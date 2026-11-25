import { readConfig } from './config.mjs';
import { createHttpServer } from './http/server.mjs';
import { startCollector } from './telemetry/collector.mjs';
import { createTelemetry } from './telemetry/history.mjs';

const config = readConfig();
const telemetry = createTelemetry(config.historySize);
const server = createHttpServer(config, telemetry);
let stopCollector;

server.listen(config.port, config.host, () => {
  stopCollector = startCollector(config.collectorPath, telemetry);
  console.log(`SentiNull is running at http://${config.host}:${config.port}`);
});

function shutdown() {
  stopCollector?.();
  server.close();
}

process.once('SIGINT', shutdown);
process.once('SIGTERM', shutdown);
