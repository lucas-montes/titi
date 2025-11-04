use std::borrow::Cow;

use axum::{
    Json, Router,
    extract::{Path, ws::{WebSocket, WebSocketUpgrade}},
    response::IntoResponse,
    routing::{get, post},
};

use chrono::{DateTime, Utc};
use fuzzer::Configuration;
use serde::{Deserialize, Serialize};
use stefn::{
    askama::Template,
    state::WebsiteState,
    website::meta_tags::{ContentSecurityPolicy, Meta},
};

use crate::website::{HtmlResult, template_to_response};

pub const CSP: ContentSecurityPolicy<'static> = ContentSecurityPolicy {
    base_uri: Cow::Borrowed("'self'"),
    child_src: Cow::Borrowed("'self'"),
    connect_src: Cow::Borrowed("'self' https://cdnjs.cloudflare.com https://cdn.jsdelivr.net"),
    default_src: Cow::Borrowed("'self'"),
    font_src: Cow::Borrowed(
        "'self' https://fonts.gstatic.com https://cdnjs.cloudflare.com https://cdn.jsdelivr.net data:",
    ),
    form_action: Cow::Borrowed("'self'"),
    frame_src: Cow::Borrowed("'self'"),
    img_src: Cow::Borrowed(
        "'self' https://ik.imagekit.io/smartlinker/images/ https://img.stackshare.io/ data: blob:",
    ),
    manifest_src: Cow::Borrowed("'self'"),
    media_src: Cow::Borrowed("'self'"),
    object_src: Cow::Borrowed("'none'"),
    report_to: Cow::Borrowed("default"),
    // Empty to disable TrustedHTML requirement (libraries don't support it)
    require_trusted_types_for: Cow::Borrowed(""),
    script_src: Cow::Borrowed(
        "'self' https://unpkg.com https://cdn.jsdelivr.net https://cdnjs.cloudflare.com https://code.jquery.com https://challenges.cloudflare.com",
    ),
    script_src_attr: Cow::Borrowed("'self' 'unsafe-hashes'"),
    script_src_elem: Cow::Borrowed(
        "'self' https://unpkg.com https://cdn.jsdelivr.net https://cdnjs.cloudflare.com https://code.jquery.com https://challenges.cloudflare.com 'sha256-YpkVwgje1aO9XrJNyzcGQAECa/MK8XpN8s0LJ7VZqOc=' 'sha256-wO9/YiVvvFYKfhfsNjHZA3MkQhTz8W7MV9wFEdTNHP8=' 'sha256-u/6IAePoKeLn7yKgppkixEUUGPftRUY5XarwphH4OWA=' 'sha256-bWvwt1kqwx6+1OLGwETVGtoGf99B4az6CLpPtjmq4XE=' 'sha256-uHAx33BX/V1CXQkQ1bv6Ye3ZapqiZ81s6DxH8fqyOc4=' 'sha256-ymPBL/Gqi0weYO11V4kCEXopcGPFjEnc6vqq+ShRjHI='",
    ),
    // NOTE: 'unsafe-hashes' allows hashes for event handlers and style attributes
    style_src: Cow::Borrowed(
        "'self' https://cdn.jsdelivr.net https://cdnjs.cloudflare.com https://fonts.googleapis.com",
    ),
    style_src_attr: Cow::Borrowed(
        "'self' 'unsafe-hashes' 'sha256-biLFinpqYMtWHmXfkA1BPeCY0/fNt46SAZ+BBk5YUog=' 'sha256-hP0ISKcRz40G33clBjx741T6tbj1HTLiD63Kt3dHg0w=' 'sha256-u2Yctlkg1MQj7vO+nL13o9rBQFOBaGRouQGeAzNGvTc=' 'sha256-wedAagVGmy5Yu77EIUr+ef8lxqcy6Os/UyFq+ZDDB7o=' 'sha256-8gF21csQ4qeV/ZE3tho1N1P5zeGJ6e6N3K0Y5KU5Eas=' 'sha256-Me+0qsmODl9NAZSdTljs1+7j0sShz+U2hWOR//JGbh0=' 'sha256-w+uGU1wwxz2eo1tX54eWDaGGRsG3jwdbNiqZVR6s2uA=' 'sha256-ysu4kz3viHcMq18cHiJ7iS4a3ML8obZjxS2ejiKT47E=' 'sha256-eARFct61EylCvIfQuUQu1OZA8h7eJvSIa9M3ebb5uJA=' 'sha256-CLC1c4VNJL2uHK1VjQZNRXp2weHQdHTmDXMOlGGxhas=' 'sha256-2EA12+9d+s6rrc0rkdIjfmjbh6p2o0ZSXs4wbZuk/tA=' 'sha256-fTSNtSIktGhitxPAqTp5/WFv/eHDMEdQSM94bm2GL5w=' 'sha256-+7shjGMzKo5tzDsPcF3K6ZIZlzvE9E9zOiWiSzKjbh0=' 'sha256-C82c7tBEEimqSaZyL0VT3yXk5pvsaOjwZH6T+IvPCpE=' 'sha256-s7Jbt2iVKScn19bYfh3U5kM6Nuy42B9GvIZzc0l4pS8=' 'sha256-kpUZYMaCb92v60Wk8bJmW9HUcL0QUIp0oRPVLUg1ckc=' 'sha256-HzJAZ0iYlehU9oowuc4mL4B5oMu1xZhTj9Uv2MLrq84=' 'sha256-cb/4s2poPUvIr4GY39QqifxBdwuNzJa+A/pyx9mo+i0=' 'sha256-c90w6TFz+K4SkTDe+9p866CkUZyZEIgzQm8P0HBDadc=' 'sha256-b/s1++VItJvEnZ5kj0t5giTcvdbQrz+Xj04P4/0ckAU=' 'sha256-IOWYPccv4+GIAWz50PQ4hgBzwty+G8ckj9XrN5jdx6g=' 'sha256-hWFDCsd01INgKZATtnNmxKvFmqJ41aYlvjn6S6IeBW8=' 'sha256-NjYDAvf3Yswi9GqXn8q5mE3okYa3Q4PuzJ0DkAhe4yQ=' 'sha256-2v45m16Zom+oK2TtTcJY+szaesAYhdXY6R5Z282p7pE=' 'sha256-73uNzxKntFn/DtVnEIUbutr5HIvkj1uAw/3gfX+jJQ8=' 'sha256-WJKicgQ+eScMkZiwZRdDEFNfpHS7hwJ+gGWIVC/ymIY=' 'sha256-xVEK7gcaeJAqZHgUjP0ktYmyVcW5brxPqD47mQeu5uw=' 'sha256-c4tDLLDQ1B0Ls+XIkqmORJWQCZ+pIdv9CnHDlfw5otA=' 'sha256-OShiAjSV3lAIOBSclt31Fojewx+uXVNt3v1TfMdz520=' 'sha256-z/JfPWZgPJvGuwZaa/FMvEw1R3/Q7Zf+WYeybn4dTOQ=' 'sha256-TuFENwSfPqkgcx1buF2Kr8jrk73OCy0EjKG+gUSrygU=' 'sha256-vBOzc3Pj55jioMIsAMlosbrlh4AtvKGJYUOTJBlREMs=' 'sha256-jFYqAZz3K7fwtDz0oDN+EYwHjORdb2pmOjjjePIhQR8=' 'sha256-AxTCQK6EZEOY9u3uW/YKPqu+T1aH0UOpDJm/YaEc0ZY=' 'sha256-Do1hZ7J1z5h1z0x4Lln9dkUzMOkFaJaL01Opa447kcM=' 'sha256-rUttUyi4zkWsDoVte8B8PfnT3HuR36RyLG9TMk3r/Bw=' 'sha256-QNPnk6B/0pM6VhmqozF8ZZIiTCvEs8FdGp3efQt/cXg=' 'sha256-jIQ1dBFSf6awxPSyMGe36SYLIV33mQhlAXMl2fbj67Y=' 'sha256-6gHgm+T2Oerkr61VJ1H0OzSAlyw9DDIvvJtOYGNRhoM='",
    ),
    style_src_elem: Cow::Borrowed(
        "'self' https://cdn.jsdelivr.net https://cdnjs.cloudflare.com https://fonts.googleapis.com 'sha256-bsV5JivYxvGywDAZ22EZJKBFip65Ng9xoJVLbBg7bdo=' 'sha256-HgfNXHMfocj7T+6Sf8drepfYA1mLM/fgAlgSFRaSHCI=' 'sha256-N9YYUEhxqoRWOg8r6hNcajPLxKHXTifLIy6ykPVyU1E=' 'sha256-SSGCrClQA5HDwURccGOBm8CCOfMt1gmANv5KkeZERXs=' 'sha256-vPVXKObmqb7tOFKmg/fCzGZLurO5wqxUQ95EbaaZZm0='",
    ),
    trusted_types: Cow::Borrowed(""),
    worker_src: Cow::Borrowed("'none'"),
};

