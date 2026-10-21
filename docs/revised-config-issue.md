# Configuration layer for OIDC discovery and runtime settings

## Overview

The configuration layer drives the OIDC discovery document output, token lifetimes, feature flags, and runtime behavior. This issue defines how configuration is loaded, validated, and served.

**Key design decisions:**

- **Immutable after loading:** Configuration is loaded once at startup into an `Arc<ConfigState>` and shared across all handlers. No runtime mutation.
- **Separate mutable secrets:** `SecretStore` trait handles mutable secrets (keys, connection strings) with hot reload. Secrets are not part of the immutable config.
- **TOML for service config:** TOML is the Rust-native format. YAML is reserved for the deployment layer (Helm chart) later.
- **Single-tenant MVP:** Multi-tenant support is out of scope for MVP. The config design is extendable (see follow-up issue).
- **Canonical OIDC issuer:** `issuer` is the full base URL (per OIDC spec). No `issuer_path_prefix` — that's a proxy concern.

## Architecture

```
Config struct (serializable, default-able)
    │
    ▼
ConfigState (Arc<ConfigState>, loaded at startup, validated)
    │
    ▼
SecretStore (trait, mutable secrets with reload)
    │
    ▼
App (holds Arc<ConfigState> + Arc<dyn SecretStore>, shared across all handlers)
```

## Configuration schema

### Required / always-configurable

| Key | Env var | Default | Notes |
|-----|---------|---------|-------|
| `issuer` | `AUSZ_ISSUER` | `http://localhost:3000` | **Must** be set in production. OIDC spec requires `https` in production. Fast-fail on startup if invalid URL. |

### Feature flags (supported values — defaults are sensible)

| Key | Env var | Default | Notes |
|-----|---------|---------|-------|
| `response_types_supported` | `AUSZ_RESPONSE_TYPES` | `[Code, CodeIdToken, IdTokenToken, CodeToken]` | Subset of `SupportedResponseType` enum |
| `grant_types_supported` | `AUSZ_GRANT_TYPES` | `[AuthorizationCode, RefreshToken, ClientCredentials, DeviceCode]` | Subset of `SupportedGrantType` enum |
| `subject_types_supported` | `AUSZ_SUBJECT_TYPES` | `[Public]` | OIDC subject types |
| `id_token_signing_alg_values_supported` | `AUSZ_SIGNING_ALG` | `[RS256, ES256]` | Depends on what keys are loaded |
| `scopes_supported` | `AUSZ_SCOPES` | `[OpenId, Profile, Email, EmailVerified, Name, PreferredUsername, Nickname, Address, Phone, OfflineAccess]` | Subset of `SupportedScope` enum |
| `code_challenge_methods_supported` | `AUSZ_PKCE_METHODS` | `[S256]` | Plain is allowed by spec but MUST NOT be used in production |

### Token lifetime defaults

| Key | Env var | Default | Notes |
|-----|---------|---------|-------|
| `access_token_ttl` | `AUSZ_ACCESS_TOKEN_TTL` | `3600` (1 hour) | Seconds |
| `refresh_token_ttl` | `AUSZ_REFRESH_TOKEN_TTL` | `2592000` (30 days) | Seconds |
| `authorization_code_ttl` | `AUSZ_AUTH_CODE_TTL` | `600` (10 min) | Seconds |
| `device_code_ttl` | `AUSZ_DEVICE_CODE_TTL` | `300` (5 min) | Seconds |
| `device_poll_interval` | `AUSZ_DEVICE_POLL_INTERVAL` | `5` | Seconds |

### Secret configuration (separate from immutable config)

Secrets are loaded via the `SecretStore` trait and can be reloaded at runtime.

| Key | Env var | Default | Notes |
|-----|---------|---------|-------|
| `jwks_key_pair_file` | `AUSZ_JWKS_KEY_PAIR_FILE` | *(auto-generate on first run)* | Path to PEM key pair file. If not set, generates in-memory keys. |
| `jwks_key_id` | `AUSZ_JWKS_KEY_ID` | *(random UUID)* | `kid` for the key |
| `jwks_key_algorithm` | `AUSZ_JWKS_KEY_ALG` | `RS256` | Must match key type |
| `jwks_auto_rotate` | `AUSZ_JWKS_AUTO_ROTATE` | `false` | Auto-generate new keys on a schedule |

### Configuration file format (TOML)

```toml
issuer = "http://localhost:3000"

[feature_flags]
response_types_supported = ["code", "code id_token", "id_token token", "code token"]
grant_types_supported = ["authorization_code", "refresh_token", "client_credentials", "urn:ietf:params:oauth:grant-type:device_code"]
subject_types_supported = ["public"]
id_token_signing_alg_values_supported = ["RS256", "ES256"]
scopes_supported = ["openid", "profile", "email", "email_verified", "name", "preferred_username", "nickname", "address", "phone", "offline_access"]
code_challenge_methods_supported = ["S256"]

[token_lifetimes]
access_token_ttl = 3600
refresh_token_ttl = 2592000
authorization_code_ttl = 600
device_code_ttl = 300
device_poll_interval = 5

[jwks]
key_pair_file = "/etc/ausz/jwks.pem"
key_id = "ausz-dev-001"
key_algorithm = "RS256"
auto_rotate = false
```

