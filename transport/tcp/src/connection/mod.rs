pub mod state;

//pub mod closed;
pub mod established;

pub enum Connection {
  Closed,
  //SynSent(syn_received::SynReceived),
  //SynSent(syn_sent::SynSent),
  Established(established::Established),
}
