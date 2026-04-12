use axum::{extract::State, http::StatusCode, response::IntoResponse};

use crate::state::AppState;

/// GET /healthz — liveness probe (always OK if process is running)
pub async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

/// GET /readyz — readiness probe (checks DB and Spur connectivity)
pub async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    // Check DB
    if sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .is_err()
    {
        return (StatusCode::SERVICE_UNAVAILABLE, "database unreachable");
    }

    // Check Spur controller
    let mut spur = state.spur.clone();
    if spur_proto::proto::slurm_controller_client::SlurmControllerClient::get_nodes(
        &mut spur,
        spur_proto::proto::GetNodesRequest::default(),
    )
    .await
    .is_err()
    {
        return (StatusCode::SERVICE_UNAVAILABLE, "spur controller unreachable");
    }

    (StatusCode::OK, "ready")
}
