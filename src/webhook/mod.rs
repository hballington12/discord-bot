use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;

// Define webhook payload structure
#[derive(Debug, Deserialize, Serialize)]
pub struct WebhookPayload {
    pub event_type: String,
    pub content: String,
    // Add additional fields as needed
}

// Define response structure
#[derive(Serialize)]
pub struct WebhookResponse {
    pub status: String,
    pub message: String,
}

// Channel for communicating with the main bot
pub type WebhookSender = mpsc::Sender<WebhookPayload>;
pub type WebhookReceiver = mpsc::Receiver<WebhookPayload>;

// AppState to share data between routes
#[derive(Clone)]
pub struct AppState {
    webhook_sender: WebhookSender,
}

// Handler for webhook POST requests
async fn handle_webhook(
    State(state): State<AppState>,
    Json(payload): Json<WebhookPayload>,
) -> (StatusCode, Json<WebhookResponse>) {
    println!("Received webhook: {:?}", payload);

    // Send the webhook payload to the main bot process
    if let Err(e) = state.webhook_sender.send(payload).await {
        eprintln!("Failed to send webhook payload: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(WebhookResponse {
                status: "error".to_string(),
                message: "Failed to process webhook".to_string(),
            }),
        );
    }

    (
        StatusCode::OK,
        Json(WebhookResponse {
            status: "success".to_string(),
            message: "Webhook received".to_string(),
        }),
    )
}

// Health check endpoint
async fn health_check() -> (StatusCode, Json<WebhookResponse>) {
    (
        StatusCode::OK,
        Json(WebhookResponse {
            status: "success".to_string(),
            message: "Webhook server is running".to_string(),
        }),
    )
}

// Start the webhook server
pub async fn start_webhook_server(port: u16) -> (WebhookSender, WebhookReceiver) {
    // Create a channel for communication
    let (webhook_sender, webhook_receiver) = mpsc::channel(100);

    // Create app state
    let state = AppState {
        webhook_sender: webhook_sender.clone(),
    };

    // Build the router
    let app = Router::new()
        .route("/webhook", post(handle_webhook))
        .route("/health", get(health_check))
        .with_state(state);

    // Start the server
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    println!("Starting webhook server on {}", addr);

    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    (webhook_sender, webhook_receiver)
}
