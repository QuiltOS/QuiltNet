use std::io::BufWriter;
use std::cmp;

use network::ipv4::strategy::RoutingTable;
use network::ipv4::Addr;

use packet::{mod, TcpPacket};
use send;

use connection::packet_buf::{
  BestPacketBuf,
  PacketBuf,
  // PacketBufIter,
  get_next_write_seq,
};
use self::timer::RetransmitData;
//use super::manager::recv::RecvMgr;
//use super::manager::send::SendMgr;


mod timer;


pub const TCP_MSS           : u16  = 536u16;
pub const TCP_BUF_SIZE      : u16  = ((1u32 << 16u) - 1u32) as u16;
pub const TCP_RECV_WND_INIT : u16  = TCP_BUF_SIZE;
pub const TCP_MAX_RETRIES   : uint = 5u;

#[deriving(Show)]
pub struct TcbState {
  pub recv_WND: u16,   // Recv Window Size
  pub send_WND: u16,   // Send Window Size
  //pub send_UP:  u32, // send urgent pointer
  pub send_WL1: u32,   // Seq number for last window update
  pub send_WL2: u32,   // Ack number for last window update
}

impl TcbState
{

  //TODO: initialize all these variables from handshake!!!
  pub fn new(our_isn: u32, their_isn: u32, their_wnd: u16) -> TcbState {
    TcbState {
      recv_WND: TCP_RECV_WND_INIT,
      send_WND: their_wnd,       //INIT: get from what they tell us
      send_WL1: their_isn,  //they should be acking what we send and vice-versa
      send_WL2: our_isn,
    }
  }
}


/// Encapsulates all connection state and data structures
pub struct TCB {
  read:  BestPacketBuf,
  write: BestPacketBuf,

  // Buffers
  //recv_mgr : RecvMgr,
  //send_mgr : SendMgr,
  state:     TcbState,
  transmit_data: RetransmitData,
}

impl TCB
{
  pub fn new(our_isn: u32, their_isn: u32, their_wnd: u16) -> TCB {
    TCB {
      read  : PacketBuf::new(their_isn),
      write : PacketBuf::new(our_isn),
      state : TcbState::new(our_isn, their_isn, their_wnd),
      transmit_data: RetransmitData::new(),
    }
  }

  // ******** TCP Module API ***********************************//

  /// Receive logic for TCP packet
  pub fn recv(&mut self, packet: TcpPacket) -> (bool, bool)
  {
    let mut can_read  = false;
    let mut can_write = false;
    // Validate (ACK, SEQ in appropriate intervals)
    // Is duplicate? -> trash TODO: quick duplicate detection
    if self.validate_packet_state(&packet) {

      debug!("Valid state for packet SEQ:{}, LEN:{}", packet.get_seq_num(), packet.get_body_len());
      let seg_SEQ = packet.get_seq_num();
      let seg_WND = packet.get_window_size();

      // If ACK
      if let Some(seg_ACK) = packet.get_ack_num() {
        debug!("is ACK packet");
        // If ACKing new data
        // FIXME: is this covered in validate_packet_state()?

        if mod_in_interval(self.write.get_next_consume_seq(),
                           get_next_write_seq(&self.write),
                           seg_ACK)
        {
          debug!("ACKing new data");

          let una = self.write.get_next_consume_seq();

            // UPDATE TIMER STATE
            // (NEED X LEVELS of these if you have X max retransmission limit)
            // if seg_ACK <= last_retransmitted
            //  don't care
            // elif seg_ACK <= send.NXT
            //  RTT_last = now() - last_transmit_time()
            // else:
            //  This is an invalid ACK....

          self.transmit_data.update_rtt_from_ack(seg_ACK);


          // Fast-forward our retransmission queue up to the ACK
          let _= self.write.consume_iter()
            .zip(::std::iter::count(una, 1))
            .take_while(|&(_, n): &(u8, u32) | n != seg_ACK)
            .last();

          //    -> if (send_WL1 < SEG.SEQ) or (send_WL1 == SEG.SEQ && send_WL2 <= SEG.ACK)
          if (self.state.send_WL1 < seg_SEQ) ||
            (self.state.send_WL1 == seg_SEQ && self.mod_leq(self.state.send_WL2, seg_ACK))
          {
            self.state.send_WND = seg_WND;
            self.state.send_WL1 = seg_SEQ;
            self.state.send_WL2 = seg_ACK;
          } else {
            debug!("valid packet with shrinking ACK?");
            //    -> else
            //      -> don't care
          }
          // We have more space to write into now
          can_write = true;
          //    | OR start timer to buffer ACKS to only annouce the highest ACK instead of notifying X
          //    times
        }
      }

      {
        let old_recv_next = get_next_write_seq(&self.read);

        // add packet to buffer
        {
          // need to let, because packet will be moved
          let seq    = packet.get_seq_num();
          let offset = packet.get_payload_offset();
          self.read.add_vec(seq, packet.to_vec(), offset);
        }

        let recv_next = get_next_write_seq(&self.read);
        //debug!("read buf head changed from {} to {}", old_recv_next, recv_next);

        if old_recv_next != recv_next // != cause wrap arounds
        {
          can_read = true;
        }

        //  ->TODO Will require method in RecvMgr that can iterate over first contiguous block from RCV.NXT
        //  -  and do things at each packet (copy to buf), optionally deleting from queue once consumed

        // -> Send ACK TODO: efficient way of ACKing - single ACK value at any time, should just update
        // ACK number in timer
      }
    }
    else
    {
      debug!("Invalid Packet ::State: TCB: {}, SEG:<ACK:{}, SEQ:{}>", self.state,
             packet.get_ack_num(),
             packet.get_seq_num());
    }
    (can_read, can_write)
  }

