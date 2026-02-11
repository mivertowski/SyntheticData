// k6-streaming.js -- WebSocket test for /ws/events event streaming.
//
// Connects 5 VUs to the WebSocket endpoint and verifies that event messages
// are received over a 1-minute window.
//
// Run:
//   k6 run tests/load/k6-streaming.js
//   k6 run -e BASE_URL=http://my-server:3000 tests/load/k6-streaming.js

import { check, sleep } from 'k6';
import ws from 'k6/ws';
import { Counter, Trend } from 'k6/metrics';
import { WS_URL } from './k6-common.js';

// ---------------------------------------------------------------------------
// Custom metrics
// ---------------------------------------------------------------------------

const messagesReceived = new Counter('ws_messages_received');
const messageLatency   = new Trend('ws_message_parse_time', true);

// ---------------------------------------------------------------------------
// Options
// ---------------------------------------------------------------------------

export const options = {
  vus: 5,
  duration: '1m',
  thresholds: {
    ws_messages_received: ['count>0'],            // Must receive at least 1 message
    checks:              ['rate>0.90'],            // 90 % of checks should pass
  },
};

// ---------------------------------------------------------------------------
// Test function
// ---------------------------------------------------------------------------

export default function () {
  const url = `${WS_URL}/ws/events`;

  const res = ws.connect(url, {}, function (socket) {
    let msgCount = 0;

    socket.on('open', () => {
      // Connection established; the server streams events automatically.
    });

    socket.on('message', (data) => {
      msgCount++;
      messagesReceived.add(1);

      const parseStart = Date.now();
      try {
        const event = JSON.parse(data);
        const parseEnd = Date.now();
        messageLatency.add(parseEnd - parseStart);

        // Validate event structure (EventUpdate from websocket.rs)
        check(event, {
          'ws: event has sequence':     (e) => e.sequence !== undefined,
          'ws: event has timestamp':    (e) => e.timestamp !== undefined,
          'ws: event has event_type':   (e) => e.event_type !== undefined,
          'ws: event has document_id':  (e) => e.document_id !== undefined,
          'ws: event has company_code': (e) => e.company_code !== undefined,
          'ws: event has amount':       (e) => e.amount !== undefined,
        });
      } catch (_) {
        // Non-JSON message; skip
      }
    });

    socket.on('error', (e) => {
      console.error(`WebSocket error: ${e.error()}`);
    });

    // Keep connection open for the VU iteration duration.
    // The socket will close when the timeout fires.
    socket.setTimeout(function () {
      check(msgCount, {
        'ws: received at least 1 message': (c) => c > 0,
      });
      socket.close();
    }, 55000);  // 55 s (leave 5 s buffer inside the 1 m duration)
  });

  check(res, {
    'ws: connection status is 101': (r) => r && r.status === 101,
  });

  sleep(1);
}
