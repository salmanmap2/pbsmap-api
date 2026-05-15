use rand::Rng;
use uuid::Uuid;
use chrono::Utc;
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use std::env;

use crate::models::Claims;

/// Generate a random 6-digit OTP
pub fn generate_otp() -> String {
    let mut rng = rand::thread_rng();
    format!("{:06}", rng.gen_range(100000..=999999))
}

/// Generate username from email prefix + 4 random hex chars
/// e.g. email "salman@gmail.com" → "salman_a1b2"
pub fn generate_username(email: Option<&str>) -> String {
    let mut rng = rand::thread_rng();
    let suffix: u16 = rng.gen();
    let suffix_hex = format!("{:04x}", suffix);

    match email {
        Some(e) => {
            // Take the part before @, keep only alphanumeric, max 12 chars
            let prefix: String = e
                .split('@')
                .next()
                .unwrap_or("user")
                .chars()
                .filter(|c| c.is_alphanumeric())
                .take(12)
                .collect::<String>()
                .to_lowercase();
            let prefix = if prefix.is_empty() { "user".to_string() } else { prefix };
            format!("{}_{}", prefix, suffix_hex)
        }
        None => {
            // Google login without email
            let id = Uuid::new_v4().to_string().replace('-', "");
            format!("u{}_{}", &id[..6], suffix_hex)
        }
    }
}

/// Generate API key: pbsnet-<16 hex chars>
pub fn generate_api_key() -> String {
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..8).map(|_| rng.gen::<u8>()).collect();
    format!("pbsnet-{}", hex::encode(bytes))
}

/// Hash password using bcrypt
pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let hashed = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;
    Ok(hashed)
}

/// Verify bcrypt password
pub fn verify_password(password: &str, hash: &str) -> bool {
    bcrypt::verify(password, hash).unwrap_or(false)
}

/// Create JWT token
pub fn create_jwt(username: &str) -> anyhow::Result<String> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret_change_me".to_string());
    let expiry_hours: i64 = env::var("JWT_EXPIRY_HOURS")
        .unwrap_or_else(|_| "720".to_string())
        .parse()
        .unwrap_or(720);

    let now = Utc::now().timestamp() as usize;
    let exp = (Utc::now().timestamp() + expiry_hours * 3600) as usize;

    let claims = Claims {
        sub: username.to_string(),
        exp,
        iat: now,
        session_id: Uuid::new_v4().to_string(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;
    Ok(token)
}

/// Validate JWT token and return Claims
pub fn validate_jwt(token: &str) -> anyhow::Result<Claims> {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret_change_me".to_string());
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

/// Generate office_id: first 3 digits = pbs_id, last 2 = seq
pub fn generate_office_id(pbs_id: u32, seq: u32) -> String {
    format!("{:03}{:02}", pbs_id, seq)
}

/// Generate account_id: 5-digit office_id + 8 random digits
pub fn generate_account_id(office_id: &str) -> String {
    let mut rng = rand::thread_rng();
    let suffix: u64 = rng.gen_range(10000000..=99999999);
    format!("{}{}", office_id, suffix)
}
