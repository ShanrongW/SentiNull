use std::io::{self, Write};

use crate::snapshot::{Capacity, Host, Snapshot};

const KIB: f64 = 1024.0;
const MIB: f64 = KIB * KIB;
const GIB: f64 = MIB * KIB;

pub(crate) fn host(out: &mut impl Write, host: &Host) -> io::Result<()> {
    for (label, value) in [
        ("Host", &host.name),
        ("OS", &host.os),
        ("Kernel", &host.kernel),
        ("CPU Model", &host.cpu_model),
        ("CPU Manufacturer", &host.cpu_vendor),
    ] {
        if let Some(value) = value {
            writeln!(out, "{label}: {value}")?;
        }
    }
    writeln!(out, "CPU Architecture: {}", host.architecture)?;
    if let Some(physical) = host.physical_cores {
        writeln!(out, "Physical CPU Cores: {physical}")?;
    }
    writeln!(out, "Logical CPUs: {}", host.logical_cpus)
}

pub(crate) fn sample(out: &mut impl Write, snapshot: &Snapshot<'_>) -> io::Result<()> {
    writeln!(
        out,
        "\nMeasurement time: {}",
        snapshot.collected_at.to_rfc3339()
    )?;
    writeln!(out, "Uptime: {} s", snapshot.uptime_seconds)?;
    writeln!(out, "CPU Usage: {}", percent(snapshot.cpu.usage_percent))?;
    if !snapshot.cpu.logical_usage_percent.is_empty() {
        let cores: Vec<_> = snapshot
            .cpu
            .logical_usage_percent
            .iter()
            .enumerate()
            .map(|(index, usage)| format!("CPU {index} {usage:.1}%"))
            .collect();
        writeln!(out, "Logical CPU Usage: {}", cores.join(", "))?;
    }
    if snapshot.memory.ram.total_bytes == 0 {
        writeln!(out, "RAM Used: unavailable")?;
    } else {
        writeln!(out, "RAM Used: {}", capacity(&snapshot.memory.ram, false))?;
    }
    writeln!(out, "Swap Used: {}", capacity(&snapshot.memory.swap, false))?;
    if snapshot.storage.is_empty() {
        writeln!(out, "Storage: unavailable")?;
    }
    for volume in &snapshot.storage {
        writeln!(
            out,
            "Storage {}: {}",
            volume.mount,
            capacity(&volume.capacity, true)
        )?;
    }
    if snapshot.network.is_empty() {
        writeln!(out, "Network: unavailable")?;
    }
    for interface in &snapshot.network {
        let label = if interface.wifi { "Wi-Fi" } else { "Network" };
        writeln!(
            out,
            "{label} {}: RX {:.2} KiB/s ({:.2} MiB total), TX {:.2} KiB/s ({:.2} MiB total)",
            interface.name,
            interface.received_bytes_per_second / KIB,
            interface.received_bytes as f64 / MIB,
            interface.transmitted_bytes_per_second / KIB,
            interface.transmitted_bytes as f64 / MIB
        )?;
    }
    if !snapshot.network.iter().any(|interface| interface.wifi) {
        let status = if cfg!(target_os = "linux") {
            "no interface detected"
        } else {
            "interface detection unavailable"
        };
        writeln!(out, "Wi-Fi: {status}")?;
    }
    if snapshot.gpus.is_empty() {
        writeln!(out, "GPU: unavailable")?;
    }
    for gpu in &snapshot.gpus {
        let memory = match (gpu.memory_used, gpu.memory_total) {
            (Some(used), Some(total)) => {
                format!("{:.2} / {:.2} GiB", used as f64 / GIB, total as f64 / GIB)
            }
            (Some(used), None) => format!("{:.2} GiB used", used as f64 / GIB),
            (None, Some(total)) => format!("{:.2} GiB total", total as f64 / GIB),
            (None, None) => "unavailable".to_owned(),
        };
        let temperature = gpu.temperature_celsius.map_or_else(
            || "unavailable".to_owned(),
            |value| format!("{value:.1} °C"),
        );
        writeln!(
            out,
            "GPU {}: usage {}, VRAM {memory}, temperature {temperature}",
            gpu.name,
            percent(gpu.usage_percent)
        )?;
    }
    for sensor in &snapshot.sensors {
        writeln!(
            out,
            "Sensor {}: {:.1} °C",
            sensor.label, sensor.temperature_celsius
        )?;
    }
    Ok(())
}

fn percent(value: Option<f32>) -> String {
    value.map_or_else(|| "unavailable".to_owned(), |value| format!("{value:.1}%"))
}

fn capacity(value: &Capacity, show_used: bool) -> String {
    let used_label = if show_used { " used" } else { "" };
    format!(
        "{:.2} GiB{used_label} / {:.2} GiB",
        value.used_bytes as f64 / GIB,
        value.total_bytes as f64 / GIB
    )
}
