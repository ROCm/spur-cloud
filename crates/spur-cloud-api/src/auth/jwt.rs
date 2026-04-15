use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::principal::Principal;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user ID
    pub email: String,
    pub username: String,
    pub admin: bool,
    pub exp: i64, // expiry (unix timestamp)
    pub iat: i64, // issued at
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

pub fn verify_token(secret: &str, token: &str) -> anyhow::Result<Principal> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    let user_id = Uuid::parse_str(&data.claims.sub)?;
    Ok(Principal {
        user_id,
        email: data.claims.email,
        username: data.claims.username,
        is_admin: data.claims.admin,
    })
}

#[cfg(test)]
mod tests {
    use super::{generate_token, verify_token};
    use crate::auth::principal::Principal;
    use uuid::Uuid;

    fn assert_principal(_: &Principal) {}

    #[test]
    fn verify_token_returns_principal_with_expected_claims() {
        let user_id = Uuid::new_v4();
        let token = generate_token(
            "test-secret",
            user_id,
            "user@example.com",
            "alice",
            true,
            1,
        )
        .unwrap();

        let principal = verify_token("test-secret", &token).unwrap();

        assert_principal(&principal);
        assert_eq!(principal.user_id, user_id);
        assert_eq!(principal.email, "user@example.com");
        assert_eq!(principal.username, "alice");
        assert!(principal.is_admin);
    }

    #[test]
    fn verify_token_rejects_wrong_secret() {
        let token = generate_token(
            "test-secret",
            Uuid::new_v4(),
            "user@example.com",
            "alice",
            false,
            1,
        )
        .unwrap();

        assert!(verify_token("wrong-secret", &token).is_err());
    }
}
