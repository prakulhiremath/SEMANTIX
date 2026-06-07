// Configuration management
use crate::semantic_anchors::AnchorConfig;
use crate::cost_model::CostModelConfig;
use crate::scheduler::SchedulerConfig;
use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::Result;

/// Top-level SEMANTIX configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticixConfig {
    pub anchor_config: AnchorConfig,
    pub cost_model_config: CostModelConfig,
    pub scheduler_config: SchedulerConfig,
    pub database_url: String,
    pub log_level: String,
}

impl Default for SemanticixConfig {
    fn default() -> Self {
        Self {
            anchor_config: AnchorConfig::default(),
            cost_model_config: CostModelConfig::default(),
            scheduler_config: SchedulerConfig::default(),
            database_url: "postgresql://localhost/semantix".to_string(),
            log_level: "info".to_string(),
        }
    }
}

impl SemanticixConfig {
    /// Load configuration from file or environment
    pub fn load() -> Result<Self> {
        // Try to load from config file
        if let Ok(config_str) = fs::read_to_string("semantix.toml") {
            let config = toml::from_str(&config_str)?;
            return Ok(config);
        }

        // Fall back to environment variables
        if let Ok(config_str) = std::env::var("SEMANTIX_CONFIG") {
            let config = toml::from_str(&config_str)?;
            return Ok(config);
        }

        // Use defaults
        Ok(Self::default())
    }

    /// Save configuration to file
    pub fn save(&self, path: &str) -> Result<()> {
        let config_str = toml::to_string_pretty(self)?;
        fs::write(path, config_str)?;
        Ok(())
    }
}
