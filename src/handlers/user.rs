use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use serde_json::json;

use crate::AppState;
use crate::db;
use crate::utils;
use crate::errors;
use crate::middleware_auth::extract_claims;

#[derive(Deserialize)]
pub struct UpdateProfileRequest {
    pub mobile_number: Option<String>,
    pub full_name: Option<String>,
    pub profile_pic_url: Option<String>,
    pub active_office: Option<String>,
    pub designation: Option<String>,
    pub pbs_name: Option<String>,
    pub office_name: Option<String>,
    pub whatsapp: Option<String>,
    pub facebook: Option<String>,
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Deserialize)]
pub struct JoinOfficeRequest {
    pub office_id: String,
}

/// GET /api/user/profile  (JWT required)
pub async fn get_profile(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> HttpResponse {
    let claims = match extract_claims(&req) {
        Ok(c) => c,
        Err(e) => return errors::unauthorized(&e),
    };

    let user = match db::find_user_by_username(&state.db, &claims.sub).await {
        Ok(Some(u)) => u,
        Ok(None) => return errors::not_found("User not found"),
        Err(_) => return errors::internal_error("Database error"),
    };

    let user_json: serde_json::Value = user.user_json.clone().unwrap_or(json!({}));

    errors::ok("Profile fetched", json!({
        "username": user.username,
        "email": user.email,
        "mobile_number": user.mobile_number,
        "user_json": user_json,
        "active_office": user.active_office,
        "user_api_key": user.user_api_key
    }))
}

/// PUT /api/user/profile  (JWT required)
pub async fn update_profile(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<UpdateProfileRequest>,
) -> HttpResponse {
    let claims = match extract_claims(&req) {
        Ok(c) => c,
        Err(e) => return errors::unauthorized(&e),
    };

    let user = match db::find_user_by_username(&state.db, &claims.sub).await {
        Ok(Some(u)) => u,
        Ok(None) => return errors::not_found("User not found"),
        Err(_) => return errors::internal_error("Database error"),
    };

    // Merge user_json fields
    let mut user_json: serde_json::Value = user.user_json.clone().unwrap_or(json!({}));

    if let Some(ref full_name) = body.full_name {
        user_json["full_name"] = json!(full_name);
    }
    if let Some(ref pic) = body.profile_pic_url {
        user_json["profile_pic_url"] = json!(pic);
    }
    if let Some(ref v) = body.designation {
        user_json["designation"] = json!(v);
    }
    if let Some(ref v) = body.pbs_name {
        user_json["pbs_name"] = json!(v);
    }
    if let Some(ref v) = body.office_name {
        user_json["office_name"] = json!(v);
    }
    if let Some(ref v) = body.whatsapp {
        user_json["whatsapp"] = json!(v);
    }
    if let Some(ref v) = body.facebook {
        user_json["facebook"] = json!(v);
    }

    let user_json_str = user_json.to_string();

    if let Err(e) = db::update_user_profile(
        &state.db,
        &claims.sub,
        body.mobile_number.as_deref(),
        Some(&user_json_str),
        body.active_office.as_deref(),
    ).await {
        log::error!("Update profile error: {}", e);
        return errors::internal_error("Failed to update profile");
    }

    errors::ok_simple("Profile updated successfully")
}

/// POST /api/user/change-password  (JWT required)
pub async fn change_password(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<ChangePasswordRequest>,
) -> HttpResponse {
    let claims = match extract_claims(&req) {
        Ok(c) => c,
        Err(e) => return errors::unauthorized(&e),
    };

    if body.new_password.len() < 6 {
        return errors::bad_request("New password must be at least 6 characters");
    }

    let user = match db::find_user_by_username(&state.db, &claims.sub).await {
        Ok(Some(u)) => u,
        Ok(None) => return errors::not_found("User not found"),
        Err(_) => return errors::internal_error("Database error"),
    };

    let stored_hash = match &user.password {
        Some(h) => h,
        None => return errors::bad_request("This account uses Google login. Set a password via forgot-password flow."),
    };

    if !utils::verify_password(&body.old_password, stored_hash) {
        return errors::unauthorized("Old password is incorrect");
    }

    let new_hash = match utils::hash_password(&body.new_password) {
        Ok(h) => h,
        Err(_) => return errors::internal_error("Failed to hash password"),
    };

    if let Err(e) = db::update_user_password(&state.db, &claims.sub, &new_hash).await {
        log::error!("Change password error: {}", e);
        return errors::internal_error("Failed to change password");
    }

    errors::ok_simple("Password changed successfully")
}

/// POST /api/user/regenerate-api-key  (JWT required)
pub async fn regenerate_api_key(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> HttpResponse {
    let claims = match extract_claims(&req) {
        Ok(c) => c,
        Err(e) => return errors::unauthorized(&e),
    };

    let new_key = utils::generate_api_key();

    if let Err(e) = db::update_user_api_key(&state.db, &claims.sub, &new_key).await {
        log::error!("Regenerate API key error: {}", e);
        return errors::internal_error("Failed to regenerate API key");
    }

    errors::ok("API key regenerated", json!({ "user_api_key": new_key }))
}

/// POST /api/user/join-office  (JWT required)
/// Adds user to office's pending_users list
pub async fn join_office(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<JoinOfficeRequest>,
) -> HttpResponse {
    let claims = match extract_claims(&req) {
        Ok(c) => c,
        Err(e) => return errors::unauthorized(&e),
    };

    let office_id = body.office_id.trim();
    if office_id.is_empty() {
        return errors::bad_request("office_id is required");
    }

    let office = match db::get_office_by_id(&state.db, office_id).await {
        Ok(Some(o)) => o,
        Ok(None) => return errors::not_found("Office not found"),
        Err(_) => return errors::internal_error("Database error"),
    };

    let mut office_user_json: serde_json::Value = office.office_user_json.clone().unwrap_or(json!({
            "pending_users": [],
            "admin_users": [],
            "editor_users": [],
            "viewer_users": []
        }));

    let username = &claims.sub;

    // Check if already in any list
    for key in &["admin_users", "editor_users", "viewer_users", "pending_users"] {
        if let Some(arr) = office_user_json[key].as_array() {
            if arr.iter().any(|v| v.as_str() == Some(username)) {
                return errors::bad_request("You are already associated with this office");
            }
        }
    }

    // Add to pending
    if let Some(arr) = office_user_json["pending_users"].as_array_mut() {
        arr.push(json!(username));
    }

    if let Err(e) = db::update_office(
        &state.db,
        office_id,
        None,
        None,
        Some(&office_user_json.to_string()),
    ).await {
        log::error!("Join office error: {}", e);
        return errors::internal_error("Failed to join office");
    }

    errors::ok_simple("Join request submitted. Waiting for admin approval.")
}
