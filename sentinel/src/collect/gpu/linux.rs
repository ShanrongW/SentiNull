use super::{Gpu, PciAddress, percent};
use std::{collections::HashSet, fs, path::Path};

pub(super) fn sample(root: &Path, nvidia: &[(Option<PciAddress>, Gpu)]) -> Vec<Gpu> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };

    let mut cards: Vec<_> = entries
        .flatten()
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .and_then(|name| name.strip_prefix("card"))
                .is_some_and(|index| {
                    !index.is_empty() && index.bytes().all(|byte| byte.is_ascii_digit())
                })
        })
        .map(|entry| entry.path())
        .collect();
    cards.sort_unstable();

    let mut seen_devices = HashSet::new();
    let mut gpus = Vec::new();
    for card in cards {
        let Ok(device) = card.join("device").canonicalize() else {
            continue;
        };

        let address = device
            .file_name()
            .and_then(|name| PciAddress::parse(name.to_str()?));
        if address.as_ref().is_some_and(|address| {
            nvidia
                .iter()
                .any(|(other, _)| other.as_ref() == Some(address))
        }) || !seen_devices.insert(device.clone())
        {
            continue;
        }

        let card_name = card.file_name().unwrap_or_default().to_string_lossy();
        gpus.push(Gpu {
            name: format!("{} ({card_name})", device_name(&device)),
            usage_percent: read(&device.join("gpu_busy_percent")).and_then(|value| percent(&value)),
            memory_used: read(&device.join("mem_info_vram_used"))
                .and_then(|value| value.parse().ok()),
            memory_total: read(&device.join("mem_info_vram_total"))
                .and_then(|value| value.parse().ok()),
            temperature_celsius: temperature(&device),
        });
    }

    gpus
}

fn device_name(device: &Path) -> String {
    if let Some(name) = read(&device.join("product_name")) {
        return name;
    }

    let vendor = read(&device.join("vendor"));
    let model = read(&device.join("device"));
    match (vendor.as_deref(), model.as_deref()) {
        (Some("0x1002"), Some(model)) => format!("AMD {model}"),
        (Some("0x10de"), Some(model)) => format!("NVIDIA {model}"),
        (Some("0x8086"), Some(model)) => format!("Intel {model}"),
        (Some(vendor), Some(model)) => format!("PCI {vendor}:{model}"),
        _ => fs::read_link(device.join("driver"))
            .ok()
            .and_then(|path| Some(path.file_name()?.to_str()?.to_owned()))
            .unwrap_or_else(|| "DRM adapter".to_owned()),
    }
}

fn temperature(device: &Path) -> Option<f32> {
    fs::read_dir(device.join("hwmon"))
        .ok()?
        .flatten()
        .find_map(|entry| {
            let millidegrees = read(&entry.path().join("temp1_input"))?
                .parse::<i32>()
                .ok()?;
            Some(millidegrees as f32 / 1000.0)
        })
}

fn read(path: &Path) -> Option<String> {
    let value = fs::read_to_string(path).ok()?;
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}
