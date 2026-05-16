use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use serde_json::json;
use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::AppState;
use crate::db;
use crate::errors;
use crate::middleware_auth::extract_claims;
use crate::handlers::office::user_has_role;

#[derive(Deserialize)]
pub struct NewReadingRequest {
    pub account_id: String,
    pub reading_json: serde_json::Value,
}

#[derive(Deserialize)]
pub struct EditReadingRequest {
    pub reading_id: String,
    pub account_id: Option<String>,
    pub date_time: Option<String>,
    pub reading_json: Option<serde_json::Value>,
    pub reader_username: Option<String>,
}

#[derive(Deserialize)]
pub struct GetAllReadingsRequest {
    pub office_id: String,
    pub timestamp_start: Option<String>,
    pub timestamp_end: Option<String>,
}

/// POST /api/reading/new  (JWT required, admin/editor/viewer)
pub async fn new_reading(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<NewReadingRequest>,
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
        Ok(None) => return errors::not_found("Account not found"),
        Err(_) => return errors::internal_error("Database error"),
    };

    let office = match db::get_office_by_id(&state.db, &meter.office_id).await {
        Ok(Some(o)) => o,
        Ok(None) => return errors::not_found("Office not found"),
        Err(_) => return errors::internal_error("Database error"),
    };

    let office_user_json: serde_json::Value = office.office_user_json.clone().unwrap_or(json!({}));

    if !user_has_role(&office_user_json, &claims.sub, &["admin", "editor", "viewer"]) {
        return errors::forbidden("You are not a member of this office");
    }

    let reading_id = Uuid::new_v4().to_string();

    if let Err(e) = db::create_reading(
        &state.db,
        &reading_id,
        account_id,
        &body.reading_json.to_string(),
        &claims.sub,
    ).await {
        log::error!("New reading error: {}", e);
        return errors::internal_error("Failed to save reading");
    }

    errors::ok("Reading saved", json!({ "reading_id": reading_id }))
}

/// POST /api/reading/edit  (JWT required, admin/editor)
pub async fn edit_reading(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<EditReadingRequest>,
) -> HttpResponse {
    let claims = match extract_claims(&req) {
        Ok(c) => c,
        Err(e) => return errors::unauthorized(&e),
    };

    let reading_id = body.reading_id.trim();
    if reading_id.is_empty() {
        return errors::bad_request("reading_id is required");
    }

    // We need to find the office from the reading's account_id
    // First get the reading to find account_id
    let account_id_to_check = body.account_id.as_deref().unwrap_or("");

    // If account_id provided, verify office membership
    if !account_id_to_check.is_empty() {
        if let Ok(Some(meter)) = db::get_meter_by_account_id(&state.db, account_id_to_check).await {
            if let Ok(Some(office)) = db::get_office_by_id(&state.db, &meter.office_id).await {
                let office_user_json: serde_json::Value = office.office_user_json.clone().unwrap_or(json!({}));

                if !user_has_role(&office_user_json, &claims.sub, &["admin", "editor"]) {
                    return errors::forbidden("You must be an admin or editor");
                }
            }
        }
    }

    let reading_json_str = body.reading_json.as_ref().map(|v| v.to_string());

    match db::update_reading(
        &state.db,
        reading_id,
        body.account_id.as_deref(),
        body.date_time.as_deref(),
        reading_json_str.as_deref(),
        body.reader_username.as_deref(),
    ).await {
        Ok(true) => errors::ok_simple("Reading updated"),
        Ok(false) => errors::not_found("Reading not found"),
        Err(e) => {
            log::error!("Edit reading error: {}", e);
            errors::internal_error("Database error")
        }
    }
}

/// POST /api/reading/all  (JWT required, office member)
pub async fn get_all_readings(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<GetAllReadingsRequest>,
) -> HttpResponse {
    let claims = match extract_claims(&req) {
        Ok(c) => c,
        Err(e) => return errors::unauthorized(&e),
    };

    let office_id = body.office_id.trim();
    if office_id.len() < 5 {
        return errors::bad_request("office_id must be at least 5 digits");
    }

    let office = match db::get_office_by_id(&state.db, office_id).await {
        Ok(Some(o)) => o,
        Ok(None) => return errors::not_found("Office not found"),
        Err(_) => return errors::internal_error("Database error"),
    };

    let office_user_json: serde_json::Value = office.office_user_json.clone().unwrap_or(json!({}));

    if !user_has_role(&office_user_json, &claims.sub, &["admin", "editor", "viewer"]) {
        return errors::forbidden("You are not a member of this office");
    }

    let ts_start = body.timestamp_start.as_deref().and_then(|s| {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok()
    });
    let ts_end = body.timestamp_end.as_deref().and_then(|s| {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S").ok()
    });

    // Use first 5 chars of office_id as prefix for account_id matching
    let prefix = &office_id[..5];

    let readings = match db::get_readings_by_office(&state.db, prefix, ts_start, ts_end).await {
        Ok(r) => r,
        Err(e) => {
            log::error!("Get readings error: {}", e);
            return errors::internal_error("Database error");
        }
    };

    let result: Vec<serde_json::Value> = readings.iter().map(|r| {
        let reading_data = r.reading_json.clone().unwrap_or(json!({}));
        json!({
            "reading_id": r.reading_id,
            "account_id": r.account_id,
            "date_time": r.date_time.map(|t| t.to_string()),
            "reading_json": reading_data,
            "reader_username": r.reader_username
        })
    }).collect();

    errors::ok("Readings fetched", json!(result))
}
