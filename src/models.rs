use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::NaiveDateTime;

// ─── User Profile ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserProfile {
    pub username: String,
    pub jwt_token: Option<String>,
    pub mobile_number: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub app_json: Option<Value>,
    pub user_json: Option<Value>,
    pub active_office: Option<String>,
    pub user_api_key: String,
    pub google_id: Option<String>,
}

// ─── PBS ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Pbs {
    pub pbs_id: u32,
    pub pbs_name: String,
}

// ─── Office ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Office {
    pub office_id: String,
    pub pbs_id: u32,
    pub office_name: String,
    pub office_info_json: Option<Value>,
    pub office_user_json: Option<Value>,
}

// ─── Meter Info ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct MeterInfo {
    pub account_id: String,
    pub office_id: String,
    pub account_number: String,
    pub account_info_json: Option<Value>,
    pub route_number: Option<String>,
    pub village: Option<String>,
    pub meter_number: Option<String>,
    pub meter_info_json: Option<Value>,
    pub gps_location: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
}

// ─── Note ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Note {
    pub note_id: String,
    pub account_id: String,
    pub note_json: Option<Value>,
    pub note_creator: String,
    pub timestamp: Option<NaiveDateTime>,
}

// ─── Meter Reading ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct MeterReading {
    pub reading_id: String,
    pub account_id: String,
    pub date_time: Option<NaiveDateTime>,
    pub reading_json: Option<Value>,
    pub reader_username: String,
}

// ─── JWT Claims ──────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,   // username
    pub exp: usize,
    pub iat: usize,
    pub session_id: String,
}
