use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::auth::jwt::Identity;
use crate::db::billing_repo;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct UsageParams {
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
}

/// GET /api/billing/usage — list usage records
pub async fn get_usage(
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(params): Query<UsageParams>,
) -> impl IntoResponse {
    match billing_repo::get_usage(&state.db, identity.user_id, params.since, params.until).await {
        Ok(records) => Json(records).into_response(),
        Err(e) => {
            tracing::error!("billing query failed: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to fetch usage").into_response()
        }
    }
}

/// GET /api/billing/summary — aggregated usage summary
pub async fn get_summary(
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(params): Query<UsageParams>,
) -> impl IntoResponse {
    match billing_repo::get_usage_summary(&state.db, identity.user_id, params.since).await {
        Ok(summary) => Json(summary).into_response(),
        Err(e) => {
            tracing::error!("billing summary failed: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to fetch summary").into_response()
        }
    }
}
