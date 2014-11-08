pub mod state;

pub mod closed;

pub enum Connection {
  Closed(closed::Closed),
  //SynSent(syn_received::SynReceived),
  //SynSent(syn_sent::SynSent),
  //Established(established::Established),
}
