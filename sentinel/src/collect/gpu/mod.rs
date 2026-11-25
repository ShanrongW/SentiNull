use std::process::Command;

use crate::snapshot::Gpu;

#[cfg(target_os = "linux")]
mod linux;

#[derive(PartialEq)]
struct PciAddress {
    domain: u32,
    bus: u8,
    device: u8,
    function: u8,
}

impl PciAddress {
    fn parse(value: &str) -> Option<Self> {
        let (domain, address) = value.trim().split_once(':')?;
        let (bus, address) = address.split_once(':')?;
        let (device, function) = address.split_once('.')?;
        Some(Self {
            domain: u32::from_str_radix(domain, 16).ok()?,
            bus: u8::from_str_radix(bus, 16).ok()?,
            device: u8::from_str_radix(device, 16).ok()?,
            function: u8::from_str_radix(function, 16).ok()?,
        })
    }
}

pub(crate) fn sample() -> Vec<Gpu> {
    let nvidia = nvidia_gpus();

    #[cfg(target_os = "linux")]
    let drm = linux::sample(std::path::Path::new("/sys/class/drm"), &nvidia);
    #[cfg(not(target_os = "linux"))]
    let drm = [];

    nvidia.into_iter().map(|(_, gpu)| gpu).chain(drm).collect()
}

fn nvidia_gpus() -> Vec<(Option<PciAddress>, Gpu)> {
    let query = |program: &str| {
        Command::new(program)
            .args([
                "--query-gpu=pci.bus_id,name,utilization.gpu,memory.used,memory.total,temperature.gpu",
                "--format=csv,noheader,nounits",
            ])
            .output()
    };
    let result = query("nvidia-smi");
    #[cfg(target_os = "linux")]
    let result = result.or_else(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            query("/usr/lib/wsl/lib/nvidia-smi")
        } else {
            Err(error)
        }
    });
    let Ok(output) = result else {
        return Vec::new();
    };

    if !output.status.success() {
        return Vec::new();
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(parse_nvidia_gpu)
        .collect()
}

fn parse_nvidia_gpu(row: &str) -> Option<(Option<PciAddress>, Gpu)> {
    let (address, fields) = row.split_once(',')?;
    let mut fields = fields.rsplitn(5, ',').map(str::trim);
    let temperature_celsius = fields.next()?.parse::<f32>().ok().filter(|v| v.is_finite());
    let memory_total = mib_to_bytes(fields.next()?);
    let memory_used = mib_to_bytes(fields.next()?);
    let usage_percent = percent(fields.next()?);
    let name = fields.next()?.trim_matches('"');
    if name.is_empty() {
        return None;
    }

    Some((
        PciAddress::parse(address),
        Gpu {
            name: name.to_owned(),
            usage_percent,
            memory_used,
            memory_total,
            temperature_celsius,
        },
    ))
}

fn mib_to_bytes(value: &str) -> Option<u64> {
    value.parse::<u64>().ok()?.checked_mul(1024 * 1024)
}

fn percent(value: &str) -> Option<f32> {
    value.parse().ok().filter(|v| (0.0..=100.0).contains(v))
}

#[cfg(test)]
mod tests {
    use super::{PciAddress, parse_nvidia_gpu};

    #[test]
    fn nvidia_samples_preserve_missing_metrics_and_convert_memory_to_bytes() {
        let (address, gpu) =
            parse_nvidia_gpu("00000000:0A:00.0, NVIDIA GPU, 0, 256, 8192, [N/A]").unwrap();
        assert!(address.is_some() && address == PciAddress::parse("0000:0a:00.0"));
        assert_eq!(gpu.name, "NVIDIA GPU");
        assert_eq!(gpu.usage_percent, Some(0.0));
        assert_eq!(gpu.memory_used, Some(268_435_456));
        assert_eq!(gpu.memory_total, Some(8_589_934_592));
        assert_eq!(gpu.temperature_celsius, None);

        let (_, gpu) =
            parse_nvidia_gpu("00000000:0B:00.0, NVIDIA, GPU, [N/A], [N/A], 4096, 61").unwrap();
        assert_eq!(gpu.name, "NVIDIA, GPU");
        assert_eq!(gpu.usage_percent, None);
        assert_eq!(gpu.memory_used, None);
        assert_eq!(gpu.temperature_celsius, Some(61.0));
        assert!(parse_nvidia_gpu("No devices were found").is_none());
    }
}
