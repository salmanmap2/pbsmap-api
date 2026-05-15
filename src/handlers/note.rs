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
pub struct AddNoteRequest {
    pub account_id: String,
    pub note_json: serde_json::Value,
}

#[derive(Deserialize)]
pub struct DeleteNoteRequest {
    pub note_id: String,
}

#[derive(Deserialize)]
pub struct GetAllNotesRequest {
    pub account_id: String,
    pub last_time: Option<String>,
}

/// POST /api/note/add  (JWT required, admin/editor/viewer)
pub async fn add_note(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<AddNoteRequest>,
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

    let office_user_json: serde_json::Value = office.office_user_json
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or(json!({}));

    if !user_has_role(&office_user_json, &claims.sub, &["admin", "editor", "viewer"]) {
        return errors::forbidden("You are not a member of this office");
    }

    let note_id = Uuid::new_v4().to_string();

    if let Err(e) = db::create_note(
        &state.db,
        &note_id,
        account_id,
        &body.note_json.to_string(),
        &claims.sub,
    ).await {
        log::error!("Add note error: {}", e);
        return errors::internal_error("Failed to add note");
    }

    errors::ok("Note added", json!({ "note_id": note_id }))
}

/// POST /api/note/delete  (JWT required, note creator only)
pub async fn delete_note(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<DeleteNoteRequest>,
) -> HttpResponse {
    let claims = match extract_claims(&req) {
        Ok(c) => c,
        Err(e) => return errors::unauthorized(&e),
    };

    let note_id = body.note_id.trim();
    if note_id.is_empty() {
        return errors::bad_request("note_id is required");
    }

    match db::delete_note(&state.db, note_id, &claims.sub).await {
        Ok(true) => errors::ok_simple("Note deleted"),
        Ok(false) => errors::forbidden("Note not found or you are not the creator"),
        Err(e) => {
            log::error!("Delete note error: {}", e);
            errors::internal_error("Database error")
        }
    }
}

/// POST /api/note/all  (JWT required, office member)
pub async fn get_all_notes(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<GetAllNotesRequest>,
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

    let notes = match db::get_notes_by_account(&state.db, account_id, after_ts).await {
        Ok(n) => n,
        Err(e) => {
            log::error!("Get notes error: {}", e);
            return errors::internal_error("Database error");
        }
    };

    let result: Vec<serde_json::Value> = notes.iter().map(|n| {
        let note_data: serde_json::Value = n.note_json
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or(json!({}));
        json!({
            "note_id": n.note_id,
            "account_id": n.account_id,
            "note_json": note_data,
            "note_creator": n.note_creator,
            "timestamp": n.timestamp.map(|t| t.to_string())
        })
    }).collect();

    errors::ok("Notes fetched", json!(result))
}
