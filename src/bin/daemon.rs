// SEMANTIX Daemon - Main server for semantic query optimization
use semantix::SemanticQueryOptimizer;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting SEMANTIX Query Optimizer Daemon");

    // Load configuration
    let config = semantix::config::SemanticixConfig::load()?;
    info!("Configuration loaded: {:?}", config);

    // Initialize optimizer
    let mut optimizer = SemanticQueryOptimizer::new(&config.database_url).await?;
    info!("Semantic Query Optimizer initialized");

    // Example queries for demonstration
    let test_queries = vec![
        "SELECT * FROM orders WHERE custkey = 1",
        "SELECT o_orderkey, o_custkey, o_totalprice FROM orders WHERE o_orderdate > '2023-01-01'",
        "SELECT c_custkey, c_name FROM customer WHERE c_nationkey = 5",
    ];

    for (i, query) in test_queries.iter().enumerate() {
        info!("Processing query {}: {}", i + 1, query);

        match optimizer.optimize_and_execute(query).await {
            Ok(result) => {
                let metrics = optimizer.get_metrics();
                info!(
                    "Execution complete: {} rows, {} tokens, {:.2}ms, accuracy: {:.2}%",
                    result.result_rows,
                    result.actual_tokens,
                    result.actual_latency_ms,
                    metrics.avg_semantic_accuracy * 100.0
                );
            }
            Err(e) => {
                info!("Query execution failed: {}", e);
            }
        }
    }

    // Print final metrics
    let final_metrics = optimizer.get_metrics();
    info!(
        "Final metrics: {} queries, {:.0} avg tokens, {:.2}ms avg latency",
        final_metrics.total_queries, final_metrics.avg_token_cost, final_metrics.avg_latency_ms
    );

    info!("SEMANTIX Daemon shutting down gracefully");
    Ok(())
}
