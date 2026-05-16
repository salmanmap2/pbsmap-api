use actix_web::{web, HttpResponse};
use serde::Deserialize;
use serde_json::{json, Value};
use chrono::Utc;
use std::env;

use crate::AppState;
use crate::db;
use crate::utils;
use crate::errors;

// ─── Request / Response structs ──────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    pub full_name: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    /// Can be email, username, or mobile_number
    pub identifier: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct GoogleLoginRequest {
    /// Google OAuth access token (from frontend)
    pub google_token: String,
}

#[derive(Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Deserialize)]
pub struct VerifyOtpRequest {
    pub email: String,
    pub otp: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordRequest {
    pub email: String,
    pub otp: String,
    pub new_password: String,
}

// ─── Handlers ────────────────────────────────────────────────────────────────

/// POST /api/auth/signup
/// Register with email + password. No verification required.
pub async fn signup(
    state: web::Data<AppState>,
    body: web::Json<SignupRequest>,
) -> HttpResponse {
    let email = body.email.trim().to_lowercase();
    let password = body.password.trim();

    if email.is_empty() || password.is_empty() {
        return errors::bad_request("Email and password are required");
    }
    if password.len() < 6 {
        return errors::bad_request("Password must be at least 6 characters");
    }

    // Check if email already exists
    match db::find_user_by_email(&state.db, &email).await {
        Ok(Some(_)) => return errors::bad_request("Email already registered"),
        Err(e) => {
            log::error!("DB error: {}", e);
            return errors::internal_error("Database error");
        }
        _ => {}
    }

    let username = utils::generate_username(Some(&email));
    let password_hash = match utils::hash_password(password) {
        Ok(h) => h,
        Err(_) => return errors::internal_error("Failed to hash password"),
    };
    let api_key = utils::generate_api_key();

    // Build user_json with full_name if provided
    let user_json = json!({
        "full_name": body.full_name.clone().unwrap_or_default(),
        "profile_pic_url": ""
    });

    if let Err(e) = db::create_user(
        &state.db,
        &username,
        Some(&email),
        Some(&password_hash),
        None,
        Some(&user_json.to_string()),
        &api_key,
    ).await {
        log::error!("Create user error: {}", e);
        return errors::internal_error("Failed to create user");
    }

    let token = match utils::create_jwt(&username) {
        Ok(t) => t,
        Err(_) => return errors::internal_error("Failed to create token"),
    };

    errors::ok("Signup successful", json!({
        "username": username,
        "email": email,
        "token": token,
        "user_api_key": api_key
    }))
}

/// POST /api/auth/login
/// Login with email/username/mobile + password
pub async fn login(
    state: web::Data<AppState>,
    body: web::Json<LoginRequest>,
) -> HttpResponse {
    let identifier = body.identifier.trim().to_lowercase();
    let password = body.password.trim();

    if identifier.is_empty() || password.is_empty() {
        return errors::bad_request("Identifier and password are required");
    }

    // Try email first, then username, then mobile
    let user_opt = if identifier.contains('@') {
        db::find_user_by_email(&state.db, &identifier).await
    } else if identifier.chars().all(|c| c.is_ascii_digit()) {
        db::find_user_by_mobile(&state.db, &identifier).await
    } else {
        db::find_user_by_username(&state.db, &identifier).await
    };

    let user = match user_opt {
        Ok(Some(u)) => u,
        Ok(None) => return errors::unauthorized("Invalid credentials"),
        Err(e) => {
            log::error!("DB error: {}", e);
            return errors::internal_error("Database error");
        }
    };

    // Check password
    let stored_hash = match &user.password {
        Some(h) => h,
        None => return errors::unauthorized("This account uses Google login. Please use Google sign-in."),
    };

    if !utils::verify_password(password, stored_hash) {
        return errors::unauthorized("Invalid credentials");
    }

    let token = match utils::create_jwt(&user.username) {
        Ok(t) => t,
        Err(_) => return errors::internal_error("Failed to create token"),
    };

    let user_json: Value = user.user_json.clone().unwrap_or(json!({}));

    errors::ok("Login successful", json!({
        "username": user.username,
        "email": user.email,
        "token": token,
        "user_json": user_json,
        "active_office": user.active_office,
        "user_api_key": user.user_api_key
    }))
}

/// POST /api/auth/login/google
/// Login or register with Google OAuth token
pub async fn google_login(
    state: web::Data<AppState>,
    body: web::Json<GoogleLoginRequest>,
) -> HttpResponse {
    // Verify Google token by calling Google's tokeninfo endpoint
    let google_token = body.google_token.trim();
    if google_token.is_empty() {
        return errors::bad_request("Google token is required");
    }

    let google_info = match verify_google_token(google_token).await {
        Ok(info) => info,
        Err(e) => {
            log::error!("Google token verification failed: {}", e);
            return errors::unauthorized("Invalid Google token");
        }
    };

    let google_id = match google_info.get("sub").and_then(|v| v.as_str()) {
        Some(id) => id.to_string(),
        None => return errors::unauthorized("Invalid Google token: missing sub"),
    };
    let email = google_info.get("email").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let full_name = google_info.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let picture = google_info.get("picture").and_then(|v| v.as_str()).unwrap_or("").to_string();

    // 1. Check if user exists by google_id
    if let Ok(Some(user)) = db::find_user_by_google_id(&state.db, &google_id).await {
        let token = match utils::create_jwt(&user.username) {
            Ok(t) => t,
            Err(_) => return errors::internal_error("Failed to create token"),
        };
        let user_json: Value = user.user_json.clone().unwrap_or(json!({}));
        return errors::ok("Login successful", json!({
            "username": user.username,
            "email": user.email,
            "token": token,
            "user_json": user_json,
            "active_office": user.active_office,
            "user_api_key": user.user_api_key
        }));
    }

    // 2. Check if user exists by email (link Google account)
    if !email.is_empty() {
        if let Ok(Some(user)) = db::find_user_by_email(&state.db, &email).await {
            // Link google_id to existing account
            let _ = db::update_google_id_for_email(&state.db, &email, &google_id).await;
            let token = match utils::create_jwt(&user.username) {
                Ok(t) => t,
                Err(_) => return errors::internal_error("Failed to create token"),
            };
            let user_json: Value = user.user_json.clone().unwrap_or(json!({}));
            return errors::ok("Login successful (Google linked to existing account)", json!({
                "username": user.username,
                "email": user.email,
                "token": token,
                "user_json": user_json,
                "active_office": user.active_office,
                "user_api_key": user.user_api_key
            }));
        }
    }

    // 3. Create new user
    let username = utils::generate_username(if email.is_empty() { None } else { Some(&email) });
    let api_key = utils::generate_api_key();
    let user_json = json!({
        "full_name": full_name,
        "profile_pic_url": picture
    });

    if let Err(e) = db::create_user(
        &state.db,
        &username,
        if email.is_empty() { None } else { Some(&email) },
        None,
        Some(&google_id),
        Some(&user_json.to_string()),
        &api_key,
    ).await {
        log::error!("Create user error: {}", e);
        return errors::internal_error("Failed to create user");
    }

    let token = match utils::create_jwt(&username) {
        Ok(t) => t,
        Err(_) => return errors::internal_error("Failed to create token"),
    };

    errors::ok("Signup with Google successful", json!({
        "username": username,
        "email": email,
        "token": token,
        "user_json": user_json,
        "user_api_key": api_key
    }))
}

/// POST /api/auth/forgot-password
/// Send OTP to email via Gmail SMTP
pub async fn forgot_password(
    state: web::Data<AppState>,
    body: web::Json<ForgotPasswordRequest>,
) -> HttpResponse {
    let email = body.email.trim().to_lowercase();
    if email.is_empty() {
        return errors::bad_request("Email is required");
    }

    let user = match db::find_user_by_email(&state.db, &email).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            // Don't reveal if email exists
            return errors::ok_simple("If this email is registered, an OTP has been sent");
        }
        Err(e) => {
            log::error!("DB error: {}", e);
            return errors::internal_error("Database error");
        }
    };

    let otp = utils::generate_otp();
    let otp_time = Utc::now().timestamp();

    // Update app_json with OTP
    let current_app_json: Value = user.app_json.clone().unwrap_or(json!({}));

    let mut app_json = current_app_json;
    app_json["temp_reset_otp"] = json!(otp);
    app_json["temp_reset_otp_time"] = json!(otp_time);

    if let Err(e) = db::update_user_app_json(&state.db, &user.username, &app_json.to_string()).await {
        log::error!("Update app_json error: {}", e);
        return errors::internal_error("Failed to save OTP");
    }

    // Send email
    if let Err(e) = send_otp_email(&email, &otp).await {
        log::error!("Email send error: {}", e);
        return errors::internal_error("Failed to send OTP email");
    }

    errors::ok_simple("OTP sent to your email. Valid for 10 minutes.")
}

