use request_coalescing::seed_database;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    seed_database().await?;
    Ok(())
}