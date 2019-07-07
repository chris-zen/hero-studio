use crate::midi::drivers::MidiEndpoint;
use std::collections::{HashMap, HashSet};

pub type EndpointId = usize;

pub struct Endpoints<T>
where
  T: MidiEndpoint + ?Sized,
{
  next_id: EndpointId,
  ids_by_name: HashMap<String, EndpointId>,
  endpoints_by_id: HashMap<EndpointId, Box<T>>,
}

impl<T> Endpoints<T>
where
  T: MidiEndpoint + ?Sized,
{
  pub fn new() -> Self {
    Endpoints {
      next_id: 0,
      ids_by_name: HashMap::<String, EndpointId>::new(),
      endpoints_by_id: HashMap::<EndpointId, Box<T>>::new(),
    }
  }

  pub fn next_id(&self) -> EndpointId {
    self.next_id
  }

  pub fn ids(&self) -> impl Iterator<Item = &EndpointId> {
    self.endpoints_by_id.keys()
  }

  pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Box<T>> {
    self.endpoints_by_id.values_mut()
  }

  pub fn get_id_from_name(&self, name: &str) -> Option<EndpointId> {
    self.ids_by_name.get(name).cloned()
  }

  pub fn add<U>(&mut self, name: U, endpoint: Box<T>) -> EndpointId
  where
    U: Into<String>,
  {
    let id = self.next_id;
    self.next_id += 1;
    self.ids_by_name.insert(name.into(), id);
    self.endpoints_by_id.insert(id, endpoint);
    id
  }

  pub fn remove<F>(&mut self, ids: HashSet<EndpointId>, handler: F)
  where
    F: Fn(&str, EndpointId),
  {
    for id in ids.iter() {
      let maybe_name = self
        .endpoints_by_id
        .get(id)
        .map(|port| port.name().to_string());
      maybe_name.into_iter().for_each(|name| {
        self.ids_by_name.remove(&name);
        self.endpoints_by_id.remove(id);
        (handler)(&name, *id)
      });
    }
  }

  pub fn get_mut(&mut self, id: EndpointId) -> Option<&mut T> {
    self.endpoints_by_id.get_mut(&id).map(Box::as_mut)
  }
}