/// POST /api/auth/verify-otp
/// Verify OTP (check only, does not reset password)
pub async fn verify_otp(
    state: web::Data<AppState>,
    body: web::Json<VerifyOtpRequest>,
) -> HttpResponse {
    let email = body.email.trim().to_lowercase();
    let otp = body.otp.trim();

    let user = match db::find_user_by_email(&state.db, &email).await {
        Ok(Some(u)) => u,
        Ok(None) => return errors::bad_request("Invalid email or OTP"),
        Err(_) => return errors::internal_error("Database error"),
    };

    match validate_otp(&user, otp) {
        Ok(_) => errors::ok_simple("OTP verified"),
        Err(msg) => errors::bad_request(&msg),
    }
}

/// POST /api/auth/reset-password
/// Verify OTP and reset password
pub async fn reset_password(
    state: web::Data<AppState>,
    body: web::Json<ResetPasswordRequest>,
) -> HttpResponse {
    let email = body.email.trim().to_lowercase();
    let otp = body.otp.trim();
    let new_password = body.new_password.trim();

    if new_password.len() < 6 {
        return errors::bad_request("Password must be at least 6 characters");
    }

    let user = match db::find_user_by_email(&state.db, &email).await {
        Ok(Some(u)) => u,
        Ok(None) => return errors::bad_request("Invalid email or OTP"),
        Err(_) => return errors::internal_error("Database error"),
    };

    if let Err(msg) = validate_otp(&user, otp) {
        return errors::bad_request(&msg);
    }

    let password_hash = match utils::hash_password(new_password) {
        Ok(h) => h,
        Err(_) => return errors::internal_error("Failed to hash password"),
    };

    if let Err(e) = db::update_user_password(&state.db, &user.username, &password_hash).await {
        log::error!("Update password error: {}", e);
        return errors::internal_error("Failed to update password");
    }

    // Clear OTP from app_json
    let mut app_json: Value = user.app_json.clone().unwrap_or(json!({}));
    if let Some(o) = app_json.as_object_mut() {
        o.remove("temp_reset_otp");
        o.remove("temp_reset_otp_time");
    }
    let _ = db::update_user_app_json(&state.db, &user.username, &app_json.to_string()).await;

    errors::ok_simple("Password reset successful")
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn validate_otp(user: &crate::models::UserProfile, otp: &str) -> Result<(), String> {
    let app_json: Value = user.app_json.clone().unwrap_or(json!({}));

    let stored_otp = app_json["temp_reset_otp"].as_str().unwrap_or("");
    let stored_time = app_json["temp_reset_otp_time"].as_i64().unwrap_or(0);

    if stored_otp.is_empty() {
        return Err("No OTP requested".to_string());
    }

    let now = Utc::now().timestamp();
    if now - stored_time > 600 {
        return Err("OTP has expired. Please request a new one.".to_string());
    }

    if stored_otp != otp {
        return Err("Invalid OTP".to_string());
    }

    Ok(())
}

async fn verify_google_token(token: &str) -> anyhow::Result<Value> {
    let client = reqwest::Client::new();
    let url = format!("https://oauth2.googleapis.com/tokeninfo?id_token={}", token);
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("Google token verification failed");
    }
    let info: Value = resp.json().await?;
    Ok(info)
}

async fn send_otp_email(to_email: &str, otp: &str) -> anyhow::Result<()> {
    use lettre::{
        transport::smtp::authentication::Credentials,
        Message, SmtpTransport, Transport,
    };

    let smtp_user = env::var("SMTP_USER").expect("SMTP_USER must be set");
    let smtp_pass = env::var("SMTP_PASS").expect("SMTP_PASS must be set");
    let smtp_from = env::var("SMTP_FROM").unwrap_or_else(|_| smtp_user.clone());

    let email = Message::builder()
        .from(format!("Auth System <{}>", smtp_from).parse()?)
        .to(to_email.parse()?)
        .subject("Password Reset OTP")
        .body(format!(
            "Your password reset OTP is: {}\n\nThis OTP is valid for 10 minutes.\n\nIf you did not request this, please ignore this email.",
            otp
        ))?;

    let creds = Credentials::new(smtp_user, smtp_pass);
    let mailer = SmtpTransport::relay("smtp.gmail.com")?
        .credentials(creds)
        .build();

    mailer.send(&email)?;
    Ok(())
}
