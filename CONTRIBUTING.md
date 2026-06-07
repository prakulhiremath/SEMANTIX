# Contributing to SEMANTIX

Thank you for your interest in contributing to SEMANTIX! This document provides guidelines and instructions for contributing.

## Code of Conduct

We are committed to providing a welcoming and inspiring community for all. Please read and follow our Code of Conduct.

## How to Contribute

### Reporting Bugs

Before creating bug reports, check the issue list as you might find out that you don't need to create one.

**How Do I Submit A Good Bug Report?**

- Use a clear and descriptive title
- Describe the exact steps which reproduce the problem
- Provide specific examples to demonstrate the steps
- Describe the behavior you observed after following the steps
- Explain which behavior you expected to see instead and why
- Include screenshots if possible
- Include your system information (OS, Rust version, PostgreSQL version)

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, include:

- A clear and descriptive title
- A step-by-step description of the suggested enhancement
- Specific examples to demonstrate the steps
- A description of the current behavior and expected behavior
- Possible implementation approaches

## Development Setup

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- PostgreSQL 14+ ([install](https://www.postgresql.org/download/))
- Git

### Getting Started

1. **Fork and clone the repository**

```bash
git clone https://github.com/your-username/learned-semantic-costs.git
cd semantix
```

2. **Create a development branch**

```bash
git checkout -b feature/your-feature-name
```

3. **Set up development environment**

```bash
# Copy default config
cp semantix.toml.example semantix.toml

# Create and initialize database
createdb semantix_dev
psql -d semantix_dev -f schema/tpch_schema.sql

# Build and test
cargo build
cargo test
```

### Coding Standards

- **Format**: Run `cargo fmt` before committing
- **Linting**: Fix all clippy warnings with `cargo clippy`
- **Tests**: All new code must have corresponding tests
- **Documentation**: Document public APIs and complex logic
- **Comments**: Explain "why", not "what" - code should be self-documenting
- **Error Handling**: Use `Result<T>` and proper error types, avoid unwrap()

### Code Style Guide

```rust
// Good: Use type inference when possible
let optimizer = SemanticQueryOptimizer::new(db_url).await?;

// Good: Clear variable names
let semantic_entropy = H(intermediate_relation, context);

// Bad: Unclear naming
let x = compute_something();

// Good: Proper error handling
match parse_query(sql) {
    Ok(plan) => { /* ... */ },
    Err(e) => eprintln!("Parse error: {}", e),
}

// Bad: Panics in library code
let plan = parse_query(sql).unwrap();
```

### Testing Requirements

- **Unit Tests**: Place in `mod tests` at end of source file
- **Integration Tests**: Place in `tests/` directory
- **Test Coverage**: Aim for >80% coverage on new code
- **Performance Tests**: Use `criterion` for benchmarks

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_estimation() {
        let estimator = LearnedCostEstimator::new(&default_config()).unwrap();
        assert!(estimator.estimate(&test_plan).is_ok());
    }
}
```

### Documentation

- Update README.md for user-facing changes
- Add doc comments to public APIs
- Document complex algorithms with references to paper equations
- Update CHANGELOG.md with your changes

Example documentation:

```rust
/// Estimate semantic token cost for a query plan.
///
/// Implements Equation (1) from paper:
/// C_sem(π, σ) = Σ_i [H(i | Σ^ctx(i)) + γ·delay + β·staleness]
///
/// # Arguments
/// * `plan` - Logical query plan with cost decorations
/// * `schedule` - Execution schedule
///
/// # Returns
/// Vector of token costs per operator
///
/// # Errors
/// Returns `SemanticixError` if cost estimation fails
pub fn estimate_with_schedule(
    &self,
    plan: &LogicalPlanWithCosts,
    schedule: &ScheduleOutput,
) -> Result<Vec<u32>> {
    // Implementation...
}
```

### Commit Messages

Follow the conventional commits format:

```
<type>(<scope>): <subject>

<body>

<footer>
```

Examples:
```
feat(scheduler): implement Lagrangian relaxation algorithm

- Add Algorithm 1: Adaptive Token Scheduling
- Implement gradient-based dual variable update
- Add convergence testing

Fixes #123
Implements Equation (5) from paper
```

```
fix(cost_model): correct entropy estimation for joins

The entropy calculation was not conditioning on relational context.
This fix applies conditional entropy from Equation (3).
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

## Pull Request Process

1. **Before Creating PR**: Ensure your code passes all checks

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo doc --no-deps
```

2. **Create PR with clear description**

```markdown
## Description
Brief summary of changes

## Type of Change
- [x] Bug fix
- [x] New feature
- [x] Breaking change
- [x] Documentation

## Related Issues
Fixes #123

## Testing
- [x] Added tests for new functionality
- [x] Existing tests pass
- [x] Code coverage maintained (>80%)

## Checklist
- [x] Code formatted with `cargo fmt`
- [x] Clippy passes with no warnings
- [x] Documentation updated
- [x] Tests added/updated
```

3. **Respond to review feedback**

4. **Maintainers will merge when ready**

## Project Structure Guidelines

When adding features, follow existing structure:

```
src/
├── new_module.rs          # Implementation
├── lib.rs                 # Export in module list
└── tests/
    └── new_module_test.rs # Comprehensive tests

docs/
└── new_module.md          # User documentation
```

## Performance Considerations

- Profile before optimizing: `cargo flamegraph`
- Use benches for critical paths: `cargo bench`
- Test on realistic data sizes (TPC-H scale factor 10+)
- Document performance characteristics in docstrings

## Security Guidelines

- No hardcoded credentials
- Validate all external inputs
- Use constant-time comparisons for sensitive data
- Run `cargo audit` before submitting
- Report security issues privately (don't use public issues)

## Licensing

By contributing, you agree that your contributions will be licensed under the Apache License 2.0.

## Questions?

- Check existing issues and discussions
- Email: novas-workshop-2026@vldb.org
- Create an issue with `[QUESTION]` label

---

Thank you for contributing to SEMANTIX! 🚀
