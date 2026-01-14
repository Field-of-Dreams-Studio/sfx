# SFX Auth Endpoint Alignment Plan

## Overview

Align sfx's auth endpoints with questboard's `/auth/v1/` API pattern, supporting:
1. **Local inline auth** - sfx's minimal `LOCAL_AUTH` for local users
2. **External auth servers** - questboard-style servers via HTTP proxy
3. **Server selection at login** - user/app chooses which auth server
4. **Unified user ID** - `u128` type for both local (`u32`) and external (`UUID`)

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         SFX Application                          │
│                                                                   │
│   Login: { id: "user", password: "...", server: "..." }          │
│                              │                                    │
│              ┌───────────────┴───────────────┐                   │
│              ▼                               ▼                    │
│   ┌──────────────────────┐      ┌──────────────────────────────┐ │
│   │  server = "" (local)  │      │  server = "dept-a.local"    │ │
│   │                       │      │                              │ │
│   │  LOCAL_AUTH           │      │  HTTP Proxy to external      │ │
│   │  u128 from u32        │      │  POST {server}/auth/v1/...   │ │
│   └───────────┬───────────┘      └──────────────┬───────────────┘ │
│               │                                  │                │
│               ▼                                  ▼                │
│   ┌─────────────────────────────────────────────────────────────┐ │
│   │  Unified Session Storage                                     │ │
│   │                                                              │ │
│   │  UserRef { server: String, id: u128 }                       │ │
│   │  - Local:  "local@42"        (u32 as u128)                  │ │
│   │  - Remote: "dept.local@..."  (UUID as u128)                 │ │
│   │                                                              │ │
│   │  Token { value, server, user_id, expires }                  │ │
│   └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

**Key Design:**
- `u128` unifies both `u32` (local) and `UUID` (external)
- Same endpoints across all servers: `/auth/v1/login`, `/auth/v1/me`, etc.
- Server context stored with token for refresh/logout routing

---

## 1. Endpoint Migration

| Current | New | HTTP |
|---------|-----|------|
| `/auth/login` | `/auth/v1/login` | POST |
| `/auth/logout` | `/auth/v1/logout` | POST |
| `/auth/refresh` | `/auth/v1/refresh` | POST |
| `/users/me` | `/auth/v1/me` | GET |
| `/users/me/password` | `/auth/v1/change-password` | POST |
| `/users` | `/auth/v1/users` | POST |
| `/health` | `/auth/v1/health` | GET |

---

## 2. New Type: `UserRef` (unified user identification)

Create `src/local_auth/user_ref.rs`:

```rust
//! Unified user reference for cross-server identification.
//!
//! Format: "server@id" where id is u128 (hex for external, decimal for local)
//! Examples:
//! - "local@42" - local user with uid 42
//! - "dept-a.local@550e8400e29b41d4a716446655440000" - external UUID

/// Unified user reference across servers.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UserRef {
    pub server: String,  // "local" or server URL
    pub id: u128,
}

impl UserRef {
    /// Create a local user reference.
    pub fn local(uid: u32) -> Self {
        Self {
            server: "local".to_string(),
            id: uid as u128,
        }
    }

    /// Create an external user reference.
    pub fn external(server: impl Into<String>, uuid: u128) -> Self {
        Self {
            server: server.into(),
            id: uuid,
        }
    }

    /// Parse from string format "server@id".
    pub fn parse(s: &str) -> Option<Self> {
        let (server, id_str) = s.split_once('@')?;
        let id = if server == "local" {
            // Local IDs are decimal
            id_str.parse::<u32>().ok()? as u128
        } else {
            // External IDs are hex (UUID without dashes)
            u128::from_str_radix(id_str, 16).ok()?
        };
        Some(Self {
            server: server.to_string(),
            id,
        })
    }

    /// Check if this is a local user.
    pub fn is_local(&self) -> bool {
        self.server == "local"
    }

    /// Get as u32 (only valid for local users).
    pub fn as_local_uid(&self) -> Option<u32> {
        if self.is_local() && self.id <= u32::MAX as u128 {
            Some(self.id as u32)
        } else {
            None
        }
    }
}

impl std::fmt::Display for UserRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_local() {
            write!(f, "local@{}", self.id)
        } else {
            write!(f, "{}@{:032x}", self.server, self.id)
        }
    }
}
```

---

## 3. New Module: `src/local_auth/proxy.rs` (external auth proxy)

