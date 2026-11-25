mod gpu;
mod network;

use chrono::Utc;
use std::time::Duration;
use sysinfo::{Components, DiskRefreshKind, Disks, System};

use crate::snapshot::{Capacity, Cpu, Host, Memory, Sensor, Snapshot, Volume};
use network::NetworkSampler;

pub(crate) const INTERVAL: Duration = Duration::from_secs(2);

pub(crate) struct Collector {
    pub(crate) host: Host,
    system: System,
    network: NetworkSampler,
    disks: Disks,
    components: Components,
}

impl Collector {
    pub(crate) fn new() -> Self {
        let mut system = System::new();
        system.refresh_cpu_usage();

        let cpu = system.cpus().first();
        Self {
            host: Host {
                name: System::host_name(),
                os: System::long_os_version(),
                kernel: System::kernel_version(),
                architecture: System::cpu_arch().to_string(),
                cpu_model: cpu
                    .map(|cpu| cpu.brand())
                    .filter(|value| !value.is_empty())
                    .map(str::to_owned),
                cpu_vendor: cpu
                    .map(|cpu| cpu.vendor_id())
                    .filter(|value| !value.is_empty())
                    .map(str::to_owned),
                physical_cores: System::physical_core_count(),
                logical_cpus: system.cpus().len(),
            },
            system,
            network: NetworkSampler::new(),
            disks: Disks::new(),
            components: Components::new(),
        }
    }

    pub(crate) fn sample(&mut self) -> Snapshot<'_> {
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
        self.disks
            .refresh_specifics(true, DiskRefreshKind::nothing().with_storage());
        self.components.refresh(true);

        let mut storage: Vec<_> = self
            .disks
            .iter()
            .filter(|disk| disk.total_space() > 0)
            .collect();
        storage.sort_unstable_by(|left, right| left.mount_point().cmp(right.mount_point()));

        Snapshot {
            schema_version: 1,
            collected_at: Utc::now(),
            uptime_seconds: System::uptime(),
            host: &self.host,
            cpu: Cpu {
                usage_percent: (!self.system.cpus().is_empty())
                    .then(|| self.system.global_cpu_usage()),
                logical_usage_percent: self
                    .system
                    .cpus()
                    .iter()
                    .map(|cpu| cpu.cpu_usage())
                    .collect(),
            },
            memory: Memory {
                ram: Capacity {
                    used_bytes: self.system.used_memory(),
                    total_bytes: self.system.total_memory(),
                },
                swap: Capacity {
                    used_bytes: self.system.used_swap(),
                    total_bytes: self.system.total_swap(),
                },
            },
            storage: storage
                .into_iter()
                .map(|disk| Volume {
                    mount: disk.mount_point().to_string_lossy().into_owned(),
                    capacity: Capacity {
                        used_bytes: disk.total_space().saturating_sub(disk.available_space()),
                        total_bytes: disk.total_space(),
                    },
                })
                .collect(),
            network: self.network.sample(),
            sensors: self
                .components
                .iter()
                .filter_map(|item| {
                    let temperature_celsius =
                        item.temperature().filter(|value| value.is_finite())?;
                    Some(Sensor {
                        label: item.label().to_owned(),
                        temperature_celsius,
                    })
                })
                .collect(),
            gpus: gpu::sample(),
        }
    }
}