pub fn routes(state: WebsiteState) -> Router<WebsiteState> {
    Router::new()
        .route("/", get(dashboard))
        .route("/test-suite", get(test_suite))
        .route("/test-suite/security", get(test_suite_security))
        .route("/test-suite/compliance", get(test_suite_compliance))
        .route("/test-suite/correctness", get(test_suite_correctness))
        .route("/test-suite/performance", get(test_suite_performance))
        .route("/test-suite/upload", get(test_suite_upload))
        .route("/test-suite/metrics/{job_id}", get(test_suite_metrics))
        .route("/api/test-runs", get(api_test_runs))
        .route("/api/test-metrics", get(api_test_metrics))
        .route("/api/performance-data", get(api_performance_data))
        // Fuzzer API integration
        .route("/api/fuzzer/start", post(api_fuzzer_start))
        .route("/api/fuzzer/stop", post(api_fuzzer_stop))
        .route("/api/fuzzer/metrics/{job_id}", get(api_fuzzer_metrics))
        .route("/api/fuzzer/ws/{job_id}", get(api_fuzzer_websocket))
        .route("/api/fuzzer/jobs", get(api_fuzzer_jobs))
        .with_state(state)
}

struct User {
    username: String,
    email: String,
}

impl User {
    pub fn is_paying_customer(&self) -> bool {
        // Placeholder logic for determining if the user is a paying customer
        self.email.ends_with("@example.com")
    }
    pub fn picture(&self) -> &str {
        "https://ik.imagekit.io/smartlinker/images/avatars/user.png"
    }
    pub fn name(&self) -> &str {
        &self.username
    }
    pub fn is_admin(&self) -> bool {
        self.username == "admin"
    }
}

