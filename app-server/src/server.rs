use log::{debug, info, trace};

use std::collections::HashMap;
use std::net::{SocketAddr, TcpStream};
use std::thread;
use std::thread::JoinHandle;

use crossbeam_channel;
use crossbeam_channel::{Receiver, Select, Sender};

use failure::{Error, Fail};

use websocket::client::sync::Client;
use websocket::receiver::Reader;
use websocket::sender::Writer;
use websocket::server::upgrade::sync::Buffer;
use websocket::server::upgrade::WsUpgrade;
use websocket::sync::Server as WsServer;
use websocket::OwnedMessage;

#[derive(Debug, Fail)]
enum ServerError {
  #[fail(display = "Unable to accept connection")]
  RequestAccept { cause: String },
  #[fail(display = "Failed to retrieve client address")]
  ClientAddress { cause: String },
  #[fail(display = "Only localhost connections are allowed, but found {:?}", ip)]
  NotLocalhost { ip: String },
  #[fail(display = "Failed to split client IO")]
  ClientSplit { cause: String },
}

pub const ALL_PORTS: u16 = 0;

#[derive(Debug, Clone)]
pub enum Message {
  Connection { port: u16, sender: Sender<Message> },
  Close { port: u16 },
  Incoming { data: Vec<u8>, port: u16 },
  Outgoing { data: Vec<u8>, port: u16 },
  Stop,
}

impl Message {
  pub fn is_stop(&self) -> bool {
    match self {
      Message::Stop => true,
      _ => false,
    }
  }

  pub fn is_close(&self) -> bool {
    match self {
      Message::Close { .. } => true,
      _ => false,
    }
  }

  pub fn to_websocket_message(self) -> Option<OwnedMessage> {
    match self {
      Message::Connection { .. } => None,
      Message::Close { .. } => Some(OwnedMessage::Close(None)),
      Message::Incoming { .. } => None,
      Message::Outgoing { data, .. } => Some(OwnedMessage::Binary(data.into())),
      Message::Stop => None,
    }
  }
}

type Clients = HashMap<u16, Sender<Message>>;

pub struct Server {
  server_send_tx: Sender<Message>,
  server_receive_rx: Receiver<Message>,
  router_thread: JoinHandle<Result<(), Error>>,
  websocket_thread: JoinHandle<Result<(), Error>>,
}

impl Server {
  pub fn new(port: u16) -> Result<Server, Error> {
    let (server_send_tx, server_send_rx) = crossbeam_channel::unbounded::<Message>();
    let (server_receive_tx, server_receive_rx) = crossbeam_channel::unbounded::<Message>();
    let (client_receive_tx, client_receive_rx) = crossbeam_channel::unbounded::<Message>();

    let router_thread = Self::start_router(server_send_rx, client_receive_rx, server_receive_tx);

    let websocket_thread = Self::start_server(client_receive_tx, port);

    Ok(Server {
      server_send_tx,
      server_receive_rx,
      router_thread,
      websocket_thread,
    })
  }

  fn start_router(
    server_send_rx: Receiver<Message>,
    client_receive_rx: Receiver<Message>,
    server_receive_tx: Sender<Message>,
  ) -> JoinHandle<Result<(), Error>> {
    thread::Builder::new()
      .name("ws-router".into())
      .spawn(move || {
        let mut clients: Clients = HashMap::new();

        let mut select = Select::new();
        let server_index = select.recv(&server_send_rx);
        let client_index = select.recv(&client_receive_rx);

        loop {
          let result = match select.ready() {
            index if index == server_index => server_send_rx.try_recv(),
            index if index == client_index => client_receive_rx.try_recv(),
            _ => unreachable!(),
          };
          match result {
            Ok(msg) => {
              let is_stop = msg.is_stop();
              drop(Self::dispatch_message(
                &mut clients,
                server_receive_tx.clone(),
                msg,
              ));
              if is_stop {
                break;
              }
            }
            Err(err) => {
              debug!("Failed to select next message from the router: {:?}", err);
            }
          };
        }

        Ok(())
      })
      .unwrap()
  }

  fn dispatch_message(
    clients: &mut Clients,
    server_receive_tx: Sender<Message>,
    msg: Message,
  ) -> Result<(), Error> {
    // TODO Can we avoid the msg.clone() ?

    match msg {
      Message::Connection { port, sender } => {
        clients.insert(port, sender);
      }
      Message::Close { port } => {
        clients.remove(&port);
        // TODO what else ?
      }

      Message::Incoming { .. } => {
        drop(server_receive_tx.send(msg));
      }
      Message::Outgoing { port, .. } => {
        if port == ALL_PORTS {
          clients
            .values()
            .for_each(|send_tx| drop(send_tx.send(msg.clone())));
        } else {
          clients
            .get(&port)
            .iter()
            .for_each(|send_tx| drop(send_tx.send(msg.clone())));
        }
      }

      Message::Stop => {
        clients
          .values()
          .for_each(|send_tx| drop(send_tx.send(msg.clone())));
      }
    };
    Ok(())
  }

