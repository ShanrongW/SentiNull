use std::time::Instant;

use sysinfo::Networks;

use crate::snapshot::Interface;

pub(super) struct NetworkSampler {
    networks: Networks,
    last_refresh: Instant,
}

impl NetworkSampler {
    pub(super) fn new() -> Self {
        Self {
            networks: Networks::new_with_refreshed_list(),
            last_refresh: Instant::now(),
        }
    }

    pub(super) fn sample(&mut self) -> Vec<Interface> {
        self.networks.refresh(true);
        let now = Instant::now();
        let seconds = now
            .duration_since(self.last_refresh)
            .as_secs_f64()
            .max(0.001);
        self.last_refresh = now;

        let mut interfaces: Vec<_> = self.networks.iter().collect();
        interfaces.sort_unstable_by_key(|(name, _)| *name);
        interfaces
            .into_iter()
            .map(|(name, data)| Interface {
                name: name.clone(),
                wifi: is_wifi(name),
                received_bytes: data.total_received(),
                transmitted_bytes: data.total_transmitted(),
                received_bytes_per_second: data.received() as f64 / seconds,
                transmitted_bytes_per_second: data.transmitted() as f64 / seconds,
            })
            .collect()
    }
}

#[cfg(target_os = "linux")]
fn is_wifi(name: &str) -> bool {
    let interface = std::path::Path::new("/sys/class/net").join(name);
    interface.join("wireless").is_dir() || interface.join("phy80211").is_dir()
}

#[cfg(not(target_os = "linux"))]
fn is_wifi(_name: &str) -> bool {
    false
}
