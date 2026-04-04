//! Chat commands: send_message, stop_streaming, regenerate.

use crate::state::AppState;
use copilot_api::client::StreamEvent;
use copilot_api::types::{ChatMessage, ChatRequest};
use tauri::{AppHandle, Emitter, Manager};

/// Send a chat message and stream the response via events.
///
/// The frontend receives `streaming-token`, `streaming-complete`, or
/// `streaming-error` events as the response arrives.
#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    messages: Vec<ChatMessage>,
    model: String,
) -> Result<(), String> {
    if messages.is_empty() {
        return Err("At least one message is required".to_string());
    }
    if model.is_empty() {
        return Err("Model selection is required".to_string());
    }

    let state = app.state::<AppState>();

    let request = ChatRequest {
        model,
        messages,
        temperature: None,
        max_tokens: None,
        stream: true,
    };

    // Set up cancellation — reject concurrent sends
    {
        let lock = state.cancel_stream.lock().await;
        if lock.is_some() {
            return Err("A streaming response is already in progress".to_string());
        }
    }
    let (cancel_tx, mut cancel_rx) = tokio::sync::watch::channel(false);
    {
        let mut lock = state.cancel_stream.lock().await;
        *lock = Some(cancel_tx);
    }

    let mut rx = state
        .copilot
        .send_message_stream(request)
        .await
        .map_err(|e| e.to_string())?;

    // Consume stream events, forwarding to frontend
    loop {
        tokio::select! {
            event = rx.recv() => {
                match event {
                    Some(StreamEvent::Token(token)) => {
                        let _ = app.emit("streaming-token", &token);
                    }
                    Some(StreamEvent::RoleSet) => {
                        // First chunk — role established, no action needed
                    }
                    Some(StreamEvent::Done) => {
                        let _ = app.emit("streaming-complete", ());
                        break;
                    }
                    Some(StreamEvent::Error(err)) => {
                        let _ = app.emit("streaming-error", &err);
                        break;
                    }
                    None => {
                        // Channel closed
                        let _ = app.emit("streaming-complete", ());
                        break;
                    }
                }
            }
            _ = cancel_rx.changed() => {
                if *cancel_rx.borrow() {
                    let _ = app.emit("streaming-complete", ());
                    break;
                }
            }
        }
    }

    // Clear cancellation sender
    {
        let mut lock = state.cancel_stream.lock().await;
        *lock = None;
    }

    Ok(())
}

/// Cancel an in-flight streaming response.
#[tauri::command]
pub async fn stop_streaming(app: AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let lock = state.cancel_stream.lock().await;
    if let Some(ref tx) = *lock {
        let _ = tx.send(true);
    }
    Ok(())
}
