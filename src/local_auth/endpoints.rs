pub use starberry::prelude::*; 
use crate::op::APP;
use super::analyze::get_auth_token; 
use crate::admin::check_is_admin; 

use super::LOCAL_AUTH; 

/// POST /users - Register a new user 
/// Request body: Json -> {"username": "Aaa", "email": "example@example.com", "password": "Aa333333"} 
/// Auth token of a admin should be included in the request header 
/// Response (1): {"success": false, "error": "Method not allowed"/"Missing information"/"Unauthorized"} 
/// Response (2): {"success": true, "username": "Aaa"} 
#[url(APP.lit_url("/users"))] 
pub async fn create_user() -> HttpResponse { 
    if req.method() != POST {
        return akari_json!({ success: false, error: "Method not allowed" }).status(401);
    } 
    if !check_is_admin(req).await {
        return akari_json!({ success: false, error: "Unauthorized" }).status(403);
    } 
    let mut json = req.json_or_default().await; 
    let username = json.get("username").string(); 
    let email = json.get("email").string(); 
    let password = json.get("password").string(); 
    let result = LOCAL_AUTH.register_user(&username, &email, &password).await; 
    match result {
        Ok(_) => akari_json!({ success: true, username: username }),
        Err(err) => akari_json!({ success: false, error: err.to_string() }),
    } 
}

/// GET /users/me - Get current user info 
/// Request header should include a bearer token 
/// Response (1): {"success": false, "error": "Token invalid"/"System Error"/"Error fetching uid"} 
/// Response (2): {"success": true, "username": username, "uid": userid, "email": email} 
#[url(APP.lit_url("/users/me"))] 
pub async fn user_me() -> HttpResponse { 
    let token = get_auth_token(req); 
    println!("{:?}", token); 
    if token.is_none() {
        return akari_json!({ success: false, error: "Token invalid" }).status(401);
    } 
    let token = token.unwrap(); 
    match LOCAL_AUTH.get_user_info(token).await { 
        Ok(mut user) => { 
            user += object!({ is_active: true, is_verified: true });
            akari_json!({ success: true, user: user }) 
        },
        Err(err) => { 
            println!("Error fetching user info: {}", err.to_string());
            akari_json!({ success: false, error: err.to_string() }).status(401)
        } 
    }
} 

/// POST /users/me/password - Change user's password 
/// Request header should include a bearer token 
/// Request: {"old_password": old_password, "new_password": new_password} 
/// Response (1): {"success": false, "error": "Token invalid"/"System Error"/"Error fetching uid"/"Invalid old or new password"} 
/// Response (2): {"success": true} 
#[url(APP.lit_url("/users/me/password"))] 
pub async fn change_password() -> HttpResponse { 
    let token = get_auth_token(req); 
    if token.is_none() {
        return akari_json!({ success: false, error: "Token invalid" }).status(403);
    } 
    let json = req.json_or_default().await; 
    let old_password = json.get("old_password").string(); 
    let new_password = json.get("new_password").string(); 
    if old_password.is_empty() || new_password.is_empty() {
        return akari_json!({ success: false, error: "Invalid old or new password" }).status(400);
    } 
    let token = token.unwrap(); 
    let uid = match LOCAL_AUTH.authenticate_user(&token).await {
        Ok(uid) => uid,
        Err(err) => return akari_json!({ success: false, error: err.to_string() }).status(400),
    }; 
    match LOCAL_AUTH.change_password(&token, &old_password, &new_password).await {
        Ok(_) => akari_json!({ success: true }),
        Err(err) => akari_json!({ success: false, error: err.to_string() }).status(400),
    } 
}

/// GET/POST /auth/refresh - Get a new token 
/// Request header should include a bearer token 
/// Response (1): {"success": false, "error": "Token invalid"/"System Error"/"Error fetching uid"} 
/// Response (2): {"success": true, "access_token": access, "token_type": "Bearer" } 
#[url(APP.lit_url("/auth/refresh"))] 
pub async fn refresh_token() -> HttpResponse { 
    let token = get_auth_token(req);
    if token.is_none() {
        return akari_json!({ success: false, error: "Token invalid" }).status(403);
    }
    let token = token.unwrap();
    match LOCAL_AUTH.refresh_token(&token).await {
        Ok(new_token) => akari_json!({ success: true, access_token: new_token, token_type: "Bearer" }),
        Err(err) => akari_json!({ success: false, error: err.to_string() }),
    } 
} 

/// POST /auth/login - Login to the server and return a token 
/// Request (1): {"id": uid/username/email, "password": password} 
/// Request (2): {"username": username, "password": password} (Legacy support) 
/// Response (1): {success: false, message: "Invalid username or password"/"Error during authing"} 
/// Response (2): {success: true, access_token: access, token_type: "Bearer"}
#[url(APP.lit_url("/auth/login"))] 
pub async fn login() -> HttpResponse { 
    if req.method() != POST {
        return akari_json!({ success: false, message: "Method not allowed" }).status(405);
    }
    let json = req.json_or_default().await;
    let id = match json.try_get("id") { 
        Ok(value) => value.string(),
        Err(_) => json.get("username").string(),
    };
    let password = json.get("password").string(); 
    let uid = LOCAL_AUTH.uid_from_username_or_email_or_uid(id).await; 
    if let Err(err) = uid {
        return akari_json!({ success: false, message: err.to_string() }).status(400);
    } 
    let uid = uid.unwrap();
    match LOCAL_AUTH.login_user(uid, &password).await {
        Ok(token) => akari_json!({ success: true, access_token: token, token_type: "Bearer" }),
        Err(err) => akari_json!({ success: false, message: err.to_string() }),
    }
}  

/// POST auth/logout - Logout and deactivate the auth token 
/// A bearer token included in header 
/// Response (1): {"success": false, "error": ""Invalid authorization header"/"Error during logout"} 
/// Response (2): { success: true, message: "Logged out" } 
#[url(APP.lit_url("/auth/logout"))] 
pub async fn logout() -> HttpResponse { 
    let token = get_auth_token(req);
    if token.is_none() {
        return akari_json!({ success: false, error: "Invalid authorization header" }).status(401);
    }
    let token = token.unwrap();
    match LOCAL_AUTH.logout_user(&token).await {
        Ok(_) => akari_json!({ success: true, message: "Logged out" }),
        Err(err) => akari_json!({ success: false, error: err.to_string() }),
    } 
}  

#[url(APP.lit_url("/health"))] 
pub async fn health_check() -> HttpResponse {
    akari_json!({ status: "ok" })
} 