  //TODO
  fn validate_packet_state(&self, packet: &TcpPacket) -> bool {
    true
  }

  #[inline]
  /// Returns the receive window: the number of bytes in the receive buffer that are not reserved
  /// by unconsumed, sequentially ordered data
  /*fn get_rcv_window(&self) -> u16 {
    self.recv_mgr.size - (self.state.recv_NXT - self.recv_mgr.usr_NXT) as u16
  }*/

  /// Send logic for TCP Packets

  // ********* Userland API ************************************//

  /// Read as many bytes as we can into buf from received TCP data
  /// until we get to n bytes, starting from the next unconsumed
  /// sequence number.
  ///
  /// Returns the number of bytes read
  pub fn read(&mut self, buf: &mut [u8]) -> uint {
//    debug!("tcb read called, buf is {}", self.read);

    // Get CONSUMING iter over bytes queued for reading
    let mut bytes_to_read = self.read.consume_iter().take(buf.len()).peekable();
    debug!("got consuming iterator");

    let mut ctr = 0u;
    let mut writer = BufWriter::new(buf);

    // Read as many bytes as we can to fill user's buf
    debug!("reading from consuming iterator");
    for b in bytes_to_read {
      writer.write_u8(b);
      ctr += 1;
    }
    debug!("done reading from sonsuming iterator");

    // Our receive window just widened by ctr bytes
    self.state.recv_WND += ctr as u16;

    ctr
  }


  /// Write as many bytes as we can into our TCP send buffer
  /// from buf, starting with `start`
  ///
  /// Returning the number of bytes we were able to successfully write
  /// NOTE: this is less than n when
  ///               n > (SND.UNA + SND.WND ) - SND.NXT
  pub fn send<A>(&mut self, buf:    &[u8],
                 state:  &::State<A>,
                 us:     ::ConAddr,
                 them:   ::ConAddr) -> uint
    where A: RoutingTable
  {
    //debug!("TCB state: {}", self.state);
    //debug!("User on <{}<{}> send data: {}", us, them, buf);

    let send_nxt = get_next_write_seq(&self.write);

    let bytes_written = self.write.add_slice(send_nxt, buf);

    self.transmit_data.update_with_interval(get_next_write_seq(&self.write));

    //TODO: will this SEQ num state get moved into PacketBuf?
    //debug!("SendBuf: {}", self.write);
    //debug!("{} bytes written", bytes_written);

    self.flush_transmit_queue(state, us, them);

    bytes_written as uint
  }

  //Iterate through bytes to be sent, packaging them into packets and sending them off
  pub fn flush_transmit_queue<A>(&mut self,
                                               state:  &::State<A>,
                                               us:     ::ConAddr,
                                               them:   ::ConAddr) -> send::Result<()>
    where A: RoutingTable
  {
    debug!("<{},{}> Flushing Transmission Queue", us, them);

    // UPDATE TIMER DATA
    // if new data, push another entry onto queue with now() as time
    // pop entry w/ MAX_RETRIES retries off

    // Send all the bytes we have up to the current send window
    let mut bytes_to_send = self.write.iter().take(self.state.send_WND as uint).peekable();

    let cur_recv_nxt = get_next_write_seq(&self.read);

    let mut ctr = self.write.get_next_consume_seq();
    // Until we run out of bytes
    while !bytes_to_send.is_empty() {

      // Make a packet builder
      let builder: for<'p> |&'p mut TcpPacket| -> send::Result<()> = |packet| {

        // Set Packet Header Params
        packet.set_ack_num(cur_recv_nxt);
        packet.set_seq_num(ctr);
        packet.set_window_size(self.state.recv_WND);

        // Counter for bytes added to payload
        {
          let mut v = packet.as_mut_vec();

          // Add up to MSS bytes to this packet
          for _ in range(0u, TCP_MSS as uint) {
            match bytes_to_send.next() {
              Some(b) => {
                v.push(b);
                ctr += 1;
              },
              None    => {
                debug!("less than MSS packet cause there is nothing else to send");
                break
              }
            }
          }
        };

        //TODO Update sent record for timers

        Ok(())
      };

      try!(send::send(&*state.ip,
                      Some(us.0),
                      us.1,
                      them,
                      Some(TCP_MSS),
                      |x| x,
                      builder));
    }
    Ok(())
  }

  fn mod_leq(&self, n: u32, m: u32) -> bool {
    n <= m || (n - m > (1 << 16))
  }
}

// Checks if n in (s, e] (mod 2^32)
// TODO: should be in utils or something?
#[inline]
pub fn mod_in_interval(s: u32, e: u32, n: u32) -> bool {
  if e < s {

    // interval is wrapped around
    s <= n || n <= e
  } else {

    // Plain old interval
    s <= n && n <= e
  }
}
