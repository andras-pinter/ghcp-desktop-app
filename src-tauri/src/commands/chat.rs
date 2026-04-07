//! Chat commands: send_message, stop_streaming, regenerate.

use crate::db::queries;
use crate::state::AppState;
use copilot_api::client::StreamEvent;
use copilot_api::types::{ChatMessage, ChatRequest, MessageRole};
use tauri::{AppHandle, Emitter, Manager};

/// Build the system prompt for an agent by combining its prompt with skill instructions.
fn build_agent_system_prompt(
    agent: &queries::Agent,
    skills: &[queries::Skill],
) -> String {
    let mut parts = vec![agent.system_prompt.clone()];

    for skill in skills {
        if skill.enabled {
            if let Some(ref instructions) = skill.instructions {
                if !instructions.is_empty() {
                    parts.push(format!("\n---\n## Skill: {}\n{}", skill.name, instructions));
                }
            }
        }
    }

    parts.join("\n")
}

/// Send a chat message and stream the response via events.
///
/// The frontend receives `streaming-token`, `streaming-complete`, or
/// `streaming-error` events as the response arrives.
///
/// When `agent_id` is provided, the agent's system prompt and enabled skill
/// instructions are prepended as a system message.
#[tauri::command]
pub async fn send_message(
    app: AppHandle,
    messages: Vec<ChatMessage>,
    model: String,
    agent_id: Option<String>,
) -> Result<(), String> {
    if messages.is_empty() {
        return Err("At least one message is required".to_string());
    }
    if model.is_empty() {
        return Err("Model selection is required".to_string());
    }

    let state = app.state::<AppState>();

    // Build messages with optional agent system prompt prepended
    let final_messages = if let Some(ref aid) = agent_id {
        let (agent_opt, skills) = {
            let db = state.db.lock().map_err(|e| e.to_string())?;
            let agent = queries::get_agent(&db, aid).map_err(|e| e.to_string())?;
            let skills = if agent.is_some() {
                queries::get_agent_skills(&db, aid).map_err(|e| e.to_string())?
            } else {
                vec![]
            };
            (agent, skills)
        };

        if let Some(agent) = agent_opt {
            let system_prompt = build_agent_system_prompt(&agent, &skills);
            let mut msgs = vec![ChatMessage {
                role: MessageRole::System,
                content: system_prompt,
                name: None,
                tool_call_id: None,
            }];
            msgs.extend(messages);
            msgs
        } else {
            messages
        }
    } else {
        messages
    };

    let request = ChatRequest {
        model,
        messages: final_messages,
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
