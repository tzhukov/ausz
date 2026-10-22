# Config Layer (#12) — Implementation Plan

## Crate dependencies to add

| Crate | Purpose |
|-------|---------|
| `serde` + `serde_derive` | Serialization/deserialization |
| `serde_toml` | TOML format |
| `url` | URL validation |
| `chrono` | DateTime support |
| `uuid` | Key ID generation |
| `thiserror` | Error types |
| `async-trait` | SecretStore trait |
| `tokio` | Already there, for async secret reload |

## Module structure

```
src/
  config/
    mod.rs          — ConfigState, Config, FeatureFlags, TokenLifetimes, DiscoveryDocument
    enums.rs        — SupportedScope, SupportedResponseType, etc.
    error.rs        — ConfigError, SecretError
    secret.rs       — SecretStore trait, InMemorySecretStore, PemKeyPair
  config.rs         — re-exports
```

## Key implementation order

1. **Error types** — `ConfigError`, `SecretError`
2. **Enum types** — `SupportedScope`, `SupportedResponseType`, `SupportedGrantType`, `SupportedSubjectType`, `SigningAlgorithm`, `CodeChallengeMethod`
3. **Config structs** — `Config`, `FeatureFlags`, `TokenLifetimes` (serializable)
4. **DiscoveryDocument** — pre-computed from Config
5. **ConfigState** — wrapper with `load()`, validation, `Arc` sharing
6. **SecretStore trait** — `get_jwks_key_pair`, `get_key_id`, `get_key_algorithm`, `reload`
7. **InMemorySecretStore** — MVP implementation
8. **TOML example config** — `config.toml` in repo root
9. **Tests** — loading, validation, discovery doc generation

## Validation rules to implement

- `issuer` is a valid URL → fast-fail (panic)
- `issuer` is `https` in production → warning, panic in CI
- `issuer` is `localhost` in production → warning
- `jwks_key_pair_file` exists + valid PEM → fast-fail
- Enum values match known variants → fast-fail

## Acceptance criteria from the issue

- [ ] `Config` struct with `Default, Clone, Deserialize, Serialize`
- [ ] `ConfigState` wrapper with `Arc` sharing
- [ ] `ConfigState::load()` from TOML or env
- [ ] Validate `issuer` on load
- [ ] Derive all endpoint URLs from `issuer`
- [ ] `ConfigState::to_discovery_document()`
- [ ] TOML config file format
- [ ] In-memory enum types
- [ ] `SecretStore` trait
- [ ] `InMemorySecretStore` for MVP
- [ ] Validate `jwks_key_pair_file` exists + valid PEM
- [ ] Log config source at startup
- [ ] No runtime mutation of `ConfigState`
- [ ] Unit tests
