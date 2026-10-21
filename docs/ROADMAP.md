# Ausz — OIDC Provider Delivery Roadmap

This document outlines a phased delivery plan for implementing the full OpenID Connect provider. All 17 open issues are mapped to phases based on dependency order and effort.

---

## Overview

**Project:** `ausz` — a Rust-based OIDC provider using axum + tokio.  
**Scope:** Full OIDC-compliant identity provider (authorization code flow, device flow, token introspection/revocation, JWKS, discovery).  
**Current state:** Bare axum skeleton — all work is ahead.

---

## Phase 1: Foundation (Week 1–2)

All other work depends on this phase.

| # | Issue | Deliverable |
|---|-------|-------------|
| **12** | Configuration layer for OIDC discovery and runtime settings | Config loading (defaults → file → env → CLI), discovery document generation, immutable `Config` struct |
| **17** | OAuth2 / OIDC standardized error formatting | `OAuth2Error` enum, `OAuth2ErrorFormatter` trait, correct HTTP status codes, JSON + redirect error formatting |
| **11** | Database persistence layer for OIDC entities | PostgreSQL + SQLite support, migrations for clients/tokens/auth codes/sessions, token hashing, connection pooling |
| **13** | Cryptographic backend: key management and JWT operations | `CryptoBackend` trait, `FileKeyStore` + `InMemoryKeyStore`, RS256/ES256 signing, JWKS document generation, key rotation |

**Deliverable:** Config loading, error types, DB schema + migrations, key management, JWT sign/verify.

---

## Phase 2: Core Utilities (Week 2–3)

Depends on Phase 1.

| # | Issue | Deliverable |
|---|-------|-------------|
| **14** | Client credential validation module | `ClientCredentialValidator` trait supporting `client_secret_post`, `client_secret_basic`, `none`, constant-time comparison |
| **16** | PKCE validation utility module | `PkceValidator` trait, S256 + plain challenge generation/verification, constant-time comparison |

**Deliverable:** Two deep utility modules with unit tests — client auth and PKCE.

---

## Phase 3: Core OIDC Flow (Week 3–5)

The minimum viable OIDC provider.

| # | Issue | Dependencies |
|---|-------|-------------|
| **2** | `/.well-known/openid-configuration` endpoint | Phase 1 (config) |
| **6** | `/jwks.json` endpoint | Phase 1 (crypto) |
| **3** | `/authorize` endpoint | Phases 1–2, plus #15 |
| **15** | Authentication flow: user login and session management | Phases 1–2 |
| **4** | `/token` endpoint | Phases 1–2, plus #15 |

**Deliverable:** Full authorization code flow with PKCE — a client can register, authenticate, and receive tokens. **This is the first usable milestone.**

---

## Phase 4: Extended Endpoints (Week 5–6)

All depend on Phase 3. Can be parallelized.

| # | Issue | Notes |
|---|-------|-------|
| **5** | `/userinfo` endpoint | Validates bearer token, returns scoped claims |
| **7** | `/revocation` endpoint | Token revocation per RFC 7009 |
| **8** | `/introspection` endpoint | Token introspection per RFC 7662 |
| **9** | `/device_authorization` endpoint | Device flow per RFC 8628 |

**Deliverable:** A fully spec-compliant OIDC provider covering core and most-used optional endpoints.

---

## Phase 5: Quality & Verification (Week 6–7)

| # | Issue | Notes |
|---|-------|-------|
| **18** | Integration tests for OIDC auth flows | 9 test suites (auth code, device, refresh, client auth, userinfo, JWKS, discovery, revocation, introspection), CI, deterministic |

**Deliverable:** Comprehensive integration test suite covering all flows, ready for CI.

---

## Dependency Graph

```
Phase 1              Phase 2           Phase 3              Phase 4              Phase 5
┌──────────┐        ┌──────────┐      ┌──────────┐        ┌──────────┐        ┌──────────┐
│  #12      │───────▶│  #14      │─────▶│  #2       │        │  #5       │        │  #18      │
│  Config   │        │  Client   │      │  Discovery│        │  UserInfo │        │  Integration│
├──────────┤        └─────┬────┘      ├──────────┤        ├──────────┤        └──────────┘
│  #17      │───────▶│  #16      │─────▶│  #6       │        │  #7       │
│  Errors   │        │  PKCE     │      │  JWKS     │        │  Revocation│
├──────────┤        └──────────┘      └─────┬────┘        ├──────────┤
│  #11      │                                │              ├──────────┤
│  Database │                                │              │  #8       │
├──────────┤                        ┌───────▼──────┐      │  Introspection│
│  #13      │─────────────────────▶│  #3 / #15    │─────▶│           │
│  Crypto   │   (sign/verify)      │  Auth Flow   │      └──────────┘
└──────────┘                       └───────┬──────┘
                                           │
                                    ┌──────▼──────┐
                                    │  #4         │
                                    │  /token     │
                                    └──────┬──────┘
                                           │
                                    ┌──────▼──────┐
                                    │  #9         │
                                    │  /device    │
                                    └─────────────┘
```

---

## Key Observations

1. **This is a big project** — 17 issues, all open, all detailed with interfaces and acceptance criteria. The spec writing is excellent.

2. **The critical path is through Phase 1 → Phase 3** — config, errors, DB, crypto, then the authorize → token flow. That's the MVP.

3. **Issue #10 is the parent tracking issue** — for all routes. It can be closed or repurposed as a meta-issue once implementation starts.

4. **Issue #18 (integration tests)** is listed last, but tests should be written incrementally alongside each phase rather than as a big bang at the end.

5. **Each issue has explicit dependency notes** — the ordering above follows what's already documented in the issues themselves.

---

## Suggested Starting Point

Begin with **Phase 1, issue #12 (Configuration layer)** — everything else reads from config. After that, #17 (errors) and #11 (DB) can proceed in parallel, with #13 (crypto) also parallelizable since it doesn't depend on DB or config.
