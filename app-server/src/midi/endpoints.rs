use std::collections::{HashMap, HashSet};

use crate::midi::drivers::{MidiOutput as MidiOutputPort};

pub type EndpointId = usize;

pub struct Endpoints<T> where T: ?Sized {
  next_id: EndpointId,
  ids_by_name: HashMap<String, EndpointId>,
  endpoints_by_id: HashMap<EndpointId, Box<T>>,
}

impl<T> Endpoints<T> where T: ?Sized {
  pub fn new() -> Self {
    Endpoints {
      next_id: 0,
      ids_by_name: HashMap::<String, EndpointId>::new(),
      endpoints_by_id: HashMap::<EndpointId, Box<T>>::new(),
    }
  }

  pub fn ids(&self) -> impl Iterator<Item = &EndpointId> {
    self.endpoints_by_id.keys()
  }

  pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Box<T>> {
    self.endpoints_by_id.values_mut()
  }

  pub fn get_id_from_name(&self, name: &str) -> Option<EndpointId> {
    self.ids_by_name.get(name).map(|id| *id)
  }

  pub fn add(&mut self, name: String, endpoint: Box<T>) -> EndpointId {
    let id = self.next_id;
    self.next_id += 1;
    self.ids_by_name.insert(name, id);
    self.endpoints_by_id.insert(id, endpoint);
    id
  }

  pub fn remove<F>(&mut self, ids: HashSet<EndpointId>, handler: F)
    where
        F: Fn(&String, EndpointId),
  {
    for (name, id) in self.ids_by_name.iter() {
      if ids.contains(id) {
        self.endpoints_by_id.remove(id);
        (handler)(name, *id)
      }
    }
  }

  pub fn get_mut(&mut self, id: EndpointId) -> Option<&mut Box<T>> {
    self.endpoints_by_id.get_mut(&id)
  }
}
