use std::borrow::Cow;

use axum::{Router, routing::get};

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
}

async fn test_suite() -> HtmlResult {
    let meta = Meta {
        meta_title: "JobyJoba".into(),
        csp_policy: CSP,
        ..Default::default()
    };
    let template = TestSuiteTemplate { meta };
    template_to_response(&template)
}
