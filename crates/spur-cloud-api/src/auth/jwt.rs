use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,        // user ID
    pub email: String,
    pub username: String,
    pub admin: bool,
    pub exp: i64,           // expiry (unix timestamp)
    pub iat: i64,           // issued at
}

/// Identity extracted from a verified JWT. Available in route handlers.
#[derive(Debug, Clone)]
pub struct Identity {
    pub user_id: Uuid,
    pub email: String,
    pub username: String,
    pub is_admin: bool,
}

pub fn generate_token(
    secret: &str,
    user_id: Uuid,
    email: &str,
    username: &str,
    is_admin: bool,
    expiry_hours: u64,
) -> anyhow::Result<String> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        username: username.to_string(),
        admin: is_admin,
        exp: (now + Duration::hours(expiry_hours as i64)).timestamp(),
        iat: now.timestamp(),
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;
    Ok(token)
}

pub fn verify_token(secret: &str, token: &str) -> anyhow::Result<Identity> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    let user_id = Uuid::parse_str(&data.claims.sub)?;
    Ok(Identity {
        user_id,
        email: data.claims.email,
        username: data.claims.username,
        is_admin: data.claims.admin,
    })
}
