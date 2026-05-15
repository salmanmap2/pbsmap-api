use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use serde_json::json;
use chrono::NaiveDateTime;

use crate::AppState;
use crate::db;
use crate::utils;
use crate::errors;
use crate::middleware_auth::extract_claims;
use crate::handlers::office::user_has_role;

#[derive(Deserialize)]
pub struct AddMeterRequest {
    pub office_id: String,
    pub account_number: String,
    pub account_info_json: Option<serde_json::Value>,
    pub route_number: Option<String>,
    pub village: Option<String>,
    pub meter_number: Option<String>,
    pub meter_info_json: Option<serde_json::Value>,
    pub gps_location: Option<String>,
}

#[derive(Deserialize)]
pub struct EditMeterRequest {
    pub account_id: String,
    pub account_number: Option<String>,
    pub account_info_json: Option<serde_json::Value>,
    pub route_number: Option<String>,
    pub village: Option<String>,
    pub meter_number: Option<String>,
    pub meter_info_json: Option<serde_json::Value>,
    pub gps_location: Option<String>,
}

#[derive(Deserialize)]
pub struct AllMeterRequest {
    pub office_id: String,
    /// ISO datetime string: "2024-01-01T00:00:00"
    pub last_time: Option<String>,
}

/// POST /api/meter/add  (JWT required, admin or editor)
pub async fn add_meter(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<AddMeterRequest>,
) -> HttpResponse {
    let claims = match extract_claims(&req) {
        Ok(c) => c,
        Err(e) => return errors::unauthorized(&e),
    };

    let office_id = body.office_id.trim();
    if office_id.is_empty() {
        return errors::bad_request("office_id is required");
    }
    if body.account_number.trim().is_empty() {
        return errors::bad_request("account_number is required");
    }

    let office = match db::get_office_by_id(&state.db, office_id).await {
        Ok(Some(o)) => o,
        Ok(None) => return errors::not_found("Office not found"),
        Err(_) => return errors::internal_error("Database error"),
    };

    let office_user_json: serde_json::Value = office.office_user_json
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or(json!({}));

    if !user_has_role(&office_user_json, &claims.sub, &["admin", "editor"]) {
        return errors::forbidden("You must be an admin or editor of this office");
    }

    let account_id = utils::generate_account_id(office_id);
    let account_info_str = body.account_info_json.as_ref().map(|v| v.to_string());
    let meter_info_str = body.meter_info_json.as_ref().map(|v| v.to_string());

    if let Err(e) = db::create_meter(
        &state.db,
        &account_id,
        office_id,
        body.account_number.trim(),
        account_info_str.as_deref(),
        body.route_number.as_deref(),
        body.village.as_deref(),
        body.meter_number.as_deref(),
        meter_info_str.as_deref(),
        body.gps_location.as_deref(),
    ).await {
        log::error!("Add meter error: {}", e);
        return errors::internal_error("Failed to add meter");
    }

    errors::ok("Meter added", json!({ "account_id": account_id }))
}

/// POST /api/meter/edit  (JWT required, admin or editor)
pub async fn edit_meter(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<EditMeterRequest>,
) -> HttpResponse {
    let claims = match extract_claims(&req) {
        Ok(c) => c,
        Err(e) => return errors::unauthorized(&e),
    };

    let account_id = body.account_id.trim();
    if account_id.is_empty() {
        return errors::bad_request("account_id is required");
    }

    // Get meter to find office_id
    let meter = match db::get_meter_by_account_id(&state.db, account_id).await {
        Ok(Some(m)) => m,
        Ok(None) => return errors::not_found("Meter not found"),
        Err(_) => return errors::internal_error("Database error"),
    };

    let office = match db::get_office_by_id(&state.db, &meter.office_id).await {
        Ok(Some(o)) => o,
        Ok(None) => return errors::not_found("Office not found"),
        Err(_) => return errors::internal_error("Database error"),
    };

    let office_user_json: serde_json::Value = office.office_user_json
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or(json!({}));

    if !user_has_role(&office_user_json, &claims.sub, &["admin", "editor"]) {
        return errors::forbidden("You must be an admin or editor of this office");
    }

    let account_info_str = body.account_info_json.as_ref().map(|v| v.to_string());
    let meter_info_str = body.meter_info_json.as_ref().map(|v| v.to_string());

    if let Err(e) = db::update_meter(
        &state.db,
        account_id,
        body.account_number.as_deref(),
        account_info_str.as_deref(),
        body.route_number.as_deref(),
        body.village.as_deref(),
        body.meter_number.as_deref(),
        meter_info_str.as_deref(),
        body.gps_location.as_deref(),
    ).await {
        log::error!("Edit meter error: {}", e);
        return errors::internal_error("Failed to update meter");
    }

    errors::ok_simple("Meter updated successfully")
}

/// POST /api/meter/all  (JWT required, any office member)
pub async fn all_meter_list(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<AllMeterRequest>,
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

    let office_user_json: serde_json::Value = office.office_user_json
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or(json!({}));

    if !user_has_role(&office_user_json, &claims.sub, &["admin", "editor", "viewer"]) {
        return errors::forbidden("You are not a member of this office");
    }

    let after_ts = body.last_time.as_deref().and_then(|s| {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok()
    });

    let meters = match db::get_meters_by_office(&state.db, office_id, after_ts).await {
        Ok(m) => m,
        Err(e) => {
            log::error!("All meter list error: {}", e);
            return errors::internal_error("Database error");
        }
    };

    let result: Vec<serde_json::Value> = meters.iter().map(|m| {
        let account_info: serde_json::Value = m.account_info_json
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or(json!({}));
        let meter_info: serde_json::Value = m.meter_info_json
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or(json!({}));
        json!({
            "account_id": m.account_id,
            "office_id": m.office_id,
            "account_number": m.account_number,
            "account_info_json": account_info,
            "route_number": m.route_number,
            "village": m.village,
            "meter_number": m.meter_number,
            "meter_info_json": meter_info,
            "gps_location": m.gps_location,
            "updated_at": m.updated_at.map(|t| t.to_string())
        })
    }).collect();

    errors::ok("Meter list fetched", json!(result))
}
