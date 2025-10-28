-- Create test users for API testing
-- Run with: psql "postgres://exchange_user:exchange_password@localhost/exchange_api" -f docs/create_test_user.sql

-- Note: Password hashes are SHA-256 for now. Consider switching to bcrypt in production.
-- Test passwords:
--   "password" -> 5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8
--   "admin123" -> 240be518fabd2724ddb6f04eeb1da5967448d7e831c08c8fa822809f74c720a9
--   "manager123" -> 2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824

-- Create an admin user
INSERT INTO users (id, merchant_id, email, password_hash, display_name, role, is_active)
VALUES 
    (
        'a1111111-e89b-12d3-a456-426614174001',
        '123e4567-e89b-12d3-a456-426614174001',  -- test merchant ID
        'admin@test-shop.com',
        '240be518fabd2724ddb6f04eeb1da5967448d7e831c08c8fa822809f74c720a9',  -- "admin123"
        'Admin User',
        'admin',
        true
    )
ON CONFLICT (merchant_id, email) DO UPDATE 
SET 
    password_hash = EXCLUDED.password_hash,
    display_name = EXCLUDED.display_name,
    role = EXCLUDED.role,
    is_active = EXCLUDED.is_active;

-- Create a manager user
INSERT INTO users (id, merchant_id, email, password_hash, display_name, role, is_active)
VALUES 
    (
        'a2222222-e89b-12d3-a456-426614174002',
        '123e4567-e89b-12d3-a456-426614174001',  -- test merchant ID
        'manager@test-shop.com',
        '2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824',  -- "manager123"
        'Manager User',
        'manager',
        true
    )
ON CONFLICT (merchant_id, email) DO UPDATE 
SET 
    password_hash = EXCLUDED.password_hash,
    display_name = EXCLUDED.display_name,
    role = EXCLUDED.role,
    is_active = EXCLUDED.is_active;

-- Create a viewer user
INSERT INTO users (id, merchant_id, email, password_hash, display_name, role, is_active)
VALUES 
    (
        'a3333333-e89b-12d3-a456-426614174003',
        '123e4567-e89b-12d3-a456-426614174001',  -- test merchant ID
        'viewer@test-shop.com',
        '5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8',  -- "password"
        'Viewer User',
        'viewer',
        true
    )
ON CONFLICT (merchant_id, email) DO UPDATE 
SET 
    password_hash = EXCLUDED.password_hash,
    display_name = EXCLUDED.display_name,
    role = EXCLUDED.role,
    is_active = EXCLUDED.is_active;

-- Create an inactive user (for testing access control)
INSERT INTO users (id, merchant_id, email, password_hash, display_name, role, is_active)
VALUES 
    (
        'a4444444-e89b-12d3-a456-426614174004',
        '123e4567-e89b-12d3-a456-426614174001',  -- test merchant ID
        'inactive@test-shop.com',
        '5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8',  -- "password"
        'Inactive User',
        'viewer',
        false
    )
ON CONFLICT (merchant_id, email) DO UPDATE 
SET 
    password_hash = EXCLUDED.password_hash,
    display_name = EXCLUDED.display_name,
    role = EXCLUDED.role,
    is_active = EXCLUDED.is_active;

-- Output confirmation
\echo ''
\echo '✅ Test users created/updated:'
\echo ''
\echo '1. Admin User'
\echo '   Email: admin@test-shop.com'
\echo '   Password: admin123'
\echo '   Role: admin (can do everything)'
\echo '   Scopes: [Viewer, Manager, Admin]'
\echo ''
\echo '2. Manager User'
\echo '   Email: manager@test-shop.com'
\echo '   Password: manager123'
\echo '   Role: manager (can edit products/orders)'
\echo '   Scopes: [Viewer, Manager]'
\echo ''
\echo '3. Viewer User'
\echo '   Email: viewer@test-shop.com'
\echo '   Password: password'
\echo '   Role: viewer (read-only access)'
\echo '   Scopes: [Viewer]'
\echo ''
\echo '4. Inactive User (cannot login)'
\echo '   Email: inactive@test-shop.com'
\echo '   Password: password'
\echo '   Status: INACTIVE (login will fail)'
\echo ''
\echo '🧪 Test Login:'
\echo '   Use the curl command from the README to test login'
\echo ''

