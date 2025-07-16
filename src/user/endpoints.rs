use std::collections::HashMap;

use super::fetch::*;
use super::user::*;
use crate::op::{self, APP};
use crate::user::Server;
use sbmstd::session::CSessionRW;
use starberry::{
    HttpBody, HttpContentType, HttpRequest, prelude::*,
    starberry_core::http::start_line::HttpStartLine,
};

/// The GET and POST endpoint for user login 
/// 
/// # Request 
/// `GET /user/login` 
/// EMPTY 
/// 
/// `POST /user/login` 
/// UrlCodedForm, 
/// host: The base server, use "local" to present local host 
/// username: UserName 
/// password: Password 
/// 
/// # Response 
/// (1) The HTML page for login 
/// (2) JSON 
/// {
///     success: false,
///     message: "Invalid response from server" // All other cases
/// } 
/// (3) JSON 
/// JSON response from the server 
/// While the auth token and the host will be added to the cookie 
#[url(reg![&APP, LitUrl("user"), LitUrl("login")])]
async fn login() -> HttpResponse {
    logout(req).await; // Ensure user is logged out before login 
    if req.method() == POST {
        let form = req.form_or_default().await;
        let host = Server::from_string(&form.get_or_default("host"));
        let username = form.get_or_default("username");
        let password = form.get_or_default("password");
        // println!("User login attempt: {} with password {}", username, password);
        // Send the request to the user login handler
        let mut meta = HttpMeta::new(HttpStartLine::request_post("/auth/login"), HashMap::new());
        meta.set_content_type(HttpContentType::ApplicationJson());
        let request_content = HttpRequest::new(
            meta,
            HttpBody::Json(object!({
                username: username,
                password: password,
            })),
        );
        println!("Server: {}, Address: {}", host, host.get_address());
        let response = HttpResCtx::send_request(&host.get_address(), request_content, HttpSafety::default())
            .await
            .unwrap();
        println!("Response: {:?}", response);
        if let HttpBody::Json(mut json) = response.body {
            set_auth_token(req, &json.get("access_token").string());
            set_host(req, &host.to_string());
            return json_response(json);
        }
        return json_response(object!({
            success: false,
            message: "Invalid response from server" // All other cases
        }));
    }
    akari_render!(
        "user/login.html",
        pageprop = op::pageprop(req, "User Login", "Login to your account"),
        path = op::into_path_l(req, vec!["home", "user", "login"]),
        hosts = op::get_host().clone(), // Get the list of host
    )
}

/// The logout endpoint 
/// 
/// # Request 
/// `GET /user/logout ` 
/// Cookie session required to be include in the header  
/// 
/// # Response 
/// A `HttpResponse` that redirects to the login-refresh flow 
/// This will clear the session and redirect to the login page 
#[url(reg![&APP, LitUrl("user"), LitUrl("logout")])]
async fn logout_route() -> HttpResponse {
    if let Some(token) = get_auth_token(req) {
        disable_token(get_host(req), token).await;
    }
    logout(req).await
}

/// The refresh endpoint 
/// 
/// # Request 
/// `GET /user/refresh?redirect=<url>` 
/// Cookie session required to be included in the header 
/// 
/// # Response 
/// A `HttpResponse` that redirects to the specified URL 
/// This will refresh the user token and redirect to the specified URL 
#[url(reg![&APP, LitUrl("user"), LitUrl("refresh")])]
async fn refresh_route() -> HttpResponse {
    refresh_user_token(req).await;
    redirect_response(&req.get_url_args("redirect").unwrap_or("/".to_string()))
}

/// Get the current user's auth token from the request context. 
/// This is not meant for production use, but for testing purposes only. 
#[url(reg![&APP, LitUrl("user"), LitUrl("token")])]
async fn get_token() -> HttpResponse {
    text_response(format!("{:?}", get_auth_token(req)))
}

/// Get the current user's information. 
/// This is not meant for production use, but for testing purposes only. 
#[url(reg![&APP, LitUrl("user"), LitUrl("info")])]
async fn get_self_uid() -> HttpResponse {
    if let Some(token) = get_auth_token(req) {
        text_response(format!(
            "User: {:?}",
            fetch_user_info(get_host(req), token).await
        ))
    } else {
        text_response("No Info")
    }
}

/// Refresh the user token and return the new token in JSON format. 
/// This is not meant for production use, but for testing purposes only. 
#[url(reg![&APP, LitUrl("user"), LitUrl("refresh_api")])]
async fn refresh_token() -> HttpResponse {
    json_response(refresh_user_token(req).await)
}

