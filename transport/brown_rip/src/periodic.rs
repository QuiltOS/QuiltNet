use std::io::Timer;
use std::io::net::ip::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use time::{Timespec, get_time};

use network::ipv4::IpState;

use super::{RIP_INFINITY, RipRow, RipTable};
use super::comm::propagate;


const UPDATE_PERIOD:     uint = 5;
const EXPIRATION_PERIOD: uint = 12;


pub fn spawn_updater(state: Arc<IpState<RipTable>>) {
  spawn(proc() {
    let mut timer = Timer::new().unwrap();
    let periodic = timer.periodic(Duration::seconds(UPDATE_PERIOD));
    loop {
      debug!("periodic update");
      periodic.recv();
      update(&*state);
    }
  })
}

fn update(state: &IpState<RipTable>) {
  let unlocked = state.routes.map.read();
  // propegate the whole damn table!
  let factory = || unlocked.iter().map(|(a,r)| (*a,r));

  // ignore errors, for now
  let _ = propagate(factory,
                    state.neighbors.keys().map(|x| *x), // tell everyone
                    &state.neighbors,
                    state.interfaces.as_slice());
}

pub fn spawn_garbage_collector(state: Arc<IpState<RipTable>>) {
  spawn(proc() {
    let mut timer = Timer::new().unwrap();
    // half the expiration period to ensure nothing lives too long
    let periodic = timer.periodic(Duration::seconds(EXPIRATION_PERIOD / 2));
    loop {
      debug!("Periodic gc");
      periodic.recv();
      collector_garbage(&*state);
    }
  })
}

fn collector_garbage(state: &IpState<RipTable>) {
  let cur_time = get_time();

  let mut bad_keys: Vec<IpAddr> = Vec::new();
  { // naked block to make sure lock is released
    let mut bad_rows: Vec<&RipRow> = Vec::new();

    let mut table = state.routes.map.write();
    for (dst, row) in table.iter_mut() {
      let deadline = Timespec {
        sec: row.time_added.sec + EXPIRATION_PERIOD,
        ..row.time_added
      };
      // allowed to forget neighbors, though the neighbor -> interface map
      // will remember them
      if row.cost == RIP_INFINITY || deadline >= cur_time
      {
        row.cost = RIP_INFINITY; // dead rows shall be poisonsed
        bad_keys.push(*dst);
        bad_rows.push(row);
      }
    }

    let zip_iter_factory = || bad_keys.iter()
      .map(|x| *x)
      .zip(bad_rows.iter().map(|x| *x));

    // ignore errors, for now
    let _ = propagate(zip_iter_factory,
                      state.neighbors.keys().map(|x| *x), // all neighbors
                      &state.neighbors,
                      state.interfaces.as_slice());
  }

  for k in bad_keys.into_iter() {
    // lock is reaquired
    let mut table = state.routes.map.write();
    table.remove(&k);
  }
}