```rust
//! HTTP proxy for external auth servers.
//!
//! Routes auth requests to external questboard-style servers.
//! Uses reqwest for outbound HTTP calls.

use hotaru::prelude::*;

/// Simple HTTP client for external auth calls.
/// (Could use reqwest or hotaru's built-in client)
async fn http_post(url: &str, body: Value, auth_token: Option<&str>) -> Result<Value, String> {
    let client = reqwest::Client::new();
    let mut req = client.post(url)
        .header("Content-Type", "application/json")
        .body(body.into_json());

    if let Some(token) = auth_token {
        req = req.header("Authorization", format!("Bearer {}", token));
    }

    match req.send().await {
        Ok(resp) => {
            let text = resp.text().await.map_err(|e| e.to_string())?;
            Value::from_json(&text).map_err(|e| e.to_string())
        }
        Err(e) => Err(format!("Connection failed: {}", e)),
    }
}

async fn http_get(url: &str, auth_token: &str) -> Result<Value, String> {
    let client = reqwest::Client::new();
    let resp = client.get(url)
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    let text = resp.text().await.map_err(|e| e.to_string())?;
    Value::from_json(&text).map_err(|e| e.to_string())
}

/// Check if response is an error.
fn is_error(response: &Value) -> Option<String> {
    if response.try_get("error").is_ok() {
        Some(response.get("message").string())
    } else {
        None
    }
}

/// Proxy login to external auth server.
pub async fn proxy_login(server: &str, id: &str, password: &str) -> Result<Value, String> {
    let url = format!("{}/auth/v1/login", server);
    let body = object!({ id: id, password: password });

    let response = http_post(&url, body, None).await?;
    if let Some(msg) = is_error(&response) {
        Err(msg)
    } else {
        Ok(response)
    }
}

/// Proxy refresh to external auth server.
pub async fn proxy_refresh(server: &str, token: &str) -> Result<Value, String> {
    let url = format!("{}/auth/v1/refresh", server);
    let response = http_post(&url, object!({}), Some(token)).await?;
    if let Some(msg) = is_error(&response) {
        Err(msg)
    } else {
        Ok(response)
    }
}

/// Proxy logout to external auth server.
pub async fn proxy_logout(server: &str, token: &str) -> Result<(), String> {
    let url = format!("{}/auth/v1/logout", server);
    let response = http_post(&url, object!({}), Some(token)).await?;
    if let Some(msg) = is_error(&response) {
        Err(msg)
    } else {
        Ok(())
    }
}

/// Proxy get user info from external auth server.
pub async fn proxy_me(server: &str, token: &str) -> Result<Value, String> {
    let url = format!("{}/auth/v1/me", server);
    let response = http_get(&url, token).await?;
    if let Some(msg) = is_error(&response) {
        Err(msg)
    } else {
        Ok(response)
    }
}

/// Proxy introspect to external auth server.
pub async fn proxy_introspect(server: &str, token: &str) -> Result<Value, String> {
    let url = format!("{}/auth/v1/introspect", server);
    let body = object!({ token: token });
    http_post(&url, body, None).await
}
```

---

## 4. New File: `src/local_auth/response.rs`

```rust
//! Standardized error responses aligned with questboard format.
//!
//! Error format: { "error": "<category>", "reason": "<code>", "message": "<text>" }

use hotaru::prelude::*;
use hotaru::http::*;
use super::fop::FopError;

/// Create a structured error response.
pub fn error_response(error: &str, reason: &str, message: &str, status: u16) -> HttpResponse {
    akari_json!({
        error: error,
        reason: reason,
        message: message
    }).status(status)
}

/// Convert FopError to structured HTTP response.
pub fn from_fop_error(err: FopError) -> HttpResponse {
    match err {
        FopError::TokenInvalid => error_response(
            "unauthorized",
            "token_invalid",
            "Token is invalid or expired",
            401
        ),
        FopError::PasswordMismatch => error_response(
            "unauthorized",
            "invalid_credentials",
            "Invalid credentials",
            401
        ),
        FopError::UserNotFound => error_response(
            "not_found",
            "user_not_found",
            "User not found",
            404
        ),
        FopError::UserNameNotValid => error_response(
            "validation",
            "invalid_username",
            "Username is not valid",
            400
        ),
        FopError::EmailNotValid => error_response(
            "validation",
            "invalid_email",
            "Email is not valid",
            400
        ),
        FopError::TooManyRequest => error_response(
            "rate_limit",
            "too_many_requests",
            "Too many requests",
            429
        ),
        FopError::UserTooBig => error_response(
            "server_error",
            "internal_error",
            "Internal server error",
            500
        ),
        FopError::Other(msg) => error_response(
            "server_error",
            "unknown_error",
            &msg,
            500
        ),
    }
}

/// Unauthorized error (missing/invalid auth header).
pub fn unauthorized(reason: &str, message: &str) -> HttpResponse {
    error_response("unauthorized", reason, message, 401)
}

/// Forbidden error (authenticated but not allowed).
pub fn forbidden(reason: &str, message: &str) -> HttpResponse {
    error_response("forbidden", reason, message, 403)
}

/// Validation error.
pub fn validation_error(reason: &str, message: &str) -> HttpResponse {
    error_response("validation", reason, message, 400)
}

/// Method not allowed error.
pub fn method_not_allowed() -> HttpResponse {
    error_response("method_error", "method_not_allowed", "Method not allowed", 405)
}
```

