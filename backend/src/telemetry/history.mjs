const STALE_AFTER_MS = 10_000;

export function createTelemetry(capacity) {
  const points = new Array(capacity);
  let size = 0;
  let cursor = 0;
  let snapshot = null;
  let timestamp = null;
  let receivedAt = null;
  let phase = 'starting';

  function record(sample) {
    timestamp = Date.parse(sample.collected_at);
    receivedAt = performance.now();
    snapshot = sample;

    const ram = sample.memory.ram;
    points[cursor] = {
      timestamp,
      cpu: sample.cpu.usage_percent,
      memory: ram.total_bytes > 0 ? 100 * ram.used_bytes / ram.total_bytes : null,
      gpu: sample.gpus[0]?.usage_percent ?? null,
    };
    cursor = (cursor + 1) % capacity;
    size = Math.min(size + 1, capacity);
  }

  function status() {
    if (phase === 'retrying' || phase === 'stopped') {
      return snapshot ? 'stale' : 'offline';
    }
    if (receivedAt === null) return 'starting';
    return performance.now() - receivedAt > STALE_AFTER_MS ? 'stale' : 'live';
  }

  function overview(windowSeconds) {
    const history = [];
    if (timestamp !== null) {
      const cutoff = timestamp - windowSeconds * 1000;
      for (let index = 0; index < size; index += 1) {
        const point = points[(cursor - size + index + capacity) % capacity];
        if (point.timestamp >= cutoff && point.timestamp <= timestamp) history.push(point);
      }
    }
    return { status: status(), snapshot, history };
  }

  return {
    record,
    status,
    overview,
    setPhase(next) {
      phase = next;
    },
  };
}
