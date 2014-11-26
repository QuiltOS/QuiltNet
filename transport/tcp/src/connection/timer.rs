use time;
use std::sync::{Arc, Weak, RWLock};
use std::io::Timer;
use std::time::duration::Duration;


use network::ipv4;
use network::ipv4::strategy::RoutingTable;


pub fn start_timer<A>(state: &Arc<::State<A>>,
                      weak:  &Arc<RWLock<super::Connection>>)
  where A: RoutingTable
{
  let state_weak = state.clone().downgrade();
  let con_weak  = weak.clone().downgrade();

  spawn(proc() {
    let mut interval = Duration::milliseconds(0);
    let mut timer = Timer::new().unwrap();

    loop {
      let oneshot  = timer.oneshot(interval);
      oneshot.recv();
      debug!("Timer firing");

      let (state, mut con) = match (state_weak.upgrade(), con_weak.upgrade()) {
        (Some(state), Some(arc)) => (state, arc),
        _                        => break,
      };

      if super::checkup(&mut *con.write(), &*state, &mut interval) {
        break;
      }
    }
  });
}
