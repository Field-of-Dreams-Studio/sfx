use crate::APP;
use crate::admin::check_is_admin;
use crate::local_auth::LOCAL_AUTH;
use crate::op::{self, into_path_l, pageprop};
use crate::user::fetch::send_http_request;
use hotaru::http::*;
use hotaru::prelude::*;

async fn admin_fetch_json(req: &mut HttpReqCtx, path: &str) -> Option<Value> {
    let full_host: String = format!("http://{}", op::BINDING.clone());
    let result = send_http_request(
        full_host.clone(),
        get_request(path)
            .add_cookie("session_id", req.get_cookie_or_default("session_id"))
            .add_cookie("session_cont", req.get_cookie_or_default("session_cont")),
        HttpSafety::default(),
    )
    .await;

    let response = match result {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(?e, path = %path, "admin_fetch_json: self-call failed");
            return None;
        }
    };

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
        let users = admin_fetch_json(req, "/admin/users").await
            .map(|j| j.get("users").clone())
            .unwrap_or(object!([]));
        akari_render!(
            "admin/panel.html",
            pageprop  = pageprop(req, "Manage Users", "Create, view, and edit users"),
            path      = into_path_l(req, vec!["home", "admin", "user"]),
            users     = users
        )
    }
}

endpoint! {
    APP.url("/admin/panel/admins"),

    pub panel_admins <HTTP> {
        if !check_is_admin(req).await {
            return redirect_response("/user/unauthorized");
        }
        akari_render!(
            "admin/admins.html",
            pageprop = pageprop(req, "Manage Admins", "Manage admin access"),
            path = into_path_l(req, vec!["home", "admin", "user"]),
        )
    }
}

endpoint! {
    APP.url("/admin/panel/<uid>"),

    pub panel_user_edit <HTTP> {
        if !check_is_admin(req).await {
            return redirect_response("/user/unauthorized");
        }

        let uid = match req.param("uid").and_then(|uid| uid.parse::<u32>().ok()) {
            Some(uid) => uid,
            None => return text_response("404 User not found").status(StatusCode::NOT_FOUND),
        };

        let user = match LOCAL_AUTH.admin_get_user(uid).await {
            Some(user) => {
                let admin_entry = object!(format!("{}@local", uid));
                object!({
                    uid: uid,
                    username: &user.username,
                    email: &user.email,
                    is_active: user.is_active,
                    is_admin: op::get_admin().contains(&admin_entry),
                })
            }
            None => return text_response("404 User not found").status(StatusCode::NOT_FOUND),
        };

        akari_render!(
            "admin/user_edit.html",
            pageprop = pageprop(req, "Edit User", "Edit user account"),
            path = into_path_l(req, vec!["home", "admin", "user"]),
            user = user,
        )
    }
}

endpoint! {
    APP.url("/admin/users/json"),

    pub panel_users_json <HTTP> {
        if !check_is_admin(req).await {
            return json_response(object!({ success: false, message: "Unauthorized" }))
                .status(StatusCode::UNAUTHORIZED);
        }
        let page = req.query("page").unwrap_or("1".to_string());
        let path = format!("/admin/users?page={}", page);
        let data = admin_fetch_json(req, &path).await
            .unwrap_or_else(|| object!({ users: [], total: 0 }));
        json_response(object!({
            users: data.get("users").clone(),
            total: data.get("total").clone(),
        }))
    }
}
