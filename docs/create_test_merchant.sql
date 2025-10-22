-- Create test merchant for API testing
-- Run with: psql "postgres://exchange_user:exchange_password@localhost/exchange_api" -f docs/create_test_merchant.sql

INSERT INTO merchants (id, shop_domain, shop_name, shop_currency, timezone)
VALUES 
    ('123e4567-e89b-12d3-a456-426614174001', 'test-shop.myshopify.com', 'Test Shop', 'USD', 'America/New_York')
ON CONFLICT (shop_domain) DO UPDATE 
SET 
    shop_name = EXCLUDED.shop_name,
    shop_currency = EXCLUDED.shop_currency,
    timezone = EXCLUDED.timezone
RETURNING id, shop_domain, shop_name, shop_currency;

-- Output confirmation
\echo ''
\echo '✅ Test merchant created/updated:'
\echo '   ID: 123e4567-e89b-12d3-a456-426614174001'
\echo '   Shop: test-shop.myshopify.com'
\echo ''
\echo 'You can now use this merchant_id in your API calls!'


