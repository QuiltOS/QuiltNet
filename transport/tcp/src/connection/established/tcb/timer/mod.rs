use time;
use std::sync::{Arc, Weak, RWLock};
use std::io::Timer;
use std::time::duration::Duration;
use std::collections::ring_buf::RingBuf;

use network::ipv4;
use network::ipv4::strategy::RoutingTable;

use connection::Connection;
use connection::established::Established;
use connection::established::tcb::{TCB,TCP_MAX_RETRIES};

pub const TCP_RTT_INIT : uint = 2_000u; // 2 seconds

#[deriving(Send)]
pub struct TransmissionEntry {

  // Sequence number of LAST byte in this block
  pub seq_num:      u32,

  // Number of times the highest sequence number in this block has been transmitted
  pub num_tries:  uint,

  // Timestamp in ns of the first transmission of the highest seq number in block
  pub ts_first:     i64,
}

#[deriving(Send)]
pub struct RetransmitData {

  // Estimate of RTT in ms
  rtt_estimate: uint,

  // Queue of (seq#, #tries, timestamp of 1st transmission) triples
  transmission_intervals: RingBuf<TransmissionEntry>,
}

impl RetransmitData {
  pub fn new() -> RetransmitData {

    // Create bounded queue for intervals, plus one spot for insertions before deletions
    let mut buf = RingBuf::new();
    buf.reserve_exact(TCP_MAX_RETRIES + 1);

    RetransmitData {
      rtt_estimate: TCP_RTT_INIT,
      transmission_intervals: buf,
    }
  }

  // Adds a new interval to the queue, popping off one of it is now expired
  pub fn update_with_interval(&mut self, seq_num: u32) -> Option<TransmissionEntry> {

    // Check to see if oldest entry is expired
    let pop_back = match self.transmission_intervals.back() {
      None      => false,
      Some(te)  => te.num_tries == TCP_MAX_RETRIES
    };

    // If so, pop it and return
    let back = if pop_back { self.transmission_intervals.pop_back() } else { None };

    // Add new interval to front of queue
    self.transmission_intervals.push( TransmissionEntry {
      seq_num:      seq_num,
      num_tries:    0u,
      ts_first:     now_millis(),
    });

    back
  }

  pub fn get_rtt_estimate(&mut self) -> uint {
    self.rtt_estimate
  }

  pub fn update_rtt_from_ack(&mut self, ack_num: u32) {

    // No update if we have no unACKed data
    if self.transmission_intervals.is_empty() { return; }

    // Get most recently sent sequence number
    let front = self.transmission_intervals.front().unwrap();

    let mut update = false;

    // If we haven't retransmitted it
    if front.num_tries <= 1 {

      // if there is more than 1 interval around, is the ACK necessarily for this interval?
      if self.transmission_intervals.len() > 1 {
        if ack_num > self.transmission_intervals[1].seq_num {
          update = true;
        }
      // ACK must be for this interval
      } else {
        update = true;
      }
    }

    // Update estimate of RTT since we are seeing new ACK of non-retransmitted data
    self.rtt_estimate = calc_rtt(front.ts_first, now_millis())

  }
}

//TODO: smooth using ALPHA
fn calc_rtt(t_sent: i64, t_recv: i64) -> uint {
  (t_recv - t_sent) as uint
}

// Hopefully returns timestamp of now in milliseconds
fn now_millis() -> i64 {
  let ts = time::get_time();
  ts.sec * 1000 + (ts.nsec / 1000) as i64
}

pub fn on_timeout<A>(est: &mut Established,
                     state: &::State<A>,
                     interval: &mut Duration)
  where A: RoutingTable
{
  let mut tcb = &mut est.tcb;

  *interval = Duration::milliseconds(tcb.transmit_data.get_rtt_estimate() as i64);

  match tcb.flush_transmit_queue(state, est.us, est.them) {
    Ok(_)  => (),
    Err(_) => debug!("failure during timeout action, ok"),
  };
}
