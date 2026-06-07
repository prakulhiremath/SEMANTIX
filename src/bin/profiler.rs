// Cost Profiler - Profile operator latency for learned cost models
use semantix::scheduler::{LatencyProfile, TokenScheduler, SchedulerConfig};
use std::time::Instant;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting SEMANTIX Cost Profiler");

    let config = SchedulerConfig::default();
    let mut scheduler = TokenScheduler::new(config);

    // Profile latency for different token allocations
    let operators = vec![
        ("Scan", 0),
        ("Filter", 1),
        ("Join", 2),
        ("Aggregate", 3),
    ];

    for (op_name, op_id) in operators {
        info!("Profiling operator: {}", op_name);

        let mut profile_points = Vec::new();

        // Test with increasing token allocations
        for tokens in (100..=5000).step_by(200) {
            let start = Instant::now();

            // Simulate operator execution
            // In production: execute actual operator
            std::thread::sleep(std::time::Duration::from_micros(100 + tokens as u64));

            let elapsed = start.elapsed().as_millis() as f32;
            profile_points.push((tokens as u32, elapsed));

            if tokens % 1000 == 0 {
                info!(
                    "  {} tokens: {:.3}ms latency",
                    tokens, elapsed
                );
            }
        }

        let profile = LatencyProfile {
            operator_id: op_id,
            tokens_to_latency: profile_points,
        };

        scheduler.add_latency_profile(profile);
    }

    info!("Profiling complete. Profiles saved to scheduler.");

    Ok(())
}
