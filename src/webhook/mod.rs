use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

// Update the webhook payload structure to match Discord's format
#[derive(Debug, Deserialize, Serialize)]
pub struct WebhookPayload {
    pub r#type: String, // Using r# prefix because "type" is a reserved keyword
    pub playerName: String,
    pub accountType: Option<String>,
    pub dinkAccountHash: String,
    pub clanName: Option<String>,
    pub seasonalWorld: bool,
    pub world: i32,
    pub regionId: i32,
    pub extra: serde_json::Value, // Use generic Value for complex nested structures
    pub embeds: Vec<Embed>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Embed {
    pub title: String,
    pub description: String,
    pub author: Option<EmbedAuthor>,
    pub color: Option<i32>,
    pub thumbnail: Option<EmbedThumbnail>,
    pub fields: Option<Vec<EmbedField>>,
    pub footer: Option<EmbedFooter>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EmbedAuthor {
    pub name: String,
    pub icon_url: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EmbedThumbnail {
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EmbedField {
    pub name: String,
    pub value: String,
    pub inline: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EmbedFooter {
    pub text: String,
    pub icon_url: Option<String>,
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
    mut multipart: Multipart,
) -> (StatusCode, Json<WebhookResponse>) {
    let mut payload_json = None;

    // Debug: Log before processing multipart form

    // Process the multipart form
    while let Ok(Some(field)) = multipart.next_field().await {
        if let Some(name) = field.name() {
            if name == "payload_json" {
                match field.text().await {
                    Ok(value) => {
                        // Parse the JSON string into your WebhookPayload struct
                        match serde_json::from_str::<WebhookPayload>(&value) {
                            Ok(payload) => {
                                payload_json = Some(payload);
                            }
                            Err(e) => {
                                eprintln!("[ERROR] Failed to parse payload_json: {}", e);
                                return (
                                    StatusCode::BAD_REQUEST,
                                    Json(WebhookResponse {
                                        status: "error".to_string(),
                                        message: "Invalid payload format".to_string(),
                                    }),
                                );
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[ERROR] Failed to read field text: {}", e);
                    }
                }
            }
        } else {
        }
    }

    // Process the payload
    if let Some(payload) = payload_json {
        // Send the webhook payload to the main bot process
        match state.webhook_sender.send(payload).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("[ERROR] Failed to send webhook payload: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(WebhookResponse {
                        status: "error".to_string(),
                        message: "Failed to process webhook".to_string(),
                    }),
                );
            }
        }

        return (
            StatusCode::OK,
            Json(WebhookResponse {
                status: "success".to_string(),
                message: "Webhook received".to_string(),
            }),
        );
    }

    (
        StatusCode::BAD_REQUEST,
        Json(WebhookResponse {
            status: "error".to_string(),
            message: "Missing payload_json field".to_string(),
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
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    println!("Starting webhook server on {}", addr);

    tokio::spawn(async move {
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => {
                if let Err(e) = axum::serve(listener, app).await {
                    eprintln!("[ERROR] Server error: {}", e);
                }
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to bind to address {}: {}", addr, e);
            }
        }
    });

    (webhook_sender, webhook_receiver)
}