---

## 3. Update `src/local_auth/mod.rs`

```rust
pub mod fop;
pub mod endpoints;
pub mod analyze;
pub mod response;   // Error response helpers
pub mod user_ref;   // UserRef type for cross-server IDs
pub mod proxy;      // External auth server proxy

use std::time::Duration;
use once_cell::sync::Lazy;

pub static LOCAL_AUTH: Lazy<fop::AuthManager> =
    Lazy::new(|| fop::AuthManager::new("programfiles/local_auth/users", Duration::from_secs(180)));
```

---

## 4. Update `src/local_auth/endpoints.rs` (Hotaru + Multi-Server)

```rust
use hotaru::prelude::*;
use hotaru::http::*;
use crate::op::APP;
use super::analyze::get_auth_token;
use super::response::{from_fop_error, unauthorized, forbidden, method_not_allowed, error_response};
use super::proxy;
use super::user_ref::UserRef;
use super::LOCAL_AUTH;
use crate::admin::check_is_admin;

endpoint! {
    APP.url("/auth/v1/users"),

    /// POST - Register new user (admin only, local only)
    pub create_user <HTTP> {
        if req.method() != POST {
            return method_not_allowed();
        }
        if !check_is_admin(&req).await {
            return forbidden("unauthorized", "Admin access required");
        }
        let json = req.json().await.unwrap_or(&Value::Null);
        let username = json.get("username").string();
        let email = json.get("email").string();
        let password = json.get("password").string();
        match LOCAL_AUTH.register_user(&username, &email, &password).await {
            Ok(_) => akari_json!({ success: true, username: &username }),
            Err(err) => from_fop_error(err),
        }
    }
}

endpoint! {
    APP.url("/auth/v1/me"),

    /// GET - Get current user info (routes to local or external)
    pub user_me <HTTP> {
        let token = match get_auth_token(&req) {
            Some(t) => t,
            None => return unauthorized("token_missing", "Authorization header required"),
        };

        let server = req.locals.get::<String>("auth_server")
            .map(|s| s.clone())
            .unwrap_or_else(|| "local".to_string());

        if server == "local" {
            match LOCAL_AUTH.get_user_info(token).await {
                Ok(mut user) => {
                    user += object!({ is_active: true, is_verified: true, server: "local" });
                    akari_json!({ success: true, user: user })
                }
                Err(err) => from_fop_error(err),
            }
        } else {
            match proxy::proxy_me(&server, &token).await {
                Ok(mut user) => {
                    user += object!({ server: &server });
                    akari_json!({ success: true, user: user })
                }
                Err(msg) => error_response("unauthorized", "external_auth_failed", &msg, 401),
            }
        }
    }
}

endpoint! {
    APP.url("/auth/v1/change-password"),

    /// POST - Change password (local users only)
    pub change_password <HTTP> {
        if req.method() != POST {
            return method_not_allowed();
        }
        let token = match get_auth_token(&req) {
            Some(t) => t,
            None => return unauthorized("token_missing", "Authorization header required"),
        };

        let server = req.locals.get::<String>("auth_server")
            .map(|s| s.clone())
            .unwrap_or_else(|| "local".to_string());

        if server != "local" {
            return error_response(
                "forbidden", "external_user",
                "External users must change password on their auth server", 403
            );
        }

        let json = req.json().await.unwrap_or(&Value::Null);
        let old_password = json.get("old_password").string();
        let new_password = json.get("new_password").string();
        if old_password.is_empty() || new_password.is_empty() {
            return super::response::validation_error(
                "invalid_input", "Both old_password and new_password are required"
            );
        }
        match LOCAL_AUTH.change_password(&token, &old_password, &new_password).await {
            Ok(_) => akari_json!({ success: true }),
            Err(err) => from_fop_error(err),
        }
    }
}

endpoint! {
    APP.url("/auth/v1/refresh"),

    /// POST - Refresh token (routes based on session)
    pub refresh_token <HTTP> {
        if req.method() != POST {
            return method_not_allowed();
        }
        let token = match get_auth_token(&req) {
            Some(t) => t,
            None => return unauthorized("token_missing", "Authorization header required"),
        };

        let server = req.locals.get::<String>("auth_server")
            .map(|s| s.clone())
            .unwrap_or_else(|| "local".to_string());

        if server == "local" {
            match LOCAL_AUTH.refresh_token(&token).await {
                Ok(new_token) => akari_json!({
                    success: true,
                    access_token: &new_token,
                    token_type: "Bearer"
                }),
                Err(err) => from_fop_error(err),
            }
        } else {
            match proxy::proxy_refresh(&server, &token).await {
                Ok(response) => akari_json!({ ..response }),
                Err(msg) => error_response("unauthorized", "refresh_failed", &msg, 401),
            }
        }
    }
}

endpoint! {
    APP.url("/auth/v1/login"),

    /// POST - Login with server selection
    /// Request: { id, password, server? }
    /// - server="" or "local" → LOCAL_AUTH
    /// - server="https://..." → proxy to external
    pub login <HTTP> {
        if req.method() != POST {
            return method_not_allowed();
        }
        let json = req.json().await.unwrap_or(&Value::Null);
        let id = match json.try_get("id") {
            Ok(value) => value.string(),
            Err(_) => json.get("username").string(),
        };
        let password = json.get("password").string();
        let server = json.get("server").string();

        if server.is_empty() || server == "local" {
            // === LOCAL AUTH ===
            let uid = match LOCAL_AUTH.uid_from_username_or_email_or_uid(id).await {
                Ok(uid) => uid,
                Err(err) => return from_fop_error(err),
            };
            match LOCAL_AUTH.login_user(uid, &password).await {
                Ok(token) => {
                    req.locals.set("auth_server", "local".to_string());
                    req.locals.set("auth_uid", uid);
                    let user_ref = UserRef::local(uid);
                    akari_json!({
                        success: true,
                        access_token: &token,
                        token_type: "Bearer",
                        server: "local",
                        user_ref: user_ref.to_string()
                    })
                }
                Err(err) => from_fop_error(err),
            }
        } else {
            // === EXTERNAL AUTH ===
            match proxy::proxy_login(&server, &id, &password).await {
                Ok(response) => {
                    req.locals.set("auth_server", server.clone());
                    let uid_str = response.get("user").get("uid").string();
                    let user_ref = if let Ok(uuid) = u128::from_str_radix(&uid_str.replace("-", ""), 16) {
                        UserRef::external(&server, uuid).to_string()
                    } else {
                        format!("{}@{}", server, uid_str)
                    };
                    akari_json!({
                        success: true,
                        access_token: response.get("access_token").string(),
                        token_type: "Bearer",
                        server: &server,
                        user_ref: &user_ref
                    })
                }
                Err(msg) => error_response("unauthorized", "invalid_credentials", &msg, 401),
            }
        }
    }
}

endpoint! {
    APP.url("/auth/v1/logout"),

    /// POST - Logout (routes based on session)
    pub logout <HTTP> {
        if req.method() != POST {
            return method_not_allowed();
        }
        let token = match get_auth_token(&req) {
            Some(t) => t,
            None => return unauthorized("token_missing", "Authorization header required"),
        };

        let server = req.locals.get::<String>("auth_server")
            .map(|s| s.clone())
            .unwrap_or_else(|| "local".to_string());

        let result = if server == "local" {
            LOCAL_AUTH.logout_user(&token).await.map_err(|e| e.to_string())
        } else {
            proxy::proxy_logout(&server, &token).await
        };

        // Clear session
        req.locals.remove::<String>("auth_server");
        req.locals.remove::<u32>("auth_uid");

        match result {
            Ok(_) => akari_json!({ success: true, message: "Logged out" }),
            Err(msg) => error_response("unauthorized", "logout_failed", &msg, 401),
        }
    }
}

endpoint! {
    APP.url("/auth/v1/health"),

    /// GET - Health check
    pub health_check <HTTP> {
        akari_json!({ status: "ok" })
    }
}

endpoint! {
    APP.url("/auth/v1/introspect"),

    /// POST - Validate token (for external apps calling sfx)
    /// Request: { token }
    /// Response: { active: true/false, user_ref, ... }
    pub introspect <HTTP> {
        if req.method() != POST {
            return method_not_allowed();
        }
        let json = req.json().await.unwrap_or(&Value::Null);
        let token = json.get("token").string();

        if token.is_empty() {
            return akari_json!({ active: false, reason: "missing_token" });
        }

        match LOCAL_AUTH.authenticate_user(&token).await {
            Ok(user) => {
                let uid = user.get("uid").as_u32().unwrap_or(0);
                let user_ref = UserRef::local(uid);
                akari_json!({
                    active: true,
                    user_ref: user_ref.to_string(),
                    uid: uid,
                    username: user.get("username").string(),
                    email: user.get("email").string()
                })
            }
            Err(_) => akari_json!({ active: false, reason: "token_invalid" }),
        }
    }
}
```

