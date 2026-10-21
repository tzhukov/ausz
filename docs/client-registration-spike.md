# Spike: Client registration story

## Overview

Clients need to be registered with the OIDC provider before they can authenticate users. This spike investigates the client registration workflow, API design, and security model.

**Out of scope for MVP:** This is a spike to decide the client registration story before implementing.

## Design questions

### 1. Who can register clients?

| Option | Pros | Cons |
|--------|------|------|
| **Admin-only** | Simple, secure, clear ownership | Requires admin tooling |
| **Self-service** | Developer-friendly, scales | Security risk, abuse potential |
| **Hybrid** (admin + self-service) | Best of both | More complex |

**Recommendation:** Admin-only for MVP. Self-service can be added later with approval workflows.

### 2. Registration mechanism

| Option | Pros | Cons |
|--------|------|------|
| **Admin API** | RESTful, integratable, scriptable | Need to build it |
| **Admin CLI** | Fast for ops, no auth needed | Less integratable |
| **Config file** | Simple, version-controlled | No runtime updates, no audit trail |
| **Admin dashboard** | User-friendly, discoverable | Heavy lift, not MVP |

**Recommendation:** Admin API as primary, admin CLI as convenience. Both hit the same backend.

### 3. Client secret handling

| Option | Pros | Cons |
|--------|------|------|
| **System-generated** | Secure, no user input | User can't choose their own |
| **User-provided** | User control | Weak secrets possible |
| **Hybrid** (system generates, user can override) | Best of both | More complex |

**Recommendation:** System-generated with argon2 hash. Secret is returned once at creation (like an API key).

### 4. Client data model

```rust
pub struct RegisteredClient {
    pub id: UUID,
    pub client_id: String,        // public identifier
    pub client_secret_hash: String, // argon2 hash
    pub redirect_uris: Vec<String>,
    pub grant_types: Vec<SupportedGrantType>,
    pub response_types: Vec<SupportedResponseType>,
    pub scopes: Vec<SupportedScope>,
    pub token_endpoint_auth_method: ClientAuthMethod,
    pub pkce_required: bool,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub deleted_at: Option<DateTime>,  // soft delete
}
```

### 5. API design

```
POST /admin/clients
  → Creates a client
  → Returns client_id + client_secret (once)
  → Auth: admin API key or JWT

GET /admin/clients
  → Lists all clients (paginated)
  → Auth: admin API key or JWT

GET /admin/clients/{client_id}
  → Gets a client (secret not included)
  → Auth: admin API key or JWT

PUT /admin/clients/{client_id}
  → Updates a client (redirect_uris, grant_types, etc.)
  → Does NOT allow secret reset (requires delete + recreate)
  → Auth: admin API key or JWT

DELETE /admin/clients/{client_id}
  → Soft-deletes a client (revokes all tokens)
  → Auth: admin API key or JWT

POST /admin/clients/{client_id}/reset-secret
  → Generates a new client_secret
  → Hashes with argon2, replaces old hash
  → Auth: admin API key or JWT
```

### 6. Admin authentication

| Option | Pros | Cons |
|--------|------|------|
| **API key** | Simple, fast | Needs secure distribution |
| **JWT (from same provider)** | Consistent with OIDC | Chicken-and-egg: how does admin auth work? |
| **mTLS** | Very secure | Complex setup |
| **Magic link** | User-friendly | Slow, not for automation |

**Recommendation:** API key for MVP. JWT later once the provider is running. API key is stored in env (`AUSZ_ADMIN_API_KEY`) and hashed with argon2.

## Spike output

After this spike, we should have:
- [ ] Decision on admin-only vs self-service
- [ ] Decision on registration mechanism (API + CLI)
- [ ] API contract for client registration endpoints
- [ ] Decision on admin authentication (API key for MVP)
- [ ] Decision on client secret generation strategy
- [ ] Client data model (as above, with adjustments)

## Dependencies

- **#14** Client credential validation — reads client data from DB
- **#11** Database — clients table (already defined in #11)
- **#4** `/token` — uses client data for auth
- **#3** `/authorize` — uses client data for validation
