// Error types
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SemanticixError {
    #[error("Semantic parsing error: {0}")]
    ParsingError(String),

    #[error("Cost estimation error: {0}")]
    CostError(String),

    #[error("Scheduling error: {0}")]
    SchedulingError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Model loading error: {0}")]
    ModelError(String),
}
