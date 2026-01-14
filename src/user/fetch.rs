//! fetch.rs
//!
//! Responsible for managing authentication tokens in the session, communicating with the
//! remote auth/user service, and caching user info in the session store.

use hotaru::prelude::*; 
use hotaru::http::*; 
use htmstd::session::CSessionRW;
use super::user::*;
use super::Server; 

/// Store the given authentication token in the HTTP-session under `"auth_token"`.
///
/// # Arguments
///
/// * `req`   – mutable reference to the current request context
/// * `token` – the raw JWT or bearer token string to persist
pub fn set_auth_token(req: &mut HttpReqCtx, token: &str) {
    tracing::info!(%token, "Setting auth token in session");
    req.params
        .get_mut::<CSessionRW>()
        .unwrap()
        .insert("auth_token".into(), token.into());
}

/// Retrieve the authentication token from the current HTTP-session, if present.
///
/// Returns `Some(String)` when a token is stored, or `None` if not set.
///
/// # Arguments
///
/// * `req` – shared reference to the current request context
pub fn get_auth_token(req: &HttpReqCtx) -> Option<String> {
    req.params
        .get::<CSessionRW>()
        .and_then(|session| session.get("auth_token"))
        .map(|token| token.string())
} 

/// Store the given authentication token in the HTTP-session under `"auth_token"`.
///
/// # Arguments
///
/// * `req`   – mutable reference to the current request context
/// * `host` – the host 
pub fn set_host(req: &mut HttpReqCtx, host: &str) {
    tracing::info!(%host, "Setting host in session");
    req.params
        .get_mut::<CSessionRW>()
        .unwrap()
        .insert("host".into(), host.into());
}

/// Retrieve the authentication token from the current HTTP-session, if present. 
/// 
/// If does not exist use the default 
///
/// # Arguments
///
/// * `req` – shared reference to the current request context 
pub fn get_host(req: &HttpReqCtx) -> Server { 
    req.params
        .get::<CSessionRW>()
        .and_then(|session| session.get("host"))
        .map(|token| token.string()) 
        .map(|s| Server::from_string(&s))
        .unwrap_or(Server::Local) 
}

/// Perform an authenticated GET on `/users/me` to fetch the remote user’s details,
/// then deserialize into our local `User` type.
///
/// Returns `Some(User)` on success, or `None` if the server returned a non-JSON body
/// or an error.
///
/// # Arguments
///
/// * `host` - the host 
/// * `auth` – the bearer token to include in the request
pub async fn fetch_user_info(host: Server, auth: String) -> Option<User> {
    println!("fetch_user_info: sending request to {}, token: {}", host.get_address(), auth);
    let request = request_with_auth_token(get_request("/users/me"), Some(auth));
    let response = HttpResCtx::send_request(
        host.get_address(),
        request,
        HttpSafety::default(),
    )
    .await;

    if response.is_err() {
        println!("fetch_user_info: request failed: {:?}", response);
        return None;
    }

    let response = response.unwrap();
    println!("fetch_user_info: response body type: {:?}", std::mem::discriminant(&response.body));

    // Try to parse the body as JSON if it's a buffer
    let body = response.body.parse_buffer(&HttpSafety::new());
    println!("fetch_user_info: parsed body: {:?}", body);

    if let HttpBody::Json(json) = body {
        if json.get("success").boolean() {
            // The JSON is assumed to be of the form { "success": true, "user": { ... } }
            let mut user_value = json.get("user").clone();
            user_value.set("server", host.clone());
            println!("fetch_user_info: returning user: {:?}", user_value);
            Some(user_value.into())
        } else {
            println!("fetch_user_info: success=false in response");
            None
        }
    } else {
        println!("fetch_user_info: unexpected response body: {:?}", body);
        None
    }
}

/// Refresh the stored token by calling `/auth/refresh`.  If no token is in-session,
/// returns a JSON error object.  On success, overwrites the session and returns
/// `{ success: true, access_token: <new> }`.
///
/// # Arguments
///
/// * `req` – mutable reference to the current request context
pub async fn refresh_user_token(req: &mut HttpReqCtx) -> Value {
    // First, pull the token out of the session
    let auth_token = match get_auth_token(req) {
        Some(t) => t,
        None => {
            return object!({
                success: false,
                message: "No authentication token available"
            });
        }
    }; 

    let host = get_host(req); 

    // Exchange it at /auth/refresh
    match get_new_token(host, auth_token).await {
        Ok(new_token) => {
            tracing::info!(%new_token, "Refreshed auth token successfully");
            set_auth_token(req, &new_token);
            object!({
                success: true,
                access_token: new_token
            })
        }
        Err(err_value) => {
            tracing::error!("Token refresh failed: {:?}", err_value);
            err_value
        }
    }
}

