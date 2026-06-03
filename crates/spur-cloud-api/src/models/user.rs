// Copyright (c) 2026 Advanced Micro Devices, Inc. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password_hash: Option<String>,
    pub github_id: Option<i64>,
    pub okta_sub: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub spur_account: String,
    pub is_admin: bool,
    /// Per-user GPU quota. NULL = unlimited.
    pub max_gpus: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_admin: bool,
    pub spur_account: String,
    /// How this user authenticates: `github`, `okta`, or `local`.
    pub auth_provider: String,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

fn auth_provider_for(user: &User) -> String {
    if user.github_id.is_some() {
        "github".into()
    } else if user.okta_sub.is_some() {
        "okta".into()
    } else {
        "local".into()
    }
}

impl From<User> for UserProfile {
    fn from(u: User) -> Self {
        let auth_provider = auth_provider_for(&u);
        Self {
            id: u.id,
            email: u.email,
            username: u.username,
            display_name: u.display_name,
            avatar_url: u.avatar_url,
            is_admin: u.is_admin,
            spur_account: u.spur_account,
            auth_provider,
            last_login_at: u.last_login_at,
            created_at: u.created_at,
        }
    }
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct SshKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub public_key: String,
    pub fingerprint: String,
    pub created_at: DateTime<Utc>,
}