## Interface

### Immutable config

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub issuer: String,
    pub feature_flags: FeatureFlags,
    pub token_lifetimes: TokenLifetimes,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FeatureFlags {
    pub response_types_supported: Vec<SupportedResponseType>,
    pub grant_types_supported: Vec<SupportedGrantType>,
    pub subject_types_supported: Vec<SupportedSubjectType>,
    pub id_token_signing_alg_values_supported: Vec<SigningAlgorithm>,
    pub scopes_supported: Vec<SupportedScope>,
    pub code_challenge_methods_supported: Vec<CodeChallengeMethod>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenLifetimes {
    pub access_token_ttl: Duration,
    pub refresh_token_ttl: Duration,
    pub authorization_code_ttl: Duration,
    pub device_code_ttl: Duration,
    pub device_poll_interval: Duration,
}

/// Immutable config state, shared via Arc across all handlers.
pub struct ConfigState {
    pub config: Config,
    pub discovery_document: DiscoveryDocument,  // pre-computed from config
}

impl ConfigState {
    pub fn load() -> Result<Self, ConfigError>;
    pub fn to_discovery_document(&self) -> &DiscoveryDocument;
}
```

### Secret store trait

```rust
#[async_trait]
pub trait SecretStore: Send + Sync {
    /// Get the current JWKS key pair (PEM bytes).
    fn get_jwks_key_pair(&self) -> Result<PemKeyPair, SecretError>;

    /// Get the current key ID (kid).
    fn get_key_id(&self) -> Result<String, SecretError>;

    /// Get the current key algorithm.
    fn get_key_algorithm(&self) -> Result<SigningAlgorithm, SecretError>;

    /// Check if auto-rotation is enabled.
    fn auto_rotate(&self) -> bool;

    /// Reload secrets from their source (file, env, etc.).
    /// Called periodically or on demand.
    async fn reload(&self) -> Result<(), SecretError>;

    /// Check if the secret store supports hot reload.
    fn supports_hot_reload(&self) -> bool;
}
```

### In-memory secret store (MVP)

```rust
pub struct InMemorySecretStore {
    key_pair: Arc<PemKeyPair>,
    key_id: String,
    key_algorithm: SigningAlgorithm,
    auto_rotate: bool,
}

#[async_trait]
impl SecretStore for InMemorySecretStore {
    // ... implementation
}
```

## Enum types (in-memory, extensible)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedScope {
    OpenId,
    Profile,
    Email,
    EmailVerified,
    Name,
    PreferredUsername,
    Nickname,
    Address,
    Phone,
    OfflineAccess,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedResponseType {
    Code,
    CodeIdToken,
    IdTokenToken,
    CodeToken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedGrantType {
    AuthorizationCode,
    RefreshToken,
    ClientCredentials,
    DeviceCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedSubjectType {
    Public,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SigningAlgorithm {
    RS256,
    ES256,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeChallengeMethod {
    S256,
    Plain,
}
```

## Discovery document

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}
```

## Implementation requirements

- [ ] Define `Config` struct with `#[derive(Default, Clone, Deserialize, Serialize)]`
- [ ] Define `ConfigState` wrapper with `Arc` sharing
- [ ] Implement `ConfigState::load()` that reads from TOML file or env vars
- [ ] Validate `issuer` on load (must be a valid URL)
- [ ] Fast-fail (panic) on `http://` in non-localhost environments
- [ ] Derive all endpoint URLs from `issuer` + convention paths
- [ ] Expose `ConfigState::to_discovery_document()` that returns pre-computed document
- [ ] Config file format: TOML (via `serde_toml` crate)
- [ ] Define in-memory enum types for scopes, response types, grant types, etc.
- [ ] Define `SecretStore` trait for mutable secrets
- [ ] Implement `InMemorySecretStore` for MVP
- [ ] Validate that `jwks_key_pair_file` exists and is a valid PEM when provided
- [ ] Log config source for each value at startup
- [ ] No runtime mutation of `ConfigState`
- [ ] Unit tests for config loading, validation, and discovery document generation

## Validation rules

| Rule | Action |
|------|--------|
| `issuer` is not a valid URL | Fast-fail (panic) on startup |
| `issuer` starts with `http://` (not `https://`) in production | Warning on startup, panic in CI |
| `issuer` is `localhost` in production | Warning on startup |
| `jwks_key_pair_file` doesn't exist | Fast-fail (panic) on startup |
| `jwks_key_pair_file` is not valid PEM | Fast-fail (panic) on startup |
| Enum values in config don't match known enum variants | Fast-fail (panic) on startup |

## Dependencies

- **#2** `/.well-known/openid-configuration` — discovery document pulls from config
- **#4** `/token` — token TTLs come from config
- **#9** `/device_authorization` — device code TTL and poll interval from config
- **#13** Crypto backend — key loading, algorithm selection via `SecretStore`
- **#11** Database — connection string from secret store

## Crates

| Crate | Purpose |
|-------|---------|
| `serde` | Serialization/deserialization |
| `serde_toml` | TOML format support |
| `url` | URL validation (not just parsing) |
| `uuid` | Key ID generation |
