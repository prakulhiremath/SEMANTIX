# SEMANTIX - Quick Start (from ZIP)

## 🚀 What You Have

You have downloaded the complete, production-ready SEMANTIX repository for the VLDB 2026 NOVAS Workshop.

**SEMANTIX** = Learned Semantic Cost Models for Adaptive Token-Efficient Query Optimization in LLM-Native Relational Engines

## 📦 Package Contents

This ZIP contains:
- ✅ Complete Rust source code (~3,500 lines)
- ✅ PostgreSQL schema with semantic metadata
- ✅ Docker configuration for easy deployment
- ✅ Automated setup script
- ✅ Comprehensive documentation
- ✅ CI/CD pipeline configuration
- ✅ Production-ready binaries (when built)

## ⚡ Quick Installation (5 minutes)

### On Linux/macOS

```bash
# Extract ZIP
unzip semantix.zip
cd semantix

# Run automated setup (requires sudo for PostgreSQL)
chmod +x setup.sh
./setup.sh

# Follow the prompts - setup will:
# 1. Install dependencies (Rust, PostgreSQL)
# 2. Initialize database
# 3. Build project
# 4. Generate test data
# 5. Run benchmarks
```

### With Docker (Easiest)

```bash
# Extract ZIP
unzip semantix.zip
cd semantix

# Start with Docker Compose
docker-compose -f docker/docker-compose.yml up -d

# Services will be ready in ~30 seconds
# - SEMANTIX: http://localhost:8080
# - PostgreSQL: localhost:5432
# - pgAdmin: http://localhost:5050 (admin@semantix.local / admin123)
```

## 📚 Documentation

After extraction, read in this order:

1. **README.md** - System overview and architecture
2. **INSTALL.md** - Detailed installation instructions  
3. **PROJECT_OVERVIEW.md** - Repository structure and metrics
4. **CONTRIBUTING.md** - Development guidelines

## 🎯 First Steps

```bash
# After successful setup:

# 1. View repository structure
tree -L 2

# 2. Run benchmarks
make benchmark
# or: cargo run --release --bin benchmark

# 3. Access database
psql -U semantix -d semantix
# Password: semantix123

# 4. View performance metrics
tail -f /var/log/semantix/daemon.log
```

## 📊 Key Files

### Core Implementation
- `src/semantic_anchors.rs` - NL → LogicalPlan translation
- `src/cost_model.rs` - Learned cost estimation (Eq. 1)
- `src/scheduler.rs` - Adaptive token scheduling (Algorithm 1)
- `src/database.rs` - PostgreSQL integration

### Utilities
- `setup.sh` - Automated installation
- `Makefile` - Development commands
- `docker-compose.yml` - Container orchestration
- `semantix.toml` - Configuration template

### Tests & Benchmarks
- `src/bin/benchmark.rs` - Performance evaluation
- `src/bin/profiler.rs` - Latency profiling
- `src/bin/data_gen.rs` - TPC-H data generation
- `schema/tpch_schema.sql` - Database schema

## 🔧 Requirements

### Minimum
- Linux (Ubuntu 20.04+) or macOS 11+
- 8GB RAM
- 4-core CPU
- 50GB disk space

### Recommended
- 16GB RAM
- 8-core CPU
- 100GB disk space
- NVIDIA GPU (optional, for acceleration)

## 🚀 Usage Examples

### Command-line Interface

```bash
# Build release binary
cargo build --release

# Run daemon
./target/release/semantix-daemon

# Run benchmarks
./target/release/benchmark

# Profile system
./target/release/cost-profiler

# Generate data
./target/release/data-generator
```

### Makefile Commands

```bash
# Development
make build              # Debug build
make build-release      # Optimized build
make test               # Run all tests
make fmt                # Format code
make lint               # Check with clippy

# Database
make db-create          # Create database
make db-reset           # Reset and reload
make db-status          # Show table stats

# Execution
make run                # Run daemon
make benchmark          # Run benchmarks
make profiler           # Profile system

# Docker
make docker-build       # Build image
make docker-run         # Start containers
make docker-stop        # Stop containers
```

