import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useMicrophoneRecorder } from "./hooks/useMicrophoneRecorder";
import "./App.css";


const apiKey = import.meta.env.VITE_DEEPGRAM_API_KEY as string;

function App() {
  const [displayText, setDisplayText] = useState("");
  const [finalText, setFinalText] = useState("");
  const [error, setError] = useState<string | null>(null);

  const { start, stop, status, error: micError } = useMicrophoneRecorder();
  const isRecording = status === "recording";

  useEffect(() => {
    if (!apiKey) {
      setError("Missing Deepgram API key");
      return;
    }

    invoke("start_deepgram_session", { apiKey }).catch((e) => {
      console.error(e);
      setError(String(e));
    });

    const unlistenPromise = listen<{
      text: string;
      isFinal: boolean;
    }>("deepgram_transcript", (event) => {
      const { text, isFinal } = event.payload;

      if (isFinal) {
        setFinalText((prev) => (prev ? `${prev} ${text}` : text));
        setDisplayText("");
      } else {
        setDisplayText(text);
      }
    });

    return () => {
      invoke("stop_deepgram_session");
      unlistenPromise.then((u) => u());
    };
  }, []);

  return (
  <div className="app">
    <header className="header">
      <h1>🎙️ Voice to Text</h1>
      <p>Real-time transcription using Rust + Deepgram</p>
    </header>

    <div className="card">
      <button
        className={`mic-btn ${isRecording ? "recording" : ""}`}
        onMouseDown={start}
        onMouseUp={stop}
        onTouchStart={start}
        onTouchEnd={stop}
      >
        {isRecording ? "🎤 Listening…" : "🎙 Hold to Talk"}
      </button>

      <p className="status">
        Status: <span>{status}</span>
      </p>

      {error && <p className="error">{error}</p>}
      {micError && <p className="error">{micError}</p>}
    </div>

    <div className="grid">
      <div className="panel">
        <h2>Live Speech</h2>
        <div className="live-text">
          {displayText || <span className="muted">Say something…</span>}
        </div>
      </div>

      <div className="panel">
        <h2>Final Transcript</h2>
        <textarea value={finalText} readOnly />
        <div className="actions">
          <button onClick={() => navigator.clipboard.writeText(finalText)}>
            Copy
          </button>
          <button
            className="secondary"
            onClick={() => {
              setFinalText("");
              setDisplayText("");
            }}
          >
            Clear
          </button>
        </div>
      </div>
    </div>
  </div>
);

}

export default App;
