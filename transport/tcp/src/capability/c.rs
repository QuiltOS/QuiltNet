use std::io::net::ip::Port;
use std::io::{
  Reader,
  Writer,
  IoResult,
  IoError,
  IoErrorKind
};

use std::sync::{Arc, Weak, Mutex, RWLock};
use std::comm::{
  Sender,
  Receiver,
};

use network::ipv4;
use network::ipv4::strategy::RoutingTable;

use connection::{
  mod,
  Connection,
};
use listener;
use send;

use super::{l, L};


/// Capability that gives synchronous access to a Connection
// TODO: not too great that ipv4::State's routing strategy is leaking this far.
pub struct C<A>
  where A: RoutingTable
{
  state:     Arc<::State<A>>,

  con:       Weak<RWLock<::connection::Connection>>,
  can_read:  Receiver<()>,
  can_write: Receiver<()>,
}

impl<A> C<A>
  where A: RoutingTable
{
  pub fn connect(state:   &Arc<::State<A>>,
                 us:      Port,
                 them:    ::ConAddr)
                 -> send::Result<C<A>>
  {
    let (handler, rd_rx, wt_rx) = make_con_handler();

    let con = try!(connection::handshaking::Handshaking::new(
      state, us, None, them,
      false, false, None, None, handler));

    Ok(new(state, con, rd_rx, wt_rx))
  }

  /// Convenience wrapper. If you always are going to do non-blocking, use the
  /// async API instead.
  ///
  /// `None` means no connection / closed. `Some(n)` is n number of bytes read.
  pub fn read_nonblock(&mut self, buf: &mut [u8]) -> Result<uint, ()>
  {
    debug!("trying to do a non-blocking read");
    let arc = match self.con.upgrade() {
      Some(a) => a,
      None    => return Err(()),
    };

    let mut lock = arc.write();
    let mut est = match &mut *lock {
      &Connection::Established(ref mut est) => est,
      _                                     => return Err(()),
    };
    Ok(est.read(&*self.state, buf))
  }

  // Returns (send, receive) window sizes for this connection
  pub fn get_window(&self) -> ((u32, u16), (u32, u16)) {
    self.con.upgrade().unwrap().read().deref().get_window()
  }
}

const EOF: IoError = IoError {
  kind:   IoErrorKind::EndOfFile,
  desc:   "TCP connection is apparently closed!",
  detail: None,
};

impl<A> Reader for C<A>
  where A: RoutingTable
{
  fn read(&mut self, mut buf: &mut [u8]) -> IoResult<uint>
  {
    let num_requested = buf.len();
    debug!("trying to do a blocking read");
    let arc = match self.con.upgrade() {
      Some(l) => l,
      None    => return Err(EOF),
    };

    let mut total_read = 0;

    loop {
      {
        let mut lock = arc.write();
        let mut est = match &mut *lock {
          &Connection::Established(ref mut est) => est,
          _ if total_read == 0 => return Err(EOF),
          _                    => return Ok(total_read), // semi-success: next call will EOF
        };
        //debug!("reading from est");
        let n       = est.read(&*self.state, buf);
        //debug!("tcb read {} bytes", n);
        {
          // TODO report this annoying situation
          let temp1 = buf;
          let temp2 = temp1[mut n..];
          buf       = temp2;
        }
        total_read += n;
        //debug!("total read so far is {}", total_read);
        //debug!("buf is {}", buf);
        if total_read == num_requested { return Ok(total_read) }; // success
        assert!(total_read < num_requested);
      };
      // block, after letting go of lock
      self.can_read.recv();
    }
  }
}


impl<A> Writer for C<A>
  where A: RoutingTable
{
  fn write(&mut self, mut buf: &[u8]) -> IoResult<()>
  {
    debug!("trying to do a blocking write");
    let arc = match self.con.upgrade() {
      Some(l) => l,
      None    => return Err(EOF),
    };

    loop {
      {
        let mut lock = arc.write();
        let mut est = match &mut *lock {
          &Connection::Established(ref mut est) => est,
          _ => return Err(EOF),
        };
        let n = est.write(&*self.state, buf);
        buf   = buf[n..];
        if buf.len() == 0 { return Ok(()) };
      };
      // block, after letting go of lock
      self.can_write.recv();
    }
  }
}


pub fn new<A>(state:   &Arc<::State<A>>,
              con:     Weak<RWLock<::connection::Connection>>,
              rd_rx:   Receiver<()>,
              wt_rx:   Receiver<()>)
              -> C<A>
  where A: RoutingTable
{
  // block on first CanRead---to signify that connection is established
  rd_rx.recv();

  C {
    state: state.clone(),

    con: con,
    can_read:  rd_rx,
    can_write: wt_rx,
  }
}

pub fn make_con_handler() -> (connection::established::Handler, Receiver<()>, Receiver<()>)
{
  use connection::established::Established;
  use connection::established::Situation;


  let (rd_tx, rd_rx) = channel::<()>();
  let (wt_tx, wt_rx) = channel::<()>();

  let handler = {
    // TODO: this mutex is not necessary
    let rd = Mutex::new(rd_tx);
    let wt = Mutex::new(wt_tx);
    box move |&mut: est: Established, situ: Situation| {
      debug!("in C-Capability Handler");
      match situ {
        Situation::CanRead  => rd.lock().send(()),
        Situation::CanWrite => wt.lock().send(()),
      };
      Connection::Established(est)
    }
  };
  (handler, rd_rx, wt_rx)
}

/*
impl fmt::Show for Capability<A>
  where A: RoutingTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    }
  }
  */
