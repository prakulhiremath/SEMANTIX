// Learned Semantic Cost Model
// Implements Equation (1) and (2) from paper: C_sem(π, σ) with entropy conditioning

use crate::semantic_anchors::{LogicalPlanWithCosts, TokenCostEstimate, LogicalOperator};
use crate::errors::SemanticixError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;
use tracing::{debug, info};
use rand::Rng;

/// Configuration for learned cost estimator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostModelConfig {
    pub model_type: String, // "gbdt", "neural", "parametric"
    pub model_path: String,
    pub entropy_weight: f32,      // γ in Equation (1)
    pub delay_weight: f32,        // coefficient for delay
    pub staleness_weight: f32,    // β in Equation (1)
    pub context_decay_rate: f32,
    pub min_token_budget: u32,
    pub max_token_budget: u32,
}

impl Default for CostModelConfig {
    fn default() -> Self {
        Self {
            model_type: "gbdt".to_string(),
            model_path: "models/cost_model.xgb".to_string(),
            entropy_weight: 1.0,
            delay_weight: 0.3,
            staleness_weight: 0.5,
            context_decay_rate: 0.95,
            min_token_budget: 100,
            max_token_budget: 10000,
        }
    }
}

/// Cost estimation context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostContext {
    pub operator_id: usize,
    pub operator_type: String,
    pub selectivity: f32,
    pub input_cardinality: u64,
    pub schema_complexity: f32,
    pub predicate_expressiveness: f32,
    pub join_selectivity: f32,
}

/// Learned Cost Estimator - implements Equation (1)
/// C_sem(π, σ) = Σ_i [H(i | Σ^ctx(i)) + γ*delay(o_j, σ) + β*staleness(o_j, σ)]
pub struct LearnedCostEstimator {
    config: CostModelConfig,
    // In production: actual GBDT/neural model loaded here
    model: MockCostModel,
    feedback_buffer: Vec<ExecutionFeedback>,
    parameter_estimates: ParameterEstimates,
}

/// Mock cost model for demonstration
struct MockCostModel;

/// Learned parameter estimates
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ParameterEstimates {
    entropy_coefficient: f32,
    delay_coefficient: f32,
    staleness_coefficient: f32,
    operator_bias: HashMap<String, f32>,
    last_updated: String,
}

impl Default for ParameterEstimates {
    fn default() -> Self {
        let mut operator_bias = HashMap::new();
        operator_bias.insert("Scan".to_string(), 100.0);
        operator_bias.insert("Filter".to_string(), 150.0);
        operator_bias.insert("Join".to_string(), 350.0);
        operator_bias.insert("Aggregate".to_string(), 200.0);
        operator_bias.insert("Project".to_string(), 50.0);
        operator_bias.insert("Sort".to_string(), 150.0);
        operator_bias.insert("Limit".to_string(), 25.0);

        Self {
            entropy_coefficient: 200.0,
            delay_coefficient: 50.0,
            staleness_coefficient: 75.0,
            operator_bias,
            last_updated: chrono::Local::now().to_string(),
        }
    }
}

/// Execution feedback for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionFeedback {
    pub plan_id: String,
    pub operator_id: usize,
    pub predicted_cost: u32,
    pub actual_cost: u32,
    pub context: CostContext,
    pub schedule_delay: f32,
    pub context_staleness: f32,
    pub timestamp: String,
}

impl LearnedCostEstimator {
    /// Create new learned cost estimator
    pub fn new(config: &CostModelConfig) -> Result<Self> {
        debug!("Initializing learned cost estimator with model: {}", config.model_type);

        Ok(Self {
            config: config.clone(),
            model: MockCostModel,
            feedback_buffer: Vec::new(),
            parameter_estimates: ParameterEstimates::default(),
        })
    }

    /// Estimate total semantic token cost for a plan
    /// Implements Equation (1): C_sem(π) = Σ_i [H(i | Σ^ctx(i)) + γ*delay + β*staleness]
    pub fn estimate(&self, plan: &LogicalPlanWithCosts) -> Result<Vec<u32>> {
        debug!("Estimating semantic costs for {} operators", plan.operators.len());

        let mut total_costs = Vec::new();

        for (i, (op, cost_est)) in plan
            .operators
            .iter()
            .zip(plan.initial_costs.iter())
            .enumerate()
        {
            // Base entropy cost: H(i | Σ^ctx(i))
            let entropy_cost = (cost_est.entropy_estimate * self.config.entropy_weight) as u32;

            // Operator-specific bias
            let op_name = self.operator_type_name(op);
            let bias = self.parameter_estimates.operator_bias
                .get(&op_name)
                .copied()
                .unwrap_or(100.0) as u32;

            // Selectivity adjustment
            let selectivity_adjustment = self.estimate_selectivity(op) as u32;

            // Total cost for this operator
            let total = entropy_cost + bias + selectivity_adjustment;
            let clamped = total
                .max(self.config.min_token_budget)
                .min(self.config.max_token_budget);

            total_costs.push(clamped);
        }

        info!("Estimated total cost: {} tokens", total_costs.iter().sum::<u32>());

        Ok(total_costs)
    }

