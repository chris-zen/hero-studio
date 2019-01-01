use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::sync::{Arc, RwLock};

use uuid::Uuid;

use crate::midi::messages::Message;
use crate::time::ClockTime;

pub type MidiBusLock = Arc<RwLock<MidiBus>>;
pub type BusNodeLock = Arc<RwLock<dyn BusNode>>;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum NodeClass {
  Source,
  Destination,
  TrackPreFX,
  TrackPostFX,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum NodeFeature {
  Default,
}

pub trait BusNode {
  fn name(&self) -> &str;
  fn class(&self) -> &NodeClass;
  fn features(&self) -> &HashSet<NodeFeature>;
  fn send(&mut self, time: ClockTime, msg: &Message);
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct BusAddress(Uuid);

impl BusAddress {
  pub fn new() -> BusAddress {
    BusAddress(Uuid::new_v4())
  }
}

#[derive(Debug, Clone)]
pub struct BusQuery {
  pub name: Option<String>,
  pub classes: Option<HashSet<NodeClass>>,
  pub features: Option<HashSet<NodeFeature>>,
}

impl BusQuery {
  pub fn new() -> BusQuery {
    BusQuery {
      name: None,
      classes: None,
      features: None,
    }
  }

  pub fn name<T>(self, name: T) -> BusQuery
  where
    T: Into<String>,
  {
    BusQuery {
      name: Some(name.into()),
      ..self
    }
  }

  pub fn classes(self, classes: HashSet<NodeClass>) -> BusQuery {
    BusQuery {
      classes: self
        .classes
        .map(|prev_classes| prev_classes.union(&classes).map(|class| *class).collect()),
      ..self
    }
  }

  pub fn class(self, class: NodeClass) -> BusQuery {
    self.classes(HashSet::from_iter(std::iter::once(class)))
  }

  pub fn features(self, features: HashSet<NodeFeature>) -> BusQuery {
    BusQuery {
      features: self.features.map(|prev_features| {
        prev_features
          .union(&features)
          .map(|feature| *feature)
          .collect()
      }),
      ..self
    }
  }

  pub fn feature(self, feature: NodeFeature) -> BusQuery {
    self.features(HashSet::from_iter(std::iter::once(feature)))
  }

  fn filter(&self, _addr: &BusAddress, node: &BusNode) -> bool {
    let name_match = self.name.as_ref().map_or(true, |name| name == node.name());
    let classes_match = self
      .classes
      .as_ref()
      .map_or(true, |classes| classes.contains(&node.class()));
    let features_match = self
      .features
      .as_ref()
      .map_or(true, |features| features.is_subset(&node.features()));
    name_match && classes_match && features_match
  }
}

pub struct MidiBus {
  nodes: HashMap<BusAddress, BusNodeLock>,
}

impl MidiBus {
  pub fn new() -> MidiBus {
    MidiBus {
      nodes: HashMap::new(),
    }
  }

  pub fn iter(&self) -> impl Iterator<Item = (BusAddress, Arc<RwLock<dyn BusNode>>)> {
    self
      .nodes
      .iter()
      .map(|(addr, node)| (addr.clone(), node.clone()))
  }

  pub fn addresses_by_query(&self, query: &BusQuery) -> Vec<BusAddress> {
    self
      .nodes
      .iter()
      .filter(move |(addr, node)| {
        node
          .read()
          .map(move |node| query.clone().filter(addr, &*node))
          .unwrap_or(false)
      })
      .map(|(addr, _node)| addr.clone())
      .collect()
  }

  pub fn get_node(&self, addr: &BusAddress) -> Option<BusNodeLock> {
    self.nodes.get(addr).map(|node| node.clone())
  }

  pub fn get_node_mut(&mut self, addr: &BusAddress) -> Option<BusNodeLock> {
    self.nodes.get_mut(addr).map(|node| node.clone())
  }

  pub fn add_node(&mut self, addr: &BusAddress, node: BusNodeLock) {
    self.nodes.insert(addr.clone(), node);
  }

  pub fn remove_node(&mut self, addr: &BusAddress) -> Option<BusNodeLock> {
    self.nodes.remove(addr)
  }

  fn node_in_classes(node: &BusNodeLock, classes: &HashSet<NodeClass>) -> bool {
    node
      .read()
      .map(|node| classes.contains(&node.class()))
      .unwrap_or(false)
  }
}
