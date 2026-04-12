use axum::{
    extract::{Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    Extension,
};
use uuid::Uuid;

use crate::auth::jwt::Identity;
use crate::db::session_repo;
use crate::state::AppState;
use crate::terminal::ws_handler;

/// GET /api/sessions/:id/terminal — upgrade to WebSocket for terminal access
pub async fn terminal_upgrade(
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Path(id): Path<Uuid>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    // Verify session belongs to user and is running
    let session = match session_repo::get_session_for_user(&state.db, id, identity.user_id).await {
        Ok(Some(s)) => s,
        Ok(None) => return (StatusCode::NOT_FOUND, "session not found").into_response(),
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "failed").into_response(),
    };

    if session.state != "running" {
        return (StatusCode::BAD_REQUEST, "session is not running").into_response();
    }

    let pod_name = match &session.pod_name {
        Some(p) => p.clone(),
        None => return (StatusCode::BAD_REQUEST, "session pod not ready").into_response(),
    };

    let namespace = state.config.server.session_namespace.clone();
    let kube_client = state.kube.clone();

    ws.on_upgrade(move |socket| {
        ws_handler::handle_terminal(socket, kube_client, namespace, pod_name)
    })
    .into_response()
}
