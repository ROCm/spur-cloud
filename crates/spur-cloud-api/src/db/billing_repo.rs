use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct UsageRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub gpu_type: String,
    pub gpu_count: i32,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub gpu_seconds: i64,
}

#[derive(Debug, Serialize)]
pub struct UsageSummary {
    pub total_gpu_seconds: i64,
    pub total_gpu_hours: f64,
    pub by_gpu_type: Vec<GpuTypeUsage>,
}

#[derive(Debug, Serialize)]
pub struct GpuTypeUsage {
    pub gpu_type: String,
    pub gpu_seconds: i64,
    pub gpu_hours: f64,
    pub session_count: i64,
}

/// Record that a session has started using GPU resources.
pub async fn record_usage_start(
    pool: &PgPool,
    user_id: Uuid,
    session_id: Uuid,
    gpu_type: &str,
    gpu_count: i32,
    start_time: DateTime<Utc>,
) -> sqlx::Result<()> {
    sqlx::query(
        r#"
        INSERT INTO usage_records (user_id, session_id, gpu_type, gpu_count, start_time)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(session_id)
    .bind(gpu_type)
    .bind(gpu_count)
    .bind(start_time)
    .execute(pool)
    .await?;
    Ok(())
}

/// Finalize usage record when a session ends. Computes gpu_seconds.
pub async fn record_usage_end(
    pool: &PgPool,
    session_id: Uuid,
    end_time: DateTime<Utc>,
) -> sqlx::Result<()> {
    sqlx::query(
        r#"
        UPDATE usage_records
        SET end_time = $2,
            gpu_seconds = EXTRACT(EPOCH FROM ($2 - start_time))::bigint * gpu_count
        WHERE session_id = $1 AND end_time IS NULL
        "#,
    )
    .bind(session_id)
    .bind(end_time)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get usage records for a user within a time range.
pub async fn get_usage(
    pool: &PgPool,
    user_id: Uuid,
    since: Option<DateTime<Utc>>,
    until: Option<DateTime<Utc>>,
) -> sqlx::Result<Vec<UsageRecord>> {
    let since = since.unwrap_or_else(|| Utc::now() - chrono::Duration::days(30));
    let until = until.unwrap_or_else(Utc::now);

    sqlx::query_as::<_, UsageRecord>(
        r#"
        SELECT * FROM usage_records
        WHERE user_id = $1 AND start_time >= $2 AND start_time <= $3
        ORDER BY start_time DESC
        "#,
    )
    .bind(user_id)
    .bind(since)
    .bind(until)
    .fetch_all(pool)
    .await
}

/// Get aggregated usage summary for a user.
pub async fn get_usage_summary(
    pool: &PgPool,
    user_id: Uuid,
    since: Option<DateTime<Utc>>,
) -> sqlx::Result<UsageSummary> {
    let since = since.unwrap_or_else(|| Utc::now() - chrono::Duration::days(30));

    #[derive(sqlx::FromRow)]
    struct Row {
        gpu_type: String,
        total_seconds: Option<i64>,
        session_count: Option<i64>,
    }

    let rows = sqlx::query_as::<_, Row>(
        r#"
        SELECT gpu_type,
               COALESCE(SUM(gpu_seconds), 0) as total_seconds,
               COUNT(*) as session_count
        FROM usage_records
        WHERE user_id = $1 AND start_time >= $2
        GROUP BY gpu_type
        "#,
    )
    .bind(user_id)
    .bind(since)
    .fetch_all(pool)
    .await?;

    let mut total_gpu_seconds: i64 = 0;
    let mut by_gpu_type = Vec::new();

    for row in rows {
        let secs = row.total_seconds.unwrap_or(0);
        total_gpu_seconds += secs;
        by_gpu_type.push(GpuTypeUsage {
            gpu_type: row.gpu_type,
            gpu_seconds: secs,
            gpu_hours: secs as f64 / 3600.0,
            session_count: row.session_count.unwrap_or(0),
        });
    }

    Ok(UsageSummary {
        total_gpu_seconds,
        total_gpu_hours: total_gpu_seconds as f64 / 3600.0,
        by_gpu_type,
    })
}
