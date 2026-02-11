// k6-jobs.js -- Job lifecycle test: submit, poll, verify completion.
//
// Each VU submits a job via POST /api/jobs/submit, then polls
// GET /api/jobs/:id until the job reaches a terminal status
// (completed, failed, or cancelled).
//
// Run:
//   k6 run tests/load/k6-jobs.js
//   k6 run -e BASE_URL=http://my-server:3000 -e API_KEY=secret tests/load/k6-jobs.js

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Trend, Counter, Rate } from 'k6/metrics';
import { BASE_URL, commonHeaders, demoJobPayload } from './k6-common.js';

// ---------------------------------------------------------------------------
// Custom metrics
// ---------------------------------------------------------------------------

const jobSubmitDuration = new Trend('job_submit_duration', true);
const jobTotalDuration  = new Trend('job_total_duration', true);
const jobsSubmitted     = new Counter('jobs_submitted');
const jobsCompleted     = new Counter('jobs_completed');
const jobsFailed        = new Counter('jobs_failed');
const jobCompletionRate = new Rate('job_completion_rate');

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

export const options = {
  vus: 10,
  duration: '2m',
  thresholds: {
    http_req_failed:       ['rate<0.01'],
    http_req_duration:     ['p(95)<500'],    // Poll requests should be fast
    job_submit_duration:   ['p(95)<2000'],   // Submit within 2 s
    job_completion_rate:   ['rate>0.80'],    // At least 80 % of jobs complete
  },
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const POLL_INTERVAL_MS = 1000;   // 1 second between polls
const MAX_POLL_ATTEMPTS = 120;   // Give up after 120 polls (2 minutes)

// ---------------------------------------------------------------------------
// Test function
// ---------------------------------------------------------------------------

export default function () {
  const headers = commonHeaders();

  // -------------------------------------------------------------------------
  // 1. Submit a job
  // -------------------------------------------------------------------------

  const submitStart = Date.now();
  const submitRes = http.post(
    `${BASE_URL}/api/jobs/submit`,
    demoJobPayload,
    { headers, tags: { endpoint: 'job_submit' } },
  );
  const submitEnd = Date.now();
  jobSubmitDuration.add(submitEnd - submitStart);

  const submitOk = check(submitRes, {
    'submit: status 200 or 201': (r) => r.status === 200 || r.status === 201,
    'submit: body has id':       (r) => {
      try {
        return JSON.parse(r.body).id !== undefined;
      } catch (_) {
        return false;
      }
    },
  });

  if (!submitOk || (submitRes.status !== 200 && submitRes.status !== 201)) {
    jobCompletionRate.add(false);
    sleep(2);
    return;
  }

  let jobId;
  try {
    jobId = JSON.parse(submitRes.body).id;
  } catch (_) {
    jobCompletionRate.add(false);
    sleep(2);
    return;
  }

  jobsSubmitted.add(1);

  // -------------------------------------------------------------------------
  // 2. Poll for completion
  // -------------------------------------------------------------------------

  let attempts = 0;
  let terminal = false;
  let finalStatus = '';

  while (attempts < MAX_POLL_ATTEMPTS) {
    sleep(POLL_INTERVAL_MS / 1000);
    attempts++;

    const pollRes = http.get(
      `${BASE_URL}/api/jobs/${jobId}`,
      { headers, tags: { endpoint: 'job_poll' } },
    );

    check(pollRes, {
      'poll: status 200': (r) => r.status === 200,
    });

    if (pollRes.status !== 200) {
      continue;
    }

    try {
      const job = JSON.parse(pollRes.body);
      const status = job.status;

      check(job, {
        'poll: body has id':     (j) => j.id === jobId,
        'poll: body has status': (j) => j.status !== undefined,
      });

      if (status === 'completed' || status === 'failed' || status === 'cancelled') {
        terminal = true;
        finalStatus = status;

        if (status === 'completed') {
          jobsCompleted.add(1);

          check(job, {
            'poll: completed job has result': (j) => j.result !== undefined,
          });
        } else if (status === 'failed') {
          jobsFailed.add(1);
        }

        break;
      }
    } catch (_) {
      // Parse error; retry on next poll
    }
  }

  // Track total time from submit to terminal state
  const totalDuration = Date.now() - submitStart;
  jobTotalDuration.add(totalDuration);

  check(terminal, {
    'job reached terminal status': (t) => t === true,
  });

  jobCompletionRate.add(terminal && finalStatus === 'completed');

  sleep(1);
}
