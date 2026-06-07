# SEMANTIX Makefile
# Convenient commands for development, testing, and deployment

.PHONY: help build clean test bench fmt lint doc setup install run benchmark profile docker

# Default target
help:
	@echo "SEMANTIX - Learned Semantic Cost Models for LLM-Native Relational Engines"
	@echo ""
	@echo "Available targets:"
	@echo ""
	@echo "Development:"
	@echo "  make build           Build project (debug mode)"
	@echo "  make build-release   Build project (optimized)"
	@echo "  make clean           Remove build artifacts"
	@echo "  make fmt             Format code with cargo fmt"
	@echo "  make lint            Check code with cargo clippy"
	@echo "  make doc             Generate documentation"
	@echo ""
	@echo "Testing:"
	@echo "  make test            Run all tests"
	@echo "  make test-unit       Run unit tests only"
	@echo "  make test-integration Run integration tests"
	@echo "  make coverage        Generate code coverage report"
	@echo ""
	@echo "Performance:"
	@echo "  make bench           Run benchmarks"
	@echo "  make profile         Profile system with flamegraph"
	@echo ""
	@echo "Deployment:"
	@echo "  make setup           Automated setup (requires sudo for PostgreSQL)"
	@echo "  make docker-build    Build Docker image"
	@echo "  make docker-run      Run with Docker Compose"
	@echo "  make docker-stop     Stop Docker containers"
	@echo ""
	@echo "Running:"
	@echo "  make run             Run SEMANTIX daemon"
	@echo "  make benchmark       Run benchmark suite"
	@echo "  make profiler        Run cost profiler"
	@echo "  make data-gen        Generate TPC-H test data"
	@echo ""

# Build targets
build:
	cargo build

build-release:
	cargo build --release

clean:
	cargo clean
	rm -rf target/
	rm -f tpch_*.csv
	rm -f *.log

# Code quality targets
fmt:
	cargo fmt

fmt-check:
	cargo fmt -- --check

lint:
	cargo clippy -- -D warnings

doc:
	cargo doc --no-deps --open

# Test targets
test:
	cargo test --all

test-unit:
	cargo test --lib

test-integration:
	cargo test --test '*'

test-verbose:
	cargo test -- --nocapture

coverage:
	cargo tarpaulin --out Html --exclude-files tests/

# Benchmark targets
bench:
	cargo bench --no-fail-fast

profile:
	@command -v flamegraph >/dev/null 2>&1 || cargo install flamegraph
	cargo flamegraph --bin benchmark
	@echo "View profile: flamegraph.svg"

# Deployment targets
setup:
	@chmod +x setup.sh
	./setup.sh

docker-build:
	docker build -f docker/Dockerfile -t semantix:latest .

docker-run:
	docker-compose -f docker/docker-compose.yml up -d
	@echo "Services starting. Check: docker ps"

docker-stop:
	docker-compose -f docker/docker-compose.yml down

docker-logs:
	docker-compose -f docker/docker-compose.yml logs -f

# Running targets
run:
	cargo run --release --bin semantix-daemon

run-debug:
	RUST_BACKTRACE=1 cargo run --bin semantix-daemon

benchmark:
	cargo run --release --bin benchmark

profiler:
	cargo run --release --bin cost-profiler

data-gen:
	cargo run --release --bin data-generator

# Database targets
db-create:
	createdb semantix

db-drop:
	dropdb semantix

db-reset:
	dropdb semantix 2>/dev/null || true
	createdb semantix
	psql -d semantix -f schema/tpch_schema.sql

db-load:
	cargo run --release --bin data-generator
	psql -d semantix -c "\COPY orders FROM 'tpch_orders_semantic.csv' CSV HEADER"
	psql -d semantix -c "\COPY customer FROM 'tpch_customer_semantic.csv' CSV HEADER"
	psql -d semantix -c "\COPY part FROM 'tpch_part_semantic.csv' CSV HEADER"

db-status:
	psql semantix -c "\dt"
	psql semantix -c "SELECT COUNT(*) as orders FROM orders; SELECT COUNT(*) as customers FROM customer; SELECT COUNT(*) as parts FROM part;"

# Development workflow
dev-setup: build-release db-reset data-gen profile
	@echo "Development environment ready!"

dev-test: fmt lint test
	@echo "All checks passed!"

dev-bench: build-release benchmark
	@echo "Benchmark complete!"

# Maintenance
update-deps:
	cargo update
	cargo outdated

check-security:
	cargo audit

check-all: fmt-check lint test coverage
	@echo "All checks passed!"

# CI/CD simulation
ci: clean build-release test lint coverage
	@echo "CI pipeline complete!"

# Help for targets
.PHONY: help-build help-test help-deploy
help-build:
	@echo "Build targets:"
	@echo "  build              - Debug build"
	@echo "  build-release      - Optimized release build"

help-test:
	@echo "Test targets:"
	@echo "  test               - Run all tests"
	@echo "  test-unit          - Run unit tests only"
	@echo "  test-integration   - Run integration tests"
	@echo "  coverage           - Generate coverage report"

help-deploy:
	@echo "Deployment targets:"
	@echo "  setup              - Automated setup (Linux/macOS)"
	@echo "  docker-build       - Build Docker image"
	@echo "  docker-run         - Start with Docker Compose"
