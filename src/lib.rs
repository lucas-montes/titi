mod api;
mod dashboard;
mod website;

use axum::{
    Router,
    http::{
        Method,
        header::{AUTHORIZATION, CONTENT_TYPE},
    },
    middleware::from_fn_with_state,
};

use stefn::{
    auth::{login_required_middleware, sessions_middleware},
    service::Service,
    state::WebsiteState,
};

use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};



pub fn create_api_service() -> Service {
    Service::api("API_", api::routes)
}

pub fn create_web_service() -> Service {
    Service::website("WEB_", routes)
}

fn routes(state: WebsiteState) -> Router<WebsiteState> {
    Router::new()
        .nest("/dashboard",dashboard::routes(state.clone()))
        .merge(website::routes(state.clone()))
        .layer(from_fn_with_state(state.clone(), sessions_middleware))
        .nest_service("/dist", ServeDir::new("dist"))
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::PATCH])
                .allow_headers([CONTENT_TYPE, AUTHORIZATION])
                .allow_origin(Any),
        )
        .with_state(state)
}
