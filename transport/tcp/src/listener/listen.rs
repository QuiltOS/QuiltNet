use misc::interface::{MyFn, /* Handler */};

use network::ipv4;
use network::ipv4::strategy::RoutingTable;

use Table;
use packet::TcpPacket;
use super::Listener;
use super::state::State;

use connection::established::RWHandler;

pub type OnConnectionAttempt = //Handler<Ip>;
  Box<MyFn<(::ConAddr, ::ConAddr,), Option<[RWHandler, ..2]>> + Send + Sync + 'static>;

pub struct Listen;

impl State for Listen
{
  fn next<A>(self,
             _state:  &::State<A>,
             _packet: TcpPacket)
             -> Listener
    where A: RoutingTable
  {
    // keep on listening
    super::Listen(super::listen::Listen)
  }
}

impl Listen
{
  fn new<A>(state:   &::State<A>,
            handler: OnConnectionAttempt)
            -> Result<(), ()>
  {

    Ok(())
  }
  
}
