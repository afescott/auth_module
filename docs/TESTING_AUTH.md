# Authentication Testing Guide

## 🚀 Quick Start

### 1. Run Database Migrations
The migrations will run automatically when you start the server:

```bash
cargo run
```

This will:
- Create the `users` table
- Create the `user_sessions` table
- Create the `api_keys` table
- Create the `audit_log` table

### 2. Create Test Users
```bash
psql "postgres://exchange_user:exchange_password@localhost/exchange_api" -f docs/create_test_user.sql
```

This creates 4 test users:

| Email | Password | Role | Status | Scopes |
|-------|----------|------|--------|--------|
| `admin@test-shop.com` | `admin123` | admin | Active | Viewer, Manager, Admin |
| `manager@test-shop.com` | `manager123` | manager | Active | Viewer, Manager |
| `viewer@test-shop.com` | `password` | viewer | Active | Viewer |
| `inactive@test-shop.com` | `password` | viewer | **Inactive** | - |

---

## 🧪 Testing Login Endpoint

### Test 1: Admin Login (Success)
```bash
curl -X POST http://localhost:8080/api/v1/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@test-shop.com",
    "password": "admin123"
  }'
```

**Expected Response:**
```json
{
  "success": true,
  "message": "Login successful",
  "data": {
    "access_token": "eyJhbGciOiJSUzI1NiIsImtpZCI6ImV4Y2hhbmdlX2FwaV9rZXlfMSIsInR5cCI6IkpXVCJ9...",
    "refresh_token": "eyJhbGciOiJSUzI1NiIsImtpZCI6ImV4Y2hhbmdlX2FwaV9rZXlfMSIsInR5cCI6IkpXVCJ9...",
    "user": {
      "id": "a1111111-e89b-12d3-a456-426614174001",
      "email": "admin@test-shop.com",
      "display_name": "Admin User",
      "role": "admin"
    }
  }
}
```

### Test 2: Manager Login (Success)
```bash
curl -X POST http://localhost:8080/api/v1/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "manager@test-shop.com",
    "password": "manager123"
  }'
```

### Test 3: Viewer Login (Success)
```bash
curl -X POST http://localhost:8080/api/v1/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "viewer@test-shop.com",
    "password": "password"
  }'
```

### Test 4: Wrong Password (Fail)
```bash
curl -X POST http://localhost:8080/api/v1/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@test-shop.com",
    "password": "wrongpassword"
  }'
```

**Expected Response:**
```json
{
  "error": "Invalid credentials",
  "message": "Invalid email or password"
}
```

### Test 5: Non-existent User (Fail)
```bash
curl -X POST http://localhost:8080/api/v1/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "notexist@test-shop.com",
    "password": "password"
  }'
```

**Expected Response:**
```json
{
  "error": "Invalid credentials",
  "message": "Invalid email or password"
}
```

### Test 6: Inactive User (Fail)
```bash
curl -X POST http://localhost:8080/api/v1/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "inactive@test-shop.com",
    "password": "password"
  }'
```

**Expected Response:**
```json
{
  "error": "Unauthorized",
  "message": "Unauthorized"
}
```

### Test 7: Invalid Email Format (Fail)
```bash
curl -X POST http://localhost:8080/api/v1/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "not-an-email",
    "password": "password"
  }'
```

**Expected Response:**
```json
{
  "error": "Validation error",
  "message": "Invalid email format"
}
```

---

## 🔑 Testing JWKS Endpoint

### Get Public Keys
```bash
curl http://localhost:8080/api/v1/jwks
```

**Expected Response:**
```json
{
  "keys": [
    {
      "alg": "RS256",
      "e": "AQAB",
      "kid": "exchange_api_key_1",
      "kty": "RSA",
      "n": "xGOr-H7A-jq...",
      "use": "sig"
    }
  ]
}
```

---

## 🔍 Inspecting JWT Tokens

### Decode the Access Token
1. Copy the `access_token` from login response
2. Go to https://jwt.io
3. Paste the token

