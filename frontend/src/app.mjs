import { createChart } from './chart.mjs';
import { renderSnapshot } from './dashboard.mjs';

const POLL_INTERVAL_MS = 2000;
const labels = {
  starting: 'Connecting',
  live: 'Live',
  stale: 'Data is stale',
  offline: 'Collector offline',
  disconnected: 'Dashboard disconnected',
  paused: 'Paused',
};

const status = document.getElementById('status');
const pause = document.getElementById('pause');
const pauseLabel = document.getElementById('pause-label');
const range = document.getElementById('range');
const chart = createChart(document.getElementById('history'), document.getElementById('chart-empty'));
const navigation = document.querySelectorAll('nav a');
let paused = false;
let request = null;
let timer;
let lastSample;

function showStatus(state) {
  if (status.dataset.state === state) return;
  status.dataset.state = state;
  status.textContent = labels[state];
}

async function refresh() {
  clearTimeout(timer);
  request?.abort();
  const controller = new AbortController();
  request = controller;
  const windowSeconds = Number(range.value);
  try {
    const response = await fetch(`/api/v1/overview?window=${windowSeconds}`, {
      signal: AbortSignal.any([controller.signal, AbortSignal.timeout(5000)]),
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const { status: state, snapshot, history } = await response.json();
    if (request !== controller) return;

    showStatus(state);
    if (snapshot && snapshot.collected_at !== lastSample) {
      renderSnapshot(snapshot);
      lastSample = snapshot.collected_at;
    }
    chart.show(history, windowSeconds);
  } catch {
    if (request === controller) showStatus('disconnected');
  } finally {
    if (request === controller) {
      request = null;
      timer = setTimeout(refresh, POLL_INTERVAL_MS);
    }
  }
}

pause.addEventListener('click', () => {
  paused = !paused;
  pause.setAttribute('aria-pressed', String(paused));
  pauseLabel.textContent = paused ? 'Resume' : 'Pause';
  range.disabled = paused;
  if (paused) {
    clearTimeout(timer);
    request?.abort();
    request = null;
    showStatus('paused');
  } else {
    showStatus('starting');
    refresh();
  }
});
range.addEventListener('change', refresh);

for (const button of document.querySelectorAll('[data-series]')) {
  button.addEventListener('click', () => chart.toggle(button.dataset.series, button));
}

function updateNavigation() {
  const hash = window.location.hash || '#overview';
  for (const link of navigation) {
    if (link.hash === hash) link.setAttribute('aria-current', 'location');
    else link.removeAttribute('aria-current');
  }
}

window.addEventListener('hashchange', updateNavigation);
updateNavigation();
refresh();
