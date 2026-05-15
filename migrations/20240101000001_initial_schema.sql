-- ============================================================
-- Migration 001: Initial Schema
-- ============================================================

-- ─── Super Admin ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS super_admin (
    id         INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    api_key    VARCHAR(256) NOT NULL UNIQUE,
    label      VARCHAR(100),
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ─── PBS (Palli Bidyut Samity) ───────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS pbs (
    pbs_id   INT UNSIGNED PRIMARY KEY,   -- 3-digit numeric, starts at 101
    pbs_name VARCHAR(100) NOT NULL UNIQUE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ─── Office ──────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS office (
    office_id        VARCHAR(5)   NOT NULL PRIMARY KEY,  -- 5-digit: first 3 = pbs_id, last 2 = seq
    pbs_id           INT UNSIGNED NOT NULL,
    office_name      VARCHAR(150) NOT NULL,
    office_info_json JSON,
    office_user_json JSON,
    FOREIGN KEY (pbs_id) REFERENCES pbs(pbs_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ─── User Profile ────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS user_profile (
    username      VARCHAR(50)  NOT NULL PRIMARY KEY,
    jwt_token     TEXT,
    mobile_number VARCHAR(20)  UNIQUE,
    email         VARCHAR(150) UNIQUE,
    password      VARCHAR(255),
    app_json      JSON,
    user_json     JSON,
    active_office VARCHAR(5),
    user_api_key  VARCHAR(128) NOT NULL UNIQUE,
    google_id     VARCHAR(100) UNIQUE,
    created_at    DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (active_office) REFERENCES office(office_id) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ─── Meter Info ──────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS meter_info (
    account_id       VARCHAR(13)  NOT NULL PRIMARY KEY,  -- 5-digit office_id + 8 random digits
    office_id        VARCHAR(5)   NOT NULL,
    account_number   VARCHAR(20)  NOT NULL,
    account_info_json JSON,
    route_number     VARCHAR(50),
    village          VARCHAR(100),
    meter_number     VARCHAR(50),
    meter_info_json  JSON,
    gps_location     VARCHAR(100),
    updated_at       DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (office_id) REFERENCES office(office_id),
    INDEX idx_meter_office  (office_id),
    INDEX idx_meter_updated (updated_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ─── Note ────────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS note (
    note_id      VARCHAR(36)  NOT NULL PRIMARY KEY,  -- UUID
    account_id   VARCHAR(13)  NOT NULL,
    note_json    JSON,
    note_creator VARCHAR(50)  NOT NULL,
    timestamp    DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (account_id) REFERENCES meter_info(account_id),
    FOREIGN KEY (note_creator) REFERENCES user_profile(username),
    INDEX idx_note_account   (account_id),
    INDEX idx_note_timestamp (timestamp)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

-- ─── Meter Reading ───────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS meter_reading (
    reading_id      VARCHAR(36)  NOT NULL PRIMARY KEY,  -- UUID
    account_id      VARCHAR(13)  NOT NULL,
    date_time       DATETIME DEFAULT CURRENT_TIMESTAMP,
    reading_json    JSON,
    reader_username VARCHAR(50)  NOT NULL,
    FOREIGN KEY (account_id) REFERENCES meter_info(account_id),
    FOREIGN KEY (reader_username) REFERENCES user_profile(username),
    INDEX idx_reading_account (account_id),
    INDEX idx_reading_dt      (date_time)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
