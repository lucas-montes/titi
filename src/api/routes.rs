use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable as ScalarServable};

use stefn::{
    auth::jwt_middleware,
    axum::{Router, middleware::from_fn_with_state},
    hyper::{
        Method,
        header::{AUTHORIZATION, CONTENT_TYPE},
    },
    state::APIState,
    tower_http::cors::{Any, CorsLayer},
};

use super::{
    auth::{self, PrivateClaims},
    docs::ApiDoc,
    test_routes,
};

pub fn routes(state: APIState) -> Router<APIState> {
    Router::new()
        .merge(Scalar::with_url("/docs", ApiDoc::openapi()))
        .nest("/api/{version}", api_routes(state.clone()))
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST])
                .allow_headers([CONTENT_TYPE, AUTHORIZATION])
                .allow_origin(Any),
        )
}

fn api_routes(state: APIState) -> Router<APIState> {
    Router::new()
        // Public routes (no authentication required)
        .nest("/test", test_routes::public_routes(state.clone()))
        .nest("/auth", auth::routes(state.clone()))
        // Protected routes (authentication required)
        .nest(
            "/protected",
            Router::new()
                .nest("/test", test_routes::protected_routes(state.clone()))
                .layer(from_fn_with_state(
                    state.clone(),
                    jwt_middleware::<PrivateClaims>,
                ))
        )
}
