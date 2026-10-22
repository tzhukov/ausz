pub mod error;
pub mod enums;

use std::time::Duration;

use url::Url;

use crate::config::enums::{
    CodeChallengeMethod, SigningAlgorithm, SupportedGrantType, SupportedResponseType,
    SupportedScope, SupportedSubjectType,
};
use crate::config::error::ConfigError;

// ---------------------------------------------------------------------------
// Config — the serializable root struct (maps to TOML)
// ---------------------------------------------------------------------------

/// Root configuration struct. Maps directly to the TOML config file.
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Config {
    pub issuer: String,
    pub feature_flags: FeatureFlags,
    pub token_lifetimes: TokenLifetimes,
    pub jwks: JwksConfig,
}

// ---------------------------------------------------------------------------
// FeatureFlags
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct FeatureFlags {
    #[serde(default = "default_response_types")]
    pub response_types_supported: Vec<String>,
    #[serde(default = "default_grant_types")]
    pub grant_types_supported: Vec<String>,
    #[serde(default = "default_subject_types")]
    pub subject_types_supported: Vec<String>,
    #[serde(default = "default_signing_algorithms")]
    pub id_token_signing_alg_values_supported: Vec<String>,
    #[serde(default = "default_scopes")]
    pub scopes_supported: Vec<String>,
    #[serde(default = "default_pkce_methods")]
    pub code_challenge_methods_supported: Vec<String>,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            response_types_supported: default_response_types(),
            grant_types_supported: default_grant_types(),
            subject_types_supported: default_subject_types(),
            id_token_signing_alg_values_supported: default_signing_algorithms(),
            scopes_supported: default_scopes(),
            code_challenge_methods_supported: default_pkce_methods(),
        }
    }
}

// ---------------------------------------------------------------------------
// TokenLifetimes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TokenLifetimes {
    #[serde(default = "default_access_token_ttl")]
    pub access_token_ttl: u64,
    #[serde(default = "default_refresh_token_ttl")]
    pub refresh_token_ttl: u64,
    #[serde(default = "default_authorization_code_ttl")]
    pub authorization_code_ttl: u64,
    #[serde(default = "default_device_code_ttl")]
    pub device_code_ttl: u64,
    #[serde(default = "default_device_poll_interval")]
    pub device_poll_interval: u64,
}

impl Default for TokenLifetimes {
    fn default() -> Self {
        Self {
            access_token_ttl: default_access_token_ttl(),
            refresh_token_ttl: default_refresh_token_ttl(),
            authorization_code_ttl: default_authorization_code_ttl(),
            device_code_ttl: default_device_code_ttl(),
            device_poll_interval: default_device_poll_interval(),
        }
    }
}

impl TokenLifetimes {
    pub fn access_token_ttl(&self) -> Duration {
        Duration::from_secs(self.access_token_ttl)
    }

    pub fn refresh_token_ttl(&self) -> Duration {
        Duration::from_secs(self.refresh_token_ttl)
    }

    pub fn authorization_code_ttl(&self) -> Duration {
        Duration::from_secs(self.authorization_code_ttl)
    }

    pub fn device_code_ttl(&self) -> Duration {
        Duration::from_secs(self.device_code_ttl)
    }

    pub fn device_poll_interval(&self) -> Duration {
        Duration::from_secs(self.device_poll_interval)
    }
}

// ---------------------------------------------------------------------------
// JwksConfig — mutable secrets (loaded via SecretStore, not part of immutable Config)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct JwksConfig {
    #[serde(default)]
    pub key_pair_file: String,
    #[serde(default = "default_key_id")]
    pub key_id: String,
    #[serde(default = "default_key_algorithm")]
    pub key_algorithm: String,
    #[serde(default)]
    pub auto_rotate: bool,
}

impl Default for JwksConfig {
    fn default() -> Self {
        Self {
            key_pair_file: String::new(),
            key_id: default_key_id(),
            key_algorithm: default_key_algorithm(),
            auto_rotate: false,
        }
    }
}

