use std::{borrow::Cow, fmt};

use axum::{Json, Router, response::IntoResponse, routing::get};

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use stefn::{
    askama::Template,
    create_error_templates,
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
    script_src_attr: Cow::Borrowed("'self'"),
    script_src_elem: Cow::Borrowed(
        "'self' https://unpkg.com https://cdn.jsdelivr.net https://cdnjs.cloudflare.com https://code.jquery.com https://challenges.cloudflare.com 'sha256-YpkVwgje1aO9XrJNyzcGQAECa/MK8XpN8s0LJ7VZqOc='",
    ),
    // NOTE: 'unsafe-hashes' allows hashes for event handlers and style attributes
    style_src: Cow::Borrowed(
        "'self' https://cdn.jsdelivr.net https://cdnjs.cloudflare.com https://fonts.googleapis.com",
    ),
    style_src_attr: Cow::Borrowed(
        "'self' 'unsafe-hashes' 'sha256-biLFinpqYMtWHmXfkA1BPeCY0/fNt46SAZ+BBk5YUog=' 'sha256-hP0ISKcRz40G33clBjx741T6tbj1HTLiD63Kt3dHg0w=' 'sha256-u2Yctlkg1MQj7vO+nL13o9rBQFOBaGRouQGeAzNGvTc='",
    ),
    style_src_elem: Cow::Borrowed(
        "'self' https://cdn.jsdelivr.net https://cdnjs.cloudflare.com https://fonts.googleapis.com 'sha256-bsV5JivYxvGywDAZ22EZJKBFip65Ng9xoJVLbBg7bdo='",
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
        .route("/api/test-runs", get(api_test_runs))
        .route("/api/test-metrics", get(api_test_metrics))
        .route("/api/performance-data", get(api_performance_data))
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
