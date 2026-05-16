use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::AppState;
use crate::db;
use crate::errors;
use crate::middleware_auth::extract_claims;

#[derive(Deserialize)]
pub struct UserChangeRequest {
    pub office_id: String,
    /// Add user to a role (removes from all other roles first)
    pub add_username: Option<String>,
    pub role: Option<String>,           // "admin" | "editor" | "viewer"
    /// Approve pending user → assign role
    pub approve_username: Option<String>,
    pub approve_role: Option<String>,
    /// Remove user from a specific role
    pub remove_username: Option<String>,
    pub remove_role: Option<String>,    // "admin" | "editor" | "viewer" | "pending"
}

/// POST /api/office/user-change  (JWT required, must be office admin)
pub async fn user_change(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<UserChangeRequest>,
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

    let mut users: Value = office.office_user_json.clone().unwrap_or(json!({}));

    ensure_user_json_keys(&mut users);

    // Requester must be admin
    let is_admin = users["admin_users"]
        .as_array()
        .map(|arr| arr.iter().any(|v| v.as_str() == Some(&claims.sub)))
        .unwrap_or(false);

    if !is_admin {
        return errors::forbidden("You are not an admin of this office");
    }

    // Add user to role
    if let (Some(ref add_user), Some(ref role)) = (&body.add_username, &body.role) {
        let role_key = match role.as_str() {
            "admin"  => "admin_users",
            "editor" => "editor_users",
            "viewer" => "viewer_users",
            _ => return errors::bad_request("Invalid role. Use: admin, editor, viewer"),
        };
        add_user_to_role(&mut users, add_user, role_key);
    }

    // Approve pending user
    if let (Some(ref approve_user), Some(ref approve_role)) = (&body.approve_username, &body.approve_role) {
        let role_key = match approve_role.as_str() {
            "admin"  => "admin_users",
            "editor" => "editor_users",
            "viewer" => "viewer_users",
            _ => return errors::bad_request("Invalid role"),
        };
        // Remove from pending first
        if let Some(arr) = users["pending_users"].as_array_mut() {
            arr.retain(|v| v.as_str() != Some(approve_user.as_str()));
        }
        // Add to role if not already there
        if let Some(arr) = users[role_key].as_array_mut() {
            if !arr.iter().any(|v| v.as_str() == Some(approve_user.as_str())) {
                arr.push(Value::String(approve_user.clone()));
            }
        }
    }

    // Remove user from role
    if let (Some(ref remove_user), Some(ref remove_role)) = (&body.remove_username, &body.remove_role) {
        let role_key = match remove_role.as_str() {
            "admin"   => "admin_users",
            "editor"  => "editor_users",
            "viewer"  => "viewer_users",
            "pending" => "pending_users",
            _ => return errors::bad_request("Invalid role"),
        };
        remove_user_from_role(&mut users, remove_user, role_key);
    }

    if let Err(e) = db::update_office(&state.db, office_id, None, None, Some(&users.to_string())).await {
        log::error!("User change error: {}", e);
        return errors::internal_error("Failed to update office users");
    }

    errors::ok("Office users updated", users)
}

// ─── Shared helpers (also used by developer.rs via pub) ──────────────────────

pub fn add_user_to_role(obj: &mut Value, username: &str, role_key: &str) {
    for key in &["admin_users", "editor_users", "viewer_users", "pending_users"] {
        if let Some(arr) = obj[key].as_array_mut() {
            arr.retain(|v| v.as_str() != Some(username));
        }
    }
    if let Some(arr) = obj[role_key].as_array_mut() {
        if !arr.iter().any(|v| v.as_str() == Some(username)) {
            arr.push(Value::String(username.to_string()));
        }
    }
}

pub fn remove_user_from_role(obj: &mut Value, username: &str, role_key: &str) {
    if let Some(arr) = obj[role_key].as_array_mut() {
        arr.retain(|v| v.as_str() != Some(username));
    }
}

pub fn ensure_user_json_keys(obj: &mut Value) {
    for key in &["pending_users", "admin_users", "editor_users", "viewer_users"] {
        if obj[key].is_null() || !obj[key].is_array() {
            obj[key] = json!([]);
        }
    }
}

/// Check if username has any of the given roles in office_user_json
pub fn user_has_role(office_user_json: &Value, username: &str, roles: &[&str]) -> bool {
    for role in roles {
        let key = match *role {
            "admin"  => "admin_users",
            "editor" => "editor_users",
            "viewer" => "viewer_users",
            _ => continue,
        };
        if let Some(arr) = office_user_json[key].as_array() {
            if arr.iter().any(|v| v.as_str() == Some(username)) {
                return true;
            }
        }
    }
    false
}
