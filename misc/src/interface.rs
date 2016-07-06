// TODO: real network card may consolidate multiple packets per interrupt.
pub type Handler<Packet> = Box<Fn(Packet) + Send + Sync + 'static>;

pub trait Interface {
  type Error;
}
