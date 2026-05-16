use actix_web::{web, HttpResponse};
use serde_json::json;

use crate::AppState;
use crate::db;
use crate::errors;

/// GET /api/public/pbs-list
pub async fn all_pbs_list(state: web::Data<AppState>) -> HttpResponse {
    match db::get_all_pbs(&state.db).await {
        Ok(list) => errors::ok("PBS list", json!(list)),
        Err(e) => {
            log::error!("PBS list error: {}", e);
            errors::internal_error("Database error")
        }
    }
}

/// GET /api/public/offices/{pbs_id}
pub async fn offices_by_pbs(
    state: web::Data<AppState>,
    path: web::Path<u32>,
) -> HttpResponse {
    let pbs_id = path.into_inner();
    match db::get_offices_by_pbs(&state.db, pbs_id).await {
        Ok(offices) => {
            let result: Vec<serde_json::Value> = offices.iter().map(|o| {
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
            log::error!("Offices by PBS error: {}", e);
            errors::internal_error("Database error")
        }
    }
}

/// GET /api/public/office/{office_id}
pub async fn office_by_id(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let office_id = path.into_inner();
    match db::get_office_by_id(&state.db, &office_id).await {
        Ok(Some(o)) => {
            let info = o.office_info_json.clone().unwrap_or(json!({}));
            let users = o.office_user_json.clone().unwrap_or(json!({}));
            errors::ok("Office fetched", json!({
                "office_id": o.office_id,
                "pbs_id": o.pbs_id,
                "office_name": o.office_name,
                "office_info_json": info,
                "office_user_json": users
            }))
        }
        Ok(None) => errors::not_found("Office not found"),
        Err(e) => {
            log::error!("Office by ID error: {}", e);
            errors::internal_error("Database error")
        }
    }
}

/// GET /api/public/user-by-mobile/{mobile}
pub async fn user_by_mobile(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> HttpResponse {
    let mobile = path.into_inner();
    match db::find_user_by_mobile(&state.db, &mobile).await {
        Ok(Some(user)) => {
            let user_json = user.user_json.clone().unwrap_or(json!({}));
            errors::ok("User found", json!({
                "username": user.username,
                "mobile_number": user.mobile_number,
                "user_json": user_json
            }))
        }
        Ok(None) => errors::not_found("User not found"),
        Err(e) => {
            log::error!("User by mobile error: {}", e);
            errors::internal_error("Database error")
        }
    }
}
