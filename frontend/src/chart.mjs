import { percent, time } from './format.mjs';

const series = [
  { key: 'cpu', label: 'CPU', color: '#70e4c3' },
  { key: 'memory', label: 'Memory', color: '#b7a1ff' },
  { key: 'gpu', label: 'GPU', color: '#f1bf77' },
];

export function createChart(canvas, empty) {
  const context = canvas.getContext('2d');
  const enabled = new Set(series.map(({ key }) => key));
  let points = [];
  let windowSeconds = 300;

  function draw() {
    const { width, height } = canvas.getBoundingClientRect();
    const scale = window.devicePixelRatio || 1;
    if (width < 80 || height < 70) return;

    canvas.width = Math.round(width * scale);
    canvas.height = Math.round(height * scale);
    context.setTransform(scale, 0, 0, scale, 0, 0);
    context.font = '11px monospace';
    context.lineWidth = 1;

    const left = 38;
    const right = width - 8;
    const top = 11;
    const bottom = height - 27;
    const y = (value) => bottom - (bottom - top) * Math.min(100, Math.max(0, value)) / 100;

    context.fillStyle = '#a2aabd';
    context.strokeStyle = '#2b303d';
    context.textBaseline = 'middle';
    for (const value of [100, 75, 50, 25, 0]) {
      const row = y(value);
      context.textAlign = 'right';
      context.fillText(`${value}%`, left - 8, row);
      context.beginPath();
      context.moveTo(left, row);
      context.lineTo(right, row);
      context.stroke();
    }

    const visible = series.filter(({ key }) => enabled.has(key));
    const hasValues = visible.some(({ key }) => points.some((point) => Number.isFinite(point[key])));
    empty.hidden = hasValues;
    if (!hasValues) {
      empty.textContent = !visible.length ? 'Select a series to plot' : points.length ? 'No readings for selected series' : 'Waiting for samples';
      canvas.setAttribute('aria-label', `Utilization history. ${empty.textContent}.`);
    }
    if (!points.length) return;

    const latest = points.at(-1);
    const duration = Math.max(30_000, latest.timestamp - points[0].timestamp);
    const span = Math.min(windowSeconds * 1000, duration);
    const start = latest.timestamp - span;
    const end = latest.timestamp;
    const x = (timestamp) => left + (right - left) * (timestamp - start) / span;

    context.textBaseline = 'top';
    context.textAlign = 'left';
    context.fillText(time(start), left, bottom + 9);
    context.textAlign = 'right';
    context.fillText(time(end), right, bottom + 9);

    if (hasValues) {
      canvas.setAttribute('aria-label', `Utilization history. Latest values: ${visible.map(({ key, label }) => `${label} ${latest[key] === null ? 'unavailable' : percent(latest[key])}`).join(', ')}.`);
    }

    for (const { key, color } of visible) {
      let previous = null;
      context.beginPath();
      context.strokeStyle = color;
      context.fillStyle = color;
      context.lineWidth = 2;

      for (const point of points) {
        if (!Number.isFinite(point[key])) {
          previous = null;
          continue;
        }
        const px = x(point.timestamp);
        const py = y(point[key]);
        if (previous === null || point.timestamp - previous > 10_000) {
          context.moveTo(px, py);
          context.fillRect(px - 1.5, py - 1.5, 3, 3);
        } else {
          context.lineTo(px, py);
        }
        previous = point.timestamp;
      }
      context.stroke();
    }
  }

  new ResizeObserver(draw).observe(canvas);
  return {
    show(history, seconds) {
      points = history;
      windowSeconds = seconds;
      draw();
    },
    toggle(key, button) {
      if (enabled.has(key)) enabled.delete(key);
      else enabled.add(key);
      button.setAttribute('aria-pressed', String(enabled.has(key)));
      draw();
    },
  };
}