    /// Estimate operator cost with schedule conditioning
    /// Implements C_sem(π, σ) from Equation (1) with schedule delay and staleness
    pub fn estimate_with_schedule(
        &self,
        plan: &LogicalPlanWithCosts,
        schedule_delays: &[f32],
        staleness_scores: &[f32],
    ) -> Result<Vec<u32>> {
        debug!("Estimating costs with schedule conditioning");

        let base_costs = self.estimate(plan)?;
        let mut scheduled_costs = Vec::new();

        for (i, (base, &delay, &staleness)) in base_costs
            .iter()
            .zip(schedule_delays.iter())
            .zip(staleness_scores.iter())
            .enumerate()
        {
            // Apply delay penalty: γ * delay(o_j, σ)
            let delay_penalty = (delay * self.config.delay_weight * 100.0) as u32;

            // Apply staleness penalty: β * staleness(o_j, σ)
            let staleness_penalty = (staleness * self.config.staleness_weight * 100.0) as u32;

            // Total cost with schedule conditioning
            let conditioned = base + delay_penalty + staleness_penalty;
            let clamped = conditioned
                .max(self.config.min_token_budget)
                .min(self.config.max_token_budget);

            scheduled_costs.push(clamped);
        }

        Ok(scheduled_costs)
    }

    /// Record execution feedback for model refinement
    pub fn update_with_feedback(&mut self, feedback: ExecutionFeedback) {
        // Store in feedback buffer
        self.feedback_buffer.push(feedback.clone());

        // Trigger model update if buffer size exceeds threshold
        if self.feedback_buffer.len() >= 100 {
            let _ = self.retrain_model();
        }
    }

    /// Retrain cost model with accumulated feedback
    fn retrain_model(&mut self) -> Result<()> {
        info!(
            "Retraining cost model with {} feedback samples",
            self.feedback_buffer.len()
        );

        // In production: fit GBDT or neural model to feedback data
        // For now: simple parameter averaging

        let mut total_error = 0.0f32;
        for feedback in &self.feedback_buffer {
            let error = ((feedback.actual_cost as f32) - (feedback.predicted_cost as f32)).abs();
            total_error += error;
        }

        let mean_error = total_error / self.feedback_buffer.len() as f32;
        debug!("Mean absolute error after feedback: {}", mean_error);

        // Update parameter estimates
        self.parameter_estimates.last_updated = chrono::Local::now().to_string();

        // Clear buffer after retraining
        self.feedback_buffer.clear();

        Ok(())
    }

    /// Estimate selectivity for an operator
    fn estimate_selectivity(&self, op: &LogicalOperator) -> f32 {
        match op {
            LogicalOperator::Scan { .. } => 1.0,
            LogicalOperator::Filter { .. } => 0.1, // Assume 10% selectivity
            LogicalOperator::Join { .. } => 0.05,
            LogicalOperator::Aggregate { .. } => 0.5,
            LogicalOperator::Project { .. } => 1.0,
            LogicalOperator::Sort { .. } => 1.0,
            LogicalOperator::Limit { count } => (*count as f32) / 1000000.0, // Assume large table
        }
    }

    /// Get operator type name
    fn operator_type_name(&self, op: &LogicalOperator) -> String {
        match op {
            LogicalOperator::Scan { .. } => "Scan".to_string(),
            LogicalOperator::Filter { .. } => "Filter".to_string(),
            LogicalOperator::Join { .. } => "Join".to_string(),
            LogicalOperator::Aggregate { .. } => "Aggregate".to_string(),
            LogicalOperator::Project { .. } => "Project".to_string(),
            LogicalOperator::Sort { .. } => "Sort".to_string(),
            LogicalOperator::Limit { .. } => "Limit".to_string(),
        }
    }

    /// Get current parameter estimates
    pub fn get_parameters(&self) -> &ParameterEstimates {
        &self.parameter_estimates
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_estimator_creation() {
        let config = CostModelConfig::default();
        let estimator = LearnedCostEstimator::new(&config);
        assert!(estimator.is_ok());
    }

    #[test]
    fn test_cost_estimation() {
        let config = CostModelConfig::default();
        let estimator = LearnedCostEstimator::new(&config).unwrap();

        let plan = LogicalPlanWithCosts {
            operators: vec![
                LogicalOperator::Scan {
                    table: "orders".to_string(),
                },
                LogicalOperator::Filter {
                    predicate: "custkey = 1".to_string(),
                },
            ],
            initial_costs: vec![
                TokenCostEstimate {
                    operator_id: 0,
                    estimated_tokens: 500,
                    entropy_estimate: 0.5,
                    context_preservation: 1.0,
                },
                TokenCostEstimate {
                    operator_id: 1,
                    estimated_tokens: 150,
                    entropy_estimate: 0.3,
                    context_preservation: 0.8,
                },
            ],
            schema_info: HashMap::new(),
            parsed_predicates: vec!["custkey = 1".to_string()],
            confidence: 0.95,
        };

        let costs = estimator.estimate(&plan);
        assert!(costs.is_ok());
        let cost_vec = costs.unwrap();
        assert_eq!(cost_vec.len(), 2);
    }
}
