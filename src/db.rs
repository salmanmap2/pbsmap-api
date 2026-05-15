use sqlx::MySqlPool;
use crate::models::{UserProfile, Pbs, Office, MeterInfo, Note, MeterReading};

// ─── User ────────────────────────────────────────────────────────────────────

pub async fn find_user_by_email(pool: &MySqlPool, email: &str) -> anyhow::Result<Option<UserProfile>> {
    let user = sqlx::query_as::<_, UserProfile>(
        "SELECT username, jwt_token, mobile_number, email, password, app_json, user_json, active_office, user_api_key, google_id FROM user_profile WHERE email = ?"
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;
    Ok(user)
}

pub async fn find_user_by_username(pool: &MySqlPool, username: &str) -> anyhow::Result<Option<UserProfile>> {
    let user = sqlx::query_as::<_, UserProfile>(
        "SELECT username, jwt_token, mobile_number, email, password, app_json, user_json, active_office, user_api_key, google_id FROM user_profile WHERE username = ?"
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;
    Ok(user)
}

pub async fn find_user_by_mobile(pool: &MySqlPool, mobile: &str) -> anyhow::Result<Option<UserProfile>> {
    let user = sqlx::query_as::<_, UserProfile>(
        "SELECT username, jwt_token, mobile_number, email, password, app_json, user_json, active_office, user_api_key, google_id FROM user_profile WHERE mobile_number = ?"
    )
    .bind(mobile)
    .fetch_optional(pool)
    .await?;
    Ok(user)
}

pub async fn find_user_by_google_id(pool: &MySqlPool, google_id: &str) -> anyhow::Result<Option<UserProfile>> {
    let user = sqlx::query_as::<_, UserProfile>(
        "SELECT username, jwt_token, mobile_number, email, password, app_json, user_json, active_office, user_api_key, google_id FROM user_profile WHERE google_id = ?"
    )
    .bind(google_id)
    .fetch_optional(pool)
    .await?;
    Ok(user)
}

pub async fn create_user(
    pool: &MySqlPool,
    username: &str,
    email: Option<&str>,
    password_hash: Option<&str>,
    google_id: Option<&str>,
    user_json: Option<&str>,
    api_key: &str,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO user_profile (username, email, password, google_id, user_json, user_api_key, app_json) VALUES (?, ?, ?, ?, ?, ?, '{}')"
    )
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .bind(google_id)
    .bind(user_json)
    .bind(api_key)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_user_app_json(pool: &MySqlPool, username: &str, app_json: &str) -> anyhow::Result<()> {
    sqlx::query("UPDATE user_profile SET app_json = ? WHERE username = ?")
        .bind(app_json)
        .bind(username)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_user_password(pool: &MySqlPool, username: &str, password_hash: &str) -> anyhow::Result<()> {
    sqlx::query("UPDATE user_profile SET password = ? WHERE username = ?")
        .bind(password_hash)
        .bind(username)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_user_profile(
    pool: &MySqlPool,
    username: &str,
    mobile_number: Option<&str>,
    user_json: Option<&str>,
    active_office: Option<&str>,
) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE user_profile SET mobile_number = COALESCE(?, mobile_number), user_json = COALESCE(?, user_json), active_office = COALESCE(?, active_office) WHERE username = ?"
    )
    .bind(mobile_number)
    .bind(user_json)
    .bind(active_office)
    .bind(username)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_user_api_key(pool: &MySqlPool, username: &str, api_key: &str) -> anyhow::Result<()> {
    sqlx::query("UPDATE user_profile SET user_api_key = ? WHERE username = ?")
        .bind(api_key)
        .bind(username)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_google_id_for_email(pool: &MySqlPool, email: &str, google_id: &str) -> anyhow::Result<()> {
    sqlx::query("UPDATE user_profile SET google_id = ? WHERE email = ?")
        .bind(google_id)
        .bind(email)
        .execute(pool)
        .await?;
    Ok(())
}

// ─── PBS ─────────────────────────────────────────────────────────────────────

pub async fn get_all_pbs(pool: &MySqlPool) -> anyhow::Result<Vec<Pbs>> {
    let list = sqlx::query_as::<_, Pbs>("SELECT pbs_id, pbs_name FROM pbs ORDER BY pbs_id")
        .fetch_all(pool)
        .await?;
    Ok(list)
}

pub async fn get_pbs_by_id(pool: &MySqlPool, pbs_id: u32) -> anyhow::Result<Option<Pbs>> {
    let pbs = sqlx::query_as::<_, Pbs>("SELECT pbs_id, pbs_name FROM pbs WHERE pbs_id = ?")
        .bind(pbs_id)
        .fetch_optional(pool)
        .await?;
    Ok(pbs)
}

// ─── Office ──────────────────────────────────────────────────────────────────

pub async fn get_offices_by_pbs(pool: &MySqlPool, pbs_id: u32) -> anyhow::Result<Vec<Office>> {
    let offices = sqlx::query_as::<_, Office>(
        "SELECT office_id, pbs_id, office_name, office_info_json, office_user_json FROM office WHERE pbs_id = ?"
    )
    .bind(pbs_id)
    .fetch_all(pool)
    .await?;
    Ok(offices)
}

pub async fn get_office_by_id(pool: &MySqlPool, office_id: &str) -> anyhow::Result<Option<Office>> {
    let office = sqlx::query_as::<_, Office>(
        "SELECT office_id, pbs_id, office_name, office_info_json, office_user_json FROM office WHERE office_id = ?"
    )
    .bind(office_id)
    .fetch_optional(pool)
    .await?;
    Ok(office)
}

pub async fn get_next_office_seq(pool: &MySqlPool, pbs_id: u32) -> anyhow::Result<u32> {
    let prefix = format!("{:03}", pbs_id);
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM office WHERE office_id LIKE ?"
    )
    .bind(format!("{}%", prefix))
    .fetch_one(pool)
    .await?;
    Ok((result.0 + 1) as u32)
}

pub async fn create_office(pool: &MySqlPool, office_id: &str, pbs_id: u32, office_name: &str) -> anyhow::Result<()> {
    let default_user_json = r#"{"pending_users":[],"admin_users":[],"editor_users":[],"viewer_users":[]}"#;
    sqlx::query(
        "INSERT INTO office (office_id, pbs_id, office_name, office_info_json, office_user_json) VALUES (?, ?, ?, '{}', ?)"
    )
    .bind(office_id)
    .bind(pbs_id)
    .bind(office_name)
    .bind(default_user_json)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_office(
    pool: &MySqlPool,
    office_id: &str,
    office_name: Option<&str>,
    office_info_json: Option<&str>,
    office_user_json: Option<&str>,
) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE office SET office_name = COALESCE(?, office_name), office_info_json = COALESCE(?, office_info_json), office_user_json = COALESCE(?, office_user_json) WHERE office_id = ?"
    )
    .bind(office_name)
    .bind(office_info_json)
    .bind(office_user_json)
    .bind(office_id)
    .execute(pool)
    .await?;
    Ok(())
}

// ─── Meter ───────────────────────────────────────────────────────────────────

pub async fn create_meter(
    pool: &MySqlPool,
    account_id: &str,
    office_id: &str,
    account_number: &str,
    account_info_json: Option<&str>,
    route_number: Option<&str>,
    village: Option<&str>,
    meter_number: Option<&str>,
    meter_info_json: Option<&str>,
    gps_location: Option<&str>,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO meter_info (account_id, office_id, account_number, account_info_json, route_number, village, meter_number, meter_info_json, gps_location, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, NOW())"
    )
    .bind(account_id)
    .bind(office_id)
    .bind(account_number)
    .bind(account_info_json)
    .bind(route_number)
    .bind(village)
    .bind(meter_number)
    .bind(meter_info_json)
    .bind(gps_location)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_meter(
    pool: &MySqlPool,
    account_id: &str,
    account_number: Option<&str>,
    account_info_json: Option<&str>,
    route_number: Option<&str>,
    village: Option<&str>,
    meter_number: Option<&str>,
    meter_info_json: Option<&str>,
    gps_location: Option<&str>,
) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE meter_info SET account_number = COALESCE(?, account_number), account_info_json = COALESCE(?, account_info_json), route_number = COALESCE(?, route_number), village = COALESCE(?, village), meter_number = COALESCE(?, meter_number), meter_info_json = COALESCE(?, meter_info_json), gps_location = COALESCE(?, gps_location), updated_at = NOW() WHERE account_id = ?"
    )
    .bind(account_number)
    .bind(account_info_json)
    .bind(route_number)
    .bind(village)
    .bind(meter_number)
    .bind(meter_info_json)
    .bind(gps_location)
    .bind(account_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_meters_by_office(
    pool: &MySqlPool,
    office_id: &str,
    after_timestamp: Option<chrono::NaiveDateTime>,
) -> anyhow::Result<Vec<MeterInfo>> {
    if let Some(ts) = after_timestamp {
        let meters = sqlx::query_as::<_, MeterInfo>(
            "SELECT account_id, office_id, account_number, account_info_json, route_number, village, meter_number, meter_info_json, gps_location, updated_at FROM meter_info WHERE office_id = ? AND updated_at > ? ORDER BY updated_at ASC"
        )
        .bind(office_id)
        .bind(ts)
        .fetch_all(pool)
        .await?;
        Ok(meters)
    } else {
        let meters = sqlx::query_as::<_, MeterInfo>(
            "SELECT account_id, office_id, account_number, account_info_json, route_number, village, meter_number, meter_info_json, gps_location, updated_at FROM meter_info WHERE office_id = ? ORDER BY updated_at ASC"
        )
        .bind(office_id)
        .fetch_all(pool)
        .await?;
        Ok(meters)
    }
}

pub async fn get_meter_by_account_id(pool: &MySqlPool, account_id: &str) -> anyhow::Result<Option<MeterInfo>> {
    let meter = sqlx::query_as::<_, MeterInfo>(
        "SELECT account_id, office_id, account_number, account_info_json, route_number, village, meter_number, meter_info_json, gps_location, updated_at FROM meter_info WHERE account_id = ?"
    )
    .bind(account_id)
    .fetch_optional(pool)
    .await?;
    Ok(meter)
}

// ─── Note ────────────────────────────────────────────────────────────────────

pub async fn create_note(pool: &MySqlPool, note_id: &str, account_id: &str, note_json: &str, creator: &str) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO note (note_id, account_id, note_json, note_creator, timestamp) VALUES (?, ?, ?, ?, NOW())"
    )
    .bind(note_id)
    .bind(account_id)
    .bind(note_json)
    .bind(creator)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_note(pool: &MySqlPool, note_id: &str, username: &str) -> anyhow::Result<bool> {
    let result = sqlx::query(
        "DELETE FROM note WHERE note_id = ? AND note_creator = ?"
    )
    .bind(note_id)
    .bind(username)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn get_notes_by_account(
    pool: &MySqlPool,
    account_id: &str,
    after_timestamp: Option<chrono::NaiveDateTime>,
) -> anyhow::Result<Vec<Note>> {
    if let Some(ts) = after_timestamp {
        let notes = sqlx::query_as::<_, Note>(
            "SELECT note_id, account_id, note_json, note_creator, timestamp FROM note WHERE account_id = ? AND timestamp > ? ORDER BY timestamp ASC"
        )
        .bind(account_id)
        .bind(ts)
        .fetch_all(pool)
        .await?;
        Ok(notes)
    } else {
        let notes = sqlx::query_as::<_, Note>(
            "SELECT note_id, account_id, note_json, note_creator, timestamp FROM note WHERE account_id = ? ORDER BY timestamp ASC"
        )
        .bind(account_id)
        .fetch_all(pool)
        .await?;
        Ok(notes)
    }
}

// ─── Reading ─────────────────────────────────────────────────────────────────

pub async fn create_reading(pool: &MySqlPool, reading_id: &str, account_id: &str, reading_json: &str, reader_username: &str) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO meter_reading (reading_id, account_id, date_time, reading_json, reader_username) VALUES (?, ?, NOW(), ?, ?)"
    )
    .bind(reading_id)
    .bind(account_id)
    .bind(reading_json)
    .bind(reader_username)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_reading(
    pool: &MySqlPool,
    reading_id: &str,
    account_id: Option<&str>,
    date_time: Option<&str>,
    reading_json: Option<&str>,
    reader_username: Option<&str>,
) -> anyhow::Result<bool> {
    let result = sqlx::query(
        "UPDATE meter_reading SET account_id = COALESCE(?, account_id), date_time = COALESCE(STR_TO_DATE(?, '%Y-%m-%dT%H:%i:%s'), date_time), reading_json = COALESCE(?, reading_json), reader_username = COALESCE(?, reader_username) WHERE reading_id = ?"
    )
    .bind(account_id)
    .bind(date_time)
    .bind(reading_json)
    .bind(reader_username)
    .bind(reading_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn get_readings_by_office(
    pool: &MySqlPool,
    office_id_prefix: &str,
    ts_start: Option<chrono::NaiveDateTime>,
    ts_end: Option<chrono::NaiveDateTime>,
) -> anyhow::Result<Vec<MeterReading>> {
    let pattern = format!("{}%", office_id_prefix);
    match (ts_start, ts_end) {
        (Some(s), Some(e)) => {
            let readings = sqlx::query_as::<_, MeterReading>(
                "SELECT reading_id, account_id, date_time, reading_json, reader_username FROM meter_reading WHERE account_id LIKE ? AND date_time BETWEEN ? AND ? ORDER BY date_time ASC"
            )
            .bind(&pattern)
            .bind(s)
            .bind(e)
            .fetch_all(pool)
            .await?;
            Ok(readings)
        }
        (Some(s), None) => {
            let readings = sqlx::query_as::<_, MeterReading>(
                "SELECT reading_id, account_id, date_time, reading_json, reader_username FROM meter_reading WHERE account_id LIKE ? AND date_time >= ? ORDER BY date_time ASC"
            )
            .bind(&pattern)
            .bind(s)
            .fetch_all(pool)
            .await?;
            Ok(readings)
        }
        (None, Some(e)) => {
            let readings = sqlx::query_as::<_, MeterReading>(
                "SELECT reading_id, account_id, date_time, reading_json, reader_username FROM meter_reading WHERE account_id LIKE ? AND date_time <= ? ORDER BY date_time ASC"
            )
            .bind(&pattern)
            .bind(e)
            .fetch_all(pool)
            .await?;
            Ok(readings)
        }
        (None, None) => {
            let readings = sqlx::query_as::<_, MeterReading>(
                "SELECT reading_id, account_id, date_time, reading_json, reader_username FROM meter_reading WHERE account_id LIKE ? ORDER BY date_time ASC"
            )
            .bind(&pattern)
            .fetch_all(pool)
            .await?;
            Ok(readings)
        }
    }
}

// ─── Super Admin ─────────────────────────────────────────────────────────────

pub async fn validate_super_admin_key(pool: &MySqlPool, api_key: &str) -> anyhow::Result<bool> {
    let result: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM super_admin WHERE api_key = ?"
    )
    .bind(api_key)
    .fetch_one(pool)
    .await?;
    Ok(result.0 > 0)
}
