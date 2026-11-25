use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct Snapshot<'a> {
    pub(crate) schema_version: u8,
    pub(crate) collected_at: DateTime<Utc>,
    pub(crate) uptime_seconds: u64,
    pub(crate) host: &'a Host,
    pub(crate) cpu: Cpu,
    pub(crate) memory: Memory,
    pub(crate) storage: Vec<Volume>,
    pub(crate) network: Vec<Interface>,
    pub(crate) gpus: Vec<Gpu>,
    pub(crate) sensors: Vec<Sensor>,
}

#[derive(Serialize)]
pub(crate) struct Host {
    pub(crate) name: Option<String>,
    pub(crate) os: Option<String>,
    pub(crate) kernel: Option<String>,
    pub(crate) architecture: String,
    pub(crate) cpu_model: Option<String>,
    pub(crate) cpu_vendor: Option<String>,
    pub(crate) physical_cores: Option<usize>,
    pub(crate) logical_cpus: usize,
}

#[derive(Serialize)]
pub(crate) struct Cpu {
    pub(crate) usage_percent: Option<f32>,
    pub(crate) logical_usage_percent: Vec<f32>,
}

#[derive(Serialize)]
pub(crate) struct Memory {
    pub(crate) ram: Capacity,
    pub(crate) swap: Capacity,
}

#[derive(Serialize)]
pub(crate) struct Capacity {
    pub(crate) used_bytes: u64,
    pub(crate) total_bytes: u64,
}

#[derive(Serialize)]
pub(crate) struct Volume {
    pub(crate) mount: String,
    pub(crate) capacity: Capacity,
}

#[derive(Serialize)]
pub(crate) struct Interface {
    pub(crate) name: String,
    pub(crate) wifi: bool,
    pub(crate) received_bytes: u64,
    pub(crate) transmitted_bytes: u64,
    pub(crate) received_bytes_per_second: f64,
    pub(crate) transmitted_bytes_per_second: f64,
}

#[derive(Serialize)]
pub(crate) struct Gpu {
    pub(crate) name: String,
    pub(crate) usage_percent: Option<f32>,
    pub(crate) memory_used: Option<u64>,
    pub(crate) memory_total: Option<u64>,
    pub(crate) temperature_celsius: Option<f32>,
}

#[derive(Serialize)]
pub(crate) struct Sensor {
    pub(crate) label: String,
    pub(crate) temperature_celsius: f32,
}
