use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::auth::jwt::verify_token;
use crate::state::AppState;

/// Axum middleware that extracts and verifies JWT from Authorization header.
/// On success, inserts `Principal` into request extensions.
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    // Accept token from Authorization header or ?token= query parameter (for WebSocket)
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    let query_token = request.uri().query().and_then(|q| {
        q.split('&')
            .find(|p| p.starts_with("token="))
            .map(|p| p[6..].to_string())
    });

    let token = match auth_header {
        Some(h) if h.starts_with("Bearer ") => h[7..].to_string(),
        _ => match query_token {
            Some(t) => t,
            None => {
                return (
                    StatusCode::UNAUTHORIZED,
                    "missing or invalid authorization header",
                )
                    .into_response();
            }
        },
    };
    let token = &token;

    match verify_token(&state.config.auth.jwt_secret, token) {
        Ok(principal) => {
            request.extensions_mut().insert(principal);
            next.run(request).await
        }
        Err(_) => (StatusCode::UNAUTHORIZED, "invalid or expired token").into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::auth_middleware;
    use crate::{
        auth::{jwt::generate_token, principal::Principal},
        config::{AuthConfig, Config, DatabaseConfig, ServerConfig, SpurConfig},
        state::AppState,
    };
    use axum::{
        body::{to_bytes, Body},
        http::{Request, Response, StatusCode},
        middleware,
        response::IntoResponse,
        routing::get,
        Extension, Router,
    };
    use kube::client::Body as KubeBody;
    use spur_proto::proto::slurm_controller_client::SlurmControllerClient;
    use sqlx::postgres::PgPoolOptions;
    use std::{convert::Infallible, sync::Arc};
    use tonic::transport::Channel;
    use tower::{service_fn, util::ServiceExt};
    use uuid::Uuid;

    async fn protected(Extension(principal): Extension<Principal>) -> impl IntoResponse {
        principal.user_id.to_string()
    }

    fn test_state(secret: &str) -> AppState {
        let db = PgPoolOptions::new()
            .connect_lazy("postgresql://postgres:postgres@localhost/test")
            .unwrap();
        let spur =
            SlurmControllerClient::new(Channel::from_static("http://localhost").connect_lazy());
        let kube = kube::Client::new(
            service_fn(|_request| async {
                Ok::<_, Infallible>(
                    Response::builder()
                        .status(StatusCode::OK)
                        .body(KubeBody::empty())
                        .unwrap(),
                )
            }),
            "default",
        );

        AppState {
            db,
            spur,
            kube: Some(kube),
            config: Arc::new(Config {
                public_url: "http://localhost:3000".into(),
                database: DatabaseConfig {
                    url: "postgresql://postgres:postgres@localhost/test".into(),
                },
                spur: SpurConfig {
                    controller_addr: "http://localhost:6817".into(),
                },
                auth: AuthConfig {
                    jwt_secret: secret.into(),
                    jwt_expiry_hours: 24,
                    github: None,
                    okta: None,
                },
                server: ServerConfig::default(),
                bare_metal: None,
                update: Default::default(),
            }),
        }
    }

    fn protected_router(secret: &str) -> Router {
        let state = test_state(secret);

        Router::new()
            .route("/protected", get(protected))
            .layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ))
            .with_state(state)
    }

    #[tokio::test]
    async fn middleware_rejects_missing_authorization_header() {
        let response = protected_router("test-secret")
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn middleware_inserts_principal_for_valid_token() {
        let user_id = Uuid::new_v4();
        let token =
            generate_token("test-secret", user_id, "user@example.com", "alice", true, 1).unwrap();

        let response = protected_router("test-secret")
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(std::str::from_utf8(&body).unwrap(), user_id.to_string());
    }
}
