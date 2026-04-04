//! Model commands: get_models (model discovery).

use crate::state::AppState;
use copilot_api::types::Model;
use tauri::{AppHandle, Manager};

/// Default fallback model when API is unreachable.
const FALLBACK_MODEL: &str = "gpt-4o";

/// Fetch available Copilot models. Falls back to a default if API fails.
#[tauri::command]
pub async fn get_models(app: AppHandle) -> Result<Vec<Model>, String> {
    let state = app.state::<AppState>();

    match state.copilot.get_models().await {
        Ok(models) if !models.is_empty() => Ok(models),
        Ok(_) => {
            log::warn!("Models API returned empty list, using fallback");
            Ok(vec![Model {
                id: FALLBACK_MODEL.to_string(),
                name: Some(FALLBACK_MODEL.to_string()),
                version: None,
            }])
        }
        Err(e) => {
            log::warn!("Failed to fetch models: {e}, using fallback");
            Ok(vec![Model {
                id: FALLBACK_MODEL.to_string(),
                name: Some(FALLBACK_MODEL.to_string()),
                version: None,
            }])
        }
    }
}
