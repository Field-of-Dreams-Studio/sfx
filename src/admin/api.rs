use starberry::prelude::*;
use crate::{local_auth::LOCAL_AUTH, APP}; 
use crate::admin::check_is_admin; 
use tracing::{instrument, info, error};

/// Examples:
/// ```
/// use serde_json;
/// use crate::local_auth::models::AdminCreateUserRequest;
/// let req: AdminCreateUserRequest = serde_json::from_str(r#"{"username":"u","password":"p"}"#).unwrap();
/// assert_eq!(req.username, "u");
/// ```
#[instrument(level = "info", skip(req))]
#[url(APP.lit_url("/admin/users"))]
async fn admin_users(mut req: HttpReqCtx) -> HttpResponse {
    // Authenticate request 
    if !check_is_admin(&mut req).await {
        return json_response(object!({ success: false, message: "Unauthorized" })).status(StatusCode::UNAUTHORIZED);
    } 

    match req.meta().method() {
        GET => {
            info!(path = %req.meta().path(), "list_admin_users handler start");
            json_response(object!({ success: true, users: LOCAL_AUTH.list_users().await })).status(StatusCode::OK) 
        }
        POST => {
            info!(path = %req.meta().path(), "create_admin_user handler start"); 
            let form = req.form_or_default().await.clone();
            let username = form.get_or_default("username"); 
            let password = form.get_or_default("password"); 
            let email = form.get_or_default("email"); 
            match LOCAL_AUTH.register_user(&username, &email, &password).await {
                Ok(()) => json_response(object!({ success: true, username: username })).status(StatusCode::CREATED),
                Err(e) => {
                    println!("Error creating user: {:?}", e);
                    json_response(object!({ success: false, message: e.to_string() })).status(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        _ => json_response(object!({ success: false, message: "Method not allowed" })).status(StatusCode::METHOD_NOT_ALLOWED),
    }
}

