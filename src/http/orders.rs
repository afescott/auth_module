use crate::http::ApiContext;
use axum::{
    response::IntoResponse,
    routing::{delete, get, post, put},
    Extension, Router,
};

pub fn orders_router() -> Router {
    Router::new()
        .route("/orders", get(list_orders).post(create_order))
        .route(
            "/orders/{id}",
            get(get_order).put(update_order).delete(delete_order),
        )
}

async fn list_orders(Extension(_ctx): Extension<ApiContext>) -> impl IntoResponse {
    "ok"
}
async fn create_order(Extension(_ctx): Extension<ApiContext>) -> impl IntoResponse {
    "ok"
}
async fn get_order(Extension(_ctx): Extension<ApiContext>) -> impl IntoResponse {
    "ok"
}
async fn update_order(Extension(_ctx): Extension<ApiContext>) -> impl IntoResponse {
    "ok"
}
async fn delete_order(Extension(_ctx): Extension<ApiContext>) -> impl IntoResponse {
    "ok"
}
