# User Management - Security Model

## 🔒 Recommended Security Setup

For **user management endpoints**, I've implemented them with these security recommendations:

### Role-Based Access Control

| Endpoint | Viewer | Manager | Admin | Notes |
|----------|--------|---------|-------|-------|
| `GET /users` | ❌ No | ⚠️ Limited* | ✅ Yes | List users |
| `POST /users` | ❌ No | ❌ No | ✅ Yes | Create new user |
| `GET /users/:id` | ⚠️ Own only* | ⚠️ Own only* | ✅ Yes | View user details |
| `PUT /users/:id` | ⚠️ Own only* | ⚠️ Own only* | ✅ Yes | Update user |
| `DELETE /users/:id` | ❌ No | ❌ No | ✅ Yes | Deactivate user |

**Legend:**
- ✅ Full access
- ⚠️ Limited access (see notes)
- ❌ No access

**Notes:**
- `*` **Manager - Limited list:** Can only see users in their own merchant
- `*` **Own only:** Any user can view/edit their own profile (with limited fields)

---

## 🎯 Security Principles

### 1. **Admin-Only User Management**
Only admins can:
- Create new users
- Change user roles
- Deactivate users
- View all users across merchants

**Why:** Prevents privilege escalation and unauthorized access.

### 2. **Self-Service Profile Management**
All users can:
- View their own profile
- Update their own display name
- Change their own password

All users **cannot**:
- Change their own role
- Deactivate their own account
- See other users' information

**Why:** Balances usability with security.

### 3. **Soft Delete (Deactivation)**
The `DELETE` endpoint sets `is_active = false` instead of hard deletion.

**Benefits:**
- Preserves audit trail ("who made this change?")
- Maintains referential integrity
- Allows account reactivation if needed
- GDPR-compliant (can be fully deleted separately)

---

## 📋 Endpoints Created

### `GET /api/v1/users`
List all users for a merchant.

**Security:** 
- TODO: Add middleware for Admin access
- Alternative: Allow Managers to see their own merchant's users

**Query Parameters:**
```
merchant_id (required)
role (optional) - Filter by role: admin|manager|viewer
is_active (optional) - Filter by active status
limit (optional) - Max 100
offset (optional) - For pagination
```

**Response:** Excludes `password_hash` for security

---

### `POST /api/v1/users`
Create a new user.

**Security:** TODO: Add middleware - ADMIN ONLY

**Request Body:**
```json
{
  "merchant_id": "uuid",
  "email": "user@example.com",
  "password": "SecurePass123!",  // Optional for OAuth users
  "display_name": "John Doe",
  "role": "viewer",  // admin|manager|viewer (defaults to viewer)
  "shopify_user_id": 123456,  // Optional
  "is_active": true  // Optional (defaults to true)
}
```

**Validations:**
- ✅ Email format validation
- ✅ Password strength validation (if provided)
- ✅ Duplicate email check per merchant
- ✅ Role validation (admin/manager/viewer)
- ✅ Password hashing (SHA-256, should upgrade to bcrypt)

---

### `GET /api/v1/users/:id`
Get a specific user's details.

**Security:** 
- TODO: Add middleware - Admin access OR user viewing their own profile

**Response:** Excludes `password_hash` for security

---

### `PUT /api/v1/users/:id`
Update a user's information.

**Security:** 
- TODO: Add middleware - Admin full access OR user updating their own profile (limited fields)

**Request Body (all fields optional):**
```json
{
  "display_name": "New Name",
  "password": "NewPassword123!",  // Change password
  "role": "manager",  // Admin only
  "is_active": false  // Admin only
}
```

**For non-admin users updating themselves:**
- Allow: `display_name`, `password`
- Deny: `role`, `is_active`

---

### `DELETE /api/v1/users/:id`
Deactivate a user (soft delete).

**Security:** TODO: Add middleware - ADMIN ONLY

**Effect:** Sets `is_active = false`
- User cannot login
- Preserves audit trail
- Can be reactivated by admin later

---

## 🚧 TODO: Authentication Middleware

The endpoints are created but **not yet protected**. Next steps:

