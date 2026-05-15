# API Documentation — PBS Auth System

**Base URL:** `http://localhost:8080`  
**Content-Type:** `application/json`  
**Auth Header:** `Authorization: Bearer <jwt_token>`  
**Super Admin Header:** `X-Api-Key: <super_admin_api_key>`

---

## Response Format

All responses follow this structure:

```json
{
  "success": true | false,
  "message": "Human readable message",
  "data": { ... }   // present on success responses with data
}
```

---

## 1. Authentication Endpoints

### 1.1 Signup (Email + Password)

**POST** `/api/auth/signup`

No verification required. Username is auto-generated.

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "min6chars",
  "full_name": "John Doe"   // optional
}
```

**Response (200):**
```json
{
  "success": true,
  "message": "Signup successful",
  "data": {
    "username": "user_a1b2c3d4",
    "email": "user@example.com",
    "token": "eyJ...",
    "user_api_key": "abc123..."
  }
}
```

**Errors:** `400` email already registered | password too short

---

### 1.2 Login (Email / Username / Mobile + Password)

**POST** `/api/auth/login`

Supports multiple sessions simultaneously (each login generates a new JWT).

**Request Body:**
```json
{
  "identifier": "user@example.com",  // email, username, or mobile number
  "password": "yourpassword"
}
```

**Response (200):**
```json
{
  "success": true,
  "message": "Login successful",
  "data": {
    "username": "user_a1b2c3d4",
    "email": "user@example.com",
    "token": "eyJ...",
    "user_json": { "full_name": "John Doe", "profile_pic_url": "" },
    "active_office": "10101",
    "user_api_key": "abc123..."
  }
}
```

**Notes:**
- If identifier contains `@` → treated as email
- If identifier is all digits → treated as mobile number
- Otherwise → treated as username
- Mobile login requires mobile number to be set in profile first

**Errors:** `401` invalid credentials

---

### 1.3 Login with Google

**POST** `/api/auth/login/google`

Frontend obtains Google ID token via Google Sign-In SDK, then sends it here.

**Request Body:**
```json
{
  "google_token": "eyJ..."  // Google ID token from frontend
}
```

**Behavior:**
1. Verifies token with Google's API
2. If `google_id` exists → login
3. If email matches existing account → links Google to that account → login
4. Otherwise → creates new account with auto-generated username

**Response (200):**
```json
{
  "success": true,
  "message": "Login successful",
  "data": {
    "username": "user_a1b2c3d4",
    "email": "user@gmail.com",
    "token": "eyJ...",
    "user_json": { "full_name": "John Doe", "profile_pic_url": "https://..." },
    "user_api_key": "abc123..."
  }
}
```

**Errors:** `401` invalid Google token

---

### 1.4 Forgot Password (Send OTP)

**POST** `/api/auth/forgot-password`

Sends a 6-digit OTP to the registered email via Gmail SMTP. OTP valid for **10 minutes**.

**Request Body:**
```json
{
  "email": "user@example.com"
}
```

**Response (200):**
```json
{
  "success": true,
  "message": "If this email is registered, an OTP has been sent"
}
```

**Notes:**
- OTP stored in `app_json.temp_reset_otp` and `app_json.temp_reset_otp_time`
- Response is always 200 to prevent email enumeration

---

### 1.5 Verify OTP

**POST** `/api/auth/verify-otp`

Check if OTP is valid before showing reset password form.

**Request Body:**
```json
{
  "email": "user@example.com",
  "otp": "123456"
}
```

**Response (200):**
```json
{
  "success": true,
  "message": "OTP verified"
}
```

**Errors:** `400` invalid OTP | OTP expired

---

### 1.6 Reset Password

**POST** `/api/auth/reset-password`

Verifies OTP and sets new password. Clears OTP after success.

**Request Body:**
```json
{
  "email": "user@example.com",
  "otp": "123456",
  "new_password": "newpassword123"
}
```

**Response (200):**
```json
{
  "success": true,
  "message": "Password reset successful"
}
```

**Errors:** `400` invalid OTP | OTP expired | password too short

---

## 2. User Profile Endpoints

> All require `Authorization: Bearer <token>`

### 2.1 Get Profile

**GET** `/api/user/profile`

Returns profile without `app_json` (internal field).

**Response (200):**
```json
{
  "success": true,
  "message": "Profile fetched",
  "data": {
    "username": "user_a1b2c3d4",
    "email": "user@example.com",
    "mobile_number": "01700000000",
    "user_json": {
      "full_name": "John Doe",
      "profile_pic_url": "https://..."
    },
    "active_office": "10101",
    "user_api_key": "abc123..."
  }
}
```

---

### 2.2 Update Profile

**PUT** `/api/user/profile`

All fields optional. Only provided fields are updated.

**Request Body:**
```json
{
  "mobile_number": "01700000000",   // optional
  "full_name": "John Doe",          // optional
  "profile_pic_url": "https://...", // optional
  "active_office": "10101"          // optional, 5-digit office ID
}
```

**Response (200):**
```json
{
  "success": true,
  "message": "Profile updated successfully"
}
```

---

### 2.3 Change Password

**POST** `/api/user/change-password`

**Request Body:**
```json
{
  "old_password": "currentpassword",
  "new_password": "newpassword123"
}
```

**Response (200):**
```json
{
  "success": true,
  "message": "Password changed successfully"
}
```

**Errors:** `401` old password incorrect | `400` Google-only account

---

### 2.4 Regenerate API Key

**POST** `/api/user/regenerate-api-key`

Generates a new `user_api_key`. Old key is immediately invalidated.

**Response (200):**
```json
{
  "success": true,
  "message": "API key regenerated",
  "data": {
    "user_api_key": "new_key_here..."
  }
}
```

---

### 2.5 Join Office

**POST** `/api/user/join-office`

Adds user to office's `pending_users` list. Office admin must approve.

**Request Body:**
```json
{
  "office_id": "10101"
}
```

**Response (200):**
```json
{
  "success": true,
  "message": "Join request submitted. Waiting for admin approval."
}
```

**Errors:** `400` already associated with office | `404` office not found

---

## 3. Public Endpoints (No Auth Required)

### 3.1 All PBS List

**GET** `/api/public/pbs-list`

**Response (200):**
```json
{
  "success": true,
  "message": "PBS list",
  "data": [
    { "pbs_id": 101, "pbs_name": "Bagerhat PBS" },
    { "pbs_id": 102, "pbs_name": "Barguna PBS" }
  ]
}
```

---

### 3.2 Offices by PBS

**GET** `/api/public/offices/{pbs_id}`

**Example:** `GET /api/public/offices/121`

**Response (200):**
```json
{
  "success": true,
  "message": "Offices fetched",
  "data": [
    {
      "office_id": "12101",
      "pbs_id": 121,
      "office_name": "Dhaka PBS-1 Main Office",
      "office_info_json": { "area": "Mirpur", "map_tile_url": "https://..." },
      "office_user_json": {
        "pending_users": [],
        "admin_users": ["user_abc"],
        "editor_users": [],
        "viewer_users": []
      }
    }
  ]
}
```

---

### 3.3 Office by ID

**GET** `/api/public/office/{office_id}`

**Example:** `GET /api/public/office/12101`

**Response (200):**
```json
{
  "success": true,
  "message": "Office fetched",
  "data": {
    "office_id": "12101",
    "pbs_id": 121,
    "office_name": "Dhaka PBS-1 Main Office",
    "office_info_json": { "area": "Mirpur" },
    "office_user_json": { ... }
  }
}
```

---

### 3.4 User by Mobile Number

**GET** `/api/public/user-by-mobile/{mobile}`

**Example:** `GET /api/public/user-by-mobile/01700000000`

**Response (200):**
```json
{
  "success": true,
  "message": "User found",
  "data": {
    "username": "user_a1b2c3d4",
    "mobile_number": "01700000000",
    "user_json": { "full_name": "John Doe", "profile_pic_url": "" }
  }
}
```

---

## 4. Office Admin Endpoints

> Requires JWT. Caller must be an `admin_users` member of the target office.

### 4.1 Manage Office Users

**POST** `/api/office/user-change`

**Request Body:**
```json
{
  "office_id": "12101",

  // Add user to a role (removes from all other roles first)
  "add_username": "user_xyz",
  "role": "editor",           // "admin" | "editor" | "viewer"

  // Approve a pending user
  "approve_username": "user_abc",
  "approve_role": "viewer",

  // Remove user from a role
  "remove_username": "user_def",
  "remove_role": "editor"     // "admin" | "editor" | "viewer" | "pending"
}
```

All fields are optional. Multiple operations can be done in one request.

**Response (200):**
```json
{
  "success": true,
  "message": "Office users updated",
  "data": {
    "pending_users": [],
    "admin_users": ["user_xyz"],
    "editor_users": [],
    "viewer_users": ["user_abc"]
  }
}
```

---

## 5. Meter Endpoints

> Requires JWT.

### 5.1 Add Meter

**POST** `/api/meter/add`

Requires `admin` or `editor` role in the office.

**Request Body:**
```json
{
  "office_id": "12101",
  "account_number": "1234567",        // required, 7-digit
  "account_info_json": { "name": "Rahim", "address": "Mirpur-10" },  // optional
  "route_number": "R-01",             // optional
  "village": "Mirpur",                // optional
  "meter_number": "MTR-001",          // optional
  "meter_info_json": { "type": "digital", "phase": "single" },       // optional
  "gps_location": "23.8103,90.4125"   // optional
}
```

**Response (200):**
```json
{
  "success": true,
  "message": "Meter added",
  "data": {
    "account_id": "1210112345678"   // 13-digit: 5-digit office_id + 8 random
  }
}
```

---

### 5.2 Edit Meter

**POST** `/api/meter/edit`

Requires `admin` or `editor` role. All fields except `account_id` are optional.

**Request Body:**
```json
{
  "account_id": "1210112345678",
  "account_number": "7654321",
  "account_info_json": { "name": "Karim" },
  "route_number": "R-02",
  "village": "Banani",
  "meter_number": "MTR-002",
  "meter_info_json": { "type": "analog" },
  "gps_location": "23.7937,90.4066"
}
```

**Response (200):**
```json
{
  "success": true,
  "message": "Meter updated successfully"
}
```

---

### 5.3 All Meter List (Sync)

**POST** `/api/meter/all`

Requires any office role (admin/editor/viewer). Supports incremental sync via `last_time`.

**Request Body:**
```json
{
  "office_id": "12101",
  "last_time": "2024-06-01T00:00:00"  // optional, ISO format. Returns only meters updated after this time.
}
```

**Response (200):**
```json
{
  "success": true,
  "message": "Meter list fetched",
  "data": [
    {
      "account_id": "1210112345678",
      "office_id": "12101",
      "account_number": "1234567",
      "account_info_json": { "name": "Rahim" },
      "route_number": "R-01",
      "village": "Mirpur",
      "meter_number": "MTR-001",
      "meter_info_json": { "type": "digital" },
      "gps_location": "23.8103,90.4125",
      "updated_at": "2024-06-15 10:30:00"
    }
  ]
}
```

**Notes:**
- Returns all meters if `last_time` is omitted
- Works for 40,000+ meters (full sync)
- Use `last_time` for incremental sync to reduce data transfer

---

## 6. Note Endpoints

> Requires JWT. User must be office member (admin/editor/viewer).

### 6.1 Add Note

**POST** `/api/note/add`

**Request Body:**
```json
{
  "account_id": "1210112345678",
  "note_json": {
    "text": "Meter is faulty",
    "category": "complaint",
    "priority": "high"
  }
}
```

**Response (200):**
```json
{
  "success": true,
  "message": "Note added",
  "data": {
    "note_id": "uuid-here"
  }
}
```

---

### 6.2 Delete Note

**POST** `/api/note/delete`

Only the note creator can delete their own note.

**Request Body:**
```json
{
  "note_id": "uuid-here"
}
```

**Response (200):**
```json
{
  "success": true,
  "message": "Note deleted"
}
```

**Errors:** `403` not the creator

---

### 6.3 Get All Notes (Sync)

**POST** `/api/note/all`

**Request Body:**
```json
{
  "account_id": "1210112345678",
  "last_time": "2024-06-01T00:00:00"  // optional, returns notes after this timestamp
}
```

**Response (200):**
```json
{
  "success": true,
  "message": "Notes fetched",
  "data": [
    {
      "note_id": "uuid-here",
      "account_id": "1210112345678",
      "note_json": { "text": "Meter is faulty" },
      "note_creator": "user_a1b2c3d4",
      "timestamp": "2024-06-15 10:30:00"
    }
  ]
}
```

---

## 7. Meter Reading Endpoints

> Requires JWT. User must be office member.

### 7.1 New Reading

**POST** `/api/reading/new`

Requires admin/editor/viewer role.

**Request Body:**
```json
{
  "account_id": "1210112345678",
  "reading_json": {
    "value": 1523,
    "unit": "kWh",
    "image_url": "https://..."
  }
}
```

**Response (200):**
```json
{
  "success": true,
  "message": "Reading saved",
  "data": {
    "reading_id": "uuid-here"
  }
}
```

---

### 7.2 Edit Reading

**POST** `/api/reading/edit`

Requires admin/editor role.

**Request Body:**
```json
{
  "reading_id": "uuid-here",
  "account_id": "1210112345678",          // optional
  "date_time": "2024-06-15T10:30:00",     // optional, ISO format
  "reading_json": { "value": 1530 },      // optional
  "reader_username": "user_xyz"           // optional
}
```

**Response (200):**
```json
{
  "success": true,
  "message": "Reading updated"
}
```

---

### 7.3 Get All Readings

**POST** `/api/reading/all`

Requires office membership. Filters by office (via 5-digit office_id prefix in account_id).

**Request Body:**
```json
{
  "office_id": "12101",
  "timestamp_start": "2024-06-01T00:00:00",  // optional
  "timestamp_end": "2024-06-30T23:59:59"     // optional
}
```

**Response (200):**
```json
{
  "success": true,
  "message": "Readings fetched",
  "data": [
    {
      "reading_id": "uuid-here",
      "account_id": "1210112345678",
      "date_time": "2024-06-15 10:30:00",
      "reading_json": { "value": 1523, "unit": "kWh" },
      "reader_username": "user_a1b2c3d4"
    }
  ]
}
```

---

## 8. Developer / Super Admin Endpoints

> All require `X-Api-Key: <super_admin_api_key>` header.

### 8.1 Create Office

**POST** `/api/dev/create-office`

**Request Body:**
```json
{
  "pbs_id": 121,
  "office_name": "Dhaka PBS-1 Mirpur Office"
}
```

**Response (200):**
```json
{
  "success": true,
  "message": "Office created",
  "data": {
    "office_id": "12101",
    "pbs_id": 121,
    "office_name": "Dhaka PBS-1 Mirpur Office"
  }
}
```

**Notes:**
- `office_id` = `{pbs_id:03}{seq:02}` (e.g., PBS 121, 1st office → `12101`)
- Max 99 offices per PBS

---

### 8.2 All Offices by PBS (Dev)

**GET** `/api/dev/all-office/{pbs_id}`

**Example:** `GET /api/dev/all-office/121`

Returns full office data including `office_user_json`.

---

### 8.3 Edit Office

**POST** `/api/dev/edit-office`

**Request Body:**
```json
{
  "office_id": "12101",
  "office_name": "Updated Name",           // optional
  "office_info_json": {                    // optional, full JSON replace
    "area": "Mirpur",
    "map_tile_url": "https://tile.server/{z}/{x}/{y}.png",
    "contact": "01700000000"
  },
  "office_user_json": { ... }              // optional, full JSON replace
}
```

**Response (200):**
```json
{
  "success": true,
  "message": "Office updated successfully"
}
```

---

### 8.4 User Manage (Super Admin)

**POST** `/api/dev/user-manage`

**Request Body:**
```json
{
  "office_id": "12101",
  "add_admin": "user_abc",       // optional
  "add_editor": "user_def",      // optional
  "add_viewer": "user_ghi",      // optional
  "remove_admin": "user_xyz",    // optional
  "remove_editor": "user_uvw",   // optional
  "remove_viewer": "user_rst",   // optional
  "remove_pending": "user_pqr"   // optional
}
```

**Response (200):**
```json
{
  "success": true,
  "message": "Office users updated",
  "data": {
    "pending_users": [],
    "admin_users": ["user_abc"],
    "editor_users": ["user_def"],
    "viewer_users": ["user_ghi"]
  }
}
```

---

## 9. Data Models

### User Profile Table (`user_profile`)

| Column | Type | Description |
|--------|------|-------------|
| `username` | VARCHAR(50) PK | Auto-generated (e.g., `user_a1b2c3d4`) |
| `jwt_token` | TEXT | Not used for session (stateless JWT) |
| `mobile_number` | VARCHAR(20) UNIQUE | Optional, set after signup |
| `email` | VARCHAR(150) UNIQUE | Required for email signup |
| `password` | VARCHAR(255) | bcrypt hash, NULL for Google-only accounts |
| `app_json` | JSON | Internal: OTP storage, etc. |
| `user_json` | JSON | Public: `full_name`, `profile_pic_url`, etc. |
| `active_office` | VARCHAR(5) FK | Selected office ID |
| `user_api_key` | VARCHAR(128) UNIQUE | Auto-generated, user-regeneratable |
| `google_id` | VARCHAR(100) UNIQUE | Google OAuth sub ID |

### PBS Table (`pbs`)

| Column | Type | Description |
|--------|------|-------------|
| `pbs_id` | INT UNSIGNED PK | 3-digit, starts at 101 |
| `pbs_name` | VARCHAR(100) UNIQUE | PBS name |

### Office Table (`office`)

| Column | Type | Description |
|--------|------|-------------|
| `office_id` | VARCHAR(5) PK | 5-digit: `{pbs_id:03}{seq:02}` |
| `pbs_id` | INT UNSIGNED FK | Parent PBS |
| `office_name` | VARCHAR(150) | Office name |
| `office_info_json` | JSON | Area, map tile URL, contact, etc. |
| `office_user_json` | JSON | `{pending_users, admin_users, editor_users, viewer_users}` |

### Meter Info Table (`meter_info`)

| Column | Type | Description |
|--------|------|-------------|
| `account_id` | VARCHAR(13) PK | `{office_id:5}{random:8}` |
| `office_id` | VARCHAR(5) FK | Parent office |
| `account_number` | VARCHAR(20) | 7-digit account number |
| `account_info_json` | JSON | Customer name, address, etc. |
| `route_number` | VARCHAR(50) | Route identifier |
| `village` | VARCHAR(100) | Village name |
| `meter_number` | VARCHAR(50) | Physical meter number |
| `meter_info_json` | JSON | Meter type, phase, etc. |
| `gps_location` | VARCHAR(100) | `lat,lng` format |
| `updated_at` | DATETIME | Auto-updated on edit |

### Note Table (`note`)

| Column | Type | Description |
|--------|------|-------------|
| `note_id` | VARCHAR(36) PK | UUID |
| `account_id` | VARCHAR(13) FK | Parent meter account |
| `note_json` | JSON | Note content |
| `note_creator` | VARCHAR(50) FK | Username of creator |
| `timestamp` | DATETIME | Creation time |

### Meter Reading Table (`meter_reading`)

| Column | Type | Description |
|--------|------|-------------|
| `reading_id` | VARCHAR(36) PK | UUID |
| `account_id` | VARCHAR(13) FK | Parent meter account |
| `date_time` | DATETIME | Reading time |
| `reading_json` | JSON | Reading value, unit, image, etc. |
| `reader_username` | VARCHAR(50) FK | Who took the reading |

---

## 10. Role Permissions Summary

| Endpoint | viewer | editor | admin | super_admin |
|----------|--------|--------|-------|-------------|
| Get profile | ✓ | ✓ | ✓ | ✓ |
| Update profile | ✓ | ✓ | ✓ | ✓ |
| Join office | ✓ | ✓ | ✓ | ✓ |
| Add meter | ✗ | ✓ | ✓ | ✓ |
| Edit meter | ✗ | ✓ | ✓ | ✓ |
| View meters | ✓ | ✓ | ✓ | ✓ |
| Add note | ✓ | ✓ | ✓ | ✓ |
| Delete own note | ✓ | ✓ | ✓ | ✓ |
| View notes | ✓ | ✓ | ✓ | ✓ |
| New reading | ✓ | ✓ | ✓ | ✓ |
| Edit reading | ✗ | ✓ | ✓ | ✓ |
| View readings | ✓ | ✓ | ✓ | ✓ |
| Manage office users | ✗ | ✗ | ✓ | ✓ |
| Create/edit office | ✗ | ✗ | ✗ | ✓ |

---

## 11. Setup Guide

### Prerequisites
- Rust (latest stable)
- MySQL 8.0+
- Gmail account with App Password enabled

### Steps

```bash
# 1. Clone and enter directory
cd auth-system

# 2. Copy env file
cp .env.example .env
# Edit .env with your values

# 3. Create MySQL database
mysql -u root -p -e "CREATE DATABASE auth_system_db CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;"

# 4. Install sqlx-cli
cargo install sqlx-cli --no-default-features --features mysql

# 5. Run migrations
sqlx migrate run

# 6. Build and run
cargo run --release
```

### Gmail App Password Setup
1. Enable 2FA on your Google account
2. Go to Google Account → Security → App Passwords
3. Generate password for "Mail"
4. Use that password as `SMTP_PASS` in `.env`

### Google OAuth Setup
1. Go to [Google Cloud Console](https://console.cloud.google.com)
2. Create OAuth 2.0 Client ID (Web application)
3. Add your frontend URL to authorized origins
4. Use the Client ID in frontend Google Sign-In SDK
5. Frontend sends the `credential` (ID token) to `/api/auth/login/google`

---

## 12. Error Codes

| HTTP Code | Meaning |
|-----------|---------|
| 200 | Success |
| 400 | Bad Request (validation error) |
| 401 | Unauthorized (missing/invalid token) |
| 403 | Forbidden (insufficient permissions) |
| 404 | Not Found |
| 500 | Internal Server Error |
