# SEMANTIX Installation and Setup Guide

Complete step-by-step guide for production deployment of SEMANTIX.

## Prerequisites Checklist

- [ ] Linux (Ubuntu 20.04+, Debian 11+) or macOS 11+
- [ ] 8GB RAM minimum (16GB recommended)
- [ ] 4-core CPU minimum (8-core recommended)
- [ ] 50GB free disk space
- [ ] Internet connection for dependencies
- [ ] Bash shell

## Installation Steps

### Step 1: Install System Dependencies

#### Ubuntu/Debian

```bash
# Update package manager
sudo apt-get update
sudo apt-get upgrade -y

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install PostgreSQL
sudo apt-get install -y postgresql postgresql-contrib libpq-dev

# Install build tools
sudo apt-get install -y build-essential pkg-config openssl libssl-dev

# Optional: GPU support
sudo apt-get install -y nvidia-driver-535 cuda-toolkit-12-2
```

#### macOS

```bash
# Install Homebrew if not already installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install PostgreSQL
brew install postgresql

# Install build tools
brew install pkg-config openssl

# Start PostgreSQL
brew services start postgresql
```

### Step 2: Clone Repository

```bash
# Clone SEMANTIX
git clone https://github.com/novas-workshop-2026/learned-semantic-costs.git
cd semantix

# Create branches for development
git checkout -b development develop
```

### Step 3: Configure PostgreSQL

```bash
# Start PostgreSQL service
# Ubuntu/Debian:
sudo systemctl start postgresql
sudo systemctl enable postgresql

# macOS (if using Homebrew):
brew services start postgresql

# Create database and user
sudo -u postgres psql << EOF
CREATE USER semantix WITH PASSWORD 'semantix123';
CREATE DATABASE semantix OWNER semantix;
ALTER USER semantix CREATEDB;
GRANT ALL PRIVILEGES ON DATABASE semantix TO semantix;
EOF

# Verify connection
psql -U semantix -d semantix -c "SELECT version();"
```

### Step 4: Initialize Database Schema

```bash
# Load TPC-H schema
psql -U semantix -d semantix -f schema/tpch_schema.sql

# Verify tables created
psql -U semantix -d semantix -c "\dt"
```

### Step 5: Build SEMANTIX

```bash
# Update Rust
rustup update

# Build all binaries (release mode for production)
cargo build --release

# Verify binaries created
ls -lh target/release/semantix-* 
```

### Step 6: Generate Test Data

```bash
# Generate TPC-H dataset with semantic annotations
cargo run --release --bin data-generator

# Load data into PostgreSQL
# Ubuntu/Debian
psql -U semantix -d semantix << EOF
\COPY orders FROM tpch_orders_semantic.csv CSV HEADER;
\COPY customer FROM tpch_customer_semantic.csv CSV HEADER;
\COPY part FROM tpch_part_semantic.csv CSV HEADER;
EOF

# Verify data loaded
psql -U semantix -d semantix -c "SELECT COUNT(*) FROM orders;"
```

### Step 7: Profile System (Optional but Recommended)

```bash
# Profile operator latencies for learned cost model
cargo run --release --bin cost-profiler

# This generates latency profiles used by the scheduler
```

### Step 8: Run Benchmark

```bash
# Execute comprehensive benchmark suite
cargo run --release --bin benchmark

# Output shows performance metrics and baseline comparisons
```

## Deployment Options

### Option A: Standalone Binary Execution

```bash
# Set environment variables
export DATABASE_URL="postgresql://semantix:semantix123@localhost/semantix"
export LOG_LEVEL="info"

# Run daemon
./target/release/semantix-daemon

# In another terminal, send test query:
cargo run --release --bin benchmark
```

### Option B: Docker Deployment (Recommended)

```bash
# Build Docker image
docker build -f docker/Dockerfile -t semantix:latest .

# Run with docker-compose
cd docker
docker-compose up -d

# Wait for services to start
sleep 10

# Check status
docker ps
docker logs semantix-optimizer

# View pgAdmin (optional)
# Open http://localhost:5050
# Login: admin@semantix.local / admin123
```

### Option C: Kubernetes Deployment

```bash
# Create ConfigMap from configuration
kubectl create configmap semantix-config \
  --from-file=semantix.toml

# Create Deployment
kubectl apply -f k8s/semantix-deployment.yaml

# Expose service
kubectl expose deployment semantix-optimizer \
  --type=LoadBalancer --port=8080

# Check pod status
kubectl get pods
kubectl logs -f deployment/semantix-optimizer
```

## Configuration

### Environment Variables