### 1. Create Middleware
```rust
// src/http/middleware/auth.rs
pub async fn require_admin(
    Extension(ctx): Extension<ApiContext>,
    headers: HeaderMap,
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response, AppError> {
    let token = extract_token(&headers)?;
    let claims = ctx.auth_service.verify_token(token)?;
    
    if !claims.scope.contains(&Scope::Admin) {
        return Err(AppError::Unauthorized);
    }
    
    Ok(next.run(req).await)
}
```

### 2. Apply to Routes
```rust
Router::new()
    .route("/users", post(create_user)
        .layer(middleware::from_fn(require_admin)))
    .route("/users/:id", delete(delete_user)
        .layer(middleware::from_fn(require_admin)))
```

### 3. Self-Service Routes
```rust
pub async fn require_auth_or_self(
    Extension(ctx): Extension<ApiContext>,
    Path(user_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<(), AppError> {
    let token = extract_token(&headers)?;
    let claims = ctx.auth_service.verify_token(token)?;
    
    // Allow if admin OR accessing own profile
    if claims.scope.contains(&Scope::Admin) || 
       Uuid::parse_str(&claims.sub).ok() == Some(user_id) {
        Ok(())
    } else {
        Err(AppError::Unauthorized)
    }
}
```

---

## 🧪 Testing User Management

### 1. Create Test Admin (if not exists)
```bash
psql "postgres://exchange_user:exchange_password@localhost/exchange_api" -f docs/create_test_user.sql
```

### 2. List Users
```bash
curl "http://localhost:8080/api/v1/users?merchant_id=123e4567-e89b-12d3-a456-426614174001"
```

### 3. Create a New User
```bash
curl -X POST http://localhost:8080/api/v1/users \
  -H "Content-Type: application/json" \
  -d '{
    "merchant_id": "123e4567-e89b-12d3-a456-426614174001",
    "email": "newuser@test-shop.com",
    "password": "Password123!",
    "display_name": "New User",
    "role": "viewer"
  }'
```

### 4. Get User Details
```bash
curl "http://localhost:8080/api/v1/users/a1111111-e89b-12d3-a456-426614174001"
```

### 5. Update User
```bash
curl -X PUT http://localhost:8080/api/v1/users/a1111111-e89b-12d3-a456-426614174001 \
  -H "Content-Type: application/json" \
  -d '{
    "display_name": "Updated Name",
    "role": "manager"
  }'
```

### 6. Deactivate User
```bash
curl -X DELETE http://localhost:8080/api/v1/users/a1111111-e89b-12d3-a456-426614174001
```

---

## ⚠️ Security Warnings

### Current State (UNPROTECTED)
**WARNING:** The endpoints are currently **publicly accessible** without authentication!

**Anyone can:**
- ❌ List all users
- ❌ Create admin accounts
- ❌ Change user roles
- ❌ Deactivate any user

**You MUST add authentication middleware before production!**

### Immediate Actions Required

1. ✅ **Add authentication middleware** (top priority)
2. ✅ **Switch to bcrypt** for password hashing
3. ✅ **Add rate limiting** on user creation
4. ✅ **Add audit logging** for user management actions
5. ✅ **Test thoroughly** with different roles

---

## 🎯 Security Checklist

Before deploying to production:

- [ ] Authentication middleware applied to all endpoints
- [ ] Admin-only routes restricted to Admin scope
- [ ] Self-service routes allow users to edit only their own profile
- [ ] Role changes restricted to Admin only
- [ ] Password changes require current password verification
- [ ] Audit logging enabled for user management actions
- [ ] Rate limiting applied to prevent abuse
- [ ] bcrypt used instead of SHA-256
- [ ] Email uniqueness validated
- [ ] Strong password requirements enforced
- [ ] User activation emails sent (if applicable)
- [ ] All tests passing

---

## 📊 Summary

**Created:**
- ✅ `src/http/users.rs` - User management endpoints
- ✅ User types in `src/http/types.rs`
- ✅ Router wired into `/api/v1/users`
- ✅ Soft delete implementation
- ✅ Password hashing
- ✅ Email validation

**Security Model:**
- 🔒 Admin-only user management
- 🔒 Self-service profile updates
- 🔒 Soft delete (preserve audit trail)
- ⚠️ **Middleware not yet applied - PUBLIC ACCESS**

**Next Priority:**
1. Add authentication middleware
2. Test with different user roles
3. Add audit logging

---

**Status:** ✅ Endpoints created, ⚠️ Security middleware needed

