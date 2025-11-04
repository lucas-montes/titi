use axum::{
    Extension, Json, Router,
    extract::{Path, Query},
    response::IntoResponse,
    routing::get,
};
use serde::{Deserialize, Serialize};
use stefn::{
    service::ErrorMessage,
    state::APIState,
};
use utoipa::{self, OpenApi, ToResponse, ToSchema, IntoParams};
use std::time::Duration;
use tokio::time::sleep;

use crate::api::auth::JWTUser;

#[derive(Deserialize, Serialize, ToResponse, ToSchema)]
pub struct HealthResponse {
    status: String,
    message: String,
}

#[derive(Deserialize, Serialize, ToResponse, ToSchema)]
pub struct ServiceInfoResponse {
    service: String,
    version: String,
    description: String,
}

#[derive(Deserialize, Serialize, ToResponse, ToSchema)]
pub struct UserResponse {
    id: u64,
    username: String,
    email: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct CreateUserRequest {
    username: String,
    email: String,
}

#[derive(Deserialize, Serialize, ToSchema, IntoParams)]
pub struct UserQuery {
    #[serde(default)]
    pub search: Option<String>,
}

#[derive(Deserialize, Serialize, ToResponse, ToSchema)]
pub struct PerformanceResponse {
    endpoint: String,
    processing_time_ms: u64,
    message: String,
}

#[derive(Deserialize, Serialize, ToResponse, ToSchema)]
pub struct ComputeResponse {
    result: u64,
    iterations: u64,
    processing_time_ms: u64,
}

#[derive(Deserialize, Serialize, ToSchema, IntoParams)]
pub struct PerformanceQuery {
    /// Optional delay in milliseconds (overrides default)
    #[serde(default)]
    pub delay_ms: Option<u64>,
}

#[derive(Deserialize, Serialize, ToSchema, IntoParams)]
pub struct ComputeQuery {
    /// Limit for computation (overrides default)
    #[serde(default)]
    pub limit: Option<u64>,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        health,
        public_info,
        fast_endpoint,
        medium_endpoint,
        slow_endpoint,
        very_slow_endpoint,
        cpu_intensive,
        memory_intensive,
        protected_user_info,
        get_users,
        get_user_by_id,
        create_user,
        update_user,
        delete_user,
    ),
    components(
        schemas(
            HealthResponse,
            ServiceInfoResponse,
            UserResponse,
            CreateUserRequest,
            UserQuery,
            PerformanceResponse,
            ComputeResponse,
            PerformanceQuery,
            ComputeQuery,
        ),
        responses(
            HealthResponse,
            ServiceInfoResponse,
            UserResponse,
            PerformanceResponse,
            ComputeResponse,
        )
    ),
    tags(),
)]
pub struct ApiDoc;

/// Public routes that don't require authentication
pub fn public_routes(state: APIState) -> Router<APIState> {
    Router::new()
        .route("/health", get(health))
        .route("/info", get(public_info))
        // Performance test endpoints
        .route("/perf/fast", get(fast_endpoint))
        .route("/perf/medium", get(medium_endpoint))
        .route("/perf/slow", get(slow_endpoint))
        .route("/perf/very-slow", get(very_slow_endpoint))
        .route("/perf/cpu-intensive", get(cpu_intensive))
        .route("/perf/memory-intensive", get(memory_intensive))
        .with_state(state)
}

/// Protected routes that require JWT authentication
pub fn protected_routes(state: APIState) -> Router<APIState> {
    Router::new()
        .route("/me", get(protected_user_info))
        .route("/users", get(get_users).post(create_user))
        .route("/users/{id}", get(get_user_by_id).put(update_user).delete(delete_user))
        .with_state(state)
}

/// Health check endpoint - PUBLIC
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, body = HealthResponse, description = "Service is healthy"),
    ),
    security(),
    tag = "Public"
)]
async fn health() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok".to_string(),
        message: "Service is running".to_string(),
    })
}

/// Public information endpoint - PUBLIC
#[utoipa::path(
    get,
    path = "/info",
    responses(
        (status = 200, body = ServiceInfoResponse, description = "Public service information"),
    ),
    security(),
    tag = "Public"
)]
async fn public_info() -> impl IntoResponse {
    Json(ServiceInfoResponse {
        service: "Elerem API".to_string(),
        version: "1.0.0".to_string(),
        description: "Test API with protected and unprotected routes".to_string(),
    })
}

/// Get current user information - PROTECTED
#[utoipa::path(
    get,
    path = "/me",
    responses(
        (status = 200, body = UserResponse, description = "Current user information"),
        (status = 401, body = ErrorMessage, description = "Unauthorized"),
    ),
    security(("jwt" = [])),
    tag = "Protected"
)]
async fn protected_user_info(
    Extension(_user): JWTUser,
) -> impl IntoResponse {
    Json(UserResponse {
        id: 1,
        username: "current_user".to_string(),
        email: "user@example.com".to_string(),
    })
}

