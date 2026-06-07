# SEMANTIX: Learned Semantic Cost Models for LLM-Native Relational Engines

**A production-ready implementation of the VLDB 2026 NOVAS Workshop paper.**

> Treating AI/LLM inference as a core database engine primitive with information-theoretic cost modeling.

## Overview

SEMANTIX implements a paradigm shift in relational query optimization by treating LLM inference as a first-class database primitive, coupled with learned semantic cost estimation. Current decoupled architectures silo LLM-based retrieval from cost-aware query planning, resulting in token waste, semantic misalignment, and unbounded latency.

### Key Contributions

1. **Formal Cost Architecture** (Equation 1): Unified cost model embedding semantic entropy, relational context preservation, and execution schedule conditioning
2. **Bidirectional Semantic Anchors** (Equation 4): Learned projections mapping NL intent to cost-parametric logical plans
3. **Adaptive Token Scheduling** (Algorithm 1): Dynamic token allocation under latency constraints using Lagrangian relaxation

### Performance Results

- **3.2× reduction in inference token cost** (vs. Classical PostgreSQL)
- **1.8× speedup in latency** (25.3ms vs 45.3ms)
- **97.1% semantic accuracy** maintained
- **65.6% energy reduction** compared to classical systems

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                  SEMANTIX Query Optimizer                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Phase 1: Semantic Parsing                                 │
│  ├─ NL Query → Bidirectional Semantic Anchor              │
│  └─ Output: LogicalPlan + Initial Cost Estimates          │
│                                                             │
│  Phase 2: Cost Refinement                                  │
│  ├─ Learned Cost Model (GBDT)                             │
│  └─ Output: Refined token cost estimates                   │
│                                                             │
│  Phase 3: Adaptive Token Scheduling                        │
│  ├─ Constrained Optimization (Lagrangian)                 │
│  └─ Output: Token allocation schedule                      │
│                                                             │
│  Phase 4: Execution + Feedback Loop                        │
│  ├─ Execute with schedule                                 │
│  └─ Update cost model with actual execution data          │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Installation

### Prerequisites

