-- ============================================================
-- Migration 003: Seed initial Super Admin API key
-- IMPORTANT: Change this key before production deployment!
-- ============================================================

-- Default super admin key (replace with a secure generated key)
INSERT INTO super_admin (api_key, label) VALUES
('CHANGE_THIS_SUPER_ADMIN_KEY_BEFORE_PRODUCTION_USE_64CHARS_MINIMUM_LENGTH_HERE', 'default_super_admin');
