use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use tracing::error;

use crate::spur_client;
use crate::state::AppState;

/// GET /api/gpus — get GPU capacity across all nodes
pub async fn get_capacity(State(state): State<AppState>) -> impl IntoResponse {
    let mut spur = state.spur.clone();
    match spur_client::get_gpu_capacity(&mut spur).await {
        Ok(pools) => Json(pools).into_response(),
        Err(e) => {
            error!("GPU capacity fetch failed: {e}");
            (StatusCode::BAD_GATEWAY, "failed to fetch GPU capacity").into_response()
        }
    }
}
