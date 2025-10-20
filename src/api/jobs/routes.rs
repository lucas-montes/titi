use axum::{
    Extension, Json, Router,
    extract::{Query, State},
    routing::get,
};

use stefn::{
    service::{AppResult, ErrorMessage, PaginatedResponse},
    state::APIState,
};
use utoipa::{self, OpenApi};

use crate::api::auth::JWTUser;

use super::{
    applications::{count_jobs, fetch_jobs},
    dtos::{JobFilters, JobResponse},
};

#[derive(OpenApi)]
#[openapi(
    paths(jobs),
    components(schemas(
        JobFilters,
        JobResponse
    ),
    responses(JobResponse)),
    security(("jwt" = [])),
    tags()
)]
pub struct ApiDoc;

pub fn routes(state: APIState) -> Router<APIState> {
    Router::new().route("/", get(jobs)).with_state(state)
}

#[utoipa::path(
    get,
    path = "",
    params(JobFilters),
    responses(
        (status = 200, body = PaginatedResponse<JobResponse>, description = "Fetch jobs based on the request"),
        (status = "4XX", body = ErrorMessage, description = "Oupsi daisy, you messed up"),
        (status = "5XX", body = ErrorMessage, description = "Oupsi daisy, we messed up, sorry"),
    )
)]
async fn jobs(
    // Extension(current_user): JWTUser,
    State(state): State<APIState>,
    Query(payload): Query<JobFilters>,
) -> AppResult<PaginatedResponse<JobResponse>> {
    tokio::try_join!(
        fetch_jobs(state.database(), &payload),
        count_jobs(state.database(), &payload)
    )
    .map(|(jobs, count_result)| Json(PaginatedResponse::new(jobs, count_result as u64)))
}
