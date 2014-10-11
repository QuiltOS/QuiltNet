use std::io::Timer;
use std::sync::Arc;
use std::time::Duration;

use time::{Timespec, get_time};

use network::ipv4::IpState;

use super::{RipRow, RipTable};
use super::comm::send;

pub fn spawn_updater(state: Arc<IpState<RipTable>>) {
    spawn(proc() {
        let mut timer = Timer::new().unwrap();
        let periodic = timer.periodic(Duration::seconds(5));
        loop {
            periodic.recv();
            update(&*state);
        }
    })
}

fn update(state: &IpState<RipTable>) {
    let table_copy = {
        let guard = state.routes.map.write();
        guard.clone()
    };
    for (dst, row) in table_copy.into_iter() {
        send(state, dst, &row);
    }
}

pub fn spawn_garbage_collector(state: Arc<IpState<RipTable>>) {
    spawn(proc() {
        let mut timer = Timer::new().unwrap();
        // evert 6 seconds to ensure nothing lasts longer than 12
        let periodic = timer.periodic(Duration::seconds(6));
        loop {
            periodic.recv();
            collector_garbage(&*state);
        }
    })
}

fn collector_garbage(state: &IpState<RipTable>) {
    let cur_time = get_time();

    let mut bad_keys = Vec::new();
    { // naked block to avoid deadlock -- table needed to send packet too
        let mut table = state.routes.map.write();

        for (dst, row) in table.iter() {
            let deadline = Timespec {
                sec: row.time_added.sec + 12,
                ..row.time_added
            };
            if row.cost == 16 || deadline >= cur_time {
                bad_keys.push((*dst, *row));
            }
        }

        for &(ref k, _) in bad_keys.iter() {
            table.remove(k);
        }
    }

    for (k, ref r) in bad_keys.into_iter() {
        send(&*state, k, r);
    }
}
