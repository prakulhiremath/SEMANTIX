// Adaptive Token Scheduler
// Implements Algorithm 1 from paper: constrained token allocation under latency bounds

use crate::semantic_anchors::LogicalPlanWithCosts;
use crate::errors::SemanticixError;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tracing::{debug, info, warn};

/// Scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub max_latency_ms: u32,
    pub latency_sigma: f32,         // Latency sensitivity
    pub alpha: f32,                 // Gradient step size
    pub convergence_threshold: f32,
    pub max_iterations: usize,
    pub enable_profiling: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_latency_ms: 50,
            latency_sigma: 0.1,
            alpha: 0.01,
            convergence_threshold: 0.001,
            max_iterations: 1000,
            enable_profiling: true,
        }
    }
}

/// Token allocation for an operator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAllocation {
    pub operator_id: usize,
    pub allocated_tokens: u32,
    pub min_tokens: u32,
    pub max_tokens: u32,
    pub expected_latency_ms: f32,
}

/// Schedule output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleOutput {
    pub allocations: Vec<TokenAllocation>,
    pub total_tokens: u32,
    pub estimated_total_latency_ms: f32,
    pub schedule_delays: Vec<f32>,
    pub context_staleness: Vec<f32>,
    pub converged: bool,
    pub iterations_used: usize,
}

/// Latency profile for an operator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyProfile {
    pub operator_id: usize,
    pub tokens_to_latency: Vec<(u32, f32)>, // (token_count, latency_ms)
}

/// Token Scheduler - implements Algorithm 1
/// minimize Σ_j c_j^allocated
/// subject to Σ_j latency(o_j, c_j^allocated) ≤ L_max
///            c_j^min ≤ c_j^allocated ≤ c_j^max
pub struct TokenScheduler {
    config: SchedulerConfig,
    latency_profiles: Vec<LatencyProfile>,
}

impl TokenScheduler {
    /// Create new token scheduler
    pub fn new(config: SchedulerConfig) -> Self {
        Self {
            config,
            latency_profiles: Vec::new(),
        }
    }

    /// Add latency profile for an operator
    pub fn add_latency_profile(&mut self, profile: LatencyProfile) {
        self.latency_profiles.push(profile);
    }

    /// Optimize token allocation under latency constraints
    /// Implements Algorithm 1: Lagrangian relaxation-based scheduling
    pub fn optimize(
        &self,
        plan: &LogicalPlanWithCosts,
        base_costs: &[u32],
    ) -> Result<ScheduleOutput> {
        debug!(
            "Optimizing token allocation for {} operators",
            plan.operators.len()
        );

        let num_operators = plan.operators.len();

        // Initialize allocation at base costs
        let mut allocations: Vec<u32> = base_costs.to_vec();
        let min_budgets: Vec<u32> = base_costs.iter().map(|c| (*c * 80) / 100).collect();
        let max_budgets: Vec<u32> = base_costs.iter().map(|c| (*c * 150) / 100).collect();

        // Lagrangian relaxation: Algorithm 1
        let mut lambda = 1.0f32; // Dual variable
        let mut iteration = 0usize;
        let mut converged = false;

        for _ in 0..self.config.max_iterations {
            iteration += 1;

            // Step 1: Optimize for each operator
            for j in 0..num_operators {
                let optimal_tokens = self.solve_operator_subproblem(
                    j,
                    lambda,
                    min_budgets[j],
                    max_budgets[j],
                );
                allocations[j] = optimal_tokens;
            }

            // Step 2: Compute total latency
            let total_latency = self.compute_total_latency(&allocations);

            // Step 3: Update dual variable (Equation 5 in paper)
            let duality_gap = (total_latency - self.config.max_latency_ms as f32).abs();
            
            if duality_gap < self.config.convergence_threshold {
                converged = true;
                info!("Scheduling converged in {} iterations", iteration);
                break;
            }

            // Gradient step: λ ← λ + α(L_total - L_max)
            let gradient = total_latency - self.config.max_latency_ms as f32;
            lambda += self.config.alpha * gradient;

            // Clamp lambda to positive range
            lambda = lambda.max(0.01);

            if iteration % 100 == 0 {
                debug!(
                    "Iteration {}: total_latency = {:.2}ms, lambda = {:.4}",
                    iteration, total_latency, lambda
                );
            }
        }

        // Construct schedule output
        let total_latency = self.compute_total_latency(&allocations);
        let schedule_delays = self.compute_schedule_delays(&allocations);
        let context_staleness = self.compute_staleness(&allocations);

        let token_allocations: Vec<TokenAllocation> = allocations
            .iter()
            .enumerate()
            .map(|(i, &tokens)| TokenAllocation {
                operator_id: i,
                allocated_tokens: tokens,
                min_tokens: min_budgets[i],
                max_tokens: max_budgets[i],
                expected_latency_ms: self.latency_for_operator(i, tokens),
            })
            .collect();

        let output = ScheduleOutput {
            allocations: token_allocations,
            total_tokens: allocations.iter().sum(),
            estimated_total_latency_ms: total_latency,
            schedule_delays,
            context_staleness,
            converged,
            iterations_used: iteration,
        };

        info!(
            "Schedule output: {} total tokens, {:.2}ms latency, converged={}",
            output.total_tokens, output.estimated_total_latency_ms, output.converged
        );

        Ok(output)
    }

