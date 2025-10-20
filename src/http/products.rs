use crate::http::ApiContext;
use axum::{response::IntoResponse, routing::get, Extension, Router};

pub fn products_router() -> Router {
    Router::new()
        .route("/products", get(list_products))
        .post(create_product)
        .route(
            "/products/:id",
            get(get_product).put(update_product).delete(delete_product),
        )
}

async fn list_products(Extension(_ctx): Extension<ApiContext>) -> impl IntoResponse {
    "ok"
}
