use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::principal::Principal;
use crate::db::{ssh_key_repo, user_repo};
use crate::models::user::UserProfile;
use crate::state::AppState;

/// GET /api/users/me — get current user profile
pub async fn get_profile(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
) -> impl IntoResponse {
    match user_repo::get_user_by_id(&state.db, principal.user_id).await {
        Ok(Some(user)) => {
            let profile: UserProfile = user.into();
            Json(profile).into_response()
        }
        Ok(None) => (StatusCode::NOT_FOUND, "user not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "failed").into_response(),
    }
}

#[derive(Deserialize)]
pub struct AddSshKeyRequest {
    pub name: String,
    pub public_key: String,
}

/// GET /api/users/me/ssh-keys — list SSH keys
pub async fn list_ssh_keys(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
) -> impl IntoResponse {
    match ssh_key_repo::list_keys(&state.db, principal.user_id).await {
        Ok(keys) => Json(keys).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "failed").into_response(),
    }
}

/// POST /api/users/me/ssh-keys — add SSH key
pub async fn add_ssh_key(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Json(req): Json<AddSshKeyRequest>,
) -> impl IntoResponse {
    // Validate SSH key format
    let parts: Vec<&str> = req.public_key.trim().split_whitespace().collect();
    if parts.len() < 2 {
        return (StatusCode::BAD_REQUEST, "invalid SSH public key format").into_response();
    }

    // Compute a simple fingerprint (SHA256 of the base64 key data)
    let fingerprint = compute_fingerprint(&req.public_key);

    match ssh_key_repo::add_key(
        &state.db,
        principal.user_id,
        &req.name,
        req.public_key.trim(),
        &fingerprint,
    )
    .await
    {
        Ok(key) => (StatusCode::CREATED, Json(key)).into_response(),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("duplicate") {
                (StatusCode::CONFLICT, "SSH key already exists").into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, "failed to add key").into_response()
            }
        }
    }
}

/// DELETE /api/users/me/ssh-keys/:id — delete SSH key
pub async fn delete_ssh_key(
    State(state): State<AppState>,
    Extension(principal): Extension<Principal>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match ssh_key_repo::delete_key(&state.db, id, principal.user_id).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, "key not found").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "failed").into_response(),
    }
}

fn compute_fingerprint(public_key: &str) -> String {
    use base64::Engine;
    use sha2::{Digest, Sha256};

    let parts: Vec<&str> = public_key.trim().split_whitespace().collect();
    if parts.len() >= 2 {
        if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(parts[1]) {
            let digest = Sha256::digest(&decoded);
            return format!(
                "SHA256:{}",
                base64::engine::general_purpose::STANDARD_NO_PAD.encode(&digest)
            );
        }
    }
    // Fallback: hash the whole key
    let digest = Sha256::digest(public_key.as_bytes());
    format!(
        "SHA256:{}",
        base64::engine::general_purpose::STANDARD_NO_PAD.encode(&digest)
    )
}
