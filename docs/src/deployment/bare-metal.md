# Bare Metal Deployment

This guide covers installing and running DataSynth directly on a Linux server using SystemD.

## Prerequisites

- Linux x86_64 (Ubuntu 22.04+, Debian 12+, RHEL 9+, or equivalent)
- 2 GB RAM minimum (4 GB recommended)
- Root or sudo access for initial setup

## Binary Installation

### Option 1: Download Pre-Built Binary

```bash
# Download the latest release
curl -L https://github.com/ey-asu-rnd/SyntheticData/releases/latest/download/datasynth-server-linux-x86_64.tar.gz \
  -o datasynth-server.tar.gz

# Extract
tar xzf datasynth-server.tar.gz

# Install binaries
sudo install -m 0755 datasynth-server /usr/local/bin/
sudo install -m 0755 datasynth-data /usr/local/bin/

# Verify
datasynth-server --help
datasynth-data --version
```

### Option 2: Build from Source

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install protobuf compiler (required for gRPC)
sudo apt-get install -y protobuf-compiler   # Debian/Ubuntu
sudo dnf install -y protobuf-compiler       # RHEL/Fedora

# Clone and build
git clone https://github.com/ey-asu-rnd/SyntheticData.git
cd SyntheticData
cargo build --release -p datasynth-server -p datasynth-cli

# Install
sudo install -m 0755 target/release/datasynth-server /usr/local/bin/
sudo install -m 0755 target/release/datasynth-data /usr/local/bin/
```

To enable optional features during the build:

```bash
# With TLS support
cargo build --release -p datasynth-server --features tls

# With Redis distributed rate limiting
cargo build --release -p datasynth-server --features redis

# With OpenTelemetry
cargo build --release -p datasynth-server --features otel

# All features
cargo build --release -p datasynth-server --features "tls,redis,otel"
```

## User and Permissions

Create a dedicated service account:

```bash
# Create system user (no home dir, no login shell)
sudo useradd --system --no-create-home --shell /usr/sbin/nologin datasynth

# Create data and config directories
sudo mkdir -p /var/lib/datasynth
sudo mkdir -p /etc/datasynth
sudo mkdir -p /etc/datasynth/tls

# Set ownership
sudo chown -R datasynth:datasynth /var/lib/datasynth
sudo chmod 750 /var/lib/datasynth

sudo chown -R root:datasynth /etc/datasynth
sudo chmod 750 /etc/datasynth
sudo chmod 700 /etc/datasynth/tls
```

## Environment Configuration

Copy the example environment file:

```bash
sudo cp deploy/datasynth-server.env.example /etc/datasynth/server.env
sudo chown root:datasynth /etc/datasynth/server.env
sudo chmod 640 /etc/datasynth/server.env
```

Edit `/etc/datasynth/server.env`:

```bash
# Logging level
RUST_LOG=info

# API authentication (comma-separated keys)
DATASYNTH_API_KEYS=your-secure-key-1,your-secure-key-2

# Worker threads (0 = auto-detect from CPU count)
DATASYNTH_WORKER_THREADS=0

# TLS (requires --features tls build)
# DATASYNTH_TLS_CERT=/etc/datasynth/tls/cert.pem
# DATASYNTH_TLS_KEY=/etc/datasynth/tls/key.pem
```

## SystemD Service

The repository includes a production-ready SystemD unit at `deploy/datasynth-server.service`. Install it:

```bash
sudo cp deploy/datasynth-server.service /etc/systemd/system/
sudo systemctl daemon-reload
```

### Unit File Walkthrough

```ini
[Unit]
Description=DataSynth Synthetic Data Server
Documentation=https://github.com/ey-asu-rnd/SyntheticData
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=datasynth
Group=datasynth
EnvironmentFile=-/etc/datasynth/server.env
ExecStart=/usr/local/bin/datasynth-server \
    --host 0.0.0.0 \
    --port 50051 \
    --rest-port 3000
ExecReload=/bin/kill -HUP $MAINPID
Restart=on-failure
RestartSec=5
TimeoutStartSec=30
TimeoutStopSec=30

# Resource limits
MemoryMax=4G
CPUQuota=200%
TasksMax=512
LimitNOFILE=65536

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
PrivateTmp=true
PrivateDevices=true
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true
RestrictNamespaces=true
RestrictRealtime=true
RestrictSUIDSGID=true
ReadWritePaths=/var/lib/datasynth

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=datasynth-server

[Install]
WantedBy=multi-user.target
```

Key security directives:

| Directive | Effect |
|-----------|--------|
| `NoNewPrivileges=true` | Prevents privilege escalation |
| `ProtectSystem=strict` | Mounts filesystem read-only except `ReadWritePaths` |
| `ProtectHome=true` | Hides `/home`, `/root`, `/run/user` |
| `PrivateTmp=true` | Isolates `/tmp` |
| `PrivateDevices=true` | Restricts device access |
| `ReadWritePaths=/var/lib/datasynth` | Only writable directory |

### Enable and Start

```bash
sudo systemctl enable datasynth-server
sudo systemctl start datasynth-server
sudo systemctl status datasynth-server
```

### Common Operations

```bash
# View logs
journalctl -u datasynth-server -f

# Restart
sudo systemctl restart datasynth-server

# Reload (sends HUP signal)
sudo systemctl reload datasynth-server

# Stop
sudo systemctl stop datasynth-server
```

## Log Rotation

SystemD journal handles log rotation automatically. To configure retention:

```bash
# /etc/systemd/journald.conf.d/datasynth.conf
[Journal]
SystemMaxUse=2G
MaxRetentionSec=30d
```

Reload journald:

```bash
sudo systemctl restart systemd-journald
```

To export logs to a file for external log aggregation:

```bash
# Export today's logs as JSON
journalctl -u datasynth-server --since today -o json > /var/log/datasynth-$(date +%F).json
```

## Firewall Configuration

Open the required ports:

```bash
# UFW (Ubuntu)
sudo ufw allow 3000/tcp comment "DataSynth REST"
sudo ufw allow 50051/tcp comment "DataSynth gRPC"

# firewalld (RHEL/CentOS)
sudo firewall-cmd --permanent --add-port=3000/tcp
sudo firewall-cmd --permanent --add-port=50051/tcp
sudo firewall-cmd --reload
```

## Verifying the Installation

```bash
# Health check
curl -s http://localhost:3000/health | python3 -m json.tool

# Readiness check
curl -s http://localhost:3000/ready | python3 -m json.tool

# Prometheus metrics
curl -s http://localhost:3000/metrics

# Generate test data via CLI
datasynth-data generate --demo --output /tmp/datasynth-test
ls -la /tmp/datasynth-test/
```

## Updating

```bash
# Stop the service
sudo systemctl stop datasynth-server

# Replace the binary
sudo install -m 0755 /path/to/new/datasynth-server /usr/local/bin/

# Start the service
sudo systemctl start datasynth-server

# Verify
curl -s http://localhost:3000/health | python3 -m json.tool
```