// ---------------------------------------------------------------------------
// DiscoveryDocument — pre-computed, derived from Config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiscoveryDocument {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub jwks_uri: String,
    pub revocation_endpoint: String,
    pub introspection_endpoint: String,
    pub device_authorization_endpoint: String,
    pub response_types_supported: Vec<String>,
    pub grant_types_supported: Vec<String>,
    pub subject_types_supported: Vec<String>,
    pub id_token_signing_alg_values_supported: Vec<String>,
    pub scopes_supported: Vec<String>,
    pub code_challenge_methods_supported: Vec<String>,
    pub authorization_response_iss_parameter_supported: bool,
}

impl DiscoveryDocument {
    /// Build the discovery document from the config's issuer URL.
    /// All endpoint URLs are derived from the issuer (OIDC spec pattern).
    pub fn from_issuer(issuer: &str) -> Result<Self, ConfigError> {
        let base = Url::parse(issuer).map_err(|_| ConfigError::InvalidIssuer(issuer.to_string()))?;

        // Helper: join a path segment to the base URL
        let join = |path: &str| -> String {
            let mut url = base.clone();
            url.set_path(&format!("{}{}", base.path(), path));
            url.to_string()
        };

        Ok(Self {
            issuer: issuer.to_string(),
            authorization_endpoint: join("/authorize"),
            token_endpoint: join("/token"),
            userinfo_endpoint: join("/userinfo"),
            jwks_uri: join("/jwks.json"),
            revocation_endpoint: join("/revocation"),
            introspection_endpoint: join("/introspection"),
            device_authorization_endpoint: join("/device_authorization"),
            response_types_supported: vec!["code".to_string()],
            grant_types_supported: vec![
                "authorization_code".to_string(),
                "refresh_token".to_string(),
                "client_credentials".to_string(),
                "urn:ietf:params:oauth:grant-type:device_code".to_string(),
            ],
            subject_types_supported: vec!["public".to_string()],
            id_token_signing_alg_values_supported: vec!["RS256".to_string(), "ES256".to_string()],
            scopes_supported: vec![
                "openid".to_string(),
                "profile".to_string(),
                "email".to_string(),
                "email_verified".to_string(),
                "name".to_string(),
                "preferred_username".to_string(),
                "nickname".to_string(),
                "address".to_string(),
                "phone".to_string(),
                "offline_access".to_string(),
            ],
            code_challenge_methods_supported: vec!["S256".to_string()],
            authorization_response_iss_parameter_supported: true,
        })
    }
}

// ---------------------------------------------------------------------------
// Default value helpers
// ---------------------------------------------------------------------------

fn default_response_types() -> Vec<String> {
    vec![
        "code".to_string(),
        "code id_token".to_string(),
        "id_token token".to_string(),
        "code token".to_string(),
    ]
}

fn default_grant_types() -> Vec<String> {
    vec![
        "authorization_code".to_string(),
        "refresh_token".to_string(),
        "client_credentials".to_string(),
        "urn:ietf:params:oauth:grant-type:device_code".to_string(),
    ]
}

fn default_subject_types() -> Vec<String> {
    vec!["public".to_string()]
}

fn default_signing_algorithms() -> Vec<String> {
    vec!["RS256".to_string(), "ES256".to_string()]
}

fn default_scopes() -> Vec<String> {
    vec![
        "openid".to_string(),
        "profile".to_string(),
        "email".to_string(),
        "email_verified".to_string(),
        "name".to_string(),
        "preferred_username".to_string(),
        "nickname".to_string(),
        "address".to_string(),
        "phone".to_string(),
        "offline_access".to_string(),
    ]
}

fn default_pkce_methods() -> Vec<String> {
    vec!["S256".to_string()]
}

fn default_access_token_ttl() -> u64 {
    3600
}

fn default_refresh_token_ttl() -> u64 {
    2592000
}

fn default_authorization_code_ttl() -> u64 {
    600
}

fn default_device_code_ttl() -> u64 {
    300
}

fn default_device_poll_interval() -> u64 {
    5
}

fn default_key_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn default_key_algorithm() -> String {
    "RS256".to_string()
}
