use std::fs;
use std::path::Path;

use url::Url;

use crate::config::DiscoveryDocument;

use super::Config;

/// Immutable configuration state, shared across all handlers via Arc.
/// Loaded once at startup, never mutated after.
#[derive(Debug, Clone)]
pub struct ConfigState {
    pub config: Config,
    pub discovery_document: DiscoveryDocument,
}

impl ConfigState {
    /// Load configuration from a TOML file or environment variables.
    ///
    /// Priority: config file values override env vars, which override defaults.
    /// Validation runs immediately — invalid config causes a panic.
    pub fn load(config_path: Option<&str>) -> Self {
        let config = match config_path {
            Some(path) => {
                let content = fs::read_to_string(path).unwrap_or_else(|e| {
                    panic!("failed to read config file '{path}': {e}");
                });
                toml::from_str(&content).unwrap_or_else(|e| {
                    panic!("failed to parse config file '{path}' as TOML: {e}");
                })
            }
            None => Config::default(),
        };

        // Merge environment variable overrides
        let config = Config::from_env(config);

        Self::validate_and_build(config)
    }

    /// Load configuration with explicit validation, useful for testing.
    pub fn load_with_validation(config: Config) -> Self {
        Self::validate_and_build(config)
    }

    /// Validate config and build the ConfigState.
    /// Panics on validation failure.
    fn validate_and_build(config: Config) -> Self {
        // Validate issuer URL
        let issuer = &config.issuer;
        let parsed = Url::parse(issuer).unwrap_or_else(|_| {
            panic!("invalid issuer URL '{issuer}': must be a valid URL");
        });

        // Check for http:// in non-localhost
        if parsed.scheme() == "http" {
            let host = parsed.host().map(|h| h.to_string()).unwrap_or_default();
            if host != "localhost" && host != "127.0.0.1" {
                panic!("issuer URL must use https:// in production, got: {issuer}");
            }
            eprintln!(
                "warning: issuer URL uses http:// (host={host}). Use https:// in production."
            );
        }

        // Validate JWKS key pair file if provided
        if !config.jwks.key_pair_file.is_empty()
            && !Path::new(&config.jwks.key_pair_file).exists()
        {
            panic!("JWKS key pair file not found: {}", config.jwks.key_pair_file);
        }

        // Log config at startup
        Self::log_config(&config);

        // Build discovery document from issuer
        let discovery_document = DiscoveryDocument::from_issuer(issuer)
            .expect("failed to build discovery document from issuer");

        Self {
            config,
            discovery_document,
        }
    }

    /// Get the discovery document (returns a reference to the pre-computed document).
    pub fn discovery_document(&self) -> &DiscoveryDocument {
        &self.discovery_document
    }

    /// Log the loaded configuration at startup.
    fn log_config(config: &Config) {
        println!("ausz config:");
        println!("  issuer: {}", config.issuer);
        println!(
            "  response_types: {:?}",
            config.feature_flags.response_types_supported
        );
        println!("  grant_types: {:?}", config.feature_flags.grant_types_supported);
        println!("  scopes: {:?}", config.feature_flags.scopes_supported);
        println!("  pkce_methods: {:?}", config.feature_flags.code_challenge_methods_supported);
        println!("  signing_algorithms: {:?}", config.feature_flags.id_token_signing_alg_values_supported);
        println!("  access_token_ttl: {}s", config.token_lifetimes.access_token_ttl);
        println!("  refresh_token_ttl: {}s", config.token_lifetimes.refresh_token_ttl);
        println!("  auth_code_ttl: {}s", config.token_lifetimes.authorization_code_ttl);
        println!("  device_code_ttl: {}s", config.token_lifetimes.device_code_ttl);
        if !config.jwks.key_pair_file.is_empty() {
            println!("  jwks_key_pair_file: {}", config.jwks.key_pair_file);
        } else {
            println!("  jwks_key_pair_file: (auto-generated)");
        }
        println!("  jwks_key_id: {}", config.jwks.key_id);
        println!("  jwks_auto_rotate: {}", config.jwks.auto_rotate);
    }
}

// ---------------------------------------------------------------------------
// Env var overrides
// ---------------------------------------------------------------------------

