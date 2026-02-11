// k6-soak.js -- 30-minute soak test for DataSynth server.
//
// Runs 5 VUs for 30 minutes, alternating between /health probes and
// POST /api/generate/bulk requests.  Periodically scrapes /metrics to
// record memory and entry counters for leak detection.
//
// Run:
//   k6 run tests/load/k6-soak.js
//   k6 run -e BASE_URL=http://my-server:3000 -e API_KEY=secret tests/load/k6-soak.js
//
// Tip: pipe output to k6 cloud or InfluxDB for time-series analysis of the
// custom memory metrics.

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Trend, Gauge, Counter } from 'k6/metrics';
import { BASE_URL, commonHeaders, demoBulkPayload } from './k6-common.js';

// ---------------------------------------------------------------------------
// Custom metrics
// ---------------------------------------------------------------------------

const bulkDuration       = new Trend('soak_bulk_duration', true);
const healthDuration     = new Trend('soak_health_duration', true);
const entriesTotal       = new Gauge('soak_entries_total');
const uptimeGauge        = new Gauge('soak_uptime_seconds');
const metricsScrapeFails = new Counter('soak_metrics_scrape_failures');

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

export const options = {
  vus: 5,
  duration: '30m',
  thresholds: {
    http_req_duration:     ['p(95)<5000'],    // Generous for soak
    http_req_failed:       ['rate<0.02'],     // Allow up to 2 % errors over 30 min
    soak_health_duration:  ['p(95)<200'],     // Health checks stay fast
    soak_bulk_duration:    ['p(95)<10000'],   // Bulk gen p95 < 10 s under sustained load
  },
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/**
 * Scrape the Prometheus /metrics endpoint and extract numeric gauge/counter
 * values.  Returns a map of metric_name -> number.
 */
function scrapeMetrics(headers) {
  const res = http.get(`${BASE_URL}/metrics`, {
    headers,
    tags: { endpoint: 'metrics_scrape' },
  });

  if (res.status !== 200) {
    metricsScrapeFails.add(1);
    return {};
  }

  const values = {};
  const lines = res.body.split('\n');
  for (const line of lines) {
    // Skip comments and empty lines
    if (line.startsWith('#') || line.trim() === '') continue;

    // Handle lines with labels: metric_name{labels} value
    // and plain lines:          metric_name value
    const match = line.match(/^([a-zA-Z_][a-zA-Z0-9_]*)(?:\{[^}]*\})?\s+([0-9eE.+-]+)/);
    if (match) {
      values[match[1]] = parseFloat(match[2]);
    }
  }

  return values;
}

// ---------------------------------------------------------------------------
// Test function
// ---------------------------------------------------------------------------

export default function () {
  const headers = commonHeaders();
  const iteration = __ITER;

  // -------------------------------------------------------------------------
  // Every iteration: health check
  // -------------------------------------------------------------------------

  const healthStart = Date.now();
  const healthRes = http.get(`${BASE_URL}/health`, {
    headers,
    tags: { endpoint: 'health' },
  });
  healthDuration.add(Date.now() - healthStart);

  check(healthRes, {
    'soak health: status 200': (r) => r.status === 200,
    'soak health: healthy':    (r) => {
      try {
        return JSON.parse(r.body).healthy === true;
      } catch (_) {
        return false;
      }
    },
  });

  // -------------------------------------------------------------------------
  // Every iteration: bulk generate
  // -------------------------------------------------------------------------

  const bulkStart = Date.now();
  const bulkRes = http.post(
    `${BASE_URL}/api/generate/bulk`,
    demoBulkPayload,
    { headers, tags: { endpoint: 'bulk_generate' }, timeout: '30s' },
  );
  bulkDuration.add(Date.now() - bulkStart);

  check(bulkRes, {
    'soak bulk: status 200':      (r) => r.status === 200,
    'soak bulk: success is true': (r) => {
      try {
        return JSON.parse(r.body).success === true;
      } catch (_) {
        return false;
      }
    },
  });

  // -------------------------------------------------------------------------
  // Every 10th iteration: scrape /metrics for memory leak monitoring
  // -------------------------------------------------------------------------

  if (iteration % 10 === 0) {
    const metrics = scrapeMetrics(headers);

    if (metrics.synth_entries_generated_total !== undefined) {
      entriesTotal.add(metrics.synth_entries_generated_total);
    }
    if (metrics.synth_uptime_seconds !== undefined) {
      uptimeGauge.add(metrics.synth_uptime_seconds);
    }

    // Log a checkpoint to stdout for manual review
    const checkpoint = {
      iteration,
      uptime: metrics.synth_uptime_seconds || 'N/A',
      entries: metrics.synth_entries_generated_total || 'N/A',
      anomalies: metrics.synth_anomalies_injected_total || 'N/A',
      active_streams: metrics.synth_active_streams || 'N/A',
    };
    console.log(`[soak checkpoint] ${JSON.stringify(checkpoint)}`);
  }

  // Pause between iterations to maintain a steady, sustainable pace
  sleep(2);
}
