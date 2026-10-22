use thiserror::Error;

/// Errors that can occur during config loading and validation.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to load config file: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to parse config as TOML: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("invalid issuer URL: {0}")]
    InvalidIssuer(String),

    #[error("JWKS key pair file not found: {0}")]
    KeyFileNotFound(String),

    #[error("JWKS key pair is not a valid PEM: {0}")]
    InvalidKeyPair(String),

    #[error("config validation failed: {0}")]
    Validation(String),
}

/// Errors that can occur during secret operations.
#[derive(Debug, Error)]
pub enum SecretError {
    #[error("key not found: {0}")]
    KeyNotFound(String),

    #[error("failed to load secret: {0}")]
    Load(#[from] std::io::Error),

    #[error("invalid secret format: {0}")]
    InvalidFormat(String),

    #[error("secret operation not supported: {0}")]
    NotSupported(String),
}
