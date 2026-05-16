use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::AppState;
use crate::db;
use crate::utils;
use crate::errors;
use crate::middleware_auth::extract_super_api_key;
use crate::handlers::office::{add_user_to_role, remove_user_from_role, ensure_user_json_keys};

/// Validate super admin API key from X-Api-Key header
async fn require_super_admin(req: &HttpRequest, state: &AppState) -> Result<(), HttpResponse> {
    let key = match extract_super_api_key(req) {
        Some(k) => k,
        None => return Err(errors::unauthorized("X-Api-Key header is required")),
    };
    match db::validate_super_admin_key(&state.db, &key).await {
        Ok(true) => Ok(()),
        Ok(false) => Err(errors::forbidden("Invalid super admin API key")),
        Err(e) => {
            log::error!("Super admin key validation error: {}", e);
            Err(errors::internal_error("Database error"))
        }
    }
}

// ─── Request structs ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateOfficeRequest {
    pub pbs_id: u32,
    pub office_name: String,
}

#[derive(Deserialize)]
pub struct EditOfficeRequest {
    pub office_id: String,
    pub office_name: Option<String>,
    pub office_info_json: Option<Value>,
    pub office_user_json: Option<Value>,
}

#[derive(Deserialize)]
pub struct UserManageRequest {
    pub office_id: String,
    pub add_admin: Option<String>,
    pub add_editor: Option<String>,
    pub add_viewer: Option<String>,
    pub remove_admin: Option<String>,
    pub remove_editor: Option<String>,
    pub remove_viewer: Option<String>,
    pub remove_pending: Option<String>,
}

// ─── Handlers ────────────────────────────────────────────────────────────────

/// POST /api/dev/create-office  (Super Admin only)
pub async fn create_office(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<CreateOfficeRequest>,
) -> HttpResponse {
    if let Err(resp) = require_super_admin(&req, &state).await {
        return resp;
    }

    let office_name = body.office_name.trim();
    if office_name.is_empty() {
        return errors::bad_request("office_name is required");
    }

    match db::get_pbs_by_id(&state.db, body.pbs_id).await {
        Ok(None) => return errors::not_found("PBS not found"),
        Err(_) => return errors::internal_error("Database error"),
        _ => {}
    }

    let seq = match db::get_next_office_seq(&state.db, body.pbs_id).await {
        Ok(s) => s,
        Err(_) => return errors::internal_error("Failed to generate office ID"),
    };

    if seq > 99 {
        return errors::bad_request("Maximum offices (99) reached for this PBS");
    }

    let office_id = utils::generate_office_id(body.pbs_id, seq);

    if let Err(e) = db::create_office(&state.db, &office_id, body.pbs_id, office_name).await {
        log::error!("Create office error: {}", e);
        return errors::internal_error("Failed to create office");
    }

    errors::ok("Office created", json!({
        "office_id": office_id,
        "pbs_id": body.pbs_id,
        "office_name": office_name
    }))
}

/// GET /api/dev/all-office/{pbs_id}  (Super Admin only)
pub async fn all_office(
    req: HttpRequest,
    state: web::Data<AppState>,
    path: web::Path<u32>,
) -> HttpResponse {
    if let Err(resp) = require_super_admin(&req, &state).await {
        return resp;
    }

    let pbs_id = path.into_inner();
    match db::get_offices_by_pbs(&state.db, pbs_id).await {
        Ok(offices) => {
            let result: Vec<Value> = offices.iter().map(|o| {
                let info = o.office_info_json.clone().unwrap_or(json!({}));
                let users = o.office_user_json.clone().unwrap_or(json!({}));
                json!({
                    "office_id": o.office_id,
                    "pbs_id": o.pbs_id,
                    "office_name": o.office_name,
                    "office_info_json": info,
                    "office_user_json": users
                })
            }).collect();
            errors::ok("Offices fetched", json!(result))
        }
        Err(e) => {
            log::error!("All office error: {}", e);
            errors::internal_error("Database error")
        }
    }
}

/// POST /api/dev/edit-office  (Super Admin only)
pub async fn edit_office(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<EditOfficeRequest>,
) -> HttpResponse {
    if let Err(resp) = require_super_admin(&req, &state).await {
        return resp;
    }

    let office_id = body.office_id.trim();
    if office_id.is_empty() {
        return errors::bad_request("office_id is required");
    }

    match db::get_office_by_id(&state.db, office_id).await {
        Ok(None) => return errors::not_found("Office not found"),
        Err(_) => return errors::internal_error("Database error"),
        _ => {}
    }

    let info_str = body.office_info_json.as_ref().map(|v| v.to_string());
    let users_str = body.office_user_json.as_ref().map(|v| v.to_string());

    if let Err(e) = db::update_office(
        &state.db,
        office_id,
        body.office_name.as_deref(),
        info_str.as_deref(),
        users_str.as_deref(),
    ).await {
        log::error!("Edit office error: {}", e);
        return errors::internal_error("Failed to update office");
    }

    errors::ok_simple("Office updated successfully")
}

/// POST /api/dev/user-manage  (Super Admin only)
pub async fn user_manage(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<UserManageRequest>,
) -> HttpResponse {
    if let Err(resp) = require_super_admin(&req, &state).await {
        return resp;
    }

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

    if let Some(ref u) = body.add_admin    { add_user_to_role(&mut users, u, "admin_users"); }
    if let Some(ref u) = body.add_editor   { add_user_to_role(&mut users, u, "editor_users"); }
    if let Some(ref u) = body.add_viewer   { add_user_to_role(&mut users, u, "viewer_users"); }
    if let Some(ref u) = body.remove_admin   { remove_user_from_role(&mut users, u, "admin_users"); }
    if let Some(ref u) = body.remove_editor  { remove_user_from_role(&mut users, u, "editor_users"); }
    if let Some(ref u) = body.remove_viewer  { remove_user_from_role(&mut users, u, "viewer_users"); }
    if let Some(ref u) = body.remove_pending { remove_user_from_role(&mut users, u, "pending_users"); }

    if let Err(e) = db::update_office(&state.db, office_id, None, None, Some(&users.to_string())).await {
        log::error!("User manage error: {}", e);
        return errors::internal_error("Failed to update office users");
    }

    errors::ok("Office users updated", users)
}
