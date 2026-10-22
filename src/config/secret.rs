use std::sync::Arc;

use async_trait::async_trait;

use crate::config::error::SecretError;

/// Trait for managing mutable secrets (keys, client credentials, etc.).
///
/// Secrets are separated from the immutable ConfigState because they may
/// need to change at runtime (e.g., key rotation, client credential updates).
#[async_trait]
pub trait SecretStore: Send + Sync {
    /// Get a PEM-encoded private key by key ID (kid).
    async fn get_key(&self, kid: &str) -> Result<String, SecretError>;

    /// Get the list of key IDs available in the store.
    async fn list_key_ids(&self) -> Result<Vec<String>, SecretError>;

    /// Rotate or add a new key pair. Returns the new kid.
    async fn rotate_key(&mut self) -> Result<String, SecretError>;
}

/// In-memory MVP implementation backed by a file or auto-generated keys.
///
/// This is not thread-safe by itself; wrap in Arc for sharing.
pub struct InMemorySecretStore {
    key_id: String,
    algorithm: String,
    private_key_pem: Option<String>,
}

impl InMemorySecretStore {
    /// Create a new InMemorySecretStore from a file or auto-generate keys.
    pub fn new(key_pair_file: &str, key_id: &str, algorithm: &str) -> Result<Self, SecretError> {
        let private_key_pem = if !key_pair_file.is_empty() {
            Some(std::fs::read_to_string(key_pair_file).map_err(|e| {
                SecretError::KeyNotFound(format!(
                    "Failed to read key file '{}': {}",
                    key_pair_file, e
                ))
            })?)
        } else {
            None
        };

        Ok(Self {
            key_id: key_id.to_string(),
            algorithm: algorithm.to_string(),
            private_key_pem,
        })
    }
}

#[async_trait]
impl SecretStore for InMemorySecretStore {
    async fn get_key(&self, kid: &str) -> Result<String, SecretError> {
        if kid != self.key_id {
            return Err(SecretError::KeyNotFound(kid.to_string()));
        }
        match &self.private_key_pem {
            Some(pem) => Ok(pem.clone()),
            None => Err(SecretError::KeyNotFound(
                "No key loaded (auto-generation not yet supported)".to_string(),
            )),
        }
    }

    async fn list_key_ids(&self) -> Result<Vec<String>, SecretError> {
        Ok(vec![self.key_id.clone()])
    }

    async fn rotate_key(&mut self) -> Result<String, SecretError> {
        Err(SecretError::NotSupported(
            "Key rotation not supported for in-memory store".to_string(),
        ))
    }
}

/// Convenience wrapper: Arc<dyn SecretStore>
pub type SecretStoreRef = Arc<dyn SecretStore>;
