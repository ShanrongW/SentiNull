# SentiNull

SentiNull monitors the machine it runs on. The dashboard shows CPU and per-core load, RAM and swap, GPU usage and video memory, storage, network traffic, and temperature sensors. Available readings depend on the operating system, drivers, and hardware.

## Run the dashboard

Install Rust with Cargo and Node.js. The collector requires Rust 1.95 or later. The commands below were verified with Rust 1.98.1 and Node.js 24.21.0.

From the repository root, build the collector, then start the server.

```sh
cargo build --release --locked --manifest-path sentinel/Cargo.toml
node backend/src/index.mjs
```

Open [http://127.0.0.1:3000](http://127.0.0.1:3000). The first sample arrives after about two seconds. Choose a time range or toggle a chart series to inspect recent utilization. **Pause** freezes the browser display while the collector continues to run. Press Ctrl+C in the server terminal to stop the app.

The server binds to the local machine. It keeps recent chart history in memory and clears it when you restart the server. Rebuild the Rust binary after changing the collector, then restart the server.

| Environment variable | Default | Purpose |
| --- | --- | --- |
| `PORT` | `3000` | HTTP port on `127.0.0.1`. |
| `SENTINULL_HISTORY_SIZE` | `1800` | Maximum chart samples to retain, from 1 to 10,000. The default holds about an hour at a two-second interval. |
| `SENTINULL_COLLECTOR` | `sentinel/target/release/sentinel` | Path to an alternate collector binary. On Windows, the default ends in `.exe`. Relative overrides use the server's working directory. |

If the dashboard shows **Collector offline**, check the server terminal for the error and verify that the collector binary exists. The backend retries five seconds after the collector exits. The dashboard keeps the last reading visible during interruptions and labels it as stale or disconnected.

For NVIDIA readings, install the vendor driver and make `nvidia-smi` available on `PATH`. The collector also checks the standard WSL NVIDIA path when it cannot find `nvidia-smi` on `PATH`. Linux can report GPU metrics from DRM and Wi-Fi interfaces from sysfs when those interfaces are available. Individual readings appear as dashes when the collector cannot obtain them.

## Run the Rust collector directly

Run the terminal monitor or output one machine-readable snapshot.

```sh
cargo run --locked --manifest-path sentinel/Cargo.toml
cargo run --locked --manifest-path sentinel/Cargo.toml -- --json --once
```

Without `--once`, `--json` emits one JSON object per line. Each snapshot includes a schema version, a UTC timestamp, host information, and the latest readings. Capacity is in bytes, network rates are in bytes per second, utilization is a percentage, and temperature is in degrees Celsius.

## Project layout

| Folder | Responsibility |
| --- | --- |
| `sentinel/src/collect/` | Sample the host and handle platform-specific GPU and network collection. |
| `sentinel/src/output/` | Render each snapshot as terminal text or newline-delimited JSON. |
| `backend/src/telemetry/` | Manage one collector process and retain a fixed number of chart samples. |
| `backend/src/http/` | Serve the JSON API and dashboard on the same origin. |
| `frontend/src/` | Poll the API, render host readings, and draw utilization history. |
| `frontend/styles/` | Style the dashboard and its responsive layout. |

All dashboard clients share the same sampler. The backend stores one complete snapshot and compact chart points. The Node.js backend and browser frontend use their standard libraries and APIs and need no package installation or frontend build.

`GET /api/v1/overview?window=300` returns the collector status, the latest snapshot, and history within the requested number of seconds. `window` defaults to 300 and accepts integers from 30 through 3600. `GET /api/v1/health` returns HTTP 200 while samples are current and 503 while the collector is starting, stale, or offline.

## Run the Rust tests

```sh
cargo test --locked --manifest-path sentinel/Cargo.toml
```
