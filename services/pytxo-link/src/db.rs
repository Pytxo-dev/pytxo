use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    let migrator = sqlx::migrate!("./migrations");
    loop {
        match migrator.run(&pool).await {
            Ok(()) => break,
            Err(sqlx::migrate::MigrateError::VersionMismatch(version)) => {
                tracing::warn!(version, "migration checksum mismatch; repairing row and re-running");
                sqlx::query("DELETE FROM _sqlx_migrations WHERE version = $1")
                    .bind(version as i64)
                    .execute(&pool)
                    .await?;
            }
            Err(e) => return Err(e.into()),
        }
    }
    Ok(pool)
}
