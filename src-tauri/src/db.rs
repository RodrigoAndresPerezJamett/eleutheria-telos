use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

const TRASH_TTL_SECS: i64 = 30 * 24 * 60 * 60; // 30 days

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

/// Background worker that purges soft-deleted rows older than 30 days.
/// Runs once at startup, then every hour. Safe to run concurrently with
/// the inline purge in trash_list_handler — both are idempotent DELETEs.
pub async fn start_trash_ttl_worker(db: SqlitePool) {
    loop {
        let cutoff = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
            - TRASH_TTL_SECS;

        let notes_result = sqlx::query(
            "DELETE FROM notes WHERE deleted_at IS NOT NULL AND deleted_at < ?",
        )
        .bind(cutoff)
        .execute(&db)
        .await;

        let clips_result = sqlx::query(
            "DELETE FROM clipboard WHERE deleted_at IS NOT NULL AND deleted_at < ?",
        )
        .bind(cutoff)
        .execute(&db)
        .await;

        match (notes_result, clips_result) {
            (Ok(n), Ok(c)) => {
                let total = n.rows_affected() + c.rows_affected();
                if total > 0 {
                    tracing::info!("Trash TTL: purged {total} expired items ({} notes, {} clipboard)", n.rows_affected(), c.rows_affected());
                }
            }
            (Err(e), _) | (_, Err(e)) => {
                tracing::error!("Trash TTL purge failed: {e}");
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}