  fn start_server(client_receive_tx: Sender<Message>, port: u16) -> JoinHandle<Result<(), Error>> {
    thread::Builder::new()
      .name("ws-server".into())
      .spawn(move || {
        let addr = format!("127.0.0.1:{}", port);
        info!("Starting WebSocket server at {} ...", addr);
        let server = WsServer::bind(addr)?;
        for request in server.filter_map(Result::ok) {
          Self::accept_request(client_receive_tx.clone(), request);
        }
        Ok(())
      })
      .unwrap()
  }

  fn accept_request(
    client_receive_tx: Sender<Message>,
    request: WsUpgrade<TcpStream, Option<Buffer>>,
  ) {
    thread::spawn(move || {
      let (addr, receiver, sender) = request
        .accept()
        .map_err(|(_, err)| ServerError::RequestAccept {
          cause: err.to_string(),
        })
        .and_then(|mut client| {
          Self::ensure_valid_source_or_close(&mut client).and_then(|addr| {
            info!("New WebSocket connection: {}", addr.to_string());
            client
              .split()
              .map_err(|err| ServerError::ClientSplit {
                cause: err.to_string(),
              })
              .map(|(receiver, sender)| (addr, receiver, sender))
          })
        })?;

      let (client_send_tx, client_send_rx) = crossbeam_channel::unbounded::<Message>();

      drop(client_receive_tx.send(Message::Connection {
        port: addr.port(),
        sender: client_send_tx,
      }));

      let internal_tx = Self::send_messages(addr.clone(), client_send_rx, sender);

      Self::receive_messages(addr, client_receive_tx, internal_tx, receiver);

      Ok(())
    }) as JoinHandle<Result<(), Error>>;
  }

  fn send_messages(
    addr: SocketAddr,
    send_rx: Receiver<Message>,
    mut sender: Writer<TcpStream>,
  ) -> Sender<Message> {
    let (internal_tx, internal_rx) = crossbeam_channel::unbounded::<Message>();

    let thread_name = format!("ws-send-{}", addr.port());
    thread::Builder::new()
      .name(thread_name)
      .spawn(move || {
        let mut sel = Select::new();
        let send_index = sel.recv(&send_rx);
        let internal_index = sel.recv(&internal_rx);

        loop {
          trace!("{:?} Waiting for messages to be sent ...", addr);

          let try_msg = match sel.ready() {
            index if index == send_index => send_rx.try_recv(),
            index if index == internal_index => internal_rx.try_recv(),
            _ => unreachable!(),
          };

          match try_msg {
            Ok(msg) => {
              let is_close = msg.is_close();
              trace!("{:?} Send: {:?}", addr, msg);
              msg
                .to_websocket_message()
                .iter()
                .for_each(|ws_msg| drop(sender.send_message::<OwnedMessage>(ws_msg)));
              if is_close {
                break;
              }
            }
            _ => (),
          }
        }

        trace!("{:?} Finished thread for sending messages", addr);
      })
      .unwrap();

    internal_tx
  }

  fn receive_messages(
    addr: SocketAddr,
    receive_tx: Sender<Message>,
    internal_tx: Sender<Message>,
    mut receiver: Reader<TcpStream>,
  ) {
    let port = addr.port();

    for message in receiver.incoming_messages() {
      match message {
        Ok(OwnedMessage::Text(data)) => {
          trace!("{:?} Text: {:?}", addr, data);
          drop(receive_tx.send(Message::Incoming {
            port,
            data: data.into_bytes(),
          }));
        }
        Ok(OwnedMessage::Binary(data)) => {
          trace!("{:?} Binary: {:?}", addr, data);
          drop(receive_tx.send(Message::Incoming { port, data }));
        }
        Ok(OwnedMessage::Close(data)) => {
          trace!("{:?} Close: {:?}", addr, data);
          drop(internal_tx.send(Message::Close { port }));
          break;
        }
        Err(err) => {
          //A forced websocket close (client probably crashed, and the kernel cleaned up the socket)
          trace!("{:?} Err: {:?}", addr, err);
          drop(internal_tx.send(Message::Close { port }));
          break;
        }
        _ => {}
      }
    }

    trace!("{:?} Finished thread for receiving messages", addr);
  }

  fn ensure_valid_source_or_close(
    client: &mut Client<TcpStream>,
  ) -> Result<SocketAddr, ServerError> {
    client
      .peer_addr()
      .map_err(|err| ServerError::ClientAddress {
        cause: err.to_string(),
      })
      .and_then(|addr| {
        let ip = addr.ip();
        if ip.is_loopback() {
          Ok(addr)
        } else {
          drop(client.send_message(&OwnedMessage::Close(None)));
          Err(ServerError::NotLocalhost { ip: ip.to_string() })
        }
      })
  }

  pub fn receiver(&self) -> Receiver<Message> {
    self.server_receive_rx.clone()
  }

  //  pub fn sender(&self) -> Sender<Message> {
  //    self.server_send_tx.clone()
  //  }

  pub fn close(self) {
    info!("Closing server ...");

    drop(self.server_send_tx.send(Message::Close { port: ALL_PORTS }));
    // TODO figure out how to stop the websocket thread
    drop(self.websocket_thread.join());
  }
}
