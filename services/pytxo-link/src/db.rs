use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    let migrator = sqlx::migrate!("./migrations");
    // Migration history is evidence. Never delete a checksum mismatch and
    // silently replay changed SQL, especially across entitlement tables.
    migrator.run(&pool).await?;
    Ok(pool)
}