/// List all users - PROTECTED
#[utoipa::path(
    get,
    path = "/users",
    params(UserQuery),
    responses(
        (status = 200, body = Vec<UserResponse>, description = "List of users"),
        (status = 401, body = ErrorMessage, description = "Unauthorized"),
    ),
    security(("jwt" = [])),
    tag = "Protected"
)]
async fn get_users(
    Extension(_user): JWTUser,
    Query(query): Query<UserQuery>,
) -> impl IntoResponse {
    let mut users = vec![
        UserResponse {
            id: 1,
            username: "alice".to_string(),
            email: "alice@example.com".to_string(),
        },
        UserResponse {
            id: 2,
            username: "bob".to_string(),
            email: "bob@example.com".to_string(),
        },
        UserResponse {
            id: 3,
            username: "charlie".to_string(),
            email: "charlie@example.com".to_string(),
        },
    ];

    // Apply search filter if provided
    if let Some(search) = query.search {
        users.retain(|u|
            u.username.to_lowercase().contains(&search.to_lowercase()) ||
            u.email.to_lowercase().contains(&search.to_lowercase())
        );
    }

    Json(users)
}

/// Get user by ID - PROTECTED
#[utoipa::path(
    get,
    path = "/users/{id}",
    params(
        ("id" = u64, Path, description = "User ID")
    ),
    responses(
        (status = 200, body = UserResponse, description = "User details"),
        (status = 404, body = ErrorMessage, description = "User not found"),
        (status = 401, body = ErrorMessage, description = "Unauthorized"),
    ),
    security(("jwt" = [])),
    tag = "Protected"
)]
async fn get_user_by_id(
    Extension(_user): JWTUser,
    Path(id): Path<u64>,
) -> impl IntoResponse {
    Json(UserResponse {
        id,
        username: format!("user{}", id),
        email: format!("user{}@example.com", id),
    })
}

/// Create a new user - PROTECTED
#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, body = UserResponse, description = "User created successfully"),
        (status = 400, body = ErrorMessage, description = "Invalid request"),
        (status = 401, body = ErrorMessage, description = "Unauthorized"),
    ),
    security(("jwt" = [])),
    tag = "Protected"
)]
async fn create_user(
    Extension(_user): JWTUser,
    Json(payload): Json<CreateUserRequest>,
) -> impl IntoResponse {
    Json(UserResponse {
        id: 999,
        username: payload.username,
        email: payload.email,
    })
}

/// Update a user - PROTECTED
#[utoipa::path(
    put,
    path = "/users/{id}",
    params(
        ("id" = u64, Path, description = "User ID")
    ),
    request_body = CreateUserRequest,
    responses(
        (status = 200, body = UserResponse, description = "User updated successfully"),
        (status = 404, body = ErrorMessage, description = "User not found"),
        (status = 401, body = ErrorMessage, description = "Unauthorized"),
    ),
    security(("jwt" = [])),
    tag = "Protected"
)]
async fn update_user(
    Extension(_user): JWTUser,
    Path(id): Path<u64>,
    Json(payload): Json<CreateUserRequest>,
) -> impl IntoResponse {
    Json(UserResponse {
        id,
        username: payload.username,
        email: payload.email,
    })
}

/// Delete a user - PROTECTED
#[utoipa::path(
    delete,
    path = "/users/{id}",
    params(
        ("id" = u64, Path, description = "User ID")
    ),
    responses(
        (status = 204, description = "User deleted successfully"),
        (status = 404, body = ErrorMessage, description = "User not found"),
        (status = 401, body = ErrorMessage, description = "Unauthorized"),
    ),
    security(("jwt" = [])),
    tag = "Protected"
)]
async fn delete_user(
    Extension(_user): JWTUser,
    Path(_id): Path<u64>,
) -> impl IntoResponse {
    // Return 204 No Content
    (axum::http::StatusCode::NO_CONTENT, ())
}

// ============================================================================
// Performance Test Endpoints
// ============================================================================

/// Fast endpoint - ~10ms response - PUBLIC
#[utoipa::path(
    get,
    path = "/perf/fast",
    params(PerformanceQuery),
    responses(
        (status = 200, body = PerformanceResponse, description = "Fast response (~10ms)"),
    ),
    security(),
    tag = "Performance"
)]
async fn fast_endpoint(Query(query): Query<PerformanceQuery>) -> impl IntoResponse {
    let start = std::time::Instant::now();
    let delay = query.delay_ms.unwrap_or(10);
    sleep(Duration::from_millis(delay)).await;
    let elapsed = start.elapsed().as_millis() as u64;

    Json(PerformanceResponse {
        endpoint: "fast".to_string(),
        processing_time_ms: elapsed,
        message: format!("This endpoint responds in ~{}ms", delay),
    })
}

