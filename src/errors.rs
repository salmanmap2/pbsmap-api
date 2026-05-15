use actix_web::HttpResponse;
use serde_json::json;

pub fn bad_request(msg: &str) -> HttpResponse {
    HttpResponse::BadRequest().json(json!({ "success": false, "message": msg }))
}

pub fn unauthorized(msg: &str) -> HttpResponse {
    HttpResponse::Unauthorized().json(json!({ "success": false, "message": msg }))
}

pub fn forbidden(msg: &str) -> HttpResponse {
    HttpResponse::Forbidden().json(json!({ "success": false, "message": msg }))
}

pub fn not_found(msg: &str) -> HttpResponse {
    HttpResponse::NotFound().json(json!({ "success": false, "message": msg }))
}

pub fn internal_error(msg: &str) -> HttpResponse {
    HttpResponse::InternalServerError().json(json!({ "success": false, "message": msg }))
}

pub fn ok(msg: &str, data: serde_json::Value) -> HttpResponse {
    HttpResponse::Ok().json(json!({ "success": true, "message": msg, "data": data }))
}

pub fn ok_simple(msg: &str) -> HttpResponse {
    HttpResponse::Ok().json(json!({ "success": true, "message": msg }))
}
