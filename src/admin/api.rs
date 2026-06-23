use hotaru::prelude::*;
use hotaru::http::*;
use tracing::{instrument, info, error};

use crate::{local_auth::{LOCAL_AUTH, fop::FopError}, APP};
use crate::admin::check_is_admin;

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
                let users = LOCAL_AUTH.list_users().await;
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
                        let status = match &e {
                            FopError::UserNameNotValid
                            | FopError::EmailNotValid
                            | FopError::PasswordMismatch => StatusCode::BAD_REQUEST,
                            FopError::TooManyRequest => StatusCode::TOO_MANY_REQUESTS,
                            _ => {
                                error!(?e, "create_admin_user internal error");
                                StatusCode::INTERNAL_SERVER_ERROR
                            }
                        };
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
