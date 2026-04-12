use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Public URL of the platform (for OAuth callbacks)
    #[serde(default = "default_public_url")]
    pub public_url: String,

    pub database: DatabaseConfig,
    pub spur: SpurConfig,
    pub auth: AuthConfig,

    #[serde(default)]
    pub server: ServerConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,
    #[serde(default = "default_session_namespace")]
    pub session_namespace: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            listen_addr: default_listen_addr(),
            session_namespace: default_session_namespace(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpurConfig {
    /// gRPC address of spurctld (e.g., "http://spurctld:6817")
    pub controller_addr: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    #[serde(default = "default_jwt_expiry_hours")]
    pub jwt_expiry_hours: u64,

    #[serde(default)]
    pub github: Option<GitHubAuthConfig>,
    #[serde(default)]
    pub okta: Option<OktaAuthConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubAuthConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OktaAuthConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Okta issuer URL (e.g., "https://mycompany.okta.com/oauth2/default")
    pub issuer: String,
    pub client_id: String,
    pub client_secret: String,
    /// Okta groups that map to admin role
    #[serde(default)]
    pub admin_groups: Vec<String>,
}

impl Config {
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn github_callback_url(&self) -> String {
        format!("{}/api/auth/github/callback", self.public_url)
    }

    pub fn okta_callback_url(&self) -> String {
        format!("{}/api/auth/okta/callback", self.public_url)
    }
}

fn default_public_url() -> String {
    "http://localhost:8080".into()
}

fn default_listen_addr() -> String {
    "0.0.0.0:8080".into()
}

fn default_session_namespace() -> String {
    "spur-sessions".into()
}

fn default_jwt_expiry_hours() -> u64 {
    24
}

fn default_true() -> bool {
    true
}
