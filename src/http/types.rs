use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Not found")]
    NotFound,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message, message) = match &self {
            AppError::Database(e) => {
                // Log the full error for debugging
                eprintln!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error",
                    format!("Database error: {}", e)
                )
            },
            AppError::Validation(ref msg) => (StatusCode::BAD_REQUEST, "Validation error", msg.clone()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Resource not found", "Resource not found".to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized", "Unauthorized".to_string()),
            AppError::Internal(ref msg) => {
                eprintln!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error", msg.clone())
            },
        };

        let body = Json(serde_json::json!({
            "error": error_message,
            "message": message
        }));

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<Json<T>, AppError>;

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Product {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub shopify_product_id: i64,
    pub title: Option<String>,
    pub product_type: Option<String>,
    pub status: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Variant {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub shopify_variant_id: i64,
    pub shopify_product_id: i64,
    pub sku: Option<String>,
    pub title: Option<String>,
    pub barcode: Option<String>,
    pub weight: Option<f64>,
    pub weight_unit: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProductWithVariants {
    #[serde(flatten)]
    pub product: Product,
    pub variants: Vec<Variant>,
    pub variant_count: i64,
}

#[derive(Deserialize)]
pub struct ListProductsParams {
    pub merchant_id: Uuid,
    pub product_type: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Deserialize)]
pub struct CreateProductRequest {
    pub merchant_id: Uuid,
    pub shopify_product_id: i64,
    pub title: Option<String>,
    pub product_type: Option<String>,
    pub status: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateProductRequest {
    pub title: Option<String>,
    pub product_type: Option<String>,
    pub status: Option<String>,
}

#[derive(Serialize)]
pub struct ProductListResponse {
    pub products: Vec<ProductWithVariants>,
    pub total: i64,
    pub limit: i32,
    pub offset: i32,
}

// Orders
#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Order {
    pub id: i64,
    pub merchant_id: Uuid,
    pub shopify_order_id: i64,
    pub name: Option<String>,
    pub processed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub currency: Option<String>,
    pub subtotal_price: Option<rust_decimal::Decimal>,
    pub total_price: Option<rust_decimal::Decimal>,
    pub total_discounts: Option<rust_decimal::Decimal>,
    pub total_shipping_price_set_amount: Option<rust_decimal::Decimal>,
    pub total_tax: Option<rust_decimal::Decimal>,
    pub financial_status: Option<String>,
    pub cancelled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
pub struct ListOrdersParams {
    pub merchant_id: Uuid,
    pub financial_status: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Deserialize)]
pub struct CreateOrderRequest {
    pub merchant_id: Uuid,
    pub shopify_order_id: i64,
    pub name: Option<String>,
    pub processed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub currency: Option<String>,
    pub subtotal_price: Option<rust_decimal::Decimal>,
    pub total_price: Option<rust_decimal::Decimal>,
    pub total_discounts: Option<rust_decimal::Decimal>,
    pub total_shipping_price_set_amount: Option<rust_decimal::Decimal>,
    pub total_tax: Option<rust_decimal::Decimal>,
    pub financial_status: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateOrderRequest {
    pub name: Option<String>,
    pub financial_status: Option<String>,
    pub cancelled_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize)]
pub struct OrderListResponse {
    pub orders: Vec<Order>,
    pub total: i64,
    pub limit: i32,
    pub offset: i32,
}

// Inventory Items
#[derive(Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct InventoryItem {
    pub id: Uuid,
    pub merchant_id: Uuid,
    pub shopify_inventory_item_id: i64,
    pub shopify_variant_id: Option<i64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
pub struct ListInventoryItemsParams {
    pub merchant_id: Uuid,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Deserialize)]
pub struct CreateInventoryItemRequest {
    pub merchant_id: Uuid,
    pub shopify_inventory_item_id: i64,
    pub shopify_variant_id: Option<i64>,
}

#[derive(Deserialize)]
pub struct UpdateInventoryItemRequest {
    pub shopify_variant_id: Option<i64>,
}

#[derive(Serialize)]
pub struct InventoryItemListResponse {
    pub items: Vec<InventoryItem>,
    pub total: i64,
    pub limit: i32,
    pub offset: i32,
}