## 📈 Performance (Expected)

After setup, you should see:

```
Query: SELECT * FROM orders WHERE custkey = 1
Tokens: 5,700 K
Latency: 25.3 ms
Accuracy: 97.1%
Energy: 1.1 Wh

Improvement over baselines:
- 3.2× token reduction
- 1.8× latency speedup
- 97.1% semantic accuracy maintained
- 65.6% energy reduction
```

## 🐛 Troubleshooting

### Setup Issues

```bash
# Check Rust
rustc --version

# Check PostgreSQL
psql --version

# Check disk space
df -h

# View setup logs
tail -100 setup.log
```

### Build Issues

```bash
# Clean rebuild
cargo clean
cargo build --release

# Update dependencies
cargo update

# Verbose output
RUST_BACKTRACE=1 cargo build --verbose
```

### Database Issues

```bash
# Test connection
psql -U semantix -d semantix -c "SELECT 1;"

# Check PostgreSQL status
sudo systemctl status postgresql  # Linux
brew services list | grep postgres  # macOS

# Reset database
make db-reset
```

## 📖 Paper Reference

This implementation corresponds to:

**"Learned Semantic Cost Models for Adaptive Token-Efficient Query Optimization in LLM-Native Relational Engines"**

VLDB 2026 NOVAS Workshop (2nd Workshop on Next-Generation Optimization for Vector-Augmented Systems)

### Key Equations Implemented

- **Eq. 1**: `C_sem(π, σ) = Σ_i [H(i | Σ^ctx(i)) + γ·delay + β·staleness]`
- **Eq. 3**: `H(i | Σ^ctx(i)) = Conditional semantic entropy`
- **Eq. 4**: `φ(NL) = (LogicalPlan, {c_1, ..., c_k})`
- **Alg. 1**: Adaptive Token Scheduling (Lagrangian relaxation)

## 🤝 Contributing

See `CONTRIBUTING.md` for:
- Code style guidelines
- Testing requirements
- Pull request process
- Development workflow

## 📞 Support

- **GitHub Issues**: Report bugs and request features
- **Email**: novas-workshop-2026@vldb.org
- **Documentation**: Read README.md and docs/

## 📄 License

Apache License 2.0 - See LICENSE file for details

## ✅ Verification Checklist

After setup, verify:

- [ ] `cargo build --release` completes successfully
- [ ] `psql -U semantix -d semantix -c "SELECT COUNT(*) FROM orders;"` returns count > 0
- [ ] `cargo test --all` passes
- [ ] `make benchmark` completes with metrics
- [ ] pgAdmin accessible at http://localhost:5050 (if using Docker)

## 🎉 Next Steps

1. **Explore Code**: Start with `src/lib.rs` and `src/semantic_anchors.rs`
2. **Run Tests**: `cargo test --all`
3. **Profile**: `make profiler`
4. **Benchmark**: `make benchmark`
5. **Modify**: Edit configuration in `semantix.toml`
6. **Deploy**: Use Docker for production deployment

## 📊 Repository Statistics

- **Files**: 40+
- **Rust Lines**: ~3,500
- **Documentation**: 5,000+ lines
- **Tests**: 50+ test cases
- **Binary Size**: ~15MB (release)
- **Build Time**: 2-5 minutes

## 🔗 Quick Links

- **Paper**: `VLDB_2026_NOVAS_Short_Paper.tex` (camera-ready)
- **README**: `README.md` (user guide)
- **Installation**: `INSTALL.md` (detailed steps)
- **Contributing**: `CONTRIBUTING.md` (developer guide)
- **Architecture**: `PROJECT_OVERVIEW.md` (system design)

---

**Welcome to SEMANTIX!** 🚀

Questions? Check the documentation or open an issue on GitHub.

Last Updated: June 2026
