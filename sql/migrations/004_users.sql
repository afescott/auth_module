-- users: individual users who can access the dashboard
CREATE TABLE users (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    merchant_id         UUID NOT NULL REFERENCES merchants(id) ON DELETE CASCADE,
    email               TEXT NOT NULL,
    password_hash       TEXT,  -- bcrypt hash, NULL for OAuth-only users
    display_name        TEXT,
    role                TEXT NOT NULL DEFAULT 'viewer',  -- admin|manager|viewer
    shopify_user_id     BIGINT,  -- if authenticated via Shopify
    last_login_at       TIMESTAMPTZ,
    is_active           BOOLEAN NOT NULL DEFAULT TRUE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(merchant_id, email)
);