/// Internal helper: call `/auth/refresh` with an existing token, returning
/// `Ok(new_token)` on success or `Err(json_value)` on failure.
///
/// # Arguments
///
/// * `host` - the host 
/// * `token` – the bearer token to refresh
async fn get_new_token(host: Server, token: String) -> Result<String, Value> {
    tracing::info!(%token, "Requesting new token from auth server");
    let request = get_request("/auth/refresh")
        .add_header("Authorization", format!("Bearer {}", token));
    let response = HttpResCtx::send_request(
        host.get_address(), 
        request,
        HttpSafety::default(),
    )
    .await
    .unwrap();

    if let HttpBody::Json(json) = response.body.parse_buffer(&HttpSafety::new()) {
        if json.get("success").boolean() {
            Ok(json.get("access_token").string())
        } else {
            Err(json)
        }
    } else {
        Err(object!({
            success: false,
            message: "Invalid response from server or no response"
        }))
    }
}

/// Cache a `User` instance in-session under the key `"user_info_cache"`.
///
/// # Arguments
///
/// * `req`  – mutable reference to the current request context
/// * `user` – the fully populated `User` object to store
pub fn cache_user_info(req: &mut HttpReqCtx, user: User) {
    tracing::info!(user = ?user, "Caching user info in session");
    req.params
        .get_mut::<CSessionRW>()
        .unwrap()
        .insert("user_info_cache".into(), user.into());
}

/// Check the health endpoint (`/health`) of the auth server. Returns `true` if
/// the JSON `{ "status": "ok" }` is returned, else `false`.
pub async fn auth_server_health(host: Server) -> bool {
    let response = HttpResCtx::send_request(
        host.get_address(),
        get_request("/health"),
        HttpSafety::default(),
    )
    .await
    .unwrap();

    if let HttpBody::Json(json) = response.body.parse_buffer(&HttpSafety::new()) {
        json.get("status").string() == "ok"
    } else {
        false
    }
}

/// Log the user out locally by clearing session keys `"auth_token"` and
/// `"user_info_cache"`, then issue a redirect to the login-refresh flow.
///
/// # Arguments
///
/// * `req` – mutable reference to the current request context
pub async fn logout(req: &mut HttpReqCtx) -> HttpResponse {
    tracing::info!("Clearing session and redirecting to login-refresh");
    let params = req.params.get_mut::<CSessionRW>().unwrap();
    params.remove("user_info_cache");
    params.remove("auth_token");
    params.remove("host"); 
    redirect_response("/user/refresh?redirect=/user/login")
}

/// Immediately mutate `req.response` to redirect through `/user/refresh`.
///
/// # Arguments
///
/// * `req` – mutable reference to the current request context
pub fn redirect_refresh(req: &mut HttpReqCtx) {
    let redirect_url = req.path();
    req.response = redirect_response(&format!(
        "/user/refresh?redirect={}",
        redirect_url
    ));
}

/// Invalidate a token server-side by calling `/auth/logout`. Returns the JSON
/// payload the server responded with, or an error-style object.
///
/// # Arguments
///
/// * `host` - the host 
/// * `token` – the bearer token to revoke
pub async fn disable_token(host: Server, token: String) -> Value {
    let request = get_request("/auth/logout")
        .add_header("Authorization", format!("Bearer {}", token));
    let response = HttpResCtx::send_request(
        host.get_address(),
        request,
        HttpSafety::default(),
    )
    .await
    .unwrap();

    if let HttpBody::Json(json) = response.body.parse_buffer(&HttpSafety::new()) {
        json
    } else {
        object!({
            success: false,
            message: "Invalid response from server or no response"
        })
    }
}

/// Add an `Authorization: Bearer <token>` header if `token.is_some()`.
///
/// # Arguments
///
/// * `req`   – the base HTTP request
/// * `token` – optional bearer token
pub fn request_with_auth_token(mut req: HttpRequest, token: Option<String>) -> HttpRequest {
    if let Some(tok) = token {
        req = req.add_header("Authorization", format!("Bearer {}", tok));
    }
    req
}

/// Convenience: pull the current `User` from `req.params` or fall back to `guest`.
pub async fn get_user(req: &mut HttpReqCtx) -> User {
    req.params
        .get::<User>()
        .map(|u| u.clone())
        .unwrap_or_else(|| User::guest(get_host(req)))
} 

/// Convenience: pull the current `User` from `req.params` or fall back to `guest`. 
/// And then convert into UserID 
pub async fn get_user_id(req: &mut HttpReqCtx) -> UserID { 
    get_user(req).await.into() 
}
