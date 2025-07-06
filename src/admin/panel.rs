use starberry::prelude::*;
use crate::admin::check_is_admin;
use crate::op::{self, into_path_l, pageprop}; 
use starberry::HttpBody; 
use crate::APP; 

async fn admin_fetch_json(req: &mut HttpReqCtx, path: &str) -> Option<Value> {
    let full_host: String = format!("http://{}", op::BINDING); 
    let response = HttpResCtx::send_request(
        full_host.clone(),
        get_request(path)
            .add_cookie("session_id", req.get_cookie_or_default("session_id")) 
            .add_cookie("session_cont", req.get_cookie_or_default("session_cont")), 
        HttpSafety::default(),
    )
    .await
    .unwrap(); 

    if let HttpBody::Json(json2) = response.body {
        return Some(json2);
    } 
    return None 
} 

#[url(APP.lit_url("/admin/panel"))]
async fn panel_users(mut req: HttpReqCtx) -> HttpResponse {
    if !check_is_admin(req).await { 
        return redirect_response("/user/unauthorized") 
    }
    // Fetch users, default to empty list
    let users = admin_fetch_json(&mut req, "/admin/users").await
        .map(|j| j.get("users").clone())
        .unwrap_or(object!([]));
    akari_render!(
        "user/panel.html",
        pageprop  = pageprop(&mut req, "Manage Users", "Create, view, and edit users"),
        path      = into_path_l(&mut req, vec!["home", "admin", "user"]), 
        users     = users
    )
} 

#[url(APP.lit_url("/panel/users/json"))]
async fn panel_users_json(mut req: HttpReqCtx) -> HttpResponse {
    let path = format!("/admin/users?page={}", req.get_url_args("page").unwrap_or("1".to_string())); 
    let users = admin_fetch_json(&mut req, &path).await
        .map(|j| j.get("users").clone())
        .unwrap_or(object!([]));
    json_response(object!({ users: users }))
} 
