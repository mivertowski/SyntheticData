// Shared configuration and utilities for DataSynth k6 load tests.
//
// Usage: import { BASE_URL, commonHeaders, defaultThresholds, ... } from './k6-common.js';

// ---------------------------------------------------------------------------
// Base URL
// ---------------------------------------------------------------------------

// Override with:  k6 run -e BASE_URL=http://my-server:3000 k6-health.js
export const BASE_URL = __ENV.BASE_URL || 'http://localhost:3000';

// WebSocket variant (ws:// or wss:// derived from BASE_URL).
export const WS_URL = BASE_URL.replace(/^http/, 'ws');

// ---------------------------------------------------------------------------
// API key (optional - only needed when the server has auth enabled)
// ---------------------------------------------------------------------------

export const API_KEY = __ENV.API_KEY || '';

// ---------------------------------------------------------------------------
// Common headers
// ---------------------------------------------------------------------------

export function commonHeaders() {
  const headers = {
    'Content-Type': 'application/json',
    'Accept': 'application/json',
  };
  if (API_KEY) {
    headers['X-API-Key'] = API_KEY;
  }
  return headers;
}

// ---------------------------------------------------------------------------
// Default thresholds
// ---------------------------------------------------------------------------

// Standard thresholds shared across most test scripts.
// Individual scripts may override or extend these.
export const defaultThresholds = {
  http_req_duration: ['p(95)<500'],   // 95th percentile under 500 ms
  http_req_failed:   ['rate<0.01'],   // Error rate under 1 %
};

// ---------------------------------------------------------------------------
// Demo generation payload (matches BulkGenerateRequest)
// ---------------------------------------------------------------------------

export const demoBulkPayload = JSON.stringify({
  entry_count: 100,
  include_master_data: false,
  inject_anomalies: false,
});

// ---------------------------------------------------------------------------
// Demo job submission payload (matches JobRequest)
// ---------------------------------------------------------------------------

export const demoJobPayload = JSON.stringify({
  demo: true,
  seed: 42,
});

// ---------------------------------------------------------------------------
// Helper: check a JSON response for expected fields
// ---------------------------------------------------------------------------

import { check } from 'k6';

/**
 * Validate an HTTP response has the expected status and a JSON body with
 * the given top-level keys.
 *
 * @param {object} res        - k6 http response
 * @param {number} status     - expected HTTP status code
 * @param {string[]} keys     - top-level JSON keys to assert exist
 * @param {string} [label]    - optional label prefix for check names
 * @returns {boolean}         - true if all checks passed
 */
export function checkJsonResponse(res, status, keys, label) {
  const prefix = label ? `${label}: ` : '';
  const checks = {};
  checks[`${prefix}status is ${status}`] = (r) => r.status === status;
  keys.forEach((key) => {
    checks[`${prefix}body has '${key}'`] = (r) => {
      try {
        const body = JSON.parse(r.body);
        return body[key] !== undefined;
      } catch (_) {
        return false;
      }
    };
  });
  return check(res, checks);
}
