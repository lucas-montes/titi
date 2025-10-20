use std::{borrow::Cow, fmt};

use axum::{Router, routing::get};

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use stefn::{
    askama::Template,
    create_error_templates,
    state::WebsiteState,
    website::meta_tags::{ContentSecurityPolicy, Meta},
};

//TODO: create structs to hold the assets
pub const CSP: ContentSecurityPolicy<'static> = ContentSecurityPolicy {
    base_uri: Cow::Borrowed("'self'"),
    child_src: Cow::Borrowed("'self'"),
    connect_src: Cow::Borrowed("'self' https://cdnjs.cloudflare.com"),
    default_src: Cow::Borrowed("'self'"),
    font_src: Cow::Borrowed(
        "'self' https://fonts.gstatic.com https://cdnjs.cloudflare.com https://cdn.jsdelivr.net data:",
    ),
    form_action: Cow::Borrowed("'self'"),
    frame_src: Cow::Borrowed("'self'"),
    // Add blob: for dynamically generated images
    img_src: Cow::Borrowed(
        "'self' https://ik.imagekit.io/smartlinker/images/ https://img.stackshare.io/ data: blob:",
    ),
    manifest_src: Cow::Borrowed("'self'"),
    media_src: Cow::Borrowed("'self'"),
    object_src: Cow::Borrowed("'none'"),
    report_to: Cow::Borrowed("default"),
    // Remove empty require-trusted-types-for or set to "'script'"
    require_trusted_types_for: Cow::Borrowed("'script'"), // or remove this line entirely
    script_src: Cow::Borrowed(
        "'self' https://unpkg.com https://cdn.jsdelivr.net https://cdnjs.cloudflare.com https://code.jquery.com https://challenges.cloudflare.com",
    ),
    script_src_attr: Cow::Borrowed("'self'"),
    script_src_elem: Cow::Borrowed(
        "'self' https://unpkg.com https://cdn.jsdelivr.net https://cdnjs.cloudflare.com https://code.jquery.com https://challenges.cloudflare.com 'sha256-YpkVwgje1aO9XrJNyzcGQAECa/MK8XpN8s0LJ7VZqOc='",
    ),
    style_src: Cow::Borrowed("'self' https://cdn.jsdelivr.net https://cdnjs.cloudflare.com"),
    // Add the hash for the inline style (sha256-lQYA+ItD8v7Xb6ON33XO2xFXIE2AXIdrO9vbSef8g7c=)
    style_src_attr: Cow::Borrowed(
        "'self' 'unsafe-hashes' 'sha256-u2Yctlkg1MQj7vO+nL13o9rBQFOBaGRouQGeAzNGvTc=' 'sha256-hP0ISKcRz40G33clBjx741T6tbj1HTLiD63Kt3dHg0w=' 'sha256-w7ZF2PxCz+I15bGjIXh+Z0/ty6Ufyvxmu8lkhc6Af1w=' 'sha256-lQYA+ItD8v7Xb6ON33XO2xFXIE2AXIdrO9vbSef8g7c='",
    ),
    style_src_elem: Cow::Borrowed(
        "'self' https://cdn.jsdelivr.net https://cdnjs.cloudflare.com https://fonts.googleapis.com 'sha256-hP0ISKcRz40G33clBjx741T6tbj1HTLiD63Kt3dHg0w=' 'sha256-bsV5JivYxvGywDAZ22EZJKBFip65Ng9xoJVLbBg7bdo=' 'nonce-hey'",
    ),
    trusted_types: Cow::Borrowed("default"),
    worker_src: Cow::Borrowed("'none'"),
};

pub fn routes(state: WebsiteState) -> Router<WebsiteState> {
    Router::new().route("/", get(landing)).route("/legal", get(legal)).with_state(state)
}

create_error_templates!("404.html", "500.html");


#[derive(Template)]
#[template(path = "website/landing.html")]
struct LandingTemplate<'a> {
    meta: Meta<'a>,
}

async fn landing() -> HtmlResult {
    let meta = Meta {
        meta_title: "JobyJoba".into(),
        csp_policy: CSP,
        ..Default::default()
    };
    let template = LandingTemplate { meta };
    template_to_response(&template)
}

#[derive(Template)]
#[template(path = "website/legal.html")]
struct LegalTemplate<'a> {
    meta: Meta<'a>,
}

async fn legal() -> HtmlResult {
    let meta = Meta {
        meta_title: "JobyJoba".into(),
        csp_policy: CSP,
        ..Default::default()
    };
    let template = LegalTemplate { meta };
    template_to_response(&template)
}