#[derive(Template)]
#[template(path = "dashboard/home/index.html")]
struct DashboardTemplate<'a> {
    meta: Meta<'a>,
    user: User,
}

async fn dashboard() -> HtmlResult {
    let meta = Meta {
        meta_title: "JobyJoba".into(),
        csp_policy: CSP,
        ..Default::default()
    };
    let user = User {
        username: "admin".into(),
        email: "".into(),
    };
    let template = DashboardTemplate { meta, user };
    template_to_response(&template)
}

#[derive(Template)]
#[template(path = "dashboard/test-suite/index.html")]
struct TestSuiteTemplate<'a> {
    meta: Meta<'a>,
    user: User,
}

async fn test_suite() -> HtmlResult {
    let meta = Meta {
        meta_title: "JobyJoba".into(),
        csp_policy: CSP,
        ..Default::default()
    };
    let user = User {
        username: "admin".into(),
        email: "admin".into(),
    };
    let template = TestSuiteTemplate { meta, user };
    template_to_response(&template)
}

#[derive(Template)]
#[template(path = "dashboard/test-suite/security.html")]
struct SecurityTestTemplate<'a> {
    meta: Meta<'a>,
    user: User,
}

#[derive(Template)]
#[template(path = "dashboard/test-suite/compliance.html")]
struct ComplianceTestTemplate<'a> {
    meta: Meta<'a>,
    user: User,
}

#[derive(Template)]
#[template(path = "dashboard/test-suite/correctness.html")]
struct CorrectnessTestTemplate<'a> {
    meta: Meta<'a>,
    user: User,
}

#[derive(Template)]
#[template(path = "dashboard/test-suite/performance.html")]
struct PerformanceTestTemplate<'a> {
    meta: Meta<'a>,
    user: User,
}