/// Get the current user's cached information from the session. 
/// This is not meant for production use, but for testing purposes only. 
#[url(reg![&APP, LitUrl("user"), LitUrl("server_health")])]
async fn server_health() -> HttpResponse {
    akari_json!({
        ok: auth_server_health(get_host(req)).await
    })
}

/// Get the current user's cached information from the session. 
/// 
/// # Request 
/// `GET /user/cached_info` 
/// A `HttpResponse` that contains the cached user information 
/// 
/// # Response 
/// Json 
/// {
//     uid: self.id.uid,
//     server: self.id.server.to_string(),
//     username: self.username,
//     email: self.email,
//     is_active: self.is_active,
//     is_verified: self.is_verified,
//     cached_time: self.cached_at,
// } 
#[url(reg![&APP, LitUrl("user"), LitUrl("cached_info")])]
async fn get_self_cached_info() -> HttpResponse {
    let user = req
        .params
        .get::<CSessionRW>()
        .and_then(|session| session.get("user_info_cache"))
        .cloned()
        .unwrap_or(Value::None);
    json_response(user)
}

/// The usercenter redirect 
#[url(reg![&APP, LitUrl("user")])]
async fn user_index_redirect() -> HttpResponse {
    redirect_response("/user/home")
}

/// The usercenter redirect 
#[url(reg![&APP, LitUrl("user"), TrailingSlash()])]
async fn user_index() -> HttpResponse {
    redirect_response("/user/home")
} 

/// The user center home page 
/// 
/// # Request 
/// `GET /user/home` 
/// The session must contain a valid user information 
/// 
/// # Response 
/// A `HttpResponse` that contains the user home page 
/// If the user is a guest, it will redirect to the login page 
#[url(reg![&APP, LitUrl("user"), LitUrl("home")])]
async fn home() -> HttpResponse {
    if req.params.get::<User>().unwrap().get_uid() == 0 {
        return redirect_response("/user/login");
    }
    let user = req
        .params
        .get::<CSessionRW>()
        .and_then(|session| session.get("user_info_cache"))
        .map(|user| user.clone().into())
        .unwrap_or(User::guest(op::get_default_host()));
    akari_render!(
        "user/home.html",
        pageprop = op::pageprop(req, "User Home", "Welcome to your home page"),
        path = op::into_path_l(req, vec!["home", "user", "home"]),
        user = user
    )
}

/// The POST endpoint for changing the user's password. 
/// 
/// # Request
/// `POST /user/home/change_password`
///
/// The request body must contain the old and new passwords. 
/// UrlEncodedForm 
/// { 
///    old_password: String, 
///    new_password: String,  
///    host: String, // The base server, use "local" to present local host
/// } 
/// 
/// # Response 
/// 
/// A `HttpResponse` that contains the result of the password change operation 
/// 
/// If the operation is successful, it will return a JSON object with the following structure: 
/// {
///     success: true,
///     message: "Password changed successfully"
/// } 
#[url(reg![&APP, LitUrl("user"), LitUrl("home"), LitUrl("change_password")])]
pub async fn change_password(req: &mut HttpReqCtx) -> HttpResponse {
    let user = get_user(req).await;
    let host = get_host(req);
    let form = req.form_or_default().await;
    let old_password = form.get_or_default("old_password");
    let new_password = form.get_or_default("new_password");
    if old_password.is_empty() || new_password.is_empty() {
        return json_response(object!({
            success: false,
            message: "Invalid old or new password"
        }));
    }
    let response = HttpResCtx::send_request(
        host.get_address(),
        request_with_auth_token(
            json_request(
                "/users/me/password",
                object!({
                    old_password: old_password,
                    new_password: new_password,
                }),
            ),
            get_auth_token(req),
        ),
        HttpSafety::default(),
    )
    .await
    .unwrap();
    if let HttpBody::Json(json) = response.body {
        return json_response(json);
    }
    json_response(object!({
        success: false,
        message: "Invalid response from server or no response"
    }))
}

/// Unauthorized access page 
#[url(reg![&APP, LitUrl("user"), LitUrl("unauthorized")])]
pub async fn unauthorized(req: &mut HttpReqCtx) -> HttpResponse {
    akari_render!(
        "user/unauthorized.html",
        pageprop = op::pageprop(req, "Unauthorized", "Unauthorized"),
        path = op::into_path_l(req, vec!["home", "user", "unauthorized"]),
    )
}
