use hotaru::http::*;
use hotaru::prelude::*;

use crate::APP;
use crate::admin::check_is_admin;
use crate::local_auth::LOCAL_AUTH;
use crate::op;

fn normalize_admin_entry(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let entry = if trimmed.contains('@') {
        trimmed.to_string()
    } else {
        format!("{}@local", trimmed)
    };

    let mut parts = entry.splitn(2, '@');
    let uid = parts.next()?;
    let server = parts.next()?;
    if uid.parse::<u32>().is_err() || server.is_empty() {
        return None;
    }

    Some(entry)
}

async fn validate_admin_entry(entry: &str) -> Result<(), String> {
    let mut parts = entry.splitn(2, '@');
    let uid = parts
        .next()
        .and_then(|uid| uid.parse::<u32>().ok())
        .ok_or_else(|| "Invalid admin uid".to_string())?;
    let server = parts.next().unwrap_or_default();

    if server == "local" && LOCAL_AUTH.admin_get_user(uid).await.is_none() {
        return Err("Local user not found".to_string());
    }

    Ok(())
}

endpoint! {
    APP.url("/admin/admins/json"),

    pub admin_entries_json <HTTP> {
        if !check_is_admin(req).await {
            return json_response(object!({ success: false, message: "Unauthorized" }))
                .status(StatusCode::UNAUTHORIZED);
        }
        json_response(object!({
            success: true,
            admins: op::read_admin_entries(),
        }))
    }
}

endpoint! {
    APP.url("/admin/admins"),

    pub admin_entries <HTTP> {
        if !check_is_admin(req).await {
            return json_response(object!({ success: false, message: "Unauthorized" }))
                .status(StatusCode::UNAUTHORIZED);
        }
        if req.method() != POST {
            return json_response(object!({ success: false, message: "Method not allowed" }))
                .status(StatusCode::METHOD_NOT_ALLOWED);
        }

        let form = req.form_or_default().await.clone();
        let raw = form.get_or_default("uid");
        let entry = match normalize_admin_entry(&raw) {
            Some(entry) => entry,
            None => {
                return json_response(object!({ success: false, message: "Invalid admin entry" }))
                    .status(StatusCode::BAD_REQUEST);
            }
        };

        if let Err(message) = validate_admin_entry(&entry).await {
            return json_response(object!({ success: false, message: message }))
                .status(StatusCode::BAD_REQUEST);
        }

        match op::add_admin_entry(&entry) {
            Ok(()) => json_response(object!({ success: true, entry: entry })).status(StatusCode::OK),
            Err(err) => json_response(object!({ success: false, message: err.to_string() }))
                .status(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

endpoint! {
    APP.url("/admin/admins/<entry>/delete"),

    pub admin_entry_delete <HTTP> {
        if !check_is_admin(req).await {
            return json_response(object!({ success: false, message: "Unauthorized" }))
                .status(StatusCode::UNAUTHORIZED);
        }
        if req.method() != POST {
            return json_response(object!({ success: false, message: "Method not allowed" }))
                .status(StatusCode::METHOD_NOT_ALLOWED);
        }

        let entry = req
            .param("entry")
            .map(|entry| hotaru_lib::url_encoding::decode_url_owned(&entry))
            .and_then(|entry| normalize_admin_entry(&entry));

        let entry = match entry {
            Some(entry) => entry,
            None => {
                return json_response(object!({ success: false, message: "Invalid admin entry" }))
                    .status(StatusCode::BAD_REQUEST);
            }
        };

        match op::remove_admin_entry(&entry) {
            Ok(()) => json_response(object!({ success: true })).status(StatusCode::OK),
            Err(err) => json_response(object!({ success: false, message: err.to_string() }))
                .status(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}
