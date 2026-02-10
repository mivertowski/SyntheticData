# TLS & Reverse Proxy Configuration

DataSynth server supports TLS in two ways:

1. **Native TLS** (with `tls` feature flag) - direct rustls termination
2. **Reverse Proxy** - recommended for production deployments

## Native TLS

Build with TLS support:

```bash
cargo build --release -p datasynth-server --features tls
```

Run with certificate and key:

```bash
datasynth-server --tls-cert /path/to/cert.pem --tls-key /path/to/key.pem
```

## Nginx Reverse Proxy

```nginx
upstream datasynth_rest {
    server 127.0.0.1:3000;
}

upstream datasynth_grpc {
    server 127.0.0.1:50051;
}

server {
    listen 443 ssl http2;
    server_name datasynth.example.com;

    ssl_certificate /etc/ssl/certs/datasynth.pem;
    ssl_certificate_key /etc/ssl/private/datasynth-key.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    # REST API
    location / {
        proxy_pass http://datasynth_rest;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 300s;
    }

    # WebSocket
    location /ws/ {
        proxy_pass http://datasynth_rest;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_read_timeout 3600s;
    }

    # gRPC
    location /synth_server. {
        grpc_pass grpc://datasynth_grpc;
        grpc_read_timeout 300s;
    }
}
```

## Envoy Proxy

```yaml
static_resources:
  listeners:
    - name: listener_0
      address:
        socket_address:
          address: 0.0.0.0
          port_value: 443
      filter_chains:
        - transport_socket:
            name: envoy.transport_sockets.tls
            typed_config:
              "@type": type.googleapis.com/envoy.extensions.transport_sockets.tls.v3.DownstreamTlsContext
              common_tls_context:
                tls_certificates:
                  - certificate_chain:
                      filename: /etc/ssl/certs/datasynth.pem
                    private_key:
                      filename: /etc/ssl/private/datasynth-key.pem
          filters:
            - name: envoy.filters.network.http_connection_manager
              typed_config:
                "@type": type.googleapis.com/envoy.extensions.filters.network.http_connection_manager.v3.HttpConnectionManager
                stat_prefix: ingress_http
                route_config:
                  name: local_route
                  virtual_hosts:
                    - name: datasynth
                      domains: ["*"]
                      routes:
                        - match:
                            prefix: "/"
                          route:
                            cluster: datasynth_rest
                            timeout: 300s
                http_filters:
                  - name: envoy.filters.http.router
                    typed_config:
                      "@type": type.googleapis.com/envoy.extensions.filters.http.router.v3.Router

  clusters:
    - name: datasynth_rest
      connect_timeout: 5s
      type: STRICT_DNS
      load_assignment:
        cluster_name: datasynth_rest
        endpoints:
          - lb_endpoints:
              - endpoint:
                  address:
                    socket_address:
                      address: 127.0.0.1
                      port_value: 3000
```

## Health Check Configuration

For load balancers, use these health check endpoints:

| Endpoint | Purpose | Expected Response |
|----------|---------|-------------------|
| `GET /health` | Basic health | 200 OK |
| `GET /ready` | Readiness probe | 200 OK / 503 Unavailable |
| `GET /live` | Liveness probe | 200 OK |