    /// Solve subproblem for single operator
    /// minimize c + λ * latency(o_j, c)
    /// subject to c_min ≤ c ≤ c_max
    fn solve_operator_subproblem(
        &self,
        operator_id: usize,
        lambda: f32,
        c_min: u32,
        c_max: u32,
    ) -> u32 {
        // Ternary search or gradient descent
        // For simplicity: linear search through reasonable token range
        let mut best_cost = c_min;
        let mut best_objective = f32::INFINITY;

        for tokens in (c_min..=c_max).step_by(50) {
            let latency = self.latency_for_operator(operator_id, tokens);
            let objective = tokens as f32 + lambda * latency;

            if objective < best_objective {
                best_objective = objective;
                best_cost = tokens;
            }
        }

        best_cost
    }

    /// Get latency for operator with given token allocation
    /// Learned function from profiling: tokens → latency
    fn latency_for_operator(&self, operator_id: usize, tokens: u32) -> f32 {
        // In production: lookup in learned latency function
        // For now: simple model: latency ≈ base + overhead / tokens
        let base_latency = 1.0f32; // ms
        let overhead = 20.0f32;    // token-ms
        base_latency + overhead / (tokens as f32 + 1.0)
    }

    /// Compute total latency for allocation
    fn compute_total_latency(&self, allocations: &[u32]) -> f32 {
        allocations
            .iter()
            .enumerate()
            .map(|(i, &tokens)| self.latency_for_operator(i, tokens))
            .sum()
    }

    /// Compute schedule delays for each operator
    /// delay(o_j, σ) from Equation (1)
    fn compute_schedule_delays(&self, allocations: &[u32]) -> Vec<f32> {
        let mut delays = Vec::new();
        let total_tokens: u32 = allocations.iter().sum();

        for &tokens in allocations {
            // Delay proportional to token deficit from mean
            let mean_tokens = total_tokens as f32 / allocations.len() as f32;
            let delay = if tokens < mean_tokens as u32 {
                (mean_tokens - tokens as f32) / mean_tokens
            } else {
                0.0
            };
            delays.push(delay.min(1.0).max(0.0));
        }

        delays
    }

    /// Compute context staleness for each operator
    /// staleness(o_j, σ) from Equation (1)
    fn compute_staleness(&self, allocations: &[u32]) -> Vec<f32> {
        let mut staleness = Vec::new();

        for (i, &tokens) in allocations.iter().enumerate() {
            // Staleness increases with position in execution order
            // and decreases with available tokens (more tokens = more current context)
            let position_staleness = (i as f32) / (allocations.len() as f32 + 1.0);
            let token_freshness = (tokens as f32).min(1000.0) / 1000.0;
            let score = position_staleness * (1.0 - token_freshness);
            staleness.push(score);
        }

        staleness
    }

    /// Update scheduler with profiling data
    pub fn update_with_profile(&mut self, profile: LatencyProfile) {
        // Find and update existing profile, or add new one
        if let Some(idx) = self
            .latency_profiles
            .iter()
            .position(|p| p.operator_id == profile.operator_id)
        {
            self.latency_profiles[idx] = profile;
        } else {
            self.latency_profiles.push(profile);
        }

        debug!(
            "Updated latency profile for operator {}",
            profile.operator_id
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_creation() {
        let config = SchedulerConfig::default();
        let scheduler = TokenScheduler::new(config);
        assert_eq!(scheduler.config.max_latency_ms, 50);
    }

    #[test]
    fn test_schedule_optimization() {
        let config = SchedulerConfig::default();
        let scheduler = TokenScheduler::new(config);

        let plan = LogicalPlanWithCosts {
            operators: vec![],
            initial_costs: vec![],
            schema_info: Default::default(),
            parsed_predicates: vec![],
            confidence: 0.95,
        };

        let base_costs = vec![500u32, 300u32, 400u32];
        let result = scheduler.optimize(&plan, &base_costs);

        assert!(result.is_ok());
        let schedule = result.unwrap();
        assert_eq!(schedule.allocations.len(), 3);
    }

    #[test]
    fn test_schedule_delays() {
        let config = SchedulerConfig::default();
        let scheduler = TokenScheduler::new(config);

        let allocations = vec![100u32, 200u32, 150u32];
        let delays = scheduler.compute_schedule_delays(&allocations);

        assert_eq!(delays.len(), 3);
        assert!(delays.iter().all(|&d| d >= 0.0 && d <= 1.0));
    }
}
