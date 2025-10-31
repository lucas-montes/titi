use super::dtos::{JobOperation, JobOperationResponse, MetricsResponse};
use super::service::FuzzerService;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::time::{interval, Duration};

/// POST /fuzzer - Handle job operations (create, restart, stop)
async fn handle_job_operation(
    State(service): State<Arc<FuzzerService>>,
    Json(operation): Json<JobOperation>,
) -> (StatusCode, Json<JobOperationResponse>) {
    match operation {
        JobOperation::Create { config } => {
            match service.create_job(config).await {
                Ok(job_id) => (
                    StatusCode::CREATED,
                    Json(JobOperationResponse::Created { job_id }),
                ),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(JobOperationResponse::Error { message: e }),
                ),
            }
        }

        JobOperation::Restart { job_id } => {
            match service.restart_job(job_id).await {
                Ok(_) => (
                    StatusCode::OK,
                    Json(JobOperationResponse::Restarted { job_id }),
                ),
                Err(e) => (
                    StatusCode::NOT_FOUND,
                    Json(JobOperationResponse::Error { message: e }),
                ),
            }
        }

        JobOperation::Stop { job_id } => {
            match service.stop_job(job_id).await {
                Ok(_) => (
                    StatusCode::OK,
                    Json(JobOperationResponse::Stopped { job_id }),
                ),
                Err(e) => (
                    StatusCode::NOT_FOUND,
                    Json(JobOperationResponse::Error { message: e }),
                ),
            }
        }
    }
}

/// GET /fuzzer/metrics/:id - Get metrics for a job
async fn get_job_metrics(
    State(service): State<Arc<FuzzerService>>,
    Path(job_id): Path<u64>,
) -> (StatusCode, Json<MetricsResponse>) {
    match service.get_metrics(job_id).await {
        Ok(snapshot) => (
            StatusCode::OK,
            Json(MetricsResponse::Success(snapshot)),
        ),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(MetricsResponse::Error { error: e }),
        ),
    }
}

/// WebSocket /fuzzer/ws/:id - Stream metrics for a job in real-time
async fn metrics_websocket(
    ws: WebSocketUpgrade,
    State(service): State<Arc<FuzzerService>>,
    Path(job_id): Path<u64>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_metrics_socket(socket, service, job_id))
}

/// Handle WebSocket connection for metrics streaming
async fn handle_metrics_socket(socket: WebSocket, service: Arc<FuzzerService>, job_id: u64) {
    let (mut sender, mut receiver) = socket.split();

    // Send metrics every 500ms
    let mut tick = interval(Duration::from_millis(500));

    loop {
        tokio::select! {
            _ = tick.tick() => {
                // Get latest metrics
                match service.get_metrics(job_id).await {
                    Ok(snapshot) => {
                        // Serialize to JSON
                        match serde_json::to_vec(&snapshot) {
                            Ok(json) => {
                                if sender.send(Message::Binary(json.into())).await.is_err() {
                                    // Client disconnected
                                    break;
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to serialize metrics: {}", e);
                                break;
                            }
                        }
                    }
                    Err(_) => {
                        // Job not found or no metrics yet, continue polling
                    }
                }
            }

            // Handle incoming messages (ping/pong, close)
            Some(msg) = receiver.next() => {
                match msg {
                    Ok(Message::Close(_)) => break,
                    Ok(Message::Ping(data)) => {
                        if sender.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                    _ => {}
                }
            }
        }
    }

    tracing::info!("WebSocket connection closed for job {}", job_id);
}

/// Create the fuzzer routes
///
/// Routes:
/// - POST   /fuzzer           - Job operations (create, restart, stop)
/// - GET    /fuzzer/metrics/:id - Get current metrics snapshot
/// - GET    /fuzzer/ws/:id     - WebSocket for real-time metrics streaming
pub fn routes() -> Router<Arc<FuzzerService>> {
    Router::new()
        .route("/fuzzer", post(handle_job_operation))
        .route("/fuzzer/metrics/{id}", get(get_job_metrics))
        .route("/fuzzer/ws/{id}", get(metrics_websocket))
}
