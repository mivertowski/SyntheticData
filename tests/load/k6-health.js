// k6-health.js -- Smoke test for DataSynth health/readiness/liveness probes.
//
// Run:
//   k6 run tests/load/k6-health.js
//   k6 run -e BASE_URL=http://my-server:3000 tests/load/k6-health.js

import http from 'k6/http';
import { check, sleep } from 'k6';
import { BASE_URL, commonHeaders } from './k6-common.js';

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

export const options = {
  vus: 10,
  duration: '30s',
  thresholds: {
    http_req_duration: ['p(95)<100'],   // Health checks must be fast
    http_req_failed:   ['rate<0.01'],
  },
};

// ---------------------------------------------------------------------------
// Test function
// ---------------------------------------------------------------------------

export default function () {
  const headers = commonHeaders();

  // GET /health
  const healthRes = http.get(`${BASE_URL}/health`, { headers, tags: { endpoint: 'health' } });
  check(healthRes, {
    'health: status 200':       (r) => r.status === 200,
    'health: body has healthy': (r) => {
      try {
        return JSON.parse(r.body).healthy === true;
      } catch (_) {
        return false;
      }
    },
    'health: body has version': (r) => {
      try {
        return JSON.parse(r.body).version !== undefined;
      } catch (_) {
        return false;
      }
    },
  });

  // GET /ready
  const readyRes = http.get(`${BASE_URL}/ready`, { headers, tags: { endpoint: 'ready' } });
  check(readyRes, {
    'ready: status 200':      (r) => r.status === 200,
    'ready: body has ready':  (r) => {
      try {
        return JSON.parse(r.body).ready !== undefined;
      } catch (_) {
        return false;
      }
    },
    'ready: body has checks': (r) => {
      try {
        return Array.isArray(JSON.parse(r.body).checks);
      } catch (_) {
        return false;
      }
    },
  });

  // GET /live
  const liveRes = http.get(`${BASE_URL}/live`, { headers, tags: { endpoint: 'live' } });
  check(liveRes, {
    'live: status 200':         (r) => r.status === 200,
    'live: body has alive':     (r) => {
      try {
        return JSON.parse(r.body).alive === true;
      } catch (_) {
        return false;
      }
    },
    'live: body has timestamp': (r) => {
      try {
        return JSON.parse(r.body).timestamp !== undefined;
      } catch (_) {
        return false;
      }
    },
  });

  sleep(0.1);
}
