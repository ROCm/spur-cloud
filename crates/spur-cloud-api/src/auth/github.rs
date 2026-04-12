use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use tracing::{error, info};

use crate::auth::jwt;
use crate::db::user_repo;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    pub code: String,
    pub state: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubTokenResponse {
    access_token: String,
}

#[derive(Debug, Deserialize)]
struct GitHubUser {
    id: i64,
    login: String,
    name: Option<String>,
    avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubEmail {
    email: String,
    primary: bool,
    verified: bool,
}

/// GET /api/auth/github — redirect to GitHub OAuth authorize
pub async fn github_authorize(State(state): State<AppState>) -> Response {
    let github = match &state.config.auth.github {
        Some(g) if g.enabled => g,
        _ => return (StatusCode::NOT_FOUND, "GitHub auth not configured").into_response(),
    };

    // Generate CSRF state token
    let csrf_state = crate::auth::oidc_common::generate_state();
    let callback = state.config.github_callback_url();
    let url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=user:email&state={}",
        github.client_id,
        urlencoding::encode(&callback),
        csrf_state,
    );

    // Set state in cookie for validation on callback
    let cookie = format!("oauth_state={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=600", csrf_state);
    let mut resp = Redirect::temporary(&url).into_response();
    resp.headers_mut().insert(
        axum::http::header::SET_COOKIE,
        cookie.parse().unwrap(),
    );
    resp
}

/// GET /api/auth/github/callback — exchange code for token, upsert user, issue JWT
pub async fn github_callback(
    State(state): State<AppState>,
    Query(params): Query<CallbackParams>,
    headers: axum::http::HeaderMap,
) -> Response {
    // Validate CSRF state
    if let Err(e) = crate::auth::oidc_common::validate_state(&headers, params.state.as_deref()) {
        return (StatusCode::BAD_REQUEST, e).into_response();
    }

    let github = match &state.config.auth.github {
        Some(g) if g.enabled => g,
        _ => return (StatusCode::NOT_FOUND, "GitHub auth not configured").into_response(),
    };

    // Exchange code for access token
    let client = reqwest::Client::new();
    let token_resp = client
        .post("https://github.com/login/oauth/access_token")
        .header("accept", "application/json")
        .form(&[
            ("client_id", github.client_id.as_str()),
            ("client_secret", github.client_secret.as_str()),
            ("code", params.code.as_str()),
        ])
        .send()
        .await;

    let token_resp = match token_resp {
        Ok(r) => r,
        Err(e) => {
            error!("GitHub token exchange failed: {e}");
            return (StatusCode::BAD_GATEWAY, "GitHub token exchange failed").into_response();
        }
    };

    let token_data: GitHubTokenResponse = match token_resp.json().await {
        Ok(d) => d,
        Err(e) => {
            error!("GitHub token parse failed: {e}");
            return (StatusCode::BAD_GATEWAY, "invalid GitHub token response").into_response();
        }
    };

    // Fetch user profile
    let user_resp = client
        .get("https://api.github.com/user")
        .header("authorization", format!("Bearer {}", token_data.access_token))
        .header("user-agent", "spur-cloud")
        .send()
        .await;

    let gh_user: GitHubUser = match user_resp {
        Ok(r) => match r.json().await {
            Ok(u) => u,
            Err(e) => {
                error!("GitHub user parse failed: {e}");
                return (StatusCode::BAD_GATEWAY, "invalid GitHub user response").into_response();
            }
        },
        Err(e) => {
            error!("GitHub user fetch failed: {e}");
            return (StatusCode::BAD_GATEWAY, "GitHub user fetch failed").into_response();
        }
    };

    // Fetch primary email
    let emails_resp = client
        .get("https://api.github.com/user/emails")
        .header("authorization", format!("Bearer {}", token_data.access_token))
        .header("user-agent", "spur-cloud")
        .send()
        .await;

    let email = match emails_resp {
        Ok(r) => {
            let emails: Vec<GitHubEmail> = r.json().await.unwrap_or_default();
            emails
                .into_iter()
                .find(|e| e.primary && e.verified)
                .map(|e| e.email)
                .unwrap_or_else(|| format!("{}@github.local", gh_user.login))
        }
        Err(_) => format!("{}@github.local", gh_user.login),
    };

    // Upsert user
    let user = match user_repo::upsert_github_user(
        &state.db,
        gh_user.id,
        &email,
        &gh_user.login,
        gh_user.name.as_deref(),
        gh_user.avatar_url.as_deref(),
    )
    .await
    {
        Ok(u) => u,
        Err(e) => {
            error!("GitHub user upsert failed: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "user creation failed").into_response();
        }
    };

    info!(user_id = %user.id, username = %user.username, "GitHub login");

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

    // Redirect to frontend with token in fragment
    let redirect_url = format!("{}/#/auth/callback?token={}", state.config.public_url, token);
    Redirect::temporary(&redirect_url).into_response()
}

// Simple URL encoding for the redirect URI
mod urlencoding {
    pub fn encode(s: &str) -> String {
        url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
    }
}
