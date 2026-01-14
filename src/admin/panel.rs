use hotaru::prelude::*;
use hotaru::http::*;
use crate::admin::check_is_admin;
use crate::op::{self, into_path_l, pageprop}; 
use crate::APP; 

async fn admin_fetch_json(req: &mut HttpReqCtx, path: &str) -> Option<Value> {
    let full_host: String = format!("http://{}", op::BINDING.clone()); 
    let response = HttpResCtx::send_request(
        full_host.clone(),
        get_request(path)
            .add_cookie("session_id", req.get_cookie_or_default("session_id")) 
            .add_cookie("session_cont", req.get_cookie_or_default("session_cont")), 
        HttpSafety::default(),
    )
    .await
    .unwrap(); 

    if let HttpBody::Json(json2) = response.body.parse_buffer(&HttpSafety::new()) {
        return Some(json2);
    }
    None
} 

endpoint! {
    APP.url("/admin/panel"),

    pub panel_users <HTTP> {
        if !check_is_admin(req).await { 
            return redirect_response("/user/unauthorized"); 
        }
        // Fetch users, default to empty list
        let users = admin_fetch_json(req, "/admin/users").await
            .map(|j| j.get("users").clone())
            .unwrap_or(object!([]));
        akari_render!(
            "user/panel.html",
            pageprop  = pageprop(req, "Manage Users", "Create, view, and edit users"),
            path      = into_path_l(req, vec!["home", "admin", "user"]), 
            users     = users
        )
    }
}

endpoint! {
    APP.url("/panel/users/json"),

    pub panel_users_json <HTTP> {
        let page = req.query("page").unwrap_or("1".to_string());
        let path = format!("/admin/users?page={}", page);
        let users = admin_fetch_json(req, &path).await
            .map(|j| j.get("users").clone())
            .unwrap_or(object!([]));
        json_response(object!({ users: users }))
    }
}
