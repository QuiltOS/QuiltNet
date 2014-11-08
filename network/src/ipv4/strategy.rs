use std::option::Option;
use std::sync::Arc;


pub trait RoutingTable: Send + Sync {

  // initialized with the neighbor IPs
  fn init<I>(i: I) -> Self where I: Iterator<super::Addr>;

  fn lookup(&self, super::Addr) -> Option<super::Addr>;

  fn monitor(state: Arc<super::State<Self>>) -> ();

  fn dump(&self);

}

pub fn init_hack<RT, I>(i: I) -> RT
  where RT: RoutingTable, I: Iterator<super::Addr>
{
  RoutingTable::init::<I>(i)
}

pub fn monitor_hack<RT>(s: Arc<super::State<RT>>) where RT: RoutingTable
{
  RoutingTable::monitor(s);
}
