use axum::extract::State;
use axum::{http::StatusCode, response::IntoResponse, Extension, Json};
use serde::Deserialize;
use tracing::info;

use crate::auth::principal::Principal;
use crate::db::user_repo;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct SetQuotaRequest {
    pub email: String,
    /// GPU quota. null = unlimited.
    pub max_gpus: Option<i32>,
}

/// PUT /api/admin/users/quota — set per-user GPU quota (admin only)
pub async fn set_user_quota(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Json(req): Json<SetQuotaRequest>,
) -> impl IntoResponse {
    if !principal.is_admin {
        return (StatusCode::FORBIDDEN, "admin access required").into_response();
    }

    match user_repo::set_user_gpu_quota_by_email(&state.db, &req.email, req.max_gpus).await {
        Ok(true) => {
            info!(
                admin = %principal.email,
                target_email = %req.email,
                max_gpus = ?req.max_gpus,
                "GPU quota updated"
            );
            Json(serde_json::json!({
                "email": req.email,
                "max_gpus": req.max_gpus,
            }))
            .into_response()
        }
        Ok(false) => (StatusCode::NOT_FOUND, "user not found").into_response(),
        Err(e) => {
            tracing::error!("set quota failed: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "failed to set quota").into_response()
        }
    }
}

/// GET /api/admin/update-check — check for available updates (admin only)
pub async fn check_update(Extension(principal): Extension<Principal>) -> impl IntoResponse {
    if !principal.is_admin {
        return (StatusCode::FORBIDDEN, "admin access required").into_response();
    }

    match crate::update::check_for_update(env!("CARGO_PKG_VERSION")).await {
        Ok(result) => Json(serde_json::json!({
            "current_version": result.current_version,
            "latest_version": result.latest_tag,
            "update_available": result.update_available,
            "release_url": format!(
                "https://github.com/ROCm/spur-cloud/releases/tag/{}",
                result.latest_tag
            ),
        }))
        .into_response(),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("failed to check for updates: {e}"),
        )
            .into_response(),
    }
}