/// Medium endpoint - ~100ms response - PUBLIC
#[utoipa::path(
    get,
    path = "/perf/medium",
    params(PerformanceQuery),
    responses(
        (status = 200, body = PerformanceResponse, description = "Medium response (~100ms)"),
    ),
    security(),
    tag = "Performance"
)]
async fn medium_endpoint(Query(query): Query<PerformanceQuery>) -> impl IntoResponse {
    let start = std::time::Instant::now();
    let delay = query.delay_ms.unwrap_or(100);
    sleep(Duration::from_millis(delay)).await;
    let elapsed = start.elapsed().as_millis() as u64;

    Json(PerformanceResponse {
        endpoint: "medium".to_string(),
        processing_time_ms: elapsed,
        message: format!("This endpoint responds in ~{}ms", delay),
    })
}

/// Slow endpoint - ~500ms response - PUBLIC
#[utoipa::path(
    get,
    path = "/perf/slow",
    params(PerformanceQuery),
    responses(
        (status = 200, body = PerformanceResponse, description = "Slow response (~500ms)"),
    ),
    security(),
    tag = "Performance"
)]
async fn slow_endpoint(Query(query): Query<PerformanceQuery>) -> impl IntoResponse {
    let start = std::time::Instant::now();
    let delay = query.delay_ms.unwrap_or(500);
    sleep(Duration::from_millis(delay)).await;
    let elapsed = start.elapsed().as_millis() as u64;

    Json(PerformanceResponse {
        endpoint: "slow".to_string(),
        processing_time_ms: elapsed,
        message: format!("This endpoint responds in ~{}ms", delay),
    })
}

/// Very slow endpoint - ~2s response - PUBLIC
#[utoipa::path(
    get,
    path = "/perf/very-slow",
    params(PerformanceQuery),
    responses(
        (status = 200, body = PerformanceResponse, description = "Very slow response (~2s)"),
    ),
    security(),
    tag = "Performance"
)]
async fn very_slow_endpoint(Query(query): Query<PerformanceQuery>) -> impl IntoResponse {
    let start = std::time::Instant::now();
    let delay = query.delay_ms.unwrap_or(2000);
    sleep(Duration::from_millis(delay)).await;
    let elapsed = start.elapsed().as_millis() as u64;

    Json(PerformanceResponse {
        endpoint: "very_slow".to_string(),
        processing_time_ms: elapsed,
        message: format!("This endpoint responds in ~{}ms", delay),
    })
}

/// CPU intensive computation - PUBLIC
#[utoipa::path(
    get,
    path = "/perf/cpu-intensive",
    params(ComputeQuery),
    responses(
        (status = 200, body = ComputeResponse, description = "CPU intensive computation"),
    ),
    security(),
    tag = "Performance"
)]
async fn cpu_intensive(Query(query): Query<ComputeQuery>) -> impl IntoResponse {
    let start = std::time::Instant::now();

    // Compute prime numbers up to a limit (CPU intensive)
    let limit = query.limit.unwrap_or(100_000);
    let mut count = 0u64;

    for num in 2..=limit {
        let mut is_prime = true;
        for i in 2..=(num as f64).sqrt() as u64 {
            if num % i == 0 {
                is_prime = false;
                break;
            }
        }
        if is_prime {
            count += 1;
        }
    }

    let elapsed = start.elapsed().as_millis() as u64;

    Json(ComputeResponse {
        result: count,
        iterations: limit,
        processing_time_ms: elapsed,
    })
}

/// Memory intensive operation - PUBLIC
#[utoipa::path(
    get,
    path = "/perf/memory-intensive",
    params(ComputeQuery),
    responses(
        (status = 200, body = ComputeResponse, description = "Memory intensive operation"),
    ),
    security(),
    tag = "Performance"
)]
async fn memory_intensive(Query(query): Query<ComputeQuery>) -> impl IntoResponse {
    let start = std::time::Instant::now();

    // Allocate and process large data structures
    let size = query.limit.unwrap_or(1_000_000);
    let mut data: Vec<u64> = (0..size).collect();

    // Do some operations on the data
    data.sort();
    data.reverse();
    let sum: u64 = data.iter().take(1000).sum();

    let elapsed = start.elapsed().as_millis() as u64;

    Json(ComputeResponse {
        result: sum,
        iterations: size,
        processing_time_ms: elapsed,
    })
}
