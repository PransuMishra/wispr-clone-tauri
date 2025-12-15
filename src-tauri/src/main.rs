// Prevents additional console window on Windows in release, DO NOT REMOVE!!
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// fn main() {
//     wispr_clone_lib::run()
// }


// src-tauri/src/main.rs
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod deepgram;

use deepgram::DeepgramSession;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

struct DgState(Arc<Mutex<Option<DeepgramSession>>>);

#[tauri::command]
async fn start_deepgram_session<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    state: State<'_, DgState>,
    api_key: String,
) -> Result<(), String> {
    let session = DeepgramSession::new(app, api_key)
        .await
        .map_err(|e| e.to_string())?;

    *state.0.lock().await = Some(session);
    Ok(())
}

#[tauri::command]
fn send_audio_chunk(
    state: State<'_, DgState>,
    chunk: Vec<u8>,
) -> Result<(), String> {
    if let Some(session) = state.0.blocking_lock().as_ref() {
        session.send_audio(chunk).map_err(|e| e.to_string())
    } else {
        Err("No active Deepgram session".into())
    }
}

#[tauri::command]
async fn stop_deepgram_session(
    state: State<'_, DgState>,
) -> Result<(), String> {
    *state.0.lock().await = None;
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .manage(DgState(Arc::new(Mutex::new(None))))
        .invoke_handler(tauri::generate_handler![
            start_deepgram_session,
            send_audio_chunk,
            stop_deepgram_session
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri");
}