**Expected Payload:**
```json
{
  "sub": "a1111111-e89b-12d3-a456-426614174001",
  "email": "admin@test-shop.com",
  "exp": 1729612345,
  "iat": 1729611445,
  "iss": "exchange_api",
  "token_type": "Access",
  "scope": ["Viewer", "Manager", "Admin"]
}
```

### Verify Token Expiration
- **Access Token**: Expires in 15 minutes
- **Refresh Token**: Expires in 30 days

---

## 🔐 Testing with Protected Routes (Future)

Once you add authentication middleware, test like this:

### Without Token (Should Fail)
```bash
curl http://localhost:8080/api/v1/products
```

### With Valid Token (Should Succeed)
```bash
# First, login and extract token
TOKEN=$(curl -X POST http://localhost:8080/api/v1/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@test-shop.com","password":"admin123"}' \
  | jq -r '.data.access_token')

# Use token to access protected route
curl http://localhost:8080/api/v1/products \
  -H "Authorization: Bearer $TOKEN"
```

### With Expired Token (Should Fail)
Wait 15 minutes after login, then try to use the access token.

---

## 📊 Database Verification

### Check Users Table
```sql
SELECT id, email, display_name, role, is_active 
FROM users 
ORDER BY created_at;
```

### Check User Sessions (After Login)
```sql
SELECT 
    us.id,
    u.email,
    us.ip_address,
    us.expires_at,
    us.created_at
FROM user_sessions us
JOIN users u ON us.user_id = u.id
ORDER BY us.created_at DESC;
```

### Check Audit Log (If Implemented)
```sql
SELECT 
    action,
    u.email as user_email,
    ip_address,
    created_at
FROM audit_log al
LEFT JOIN users u ON al.user_id = u.id
ORDER BY created_at DESC
LIMIT 10;
```

---

## 🧩 Testing Role Scopes

### Verify Admin Scopes
Login as admin and decode JWT - should see:
```json
"scope": ["Viewer", "Manager", "Admin"]
```

### Verify Manager Scopes
Login as manager and decode JWT - should see:
```json
"scope": ["Viewer", "Manager"]
```

### Verify Viewer Scopes
Login as viewer and decode JWT - should see:
```json
"scope": ["Viewer"]
```

---

## 🐛 Troubleshooting

### "Database error" on login
- Check if migrations ran: `SELECT * FROM users LIMIT 1;`
- Verify test users exist: Run `docs/create_test_user.sql`

### "Invalid email format"
- Email must match pattern: `user@domain.com`
- Check for typos in email

### "Internal server error"
- Check server logs: `RUST_LOG=debug cargo run`
- Verify JWT keys are generated (check `public_keys.json`)

### Token not working
- Check token hasn't expired (15 min for access token)
- Verify `Authorization: Bearer <token>` header format
- Ensure no extra spaces or newlines in token

---

## 📝 Test Checklist

- [ ] Migrations run successfully
- [ ] Test users created
- [ ] Admin login works
- [ ] Manager login works
- [ ] Viewer login works
- [ ] Wrong password rejected
- [ ] Non-existent user rejected
- [ ] Inactive user rejected
- [ ] Invalid email rejected
- [ ] JWKS endpoint returns public key
- [ ] JWT tokens decode correctly
- [ ] Admin has 3 scopes
- [ ] Manager has 2 scopes
- [ ] Viewer has 1 scope
- [ ] Access token expires in 15 min
- [ ] Refresh token expires in 30 days

---

## 🚀 Next Steps After Testing

1. **Add authentication middleware** to protect routes
2. **Implement refresh token endpoint** (`POST /refresh`)
3. **Add logout endpoint** (`POST /logout`)
4. **Switch to bcrypt** for password hashing
5. **Add rate limiting** to prevent brute force
6. **Update Swagger docs** with auth examples
7. **Implement audit logging** for security events

---

**Happy Testing! 🎉**

