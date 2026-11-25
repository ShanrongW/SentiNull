mod collect;
mod output;
mod snapshot;

use std::{
    io::{self, Write},
    thread,
    time::Instant,
};

fn main() -> io::Result<()> {
    match monitor() {
        Err(error) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        result => result,
    }
}

fn monitor() -> io::Result<()> {
    let mut json = false;
    let mut once = false;
    for argument in std::env::args_os().skip(1) {
        match argument.to_str() {
            Some("--json") => json = true,
            Some("--once") => once = true,
            Some("--help" | "-h") => {
                return writeln!(
                    io::stdout().lock(),
                    "Usage: sentinel [--json] [--once]\n\n--json  Write one JSON object per line\n--once  Exit after the first sample"
                );
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("unknown option: {}", argument.to_string_lossy()),
                ));
            }
        }
    }

    let mut collector = collect::Collector::new();
    if !json {
        output::terminal::host(&mut io::stdout().lock(), &collector.host)?;
    }

    let interval = collect::INTERVAL.max(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    let mut next_sample = Instant::now() + interval;
    loop {
        thread::sleep(next_sample.saturating_duration_since(Instant::now()));
        next_sample = Instant::now() + interval;
        let snapshot = collector.sample();
        let mut out = io::stdout().lock();
        if json {
            output::json(&mut out, &snapshot)?;
        } else {
            output::terminal::sample(&mut out, &snapshot)?;
        }
        out.flush()?;
        if once {
            return Ok(());
        }
    }
}
