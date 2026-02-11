// k6-bulk-generate.js -- Ramp load test for POST /api/generate/bulk.
//
// Ramps from 1 to 50 VUs, holds, then ramps back down.
//
// Run:
//   k6 run tests/load/k6-bulk-generate.js
//   k6 run -e BASE_URL=http://my-server:3000 -e API_KEY=secret tests/load/k6-bulk-generate.js

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Trend, Counter } from 'k6/metrics';
import { BASE_URL, commonHeaders, demoBulkPayload } from './k6-common.js';

// ---------------------------------------------------------------------------
// Custom metrics
// ---------------------------------------------------------------------------

const bulkDuration     = new Trend('bulk_generate_duration', true);
const entriesGenerated = new Counter('entries_generated_total');

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

export const options = {
  stages: [
    { duration: '30s', target: 50 },   // Ramp up to 50 VUs
    { duration: '2m',  target: 50 },   // Hold at 50 VUs
    { duration: '30s', target: 1 },    // Ramp down to 1 VU
  ],
  thresholds: {
    http_req_duration:        ['p(95)<5000'],   // Bulk generation p95 < 5 s
    http_req_failed:          ['rate<0.01'],
    bulk_generate_duration:   ['p(95)<5000'],
  },
};

// ---------------------------------------------------------------------------
// Test function
// ---------------------------------------------------------------------------

export default function () {
  const headers = commonHeaders();

  const res = http.post(
    `${BASE_URL}/api/generate/bulk`,
    demoBulkPayload,
    { headers, tags: { endpoint: 'bulk_generate' }, timeout: '30s' },
  );

  check(res, {
    'bulk: status 200':           (r) => r.status === 200,
    'bulk: success is true':      (r) => {
      try {
        return JSON.parse(r.body).success === true;
      } catch (_) {
        return false;
      }
    },
    'bulk: entries_generated > 0': (r) => {
      try {
        return JSON.parse(r.body).entries_generated > 0;
      } catch (_) {
        return false;
      }
    },
    'bulk: has duration_ms':       (r) => {
      try {
        return JSON.parse(r.body).duration_ms !== undefined;
      } catch (_) {
        return false;
      }
    },
  });

  if (res.status === 200) {
    try {
      const body = JSON.parse(res.body);
      bulkDuration.add(body.duration_ms || 0);
      entriesGenerated.add(body.entries_generated || 0);
    } catch (_) {
      // ignore parse errors for metrics
    }
  }

  // Brief pause between iterations to avoid pure spin-loop
  sleep(0.5);
}