---

## 5. Update `src/user/fetch.rs`

Update endpoint paths at these locations:

```rust
// Line ~81: fetch_user_info()
// OLD: HttpStartLine::request_get("/users/me")
// NEW:
HttpStartLine::request_get("/auth/v1/me")

// Line ~154: get_new_token()
// OLD: HttpStartLine::request_get("/auth/refresh")
// NEW:
HttpStartLine::request_post("/auth/v1/refresh")

// Line ~165: Response check
// OLD: if json.get("success").boolean() {
// NEW:
if json.try_get("error").is_err() {

// Line ~197: auth_server_health()
// OLD: HttpStartLine::request_get("/health")
// NEW:
HttpStartLine::request_get("/auth/v1/health")

// Line ~246: disable_token()
// OLD: HttpStartLine::request_post("/auth/logout")
// NEW:
HttpStartLine::request_post("/auth/v1/logout")
```

---

## 6. Update `src/user/endpoints.rs`

```rust
// Line ~45: login()
// OLD: HttpStartLine::request_post("/auth/login")
// NEW:
HttpStartLine::request_post("/auth/v1/login")

// Line ~253: change_password()
// OLD: json_request("/users/me/password", ...)
// NEW:
json_request("/auth/v1/change-password", ...)
```

---

## 7. Update `src/admin/api.rs` (Optional)

