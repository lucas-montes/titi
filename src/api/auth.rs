use axum::Extension;
use serde::{Deserialize, Serialize};
use stefn::{
    auth::JWTUserRequest,
    axum::{Json, Router, extract::State, response::IntoResponse, routing::get},
    axum_extra::{
        TypedHeader,
        headers::{Authorization, authorization::Basic},
    },
    service::ErrorMessage,
    state::APIState,
};
use utoipa::{self, OpenApi, ToResponse, ToSchema};

use super::services::generate_token;

pub type JWTUser = Extension<JWTUserRequest<PrivateClaims>>;

#[derive(Clone, Deserialize, Serialize)]
pub struct PrivateClaims {
    groups: String,
    company: i64,
}

impl PrivateClaims {
    pub fn new(groups: String, company: i64) -> Self {
        Self { groups, company }
    }
}

#[derive(Deserialize, Serialize, ToResponse, ToSchema)]
pub struct JWTResponse {
    jwt: String,
}
impl JWTResponse {
    pub fn new(jwt: String) -> Self {
        Self { jwt }
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(handle_get_token),
    components(schemas(JWTResponse), responses(JWTResponse)),
    tags(),
    security(("basic" = ["handle_get_token"])),
)]
pub struct ApiDoc;

pub fn routes(state: APIState) -> Router<APIState> {
    Router::new()
        .route("/token", get(handle_get_token))
        .with_state(state)
}

#[utoipa::path(
    get,
    path = "/token",
    responses(
        (status = 200, body = JWTResponse, description = "Login to get a token"),
        (status = "4XX", body = ErrorMessage, description = "Opusi daisy, you messed up"),
        (status = "5XX", body = ErrorMessage, description = "Opusi daisy, we messed up, sorry"),
    )
)]
async fn handle_get_token(
    state: State<APIState>,
    TypedHeader(basic): TypedHeader<Authorization<Basic>>,
) -> impl IntoResponse {
    generate_token(
        state.database(),
        state.encoding(),
        basic.username(),
        basic.password(),
        state.domain(),
    )
    .await
    .map(|t| Json(JWTResponse::new(t)))
}
