use anyhow::Result;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::{
    client::IntoClientRequest,
    protocol::Message,
};

use std::sync::Arc;

#[derive(Debug)]
pub struct DeepgramSession {
    audio_tx: mpsc::UnboundedSender<Vec<u8>>,
}

#[derive(Serialize)]
struct DeepgramConfig {
    #[serde(rename = "type")]
    type_field: String,
    encoding: String,
    sample_rate: u32,
    channels: u8,
    interim_results: bool,
    punctuate: bool,
    model: String,
}

#[derive(Deserialize)]
struct DeepgramAlternative {
    transcript: String,
}

#[derive(Deserialize)]
struct DeepgramChannel {
    alternatives: Vec<DeepgramAlternative>,
}

#[derive(Deserialize)]
struct DeepgramTranscript {
    channel: DeepgramChannel,
    is_final: Option<bool>,
}

impl DeepgramSession {
    pub async fn new<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        api_key: String,
    ) -> Result<Self> {
        println!("🔗 Connecting to Deepgram…");

        // ✅ CORRECT request creation (NO http crate)
        let mut request = "wss://api.deepgram.com/v1/listen"
            .into_client_request()?;

        request
            .headers_mut()
            .insert("Authorization", format!("Token {}", api_key).parse()?);

        // ✅ connect_async infers TLS correctly
        let (ws_stream, _) = connect_async(request).await?;

        // ✅ Explicit split types (fixes E0282)
        let (sink, mut stream) = ws_stream.split();
        let sink = Arc::new(Mutex::new(sink));

        let (audio_tx, mut audio_rx) = mpsc::unbounded_channel::<Vec<u8>>();

        // ✅ Deepgram expects PCM (linear16)
        let cfg = DeepgramConfig {
            type_field: "config".into(),
            encoding: "linear16".into(),
            sample_rate: 48_000,
            channels: 1,
            interim_results: true,
            punctuate: true,
            model: "nova-3".into(),
        };

        sink.lock()
            .await
            .send(Message::Text(serde_json::to_string(&cfg)?))
            .await?;

        println!("✅ Deepgram connected");

        // 🔁 Send audio
        let sink_audio = sink.clone();
        tokio::spawn(async move {
            while let Some(chunk) = audio_rx.recv().await {
                let _ = sink_audio
                    .lock()
                    .await
                    .send(Message::Binary(chunk))
                    .await;
            }
        });

        // 🔁 Receive transcripts
        tokio::spawn(async move {
            while let Some(msg) = stream.next().await {
                if let Ok(Message::Text(txt)) = msg {
                    if let Ok(payload) =
                        serde_json::from_str::<DeepgramTranscript>(&txt)
                    {
                        if let Some(alt) =
                            payload.channel.alternatives.first()
                        {
                            let _ = app.emit(
                                "deepgram_transcript",
                                serde_json::json!({
                                    "text": alt.transcript,
                                    "isFinal": payload.is_final.unwrap_or(false)
                                }),
                            );
                        }
                    }
                }
            }
        });

        Ok(Self { audio_tx })
    }

    pub fn send_audio(&self, data: Vec<u8>) -> Result<()> {
        self.audio_tx.send(data)?;
        Ok(())
    }
}
