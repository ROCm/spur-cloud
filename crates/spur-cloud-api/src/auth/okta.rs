use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use tracing::{error, info};

use crate::auth::{jwt, oidc_common};
use crate::db::user_repo;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    pub code: String,
    pub state: Option<String>,
}

/// GET /api/auth/okta — redirect to Okta OIDC authorize
pub async fn okta_authorize(State(state): State<AppState>) -> Response {
    let okta = match &state.config.auth.okta {
        Some(o) if o.enabled => o,
        _ => return (StatusCode::NOT_FOUND, "Okta auth not configured").into_response(),
    };

    let discovery = match oidc_common::fetch_discovery(&okta.issuer).await {
        Ok(d) => d,
        Err(e) => {
            error!("Okta discovery failed: {e}");
            return (StatusCode::BAD_GATEWAY, "Okta discovery failed").into_response();
        }
    };

    // Generate CSRF state token
    let csrf_state = crate::auth::oidc_common::generate_state();
    let callback = state.config.okta_callback_url();
    let url = format!(
        "{}?client_id={}&redirect_uri={}&response_type=code&scope=openid+email+profile+groups&state={}",
        discovery.authorization_endpoint,
        okta.client_id,
        url::form_urlencoded::byte_serialize(callback.as_bytes()).collect::<String>(),
        csrf_state,
    );

    let cookie = format!(
        "oauth_state={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=600",
        csrf_state
    );
    let mut resp = Redirect::temporary(&url).into_response();
    resp.headers_mut()
        .insert(axum::http::header::SET_COOKIE, cookie.parse().unwrap());
    resp
}

/// GET /api/auth/okta/callback — exchange code, validate ID token, upsert user, issue JWT
pub async fn okta_callback(
    State(state): State<AppState>,
    Query(params): Query<CallbackParams>,
    headers: axum::http::HeaderMap,
) -> Response {
    // Validate CSRF state
    if let Err(e) = crate::auth::oidc_common::validate_state(&headers, params.state.as_deref()) {
        return (StatusCode::BAD_REQUEST, e).into_response();
    }

    let okta = match &state.config.auth.okta {
        Some(o) if o.enabled => o,
        _ => return (StatusCode::NOT_FOUND, "Okta auth not configured").into_response(),
    };

    let discovery = match oidc_common::fetch_discovery(&okta.issuer).await {
        Ok(d) => d,
        Err(e) => {
            error!("Okta discovery failed: {e}");
            return (StatusCode::BAD_GATEWAY, "Okta discovery failed").into_response();
        }
    };

    let callback = state.config.okta_callback_url();

    // Exchange code for tokens
    let tokens = match oidc_common::exchange_code(
        &discovery.token_endpoint,
        &okta.client_id,
        &okta.client_secret,
        &params.code,
        &callback,
    )
    .await
    {
        Ok(t) => t,
        Err(e) => {
            error!("Okta token exchange failed: {e}");
            return (StatusCode::BAD_GATEWAY, "Okta token exchange failed").into_response();
        }
    };

    // Decode ID token
    let id_token = match &tokens.id_token {
        Some(t) => t,
        None => {
            return (StatusCode::BAD_GATEWAY, "no id_token in Okta response").into_response();
        }
    };

    let claims = match oidc_common::decode_id_token_unverified(id_token) {
        Ok(c) => c,
        Err(e) => {
            error!("Okta ID token decode failed: {e}");
            return (StatusCode::BAD_GATEWAY, "invalid ID token").into_response();
        }
    };

    let email = claims
        .email
        .unwrap_or_else(|| format!("{}@okta.local", claims.sub));
    let username = claims
        .preferred_username
        .unwrap_or_else(|| claims.sub.clone());
    let display_name = claims.name;

    // Check admin groups
    let is_admin = claims
        .groups
        .as_ref()
        .map(|groups| groups.iter().any(|g| okta.admin_groups.contains(g)))
        .unwrap_or(false);

    // Upsert user
    let user = match user_repo::upsert_okta_user(
        &state.db,
        &claims.sub,
        &email,
        &username,
        display_name.as_deref(),
        is_admin,
    )
    .await
    {
        Ok(u) => u,
        Err(e) => {
            error!("Okta user upsert failed: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "user creation failed").into_response();
        }
    };

    info!(user_id = %user.id, username = %user.username, "Okta login");

    // Issue platform JWT
    let token = match jwt::generate_token(
        &state.config.auth.jwt_secret,
        user.id,
        &user.email,
        &user.username,
        user.is_admin,
        state.config.auth.jwt_expiry_hours,
    ) {
        Ok(t) => t,
        Err(e) => {
            error!("JWT generation failed: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "token generation failed").into_response();
        }
    };

    let redirect_url = format!(
        "{}/#/auth/callback?token={}",
        state.config.public_url, token
    );
    Redirect::temporary(&redirect_url).into_response()
}
