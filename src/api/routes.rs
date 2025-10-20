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
    jobs,
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
        .nest("/jobs", jobs::routes(state.clone()))
        // .layer(from_fn_with_state(
        //     state.clone(),
        //     jwt_middleware::<PrivateClaims>,
        // ))
        .nest("/auth", auth::routes(state.clone()))
}
