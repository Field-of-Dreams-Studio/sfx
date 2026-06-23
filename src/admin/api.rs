use hotaru::http::*;
use hotaru::prelude::*;
use tracing::{error, info, instrument};

use crate::admin::check_is_admin;
use crate::local_auth::fop::UserStorage;
use crate::op;
use crate::{
    APP,
    local_auth::{LOCAL_AUTH, fop::FopError},
};

fn admin_user_json(uid: u32, user: &UserStorage) -> Value {
    let admin_entry = object!(format!("{}@local", uid));
    object!({
        uid: uid,
        username: &user.username,
        email: &user.email,
        is_active: user.is_active,
        is_admin: op::get_admin().contains(&admin_entry),
    })
}

fn admin_error_status(error: &FopError) -> StatusCode {
    match error {
        FopError::UserNameConflict | FopError::EmailConflict => StatusCode::CONFLICT,
        FopError::UserNameNotValid | FopError::EmailNotValid | FopError::PasswordMismatch => {
            StatusCode::BAD_REQUEST
        }
        FopError::UserNotFound => StatusCode::NOT_FOUND,
        FopError::TooManyRequest => StatusCode::TOO_MANY_REQUESTS,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

endpoint! {
    APP.url("/admin/users"),

    #[instrument(level = "info", skip(req))]
    pub admin_users <HTTP> {
        if !check_is_admin(req).await {
            return json_response(object!({ success: false, message: "Unauthorized" }))
                .status(StatusCode::UNAUTHORIZED);
        }

        match req.method() {
            GET => {
                info!(path = %req.path(), "list_admin_users handler start");
                let users: Vec<Value> = LOCAL_AUTH
                    .admin_list_users()
                    .await
                    .into_iter()
                    .map(|(uid, user)| admin_user_json(uid, &user))
                    .collect();
                let total = users.len();
                json_response(object!({ success: true, users: users, total: total }))
                    .status(StatusCode::OK)
            }
            POST => {
                info!(path = %req.path(), "create_admin_user handler start");
                let form = req.form_or_default().await.clone();
                let username = form.get_or_default("username");
                let password = form.get_or_default("password");
                let email = form.get_or_default("email");
                match LOCAL_AUTH.register_user(&username, &email, &password).await {
                    Ok(()) => json_response(object!({ success: true, username: username }))
                        .status(StatusCode::CREATED),
                    Err(e) => {
                        let status = admin_error_status(&e);
                        if status == StatusCode::INTERNAL_SERVER_ERROR {
                            error!(?e, "create_admin_user internal error");
                        }
                        json_response(object!({ success: false, message: e.to_string() }))
                            .status(status)
                    }
                }
            }
            _ => json_response(object!({ success: false, message: "Method not allowed" }))
                .status(StatusCode::METHOD_NOT_ALLOWED),
        }
    }
}

endpoint! {
    APP.url("/admin/users/<uid>"),

    #[instrument(level = "info", skip(req))]
    pub admin_user_detail <HTTP> {
        if !check_is_admin(req).await {
            return json_response(object!({ success: false, message: "Unauthorized" }))
                .status(StatusCode::UNAUTHORIZED);
        }

        let uid = match req.param("uid").and_then(|uid| uid.parse::<u32>().ok()) {
            Some(uid) => uid,
            None => {
                return json_response(object!({ success: false, message: "Invalid uid" }))
                    .status(StatusCode::BAD_REQUEST);
            }
        };

        match req.method() {
            GET => {
                match LOCAL_AUTH.admin_get_user(uid).await {
                    Some(user) => json_response(object!({
                        success: true,
                        user: admin_user_json(uid, &user),
                    })).status(StatusCode::OK),
                    None => json_response(object!({ success: false, message: "User not found" }))
                        .status(StatusCode::NOT_FOUND),
                }
            }
            POST => {
                let form = req.form_or_default().await.clone();
                let username = form
                    .get("username")
                    .cloned()
                    .filter(|value| !value.is_empty());
                let email = form
                    .get("email")
                    .cloned()
                    .filter(|value| !value.is_empty());
                let is_active = form.get("is_active").map(|raw| {
                    matches!(raw.as_str(), "1" | "true" | "on" | "yes")
                });

                match LOCAL_AUTH.admin_edit_user(uid, username, email, is_active).await {
                    Ok(()) => json_response(object!({ success: true })).status(StatusCode::OK),
                    Err(e) => json_response(object!({ success: false, message: e.to_string() }))
                        .status(admin_error_status(&e)),
                }
            }
            _ => json_response(object!({ success: false, message: "Method not allowed" }))
                .status(StatusCode::METHOD_NOT_ALLOWED),
        }
    }
}

endpoint! {
    APP.url("/admin/users/<uid>/password"),

    #[instrument(level = "info", skip(req))]
    pub admin_user_password <HTTP> {
        if !check_is_admin(req).await {
            return json_response(object!({ success: false, message: "Unauthorized" }))
                .status(StatusCode::UNAUTHORIZED);
        }
        if req.method() != POST {
            return json_response(object!({ success: false, message: "Method not allowed" }))
                .status(StatusCode::METHOD_NOT_ALLOWED);
        }

        let uid = match req.param("uid").and_then(|uid| uid.parse::<u32>().ok()) {
            Some(uid) => uid,
            None => {
                return json_response(object!({ success: false, message: "Invalid uid" }))
                    .status(StatusCode::BAD_REQUEST);
            }
        };
        let form = req.form_or_default().await.clone();
        let new_password = form.get_or_default("new_password");

        match LOCAL_AUTH.admin_reset_password(uid, &new_password).await {
            Ok(()) => json_response(object!({ success: true })).status(StatusCode::OK),
            Err(e) => json_response(object!({ success: false, message: e.to_string() }))
                .status(admin_error_status(&e)),
        }
    }
}

endpoint! {
    APP.url("/admin/users/<uid>/delete"),

    #[instrument(level = "info", skip(req))]
    pub admin_user_delete <HTTP> {
        if !check_is_admin(req).await {
            return json_response(object!({ success: false, message: "Unauthorized" }))
                .status(StatusCode::UNAUTHORIZED);
        }
        if req.method() != POST {
            return json_response(object!({ success: false, message: "Method not allowed" }))
                .status(StatusCode::METHOD_NOT_ALLOWED);
        }

        let uid = match req.param("uid").and_then(|uid| uid.parse::<u32>().ok()) {
            Some(uid) => uid,
            None => {
                return json_response(object!({ success: false, message: "Invalid uid" }))
                    .status(StatusCode::BAD_REQUEST);
            }
        };

        match LOCAL_AUTH.admin_delete_user(uid).await {
            Ok(()) => json_response(object!({ success: true })).status(StatusCode::OK),
            Err(e) => json_response(object!({ success: false, message: e.to_string() }))
                .status(admin_error_status(&e)),
        }
    }
}
