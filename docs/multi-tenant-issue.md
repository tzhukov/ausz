# Multi-tenant support

## Overview

`ausz` is currently single-tenant: one issuer, one key pair, one everything. This issue tracks the design and implementation of multi-tenant support so that one `ausz` instance can serve multiple OIDC providers.

**Out of scope for MVP:** Multi-tenant is a follow-up. The config layer is designed to be extendable.

## Design questions

### 1. Tenant identification

| Approach | Pros | Cons |
|----------|------|------|
| **Subdomain** (`tenant1.auth.example.com`) | Clean, standard, easy to reason about | Requires wildcard DNS + TLS |
| **Path** (`auth.example.com/tenant1`) | Simple routing | Breaks OIDC issuer URL convention (issuer = full path) |
| **Header** (`X-Tenant-ID`) | Flexible | Not discoverable, breaks browser-based flows |

**Recommendation:** Subdomain-based. The `issuer` per tenant is `https://<tenant>.auth.example.com`. Each tenant gets its own routing path.

### 2. Key isolation

Each tenant needs its own key pair. Options:
- **Per-tenant PEM files** — loaded at startup, reloaded via `SecretStore`
- **Per-tenant keys in DB** — loaded on demand, cached in memory
- **Per-tenant keys in vault** — AWS KMS, GCP KMS, etc.

**Recommendation:** Per-tenant PEM files for MVP, vault later.

### 3. Database strategy

| Approach | Pros | Cons |
|----------|------|------|
| **Schema-per-tenant** | Full isolation, easy to drop a tenant | Schema migration per tenant |
| **Row-per-tenant** (tenant_id column) | Single DB, easy to manage | Cross-tenant queries, isolation concerns |
| **Database-per-tenant** | Maximum isolation | Operational complexity |

**Recommendation:** Row-per-tenant (`tenant_id` column on all tables) for MVP. Schema-per-tenant when tenants need full isolation.

### 4. Routing

All requests arrive at one `ausz` instance. The tenant is identified from the request:
1. Parse host header → extract tenant
2. Look up tenant config → load/refresh tenant config
3. Route to tenant's handlers with tenant-specific config

## Config impact

The current `Config` struct would become:

```rust
pub struct Config {
    pub tenants: Vec<TenantConfig>,
    pub default_tenant: String,
}

pub struct TenantConfig {
    pub id: String,
    pub issuer: String,
    pub jwks_key_pair_file: String,
    pub jwks_key_id: String,
    pub jwks_key_algorithm: SigningAlgorithm,
    pub feature_flags: FeatureFlags,
    pub token_lifetimes: TokenLifetimes,
}
```

## Implementation plan

### Phase A: Design (no code)
- [ ] Finalize tenant identification strategy (subdomain vs path)
- [ ] Design tenant config loading (file-based vs DB-backed)
- [ ] Design tenant-aware routing in axum
- [ ] Design DB schema changes (tenant_id column on all tables)
- [ ] Design key rotation per tenant

### Phase B: Config layer
- [ ] Extend `Config` to support multiple tenants
- [ ] Add `TenantConfig` struct
- [ ] Add tenant config loading from TOML (one file per tenant or one file with array)
- [ ] Add tenant discovery document generation

### Phase C: Routing layer
- [ ] Add tenant identification middleware (parse host header)
- [ ] Add tenant-aware router (route to tenant handlers)
- [ ] Add tenant config caching (in-memory, TTL-based)
- [ ] Add tenant config hot reload

### Phase D: DB layer
- [ ] Add `tenant_id` column to all tables
- [ ] Add tenant-scoped queries
- [ ] Add tenant-aware migrations

### Phase E: Crypto layer
- [ ] Multi-key `SecretStore` (one key per tenant)
- [ ] Tenant-aware key rotation
- [ ] Per-tenant JWKS document

## Dependencies

- **#11** Database — needs `tenant_id` on all tables
- **#13** Crypto backend — multi-key support
- **#2** Discovery — per-tenant discovery document
- **#12** Config — multi-tenant config (extend, don't rewrite)
