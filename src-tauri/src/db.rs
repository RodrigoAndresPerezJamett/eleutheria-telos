use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    // Ensure the DB file directory exists (project root)
    let db_url = "sqlite:../eleutheria.db?mode=rwc";

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;

    // Enable WAL mode and foreign keys for better concurrent read performance
    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA foreign_keys=ON").execute(&pool).await?;

    // Run embedded migrations from src-tauri/migrations/
    sqlx::migrate!("./migrations").run(&pool).await?;

    tracing::info!("SQLite connected and migrations applied");
    Ok(pool)
}
