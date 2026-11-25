import { bytes, percent, rate, ratio, temperature, time, uptime } from './format.mjs';

const byId = (id) => document.getElementById(id);

function text(id, value) {
  const element = byId(id);
  const next = value ?? '—';
  if (element.textContent !== next) element.textContent = next;
}

function element(tag, content, className) {
  const node = document.createElement(tag);
  if (content !== undefined) node.textContent = content;
  if (className) node.className = className;
  return node;
}

function metric(name, value, detail, usage) {
  text(`${name}-value`, value);
  text(`${name}-detail`, detail);
  byId(`${name}-detail`).title = detail;
  const meter = byId(`${name}-meter`);
  if (meter) {
    meter.hidden = !Number.isFinite(usage);
    meter.value = Number.isFinite(usage) ? usage : 0;
  }
}

function renderCards(snapshot) {
  const cpu = snapshot.cpu.usage_percent;
  const ram = snapshot.memory.ram;
  const memory = ratio(ram.used_bytes, ram.total_bytes);
  const primary = snapshot.gpus[0];
  const receive = snapshot.network.reduce((sum, item) => sum + item.received_bytes_per_second, 0);
  const send = snapshot.network.reduce((sum, item) => sum + item.transmitted_bytes_per_second, 0);

  metric('cpu', percent(cpu), `${snapshot.host.logical_cpus} logical CPUs`, cpu);
  metric('memory', percent(memory), memory === null ? 'Unavailable' : `${bytes(ram.used_bytes)} of ${bytes(ram.total_bytes)}`, memory);
  metric('gpu', percent(primary?.usage_percent), primary?.name ?? 'No GPU detected', primary?.usage_percent);
  metric('network', snapshot.network.length ? rate(receive) : '—',
    snapshot.network.length ? `${rate(send)} sent` : 'No interfaces detected');
}

function renderHost(snapshot) {
  const host = snapshot.host;
  text('heading-host', host.name ?? 'Local machine');
  text('sidebar-host', host.name ?? 'Local machine');
  text('host-name', host.name);
  text('host-os', host.os);
  text('host-kernel', host.kernel);
  text('host-architecture', host.architecture);
  text('host-cpu', host.cpu_model);
  text('host-uptime', uptime(snapshot.uptime_seconds));
  text('last-sample', `Last sample ${time(snapshot.collected_at)}`);
}

function renderCompute(snapshot) {
  const logical = snapshot.cpu.logical_usage_percent;
  const physical = snapshot.host.physical_cores;
  const topology = [`${logical.length} logical CPUs`];
  if (physical !== null) topology.push(`${physical} physical cores`);
  text('cores-detail', topology.join(' · '));

  const cores = logical.map((value, index) => {
    const core = element('div', undefined, 'core');
    const name = `CPU ${index}`;
    const meter = element('progress');
    meter.max = 100;
    meter.value = value ?? 0;
    meter.hidden = !Number.isFinite(value);
    meter.setAttribute('aria-label', name);
    core.append(element('span', name), element('strong', percent(value)), meter);
    return core;
  });
  byId('cores').replaceChildren(...(cores.length ? cores : [element('p', 'CPU readings unavailable', 'empty')]));

  const adapters = snapshot.gpus.map((gpu) => {
    const row = element('div', undefined, 'gpu-item');
    const vram = gpu.memory_total === null
      ? bytes(gpu.memory_used)
      : `${bytes(gpu.memory_used)} / ${bytes(gpu.memory_total)}`;
    row.append(element('strong', gpu.name), element('p',
      `Usage ${percent(gpu.usage_percent)} · VRAM ${vram} · ${temperature(gpu.temperature_celsius)}`));
    return row;
  });
  byId('gpus').replaceChildren(...(adapters.length ? adapters : [element('p', 'No graphics adapters detected', 'empty')]));
  const swap = snapshot.memory.swap;
  text('swap', swap.total_bytes > 0 ? `${bytes(swap.used_bytes)} / ${bytes(swap.total_bytes)}` : 'Unavailable');
}

function rows(id, items, columns, emptyText, getCells) {
  const fragment = document.createDocumentFragment();
  for (const item of items) {
    const row = element('tr');
    for (const value of getCells(item)) row.append(element('td', value));
    fragment.append(row);
  }
  if (!items.length) {
    const row = element('tr');
    const cell = element('td', emptyText);
    cell.colSpan = columns;
    row.append(cell);
    fragment.append(row);
  }
  byId(id).replaceChildren(fragment);
}

export function renderSnapshot(snapshot) {
  renderCards(snapshot);
  renderHost(snapshot);
  renderCompute(snapshot);

  rows('volumes', snapshot.storage, 4, 'No storage volumes detected', ({ mount, capacity }) => [
    mount, bytes(capacity.used_bytes), bytes(capacity.total_bytes), percent(ratio(capacity.used_bytes, capacity.total_bytes)),
  ]);
  rows('interfaces', snapshot.network, 5, 'No network interfaces detected', (item) => [
    item.wifi ? `${item.name} (Wi-Fi)` : item.name,
    rate(item.received_bytes_per_second), rate(item.transmitted_bytes_per_second),
    bytes(item.received_bytes), bytes(item.transmitted_bytes),
  ]);
  rows('temperatures', snapshot.sensors, 2, 'No temperature readings available', (sensor) => [
    sensor.label, temperature(sensor.temperature_celsius),
  ]);
}
