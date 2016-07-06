// TODO: real network card may consolidate multiple packets per interrupt.
pub type Handler<'a, Packet> = Box<Fn(Packet) + Send + Sync + 'a>;

pub trait Interface {
  type Error;
}
