use std::io::IoResult;
use std::io::net::ip::{SocketAddr, Ipv4Addr};

use std::sync::{Arc, Barrier};
use std::str::from_utf8;
use std::string::String;

use misc::interface::SenderClosure;

use dl;

use super::*;


fn mk_listener(num_threads: usize) -> IoResult<(Listener, SocketAddr)> {
  // port 0 is dynamically assign
  let mut listener = try!(Listener::new(SocketAddr { ip: Ipv4Addr(0,0,0,0), port: 0 },
                                        num_threads));
  let mut addr     = try!(listener.socket.socket_name());
  addr.ip = Ipv4Addr(127, 0, 0, 1);
  println!("made listener with addr: {}", addr);
  Ok((listener, addr))
}

fn talk_to_self_channel_helper(num_threads: usize) {
  use std::sync::mpsc::*;

  let (l1, a1) = mk_listener(num_threads).unwrap();
  let (l2, a2) = mk_listener(num_threads).unwrap();

  let (tx1, rx1) = channel::<(dl::Packet,)>();
  let (tx2, rx2) = channel::<(dl::Packet,)>();

  const M1: &'static str = "Hey Josh!";
  const M2: &'static str = "Hey Cody!";

  let interface1 = Interface::new(&l1, a2, box SenderClosure::new(tx1));
  let interface2 = Interface::new(&l2, a1, box SenderClosure::new(tx2));

  dl::Interface::send(&interface1, String::from_str(M2).into_bytes()).unwrap();
  dl::Interface::send(&interface2, String::from_str(M1).into_bytes()).unwrap();

  let (packet_1,) = rx1.recv().unwrap();
  assert_eq!(packet_1.as_slice(), M1.as_bytes());
  println!("Got the first packet");

  let (packet_2,) = rx2.recv().unwrap();
  assert_eq!(packet_2.as_slice(), M2.as_bytes());
  println!("Got the second packet");
}

#[test]
fn talk_to_self_channel() {
  talk_to_self_channel_helper(1);
}
#[test]
fn talk_to_self_channel_parallel() {
  talk_to_self_channel_helper(4);
}

fn talk_to_self_callback_helper(num_threads: usize) {

  fn mk_callback(barrier: Arc<Barrier>, msg: &'static str) -> dl::Handler {
    box move |packet: dl::Packet| {
      println!("got packet: {}", from_utf8(packet.as_slice()).unwrap());
      println!("matching against: {}", from_utf8(msg.as_bytes()).unwrap());
      assert_eq!(packet.as_slice(), msg.as_bytes());
      barrier.wait();
    }
  }

  let barrier = Arc::new(Barrier::new(3));
  
  let (l1, a1) = mk_listener(num_threads).unwrap();
  let (l2, a2) = mk_listener(num_threads).unwrap();

  const M1: &'static str = "Hey Josh!";
  const M2: &'static str = "Hey Cody!";

  let interface1 = Interface::new(&l1, a2, mk_callback(barrier.clone(), M1));
  let interface2 = Interface::new(&l2, a1, mk_callback(barrier.clone(), M2));

  dl::Interface::send(&interface1, String::from_str(M2).into_bytes()).unwrap();
  dl::Interface::send(&interface2, String::from_str(M1).into_bytes()).unwrap();

  barrier.wait();
}

#[test]
fn talk_to_self_callback() {
  talk_to_self_callback_helper(1);
}

#[test]
fn talk_to_self_callback_parallel() {
  talk_to_self_callback_helper(4);
}

#[test]
fn disable_then_cant_send() {

  fn inner() -> IoResult<()> {

    //let nop = box |&: _packet: Vec<u8>| { };

    let (l, a) = mk_listener(1).unwrap();
    let mut i = Interface::new(&l, a, box |_| {});

    dl::Interface::disable(&mut i);

    assert_eq!(dl::Interface::send(&i, Vec::new()).unwrap_err(),
               // TODO: Report bug: shouldn't need prefix with `use super::*;` above
               dl::Error::Disabled);

    Ok(())
  }
  inner().unwrap();
}
