import { spawn } from 'node:child_process';
import { createInterface } from 'node:readline';

const RETRY_DELAY_MS = 5000;

function readSample(line) {
  const sample = JSON.parse(line);
  if (sample?.schema_version !== 1 || !Number.isFinite(Date.parse(sample.collected_at))
      || !sample.host || !sample.cpu || !sample.memory?.ram
      || !['storage', 'network', 'gpus', 'sensors'].every((key) => Array.isArray(sample[key]))) {
    throw new Error('Unexpected collector snapshot format');
  }
  return sample;
}

export function startCollector(path, telemetry) {
  let child;
  let retry;
  let stopped = false;

  function launch() {
    telemetry.setPhase('running');
    child = spawn(path, ['--json'], { stdio: ['ignore', 'pipe', 'inherit'], windowsHide: true });
    const lines = createInterface({ input: child.stdout, crlfDelay: Infinity });
    let invalidOutput = false;
    lines.on('line', (line) => {
      if (stopped || invalidOutput) return;
      try {
        telemetry.record(readSample(line));
      } catch (error) {
        invalidOutput = true;
        telemetry.setPhase('retrying');
        console.error(`Collector output error: ${error.message}`);
        child.kill();
      }
    });
    child.on('error', (error) => {
      console.error(`Cannot start collector at ${path}: ${error.message}`);
    });
    child.once('close', (code, signal) => {
      lines.close();
      child = undefined;
      if (stopped) return;

      telemetry.setPhase('retrying');
      console.error(`Collector exited (${signal ?? code ?? 'start failed'}). Retrying in 5 seconds.`);
      retry = setTimeout(launch, RETRY_DELAY_MS);
    });
  }

  launch();
  return function stop() {
    stopped = true;
    clearTimeout(retry);
    telemetry.setPhase('stopped');
    if (!child) return;

    const processToStop = child;
    const timeout = setTimeout(() => processToStop.kill('SIGKILL'), 2000);
    processToStop.once('close', () => clearTimeout(timeout));
    processToStop.kill();
  };
}