Align admin error responses:

```rust
// Line ~11
// OLD: json_response(object!({ success: false, message: "Unauthorized" }))
// NEW:
json_response(object!({
    error: "forbidden",
    reason: "unauthorized",
    message: "Unauthorized"
})).status(StatusCode::FORBIDDEN)

// Line ~29
// OLD: json_response(object!({ success: false, message: e.to_string() }))
// NEW:
json_response(object!({
    error: "server_error",
    reason: "internal_error",
    message: e.to_string()
})).status(StatusCode::INTERNAL_SERVER_ERROR)

// Line ~33
// OLD: json_response(object!({ success: false, message: "Method not allowed" }))
// NEW:
json_response(object!({
    error: "method_error",
    reason: "method_not_allowed",
    message: "Method not allowed"
})).status(StatusCode::METHOD_NOT_ALLOWED)
```

---

## 8. Verification Checklist

### New Files
- [ ] Create `src/local_auth/response.rs` - Error response helpers
- [ ] Create `src/local_auth/user_ref.rs` - Unified user ID type (u128)
- [ ] Create `src/local_auth/proxy.rs` - External auth server proxy

### Module Updates
- [ ] Update `src/local_auth/mod.rs` - Export new modules
- [ ] Update `src/local_auth/endpoints.rs` - Multi-server login flow
- [ ] Update `src/user/fetch.rs` - New paths + response check
- [ ] Update `src/user/endpoints.rs` - New paths
- [ ] (Optional) Update `src/admin/api.rs` - Error format

### Testing
- [ ] Run `cargo check` - Verify compilation
- [ ] Test local login: `{ id: "user", password: "...", server: "" }`
- [ ] Test external login: `{ id: "user", password: "...", server: "https://..." }`
- [ ] Verify refresh/logout routes to correct server
- [ ] Verify user_ref format: `local@42` or `server@uuid`
