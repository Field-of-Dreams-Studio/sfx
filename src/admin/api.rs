use hotaru::prelude::*;
use hotaru::http::*;
use tracing::{instrument, info, error};

use crate::{local_auth::LOCAL_AUTH, APP}; 
use crate::admin::check_is_admin; 

endpoint! {
    APP.url("/admin/users"),

    #[instrument(level = "info", skip(req))]
    pub admin_users <HTTP> {
        // Authenticate request 
        if !check_is_admin(req).await {
            return json_response(object!({ success: false, message: "Unauthorized" }))
                .status(StatusCode::UNAUTHORIZED);
        } 

        match req.method() {
            GET => {
                info!(path = %req.path(), "list_admin_users handler start");
                json_response(object!({ success: true, users: LOCAL_AUTH.list_users().await }))
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
                        println!("Error creating user: {:?}", e);
                        json_response(object!({ success: false, message: e.to_string() }))
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                    }
                }
            }
            _ => json_response(object!({ success: false, message: "Method not allowed" }))
                .status(StatusCode::METHOD_NOT_ALLOWED),
        }
    }
}