```bash
# Database connection
export DATABASE_URL="postgresql://user:pass@localhost/semantix"

# Logging
export LOG_LEVEL="debug"  # debug, info, warn, error
export RUST_LOG="semantix=debug"

# System tuning
export SEMANTIX_WORKERS=4
export SEMANTIX_BATCH_SIZE=1000
```

### Configuration File

Edit `semantix.toml`:

```toml
[cost_model_config]
entropy_weight = 1.0       # Increase for cost-sensitive queries
delay_weight = 0.3         # Adjust for latency sensitivity
staleness_weight = 0.5     # Adjust for context freshness

[scheduler_config]
max_latency_ms = 50        # Reduce for real-time queries
max_iterations = 1000      # More iterations = better schedules
```

## Post-Installation Verification

```bash
# Check Rust installation
rustc --version
cargo --version

# Check PostgreSQL
psql --version

# Test database connection
psql -U semantix -d semantix -c "SELECT 1;"

# Test SEMANTIX binary
./target/release/semantix-daemon --help

# Run test suite
cargo test --all

# Check coverage (optional)
cargo tarpaulin --out Html
open tarpaulin-report.html
```

## Troubleshooting

### PostgreSQL Connection Issues

```bash
# Test connection
psql -U semantix -d semantix -c "SELECT 1;"

# Check PostgreSQL status
# Ubuntu:
sudo systemctl status postgresql

# macOS:
brew services list | grep postgres

# Verify credentials in DATABASE_URL
psql postgresql://semantix:semantix123@localhost/semantix
```

### Build Failures

```bash
# Clean build
cargo clean

# Update dependencies
cargo update

# Verbose build for debugging
RUST_BACKTRACE=1 cargo build --verbose 2>&1 | head -100
```

### Port Conflicts

```bash
# Check if ports are in use
lsof -i :5432   # PostgreSQL
lsof -i :8080   # SEMANTIX service

# Kill process using port
kill -9 $(lsof -t -i:8080)
```

### Database Lock Issues

```bash
# Terminate stuck connections
sudo -u postgres psql << EOF
SELECT pg_terminate_backend(pid) 
FROM pg_stat_activity 
WHERE datname = 'semantix' AND pid <> pg_backend_pid();
EOF

# Restart PostgreSQL
sudo systemctl restart postgresql
```

## Performance Tuning

### PostgreSQL Configuration

Edit `/etc/postgresql/*/main/postgresql.conf`:

```conf
# Memory settings (for 16GB system)
shared_buffers = 4GB
effective_cache_size = 12GB
maintenance_work_mem = 1GB
work_mem = 256MB

# Performance
max_parallel_workers = 8
max_parallel_workers_per_gather = 4
random_page_cost = 1.1

# Logging (for debugging)
log_statement = 'all'
log_duration = on
```

Restart PostgreSQL:
```bash
sudo systemctl restart postgresql
```

### SEMANTIX Configuration

For high-throughput scenarios:

```toml
[execution]
batch_size = 5000          # Larger batches
parallel_workers = 8       # More workers
enable_adaptive_batching = true

[performance]
cache_max_size = 50000     # Larger semantic cache
```

## Monitoring and Logging

### View Logs

```bash
# View SEMANTIX logs
tail -f /var/log/semantix/daemon.log

# View PostgreSQL logs
tail -f /var/log/postgresql/postgresql.log

# Real-time monitoring
watch -n 1 'cargo run --release --bin benchmark'
```

### Metrics Collection

```bash
# Enable metrics export (in semantix.toml)
metrics_export_interval_seconds = 60

# Metrics available at http://localhost:8080/metrics
curl http://localhost:8080/metrics
```

## Backup and Recovery

```bash
# Backup database
pg_dump -U semantix semantix > semantix_backup.sql

# Backup configuration
cp semantix.toml semantix.toml.backup

# Restore database
psql -U semantix -d semantix < semantix_backup.sql
```

## Uninstallation

```bash
# Remove SEMANTIX
rm -rf semantix/

# Drop PostgreSQL database
sudo -u postgres dropdb semantix

# Remove Rust (optional)
rustup self uninstall
```

## Support and Help

- **Documentation**: Read README.md
- **Issues**: GitHub Issues
- **Discussion**: GitHub Discussions
- **Email**: novas-workshop-2026@vldb.org

## Next Steps

1. Review [README.md](README.md) for architecture overview
2. Check [CONTRIBUTING.md](CONTRIBUTING.md) for development
3. Read paper: `VLDB_2026_NOVAS_Short_Paper.tex`
4. Run comprehensive benchmarks for your hardware
5. Adjust configuration for your workload

---

**Installation Complete!** 🎉

Your SEMANTIX system is ready for production use.