// API Data Structures
#[derive(Serialize, Deserialize)]
struct TestRun {
    id: String,
    suite_name: String,
    phase: TestPhase,
    status: TestStatus,
    duration: String,
    total_tests: u32,
    passed: u32,
    failed: u32,
    success_rate: f32,
    started_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TestPhase {
    Security,
    Compliance,
    Correctness,
    Performance,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TestStatus {
    Completed,
    Running,
    Failed,
    Queued,
}

#[derive(Serialize, Deserialize)]
struct TestMetrics {
    total_tests: u32,
    passed_tests: u32,
    failed_tests: u32,
    success_rate: f32,
}

#[derive(Serialize, Deserialize)]
struct PerformanceData {
    timestamp: DateTime<Utc>,
    requests_per_second: f32,
    avg_response_time: f32,
    error_rate: f32,
    active_users: u32,
    percentiles: ResponsePercentiles,
}

#[derive(Serialize, Deserialize)]
struct ResponsePercentiles {
    p50: f32,
    p90: f32,
    p95: f32,
    p99: f32,
    max: f32,
}

async fn test_suite_security() -> HtmlResult {
    let meta = Meta {
        meta_title: "Security Testing - JobyJoba".into(),
        csp_policy: CSP,
        ..Default::default()
    };
    let user = User {
        username: "admin".into(),
        email: "admin".into(),
    };
    let template = SecurityTestTemplate { meta, user };
    template_to_response(&template)
}

async fn test_suite_compliance() -> HtmlResult {
    let meta = Meta {
        meta_title: "Compliance Testing - JobyJoba".into(),
        csp_policy: CSP,
        ..Default::default()
    };
    let user = User {
        username: "admin".into(),
        email: "admin".into(),
    };
    let template = ComplianceTestTemplate { meta, user };
    template_to_response(&template)
}

async fn test_suite_correctness() -> HtmlResult {
    let meta = Meta {
        meta_title: "Correctness Testing - JobyJoba".into(),
        csp_policy: CSP,
        ..Default::default()
    };
    let user = User {
        username: "admin".into(),
        email: "admin".into(),
    };
    let template = CorrectnessTestTemplate { meta, user };
    template_to_response(&template)
}

async fn test_suite_performance() -> HtmlResult {
    let meta = Meta {
        meta_title: "Performance Testing - JobyJoba".into(),
        csp_policy: CSP,
        ..Default::default()
    };
    let user = User {
        username: "admin".into(),
        email: "admin".into(),
    };
    let template = PerformanceTestTemplate { meta, user };
    template_to_response(&template)
}

#[derive(Template)]
#[template(path = "dashboard/test-suite/upload.html")]
struct UploadScenarioTemplate<'a> {
    meta: Meta<'a>,
    user: User,
}

async fn test_suite_upload() -> HtmlResult {
    // For this page we allow unsafe-inline for scripts and styles
    // to simplify development. In production, you should use nonces or hashes.
    let mut upload_csp = CSP;
    upload_csp.script_src_elem = Cow::Borrowed(
        "'self' 'unsafe-inline' https://unpkg.com https://cdn.jsdelivr.net https://cdnjs.cloudflare.com https://code.jquery.com https://challenges.cloudflare.com"
    );
    upload_csp.script_src_attr = Cow::Borrowed("'self' 'unsafe-inline'");
    upload_csp.style_src_attr = Cow::Borrowed("'self' 'unsafe-inline'");

    let meta = Meta {
        meta_title: "Upload Test Scenario - JobyJoba".into(),
        csp_policy: upload_csp,
        ..Default::default()
    };
    let user = User {
        username: "admin".into(),
        email: "admin".into(),
    };
    let template = UploadScenarioTemplate { meta, user };
    template_to_response(&template)
}

#[derive(Template)]
#[template(path = "dashboard/test-suite/metrics.html")]
struct MetricsTemplate<'a> {
    meta: Meta<'a>,
    user: User,
    job_id: u64,
}

async fn test_suite_metrics(Path(job_id): Path<u64>) -> HtmlResult {
    // For this page we allow unsafe-inline for scripts and styles
    let mut metrics_csp = CSP;
    metrics_csp.script_src_elem = Cow::Borrowed(
        "'self' 'unsafe-inline' https://unpkg.com https://cdn.jsdelivr.net https://cdnjs.cloudflare.com https://code.jquery.com https://challenges.cloudflare.com"
    );
    metrics_csp.script_src_attr = Cow::Borrowed("'self' 'unsafe-inline'");
    metrics_csp.style_src_attr = Cow::Borrowed("'self' 'unsafe-inline'");

    let meta = Meta {
        meta_title: format!("Test Metrics - Job #{} - JobyJoba", job_id).into(),
        csp_policy: metrics_csp,
        ..Default::default()
    };
    let user = User {
        username: "admin".into(),
        email: "admin".into(),
    };
    let template = MetricsTemplate { meta, user, job_id };
    template_to_response(&template)
}

// API Endpoints
async fn api_test_runs() -> impl IntoResponse {
    // Mock data - replace with actual database queries
    let test_runs = vec![
        TestRun {
            id: "test-1".into(),
            suite_name: "User API Tests".into(),
            phase: TestPhase::Security,
            status: TestStatus::Completed,
            duration: "2m 34s".into(),
            total_tests: 250,
            passed: 236,
            failed: 14,
            success_rate: 94.4,
            started_at: Utc::now(),
        },
        TestRun {
            id: "test-2".into(),
            suite_name: "Product API Tests".into(),
            phase: TestPhase::Correctness,
            status: TestStatus::Running,
            duration: "1m 12s".into(),
            total_tests: 180,
            passed: 156,
            failed: 2,
            success_rate: 98.7,
            started_at: Utc::now(),
        },
    ];

    Json(test_runs)
}

