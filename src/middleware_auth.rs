use actix_web::HttpRequest;
use crate::models::Claims;
use crate::utils::validate_jwt;

/// Extract and validate JWT from Authorization header
/// Returns Claims on success
pub fn extract_claims(req: &HttpRequest) -> Result<Claims, String> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| "Missing Authorization header".to_string())?;

    let token = if auth_header.starts_with("Bearer ") {
        &auth_header[7..]
    } else {
        auth_header
    };

    validate_jwt(token).map_err(|e| format!("Invalid token: {}", e))
}

/// Extract super admin API key from X-Api-Key header
pub fn extract_super_api_key(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("X-Api-Key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}
