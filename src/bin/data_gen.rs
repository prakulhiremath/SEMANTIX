// Data Generator - Generate TPC-H dataset with semantic metadata
use tracing::info;
use std::fs::File;
use std::io::{BufWriter, Write};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Starting TPC-H Semantic Data Generator");

    // Generate sample semantic descriptions
    let semantic_templates = vec![
        "Customer {} ordered {} items for ${} on {} - {}",
        "Order {} from customer {}: {} products, total {}, status: {}",
        "Shipment {} sent to region {}, supplier {}, lineitem count: {}",
        "Part {} ({}): cost ${}, key supplier: {}",
        "Supplier {} in region {}: {} orders, {} shipments",
    ];

    // Generate orders with semantic annotations
    generate_semantic_orders(10000, &semantic_templates)?;
    info!("Generated 10,000 orders with semantic metadata");

    // Generate customers with semantic annotations
    generate_semantic_customers(1000, &semantic_templates)?;
    info!("Generated 1,000 customers with semantic metadata");

    // Generate parts with semantic annotations
    generate_semantic_parts(2000, &semantic_templates)?;
    info!("Generated 2,000 parts with semantic metadata");

    info!("Data generation complete");
    Ok(())
}

fn generate_semantic_orders(
    count: usize,
    templates: &[&str],
) -> anyhow::Result<()> {
    let file = File::create("tpch_orders_semantic.csv")?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "orderkey,custkey,totalprice,orderdate,semantic_desc")?;

    for i in 1..=count {
        let custkey = (i % 1000) + 1;
        let totalprice = ((i * 73) % 50000) as f32 / 100.0;
        let orderdate = format!("2023-{:02}-{:02}", (i % 12) + 1, (i % 28) + 1);

        let semantic = format!(
            "Order {} from customer {}: {} items, ${:.2}, placed on {}",
            i, custkey, (i % 5) + 1, totalprice, orderdate
        );

        writeln!(
            writer,
            "{},{},{:.2},{},\"{}\"",
            i, custkey, totalprice, orderdate, semantic
        )?;
    }

    Ok(())
}

fn generate_semantic_customers(
    count: usize,
    _templates: &[&str],
) -> anyhow::Result<()> {
    let file = File::create("tpch_customer_semantic.csv")?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "custkey,name,address,nationkey,semantic_desc")?;

    for i in 1..=count {
        let name = format!("Customer_{}", i);
        let address = format!("Address_{}", i);
        let nationkey = (i % 25) + 1;

        let semantic = format!(
            "Customer {} in nation {}: {} orders, high-value account",
            i, nationkey, (i % 50) + 10
        );

        writeln!(
            writer,
            "{},\"{}\",\"{}\",{},\"{}\"",
            i, name, address, nationkey, semantic
        )?;
    }

    Ok(())
}

fn generate_semantic_parts(
    count: usize,
    _templates: &[&str],
) -> anyhow::Result<()> {
    let file = File::create("tpch_part_semantic.csv")?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "partkey,name,retailprice,semantic_desc")?;

    for i in 1..=count {
        let name = format!("Part_{}", i);
        let retailprice = ((i * 17) % 5000) as f32 / 100.0;

        let semantic = format!(
            "Part {} ({}): retail ${:.2}, high-demand, supplier network: {}",
            i, name, retailprice, (i % 10) + 1
        );

        writeln!(
            writer,
            "{},\"{}\",{:.2},\"{}\"",
            i, name, retailprice, semantic
        )?;
    }

    Ok(())
}
