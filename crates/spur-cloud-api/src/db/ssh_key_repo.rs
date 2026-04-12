use sqlx::PgPool;
use uuid::Uuid;

use crate::models::user::SshKey;

pub async fn list_keys(pool: &PgPool, user_id: Uuid) -> sqlx::Result<Vec<SshKey>> {
    sqlx::query_as::<_, SshKey>(
        "SELECT * FROM ssh_keys WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn add_key(
    pool: &PgPool,
    user_id: Uuid,
    name: &str,
    public_key: &str,
    fingerprint: &str,
) -> sqlx::Result<SshKey> {
    sqlx::query_as::<_, SshKey>(
        r#"
        INSERT INTO ssh_keys (user_id, name, public_key, fingerprint)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(name)
    .bind(public_key)
    .bind(fingerprint)
    .fetch_one(pool)
    .await
}

pub async fn delete_key(pool: &PgPool, id: Uuid, user_id: Uuid) -> sqlx::Result<bool> {
    let result =
        sqlx::query("DELETE FROM ssh_keys WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn get_keys_for_user(pool: &PgPool, user_id: Uuid) -> sqlx::Result<Vec<SshKey>> {
    list_keys(pool, user_id).await
}
