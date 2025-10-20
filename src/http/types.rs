use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// Removed validator dependency - using thiserror instead

/// Standard API response wrapper for successful operations
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    pub message: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data,
            message: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn success_with_message(data: T, message: String) -> Self {
        Self {
            success: true,
            data,
            message: Some(message),
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Standard API error response
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub success: bool,
    pub error: ErrorDetails,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
    pub details: Option<HashMap<String, serde_json::Value>>,
}

impl ApiError {
    pub fn new(code: String, message: String) -> Self {
        Self {
            success: false,
            error: ErrorDetails {
                code,
                message,
                details: None,
            },
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn with_details(
        code: String,
        message: String,
        details: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            success: false,
            error: ErrorDetails {
                code,
                message,
                details: Some(details),
            },
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Simple validation error type
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Field '{field}' is invalid: {message}")]
    FieldError { field: String, message: String },
    #[error("Validation failed: {message}")]
    General { message: String },
    #[error("Invalid email format")]
    InvalidEmail,
    #[error(
        "Password does not meet requirements: must be at least 8 characters with uppercase, lowercase, digit, and special character"
    )]
    WeakPassword,
}

/// Application error types for exchange API
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Validation failed: {0}")]
    Validation(#[from] ValidationError),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("User not found")]
    UserNotFound,

    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Token expired")]
    TokenExpired,

    #[error("Insufficient permissions")]
    InsufficientPermissions,

    #[error("KYC verification required")]
    KYCVerificationRequired,

    #[error("Account locked")]
    AccountLocked,

    #[error("Account suspended")]
    AccountSuspended,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Invalid 2FA code")]
    Invalid2FACode,

    #[error("2FA not enabled")]
    TwoFactorNotEnabled,

    #[error("Session expired")]
    SessionExpired,

    #[error("Order not found")]
    OrderNotFound,

    #[error("Insufficient balance")]
    InsufficientBalance,

    #[error("Invalid order type")]
    InvalidOrderType,

    #[error("Invalid order status")]
    InvalidOrderStatus,

    #[error("Market not found")]
    MarketNotFound,

    #[error("Trading suspended")]
    TradingSuspended,

    #[error("Wallet not found")]
    WalletNotFound,

    #[error("Invalid currency")]
    InvalidCurrency,

    #[error("Withdrawal limit exceeded")]
    WithdrawalLimitExceeded,

    #[error("Deposit limit exceeded")]
    DepositLimitExceeded,

    #[error("Internal server error")]
    InternalServerError,

    #[error("External service error: {0}")]
    ExternalService(String),

    #[error("Configuration error: {0}")]
    Configuration(String),
}

impl AppError {
    pub fn error_code(&self) -> String {
        match self {
            AppError::Validation(_) => "VALIDATION_ERROR".to_string(),
            AppError::Database(_) => "DATABASE_ERROR".to_string(),
            AppError::UserNotFound => "USER_NOT_FOUND".to_string(),
            AppError::UserAlreadyExists => "USER_ALREADY_EXISTS".to_string(),
            AppError::InvalidCredentials => "INVALID_CREDENTIALS".to_string(),
            AppError::InvalidToken => "INVALID_TOKEN".to_string(),
            AppError::TokenExpired => "TOKEN_EXPIRED".to_string(),
            AppError::InsufficientPermissions => "INSUFFICIENT_PERMISSIONS".to_string(),
            AppError::KYCVerificationRequired => "KYC_VERIFICATION_REQUIRED".to_string(),
            AppError::AccountLocked => "ACCOUNT_LOCKED".to_string(),
            AppError::AccountSuspended => "ACCOUNT_SUSPENDED".to_string(),
            AppError::RateLimitExceeded => "RATE_LIMIT_EXCEEDED".to_string(),
            AppError::Invalid2FACode => "INVALID_2FA_CODE".to_string(),
            AppError::TwoFactorNotEnabled => "2FA_NOT_ENABLED".to_string(),
            AppError::SessionExpired => "SESSION_EXPIRED".to_string(),
            AppError::OrderNotFound => "ORDER_NOT_FOUND".to_string(),
            AppError::InsufficientBalance => "INSUFFICIENT_BALANCE".to_string(),
            AppError::InvalidOrderType => "INVALID_ORDER_TYPE".to_string(),
            AppError::InvalidOrderStatus => "INVALID_ORDER_STATUS".to_string(),
            AppError::MarketNotFound => "MARKET_NOT_FOUND".to_string(),
            AppError::TradingSuspended => "TRADING_SUSPENDED".to_string(),
            AppError::WalletNotFound => "WALLET_NOT_FOUND".to_string(),
            AppError::InvalidCurrency => "INVALID_CURRENCY".to_string(),
            AppError::WithdrawalLimitExceeded => "WITHDRAWAL_LIMIT_EXCEEDED".to_string(),
            AppError::DepositLimitExceeded => "DEPOSIT_LIMIT_EXCEEDED".to_string(),
            AppError::InternalServerError => "INTERNAL_SERVER_ERROR".to_string(),
            AppError::ExternalService(_) => "EXTERNAL_SERVICE_ERROR".to_string(),
            AppError::Configuration(_) => "CONFIGURATION_ERROR".to_string(),
        }
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::UserNotFound
            | AppError::OrderNotFound
            | AppError::MarketNotFound
            | AppError::WalletNotFound => StatusCode::NOT_FOUND,
            AppError::InvalidCredentials
            | AppError::InvalidToken
            | AppError::TokenExpired
            | AppError::SessionExpired => StatusCode::UNAUTHORIZED,
            AppError::InsufficientPermissions
            | AppError::KYCVerificationRequired
            | AppError::AccountLocked
            | AppError::AccountSuspended => StatusCode::FORBIDDEN,
            AppError::UserAlreadyExists => StatusCode::CONFLICT,
            AppError::InvalidOrderType
            | AppError::InvalidOrderStatus
            | AppError::InvalidCurrency => StatusCode::BAD_REQUEST,
            AppError::InsufficientBalance
            | AppError::WithdrawalLimitExceeded
            | AppError::DepositLimitExceeded => StatusCode::BAD_REQUEST,
            AppError::TradingSuspended => StatusCode::SERVICE_UNAVAILABLE,
            AppError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            AppError::Invalid2FACode | AppError::TwoFactorNotEnabled => StatusCode::BAD_REQUEST,
            AppError::Database(_)
            | AppError::InternalServerError
            | AppError::ExternalService(_)
            | AppError::Configuration(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn to_api_error(&self) -> ApiError {
        match self {
            AppError::Validation(error) => {
                let mut details = HashMap::new();
                match error {
                    ValidationError::FieldError { field, message } => {
                        let field_errors = HashMap::from([(field.clone(), vec![message.clone()])]);
                        details.insert(
                            "field_errors".to_string(),
                            serde_json::to_value(field_errors).unwrap(),
                        );
                        ApiError::with_details(self.error_code(), "Validation failed".to_string(), details)
                    }
                    ValidationError::General { message } => {
                        ApiError::new(self.error_code(), message.clone())
                    }
                    ValidationError::InvalidEmail => {
                        ApiError::new(self.error_code(), "Invalid email format".to_string())
                    }
                    ValidationError::WeakPassword => {
                        ApiError::new(self.error_code(), "Password does not meet requirements: must be at least 8 characters with uppercase, lowercase, digit, and special character".to_string())
                    }
                }
            }
            _ => ApiError::new(self.error_code(), self.to_string()),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let api_error = self.to_api_error();
        (status, Json(api_error)).into_response()
    }
}

/// Pagination metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationMeta {
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}

/// Paginated response wrapper
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub success: bool,
    pub data: Vec<T>,
    pub pagination: PaginationMeta,
    pub message: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, pagination: PaginationMeta) -> Self {
        Self {
            success: true,
            data,
            pagination,
            message: None,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Empty response for operations that don't return data
#[derive(Debug, Serialize)]
pub struct EmptyResponse {
    pub success: bool,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl EmptyResponse {
    pub fn success(message: String) -> Self {
        Self {
            success: true,
            message,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Helper trait for converting results to API responses
pub trait IntoApiResponse<T> {
    fn into_api_response(self) -> Result<Json<ApiResponse<T>>, AppError>;
}

impl<T> IntoApiResponse<T> for Result<T, AppError> {
    fn into_api_response(self) -> Result<Json<ApiResponse<T>>, AppError> {
        match self {
            Ok(data) => Ok(Json(ApiResponse::success(data))),
            Err(error) => Err(error),
        }
    }
}
