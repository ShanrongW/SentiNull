const decimal = new Intl.NumberFormat(undefined, { maximumFractionDigits: 1 });
const clock = new Intl.DateTimeFormat(undefined, { hour: '2-digit', minute: '2-digit', second: '2-digit' });

export function percent(value) {
  return Number.isFinite(value) ? `${decimal.format(value)}%` : '—';
}

export function ratio(used, total) {
  return total > 0 ? used / total * 100 : null;
}

export function bytes(value) {
  if (!Number.isFinite(value)) return '—';
  const units = ['B', 'KiB', 'MiB', 'GiB', 'TiB', 'PiB', 'EiB'];
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return `${decimal.format(value)} ${units[unit]}`;
}

export function rate(value) {
  return Number.isFinite(value) ? `${bytes(value)}/s` : '—';
}

export function temperature(value) {
  return Number.isFinite(value) ? `${decimal.format(value)} °C` : '—';
}

export function time(value) {
  return clock.format(typeof value === 'string' ? new Date(value) : value);
}

export function uptime(seconds) {
  const segments = [[86400, 'd'], [3600, 'h'], [60, 'm'], [1, 's']];
  const values = [];
  for (const [unit, label] of segments) {
    const count = Math.floor(seconds / unit);
    if (count > 0) values.push(`${count}${label}`);
    seconds %= unit;
  }
  return values.slice(0, 2).join(' ') || '0s';
}
