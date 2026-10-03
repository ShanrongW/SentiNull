use chrono::{Timelike, Utc};
use cpu_info::CpuInfo;

fn main() {
    let now = Utc::now();
    let seconds = now.second();
    let minutes = now.minute();

    let cpu_info = CpuInfo::new();
    let cpu_model = cpu_info.model;
    let cpu_fabricant = cpu_info.fabricant;
    let cpu_arch = cpu_info.architecture;

    println!("Seconds: {}", seconds);
    println!("Minutes: {}", minutes);

    println!("CPU Model: {}", cpu_model);
    println!("CPU Fabricant: {:?}", cpu_fabricant);
    println!("CPU Architecture: {:?}", cpu_arch);
}