- Rust 1.70+ ([Install](https://rustup.rs/))
- PostgreSQL 14+ ([Install](https://www.postgresql.org/download/))
- Python 3.10+ (for data generation scripts)
- 8GB RAM, 4-core CPU minimum
- NVIDIA GPU (optional, for accelerated inference)

### Quick Start

#### 1. Clone Repository

```bash
git clone https://github.com/novas-workshop-2026/learned-semantic-costs.git
cd semantix
```

#### 2. Build Project

```bash
# Release build (optimized)
cargo build --release

# Or development build
cargo build
```

#### 3. Initialize Database

```bash
# Create PostgreSQL database
createdb semantix

# Initialize schema
psql -d semantix -f schema/tpch_schema.sql
```

#### 4. Generate Test Data

```bash
# Generate TPC-H with semantic annotations
cargo run --release --bin data-generator

# Load into PostgreSQL
psql -d semantix -c "COPY orders FROM 'tpch_orders_semantic.csv' CSV HEADER;"
```

#### 5. Profile System

```bash
# Profile operator latency for learned cost models
cargo run --release --bin cost-profiler
```

#### 6. Run Benchmark

```bash
# Execute comprehensive benchmark suite
cargo run --release --bin benchmark
```

## Usage

### Running the SEMANTIX Daemon

```bash
# Start optimizer service
cargo run --release --bin semantix-daemon
```

### Programmatic API

```rust
use semantix::SemanticQueryOptimizer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize optimizer
    let mut optimizer = SemanticQueryOptimizer::new(
        "postgresql://localhost/semantix"
    ).await?;

    // Execute query with full semantic optimization
    let result = optimizer.optimize_and_execute(
        "SELECT * FROM orders WHERE custkey = 1"
    ).await?;

    // Check metrics
    let metrics = optimizer.get_metrics();
    println!("Tokens: {}, Latency: {}ms, Accuracy: {:.2}%",
        metrics.avg_token_cost,
        metrics.avg_latency_ms,
        metrics.avg_semantic_accuracy * 100.0
    );

    // Provide feedback for continuous learning
    optimizer.feedback(&result.context);

    Ok(())
}
```

### Command-Line Interface

```bash
# Profile specific query
cargo run --release --bin benchmark -- --query "SELECT * FROM orders LIMIT 100"

# Generate data with custom size
cargo run --release --bin data-generator -- --scale-factor 10

# Profile operator latencies
cargo run --release --bin cost-profiler -- --operators "Scan,Filter,Join"
```

## Configuration

### Configuration File

Create `semantix.toml`:

```toml
[anchor_config]
encoder_model_path = "models/bert-encoder-semantic.bin"
decoder_model_path = "models/bert-decoder-semantic.bin"
max_sequence_length = 512
embedding_dim = 768
semantic_drift_threshold = 0.15

[cost_model_config]
model_type = "gbdt"
model_path = "models/cost_model.xgb"
entropy_weight = 1.0
delay_weight = 0.3
staleness_weight = 0.5
min_token_budget = 100
max_token_budget = 10000

[scheduler_config]
max_latency_ms = 50
latency_sigma = 0.1
alpha = 0.01
convergence_threshold = 0.001
max_iterations = 1000

[database]
url = "postgresql://localhost/semantix"
log_level = "info"
```

### Environment Variables

```bash
export DATABASE_URL="postgresql://user:password@localhost/semantix"
export LOG_LEVEL="debug"
export SEMANTIX_CONFIG="path/to/semantix.toml"
```

## Project Structure

```
semantix/
├── Cargo.toml                 # Rust dependencies
├── src/
│   ├── lib.rs               # Main library exports
│   ├── semantic_anchors.rs  # NL → LogicalPlan translation
│   ├── cost_model.rs        # Learned cost estimation
│   ├── scheduler.rs         # Adaptive token scheduling (Algorithm 1)
│   ├── database.rs          # PostgreSQL integration
│   ├── executor.rs          # Query execution engine
│   ├── metrics.rs           # Performance tracking
│   ├── config.rs            # Configuration management
│   ├── errors.rs            # Error types
│   └── bin/
│       ├── daemon.rs        # Main optimizer service
│       ├── profiler.rs      # Latency profiler
│       ├── data_gen.rs      # TPC-H data generation
│       └── benchmark.rs     # Performance evaluation
├── schema/
│   └── tpch_schema.sql      # PostgreSQL schema
├── tests/
│   ├── integration_tests.rs # End-to-end tests
│   └── unit_tests.rs        # Component tests
├── docker/
│   ├── Dockerfile           # Container image
│   └── docker-compose.yml   # Multi-container setup
└── README.md                # This file
```

## Mathematical Foundations

### Equation 1: Semantic Token Cost

```
C_sem(π, σ) = Σ_i [H(i | Σ^ctx(i)) + γ·delay(o_j, σ) + β·staleness(o_j, σ)]
```

where:
- `H(i | Σ^ctx(i))` = Conditional semantic entropy
- `γ` = Delay weight parameter
- `β` = Staleness weight parameter
- `σ` = Execution schedule

### Equation 4: Bidirectional Semantic Anchor

```
φ(NL query) = (LogicalPlan, {c_1, ..., c_k})
```

Maps natural language to plan with cost decorations.

### Algorithm 1: Adaptive Token Scheduling

Constrained optimization using Lagrangian relaxation:

```
minimize   Σ_j c_j^allocated
subject to Σ_j latency(o_j, c_j^allocated) ≤ L_max
           c_j^min ≤ c_j^allocated ≤ c_j^max
```

## Evaluation Methodology

### Metrics

1. **Inference Token Cost** (K tokens) - Total tokens consumed
2. **End-to-End Latency** (ms) - Query execution time
3. **Semantic Accuracy** (%) - Tuple relevance accuracy
4. **Compute Energy** (Wh) - GPU/CPU energy consumption

### Baselines

1. Classical PostgreSQL - Standard cost model
2. RAG-Optimized - LLM retrieval post-planning
3. Semantic Entropy - No schedule conditioning
4. SEMANTIX - Full system (this work)

### Benchmark Queries

Uses extended TPC-H queries with semantic metadata:
- Q1-Q5: Standard TPC-H with cardinality analysis
- Q6-Q10: Join-heavy queries with semantic predicates
- Q11-Q15: Aggregation-focused queries
- Q16-Q22: Complex multi-way joins

## Docker Deployment

### Build Image

```bash
docker build -f docker/Dockerfile -t semantix:latest .
```

### Run Container

```bash
docker run -it --rm \
  -e DATABASE_URL="postgresql://postgres:password@db:5432/semantix" \
  -p 8080:8080 \
  semantix:latest
```

### Docker Compose

```bash
docker-compose -f docker/docker-compose.yml up -d
```

## Performance Profiling

### CPU Profiling

```bash
cargo install flamegraph
cargo flamegraph --bin benchmark
# Open flamegraph.svg in browser
```

### Memory Profiling

```bash
valgrind --tool=massif ./target/release/benchmark
ms_print massif.out.<pid>
```

### Latency Profiling

```bash
cargo run --release --bin cost-profiler -- --detailed-report
```

## Testing

### Run All Tests

```bash
cargo test
cargo test --doc
cargo test --all-features
```

### Integration Tests

```bash
# Start PostgreSQL first
cargo test --test integration_tests -- --test-threads=1
```

### Benchmark Tests

```bash
cargo bench
```

## Contributing

We welcome contributions! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

See CONTRIBUTING.md for detailed guidelines.

## Citation

If you use SEMANTIX in your research, please cite:

```bibtex
@inproceedings{semantix2026,
  title={Learned Semantic Cost Models for Adaptive Token-Efficient 
         Query Optimization in LLM-Native Relational Engines},
  author={Prakul Sunil Hiremath},
  year={2026}
}
```

## License

Apache License 2.0 - See LICENSE file for details.

---

**Last Updated**: June 2026  
**Status**: Production-Ready v0.1.0  
**Maintainer**: VLDB 2026 NOVAS Workshop Contributors
