mod jwks;
mod login;

use axum::Router;

pub fn auth_router() -> Router {
    Router::new()
        .merge(jwks::jwks_router())
        .merge(login::login_router())
}
