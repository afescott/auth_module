use axum::{response::IntoResponse, routing::get, Extension, Router};
use crate::http::ApiContext;

pub fn inventory_router() -> Router {
    Router::new().route("/items", get(list_items))
    /* .post(create_item))
    .route(
        "/items/:id",
        get(get_item).put(update_item).delete(delete_item),
    ) */
}

async fn list_items(Extension(_context): Extension<ApiContext>) -> impl IntoResponse { "ok" }
