use axum::http::HeaderMap;
use serde::Deserialize;
use tracing::debug;

/// OIDC Discovery document (subset of fields we need).
#[derive(Debug, Deserialize)]
pub struct OidcDiscovery {
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub jwks_uri: String,
    pub issuer: String,
}

/// Fetch OIDC discovery document from well-known endpoint.
pub async fn fetch_discovery(issuer: &str) -> anyhow::Result<OidcDiscovery> {
    let url = format!("{}/.well-known/openid-configuration", issuer.trim_end_matches('/'));
    debug!(url = %url, "fetching OIDC discovery");
    let resp = reqwest::get(&url).await?;
    let discovery: OidcDiscovery = resp.json().await?;
    Ok(discovery)
}

/// OIDC token response.
#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub id_token: Option<String>,
    pub token_type: String,
}

/// Exchange authorization code for tokens.
pub async fn exchange_code(
    token_endpoint: &str,
    client_id: &str,
    client_secret: &str,
    code: &str,
    redirect_uri: &str,
) -> anyhow::Result<TokenResponse> {
    let client = reqwest::Client::new();
    let resp = client
        .post(token_endpoint)
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("redirect_uri", redirect_uri),
        ])
        .send()
        .await?;

    let token: TokenResponse = resp.json().await?;
    Ok(token)
}

/// Decode an ID token WITHOUT cryptographic verification.
/// In production, you'd verify against the JWKS. For MVP, we trust the token
/// because it came directly from the token endpoint over TLS.
pub fn decode_id_token_unverified(id_token: &str) -> anyhow::Result<IdTokenClaims> {
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() != 3 {
        anyhow::bail!("invalid JWT format");
    }
    use base64::Engine;
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(parts[1])?;
    let claims: IdTokenClaims = serde_json::from_slice(&payload)?;
    Ok(claims)
}

#[derive(Debug, Deserialize)]
pub struct IdTokenClaims {
    pub sub: String,
    pub email: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "preferred_username")]
    pub preferred_username: Option<String>,
    pub groups: Option<Vec<String>>,
}

/// Generate a random CSRF state string for OAuth flows.
pub fn generate_state() -> String {
    use base64::Engine;
    let mut bytes = [0u8; 24];
    rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// Validate the OAuth state parameter against the cookie.
pub fn validate_state(headers: &HeaderMap, state: Option<&str>) -> Result<(), &'static str> {
    let state = match state {
        Some(s) if !s.is_empty() => s,
        _ => return Err("missing state parameter"),
    };

    // Extract oauth_state cookie
    let cookie_header = headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let cookie_state = cookie_header
        .split(';')
        .filter_map(|c| {
            let c = c.trim();
            c.strip_prefix("oauth_state=")
        })
        .next();

    match cookie_state {
        Some(cs) if cs == state => Ok(()),
        Some(_) => Err("state parameter mismatch"),
        None => Err("missing oauth_state cookie"),
    }
}
