use std::option::Option;
use std::sync::Arc;


pub trait RoutingTable: Send + Sync {

  // initialized with the neighbor IPs
  fn init<I>(i: I) -> Self where I: Iterator<Item=super::Addr>;

  fn lookup(&self, super::Addr) -> Option<super::Addr>;

  fn monitor(state: Arc<super::State<Self>>) -> ();

  fn dump(&self);

}