impl Config {
    /// Apply environment variable overrides to the config.
    /// Env vars take priority over config file values, which override defaults.
    fn from_env(mut self) -> Self {
        self.issuer = env_or(&self.issuer, "AUSZ_ISSUER");

        self.feature_flags.response_types_supported =
            env_vec_or(&self.feature_flags.response_types_supported, "AUSZ_RESPONSE_TYPES");
        self.feature_flags.grant_types_supported =
            env_vec_or(&self.feature_flags.grant_types_supported, "AUSZ_GRANT_TYPES");
        self.feature_flags.subject_types_supported =
            env_vec_or(&self.feature_flags.subject_types_supported, "AUSZ_SUBJECT_TYPES");
        self.feature_flags.id_token_signing_alg_values_supported =
            env_vec_or(
                &self.feature_flags.id_token_signing_alg_values_supported,
                "AUSZ_SIGNING_ALG",
            );
        self.feature_flags.scopes_supported =
            env_vec_or(&self.feature_flags.scopes_supported, "AUSZ_SCOPES");
        self.feature_flags.code_challenge_methods_supported =
            env_vec_or(
                &self.feature_flags.code_challenge_methods_supported,
                "AUSZ_PKCE_METHODS",
            );

        self.token_lifetimes.access_token_ttl =
            env_u64_or(self.token_lifetimes.access_token_ttl, "AUSZ_ACCESS_TOKEN_TTL");
        self.token_lifetimes.refresh_token_ttl =
            env_u64_or(self.token_lifetimes.refresh_token_ttl, "AUSZ_REFRESH_TOKEN_TTL");
        self.token_lifetimes.authorization_code_ttl =
            env_u64_or(
                self.token_lifetimes.authorization_code_ttl,
                "AUSZ_AUTH_CODE_TTL",
            );
        self.token_lifetimes.device_code_ttl =
            env_u64_or(self.token_lifetimes.device_code_ttl, "AUSZ_DEVICE_CODE_TTL");
        self.token_lifetimes.device_poll_interval =
            env_u64_or(
                self.token_lifetimes.device_poll_interval,
                "AUSZ_DEVICE_POLL_INTERVAL",
            );

        self.jwks.key_pair_file = env_or(&self.jwks.key_pair_file, "AUSZ_JWKS_KEY_PAIR_FILE");
        self.jwks.key_id = env_or(&self.jwks.key_id, "AUSZ_JWKS_KEY_ID");
        self.jwks.key_algorithm = env_or(&self.jwks.key_algorithm, "AUSZ_JWKS_KEY_ALG");

        if let Ok(v) = std::env::var("AUSZ_JWKS_AUTO_ROTATE") {
            self.jwks.auto_rotate = v == "true" || v == "1";
        }

        self
    }
}

// ---------------------------------------------------------------------------
// Helper functions for env var reading
// ---------------------------------------------------------------------------

fn env_or(default: &str, var: &str) -> String {
    std::env::var(var).unwrap_or_else(|_| default.to_string())
}

fn env_vec_or(default: &[String], var: &str) -> Vec<String> {
    match std::env::var(var) {
        Ok(v) => v.split(',').map(|s| s.trim().to_string()).collect(),
        Err(_) => default.to_vec(),
    }
}

fn env_u64_or(default: u64, var: &str) -> u64 {
    std::env::var(var)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::JwksConfig;

    #[test]
    fn test_load_default_config() {
        let state = ConfigState::load(None);
        assert_eq!(state.config.issuer, "http://localhost:3000");
        assert!(!state.discovery_document.issuer.is_empty());
    }

    #[test]
    fn test_discovery_document_from_issuer() {
        let doc = DiscoveryDocument::from_issuer("https://auth.example.com").unwrap();
        assert_eq!(doc.issuer, "https://auth.example.com");
        assert_eq!(
            doc.authorization_endpoint,
            "https://auth.example.com/authorize"
        );
        assert_eq!(doc.token_endpoint, "https://auth.example.com/token");
        assert_eq!(doc.jwks_uri, "https://auth.example.com/jwks.json");
    }

    #[test]
    fn test_discovery_document_with_path() {
        let doc =
            DiscoveryDocument::from_issuer("https://auth.example.com/auth").unwrap();
        assert_eq!(doc.issuer, "https://auth.example.com/auth");
        assert_eq!(
            doc.authorization_endpoint,
            "https://auth.example.com/auth/authorize"
        );
        assert_eq!(doc.token_endpoint, "https://auth.example.com/auth/token");
        assert_eq!(doc.jwks_uri, "https://auth.example.com/auth/jwks.json");
    }

    #[test]
    #[should_panic(expected = "invalid issuer URL")]
    fn test_invalid_issuer_panics() {
        let config = Config {
            issuer: "not-a-url".to_string(),
            ..Config::default()
        };
        ConfigState::load_with_validation(config);
    }

    #[test]
    #[should_panic(expected = "JWKS key pair file not found")]
    fn test_missing_key_file_panics() {
        let config = Config {
            issuer: "https://auth.example.com".to_string(),
            jwks: JwksConfig {
                key_pair_file: "/nonexistent/path/key.pem".to_string(),
                ..JwksConfig::default()
            },
            ..Config::default()
        };
        ConfigState::load_with_validation(config);
    }
}
