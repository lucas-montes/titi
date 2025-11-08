
use axum::Router;
use std::{net::SocketAddr, sync::Arc};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tokio::sync::mpsc;


#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Setup launcher
    let (launcher_tx, launcher_rx) = mpsc::channel(100);

    let launcher = Launcher::new(launcher_rx);
    tokio::spawn(async move {
        tracing::info!("Starting fuzzer launcher...");
        launcher.run().await;
    });

    // Create service
    let fuzzer_service = Arc::new(FuzzerService::new(launcher_tx));

    // Build application with fuzzer routes
    let app = Router::new()
        .merge(routes().with_state(fuzzer_service))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    // Run server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    tracing::info!("Server listening on http://127.0.0.1:3000");
    tracing::info!("Routes:");
    tracing::info!("  POST   /fuzzer              - Create/restart/stop jobs");
    tracing::info!("  GET    /fuzzer/metrics/:id  - Get metrics snapshot");
    tracing::info!("  WS     /fuzzer/ws/:id       - Stream metrics");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
