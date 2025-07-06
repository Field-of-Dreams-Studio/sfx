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

#[url(reg![&APP, LitUrl("user"), LitUrl("logout")])]
async fn logout_route() -> HttpResponse {
    if let Some(token) = get_auth_token(req) {
        disable_token(get_host(req), token).await;
    }
    logout(req).await
}

#[url(reg![&APP, LitUrl("user"), LitUrl("refresh")])]
async fn refresh_route() -> HttpResponse {
    refresh_user_token(req).await;
    redirect_response(&req.get_url_args("redirect").unwrap_or("/".to_string()))
}

#[url(reg![&APP, LitUrl("user"), LitUrl("token")])]
async fn get_token() -> HttpResponse {
    text_response(format!("{:?}", get_auth_token(req)))
}

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

#[url(reg![&APP, LitUrl("user"), LitUrl("refresh_api")])]
async fn refresh_token() -> HttpResponse {
    json_response(refresh_user_token(req).await)
}

#[url(reg![&APP, LitUrl("user"), LitUrl("server_health")])]
async fn server_health() -> HttpResponse {
    akari_json!({
        ok: auth_server_health(get_host(req)).await
    })
}

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

#[url(reg![&APP, LitUrl("user")])]
async fn user_index_redirect() -> HttpResponse {
    redirect_response("/user/home")
}

#[url(reg![&APP, LitUrl("user"), TrailingSlash()])]
async fn user_index() -> HttpResponse {
    redirect_response("/user/home")
}

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

#[url(reg![&APP, LitUrl("user"), LitUrl("unauthorized")])]
pub async fn unauthorized(req: &mut HttpReqCtx) -> HttpResponse {
    akari_render!(
        "user/unauthorized.html",
        pageprop = op::pageprop(req, "Unauthorized", "Unauthorized"),
        path = op::into_path_l(req, vec!["home", "user", "unauthorized"]),
    )
}