async fn api_test_metrics() -> impl IntoResponse {
    // Mock data - replace with actual calculations
    let metrics = TestMetrics {
        total_tests: 1245,
        passed_tests: 1189,
        failed_tests: 56,
        success_rate: 95.5,
    };

    Json(metrics)
}

async fn api_performance_data() -> impl IntoResponse {
    // Mock data - replace with actual performance monitoring
    let data = PerformanceData {
        timestamp: Utc::now(),
        requests_per_second: 1245.0,
        avg_response_time: 45.0,
        error_rate: 0.8,
        active_users: 500,
        percentiles: ResponsePercentiles {
            p50: 42.0,
            p90: 89.0,
            p95: 156.0,
            p99: 245.0,
            max: 1234.0,
        },
    };

    Json(data)
}

// Fuzzer API Integration
// These handlers proxy requests to the fuzzer service

/// Start a new fuzzer job
async fn api_fuzzer_start(Json(config): Json<Configuration>) -> impl IntoResponse {
    // Create the fuzzer client
    let client = reqwest::Client::new();

    // Forward the request to the fuzzer service
    match client
        .post("http://127.0.0.1:3000/fuzzer")
        .json(&serde_json::json!({
            "operation": "create",
            "config": config
        }))
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            match response.json::<serde_json::Value>().await {
                Ok(json) => (status, Json(json)),
                Err(_) => (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "result": "error",
                        "message": "Failed to parse fuzzer response"
                    }))
                ),
            }
        }
        Err(e) => (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "result": "error",
                "message": format!("Fuzzer service unavailable: {}", e)
            }))
        ),
    }
}

/// Stop a running fuzzer job
async fn api_fuzzer_stop(Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    let client = reqwest::Client::new();

    match client
        .post("http://127.0.0.1:3000/fuzzer")
        .json(&serde_json::json!({
            "operation": "stop",
            "job_id": payload["job_id"]
        }))
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            match response.json::<serde_json::Value>().await {
                Ok(json) => (status, Json(json)),
                Err(_) => (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "result": "error",
                        "message": "Failed to parse fuzzer response"
                    }))
                ),
            }
        }
        Err(e) => (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "result": "error",
                "message": format!("Fuzzer service unavailable: {}", e)
            }))
        ),
    }
}

/// Get metrics for a specific job
async fn api_fuzzer_metrics(Path(job_id): Path<u64>) -> impl IntoResponse {
    let client = reqwest::Client::new();

    match client
        .get(format!("http://127.0.0.1:3000/fuzzer/metrics/{}", job_id))
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            match response.json::<serde_json::Value>().await {
                Ok(json) => (status, Json(json)),
                Err(_) => (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": "Failed to parse fuzzer response"
                    }))
                ),
            }
        }
        Err(e) => (
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "error": format!("Fuzzer service unavailable: {}", e)
            }))
        ),
    }
}

/// WebSocket proxy for live metrics
async fn api_fuzzer_websocket(
    ws: WebSocketUpgrade,
    Path(job_id): Path<u64>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_fuzzer_websocket(socket, job_id))
}

async fn handle_fuzzer_websocket(mut socket: WebSocket, job_id: u64) {
    use axum::extract::ws::Message;

    // For now, we'll use HTTP polling to get metrics and send them via WebSocket
    // This is simpler than proxying WebSocket connections
    let client = reqwest::Client::new();

    loop {
        tokio::select! {
            // Poll metrics from fuzzer every 500ms
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(500)) => {
                match client
                    .get(format!("http://127.0.0.1:3000/fuzzer/metrics/{}", job_id))
                    .send()
                    .await
                {
                    Ok(response) => {
                        if let Ok(json) = response.json::<serde_json::Value>().await {
                            if socket.send(Message::Text(json.to_string().into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(_) => {
                        let error = serde_json::json!({"error": "Failed to fetch metrics"});
                        if socket.send(Message::Text(error.to_string().into())).await.is_err() {
                            break;
                        }
                    }
                }
            }
            // Handle incoming messages from client
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(data))) => {
                        if socket.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Get list of all jobs (for the upload page)
async fn api_fuzzer_jobs() -> impl IntoResponse {
    // This is a mock response since the fuzzer doesn't have a "list all jobs" endpoint yet
    // You can extend the fuzzer API to support this, or maintain a job registry in your main app
    Json(serde_json::json!({
        "jobs": []
    }))
}
