use scylla::SessionBuilder;

pub async fn seed_database() -> Result<(), Box<dyn std::error::Error>> {
    let uri = "127.0.0.1:9042";
    let session = SessionBuilder::new().known_node(uri).build().await?;

    let seed_data_cql = "INSERT INTO messages_keyspace.messages (channel_id, bucket, message_id, author_id, content) VALUES (?, ?, ?, ?, ?);";
    let channel_id: i64 = 23;
    let bucket = 1;
    let author_id: i64 = 45;
    let content = "wow what a cool service";

    for i in 1..100 {
        let message_id: i64 = i;
        session
            .query(
                seed_data_cql,
                (channel_id, bucket, message_id, author_id, content),
            )
            .await?;
    }
    Ok(())
}
