import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const binary = process.platform === 'win32' ? 'sentinel.exe' : 'sentinel';

function integerSetting(env, name, fallback, minimum, maximum) {
  const raw = env[name];
  if (raw === undefined) return fallback;
  const value = /^\d+$/.test(raw) ? Number(raw) : NaN;
  if (!Number.isInteger(value) || value < minimum || value > maximum) {
    throw new Error(`${name} must be an integer from ${minimum} to ${maximum}`);
  }
  return value;
}

export function readConfig(env = process.env) {
  return {
    host: '127.0.0.1',
    port: integerSetting(env, 'PORT', 3000, 1, 65535),
    historySize: integerSetting(env, 'SENTINULL_HISTORY_SIZE', 1800, 1, 10000),
    collectorPath: env.SENTINULL_COLLECTOR
      ? resolve(env.SENTINULL_COLLECTOR)
      : fileURLToPath(new URL(`../../sentinel/target/release/${binary}`, import.meta.url)),
    frontendPath: fileURLToPath(new URL('../../frontend/', import.meta.url)),
  };
}
