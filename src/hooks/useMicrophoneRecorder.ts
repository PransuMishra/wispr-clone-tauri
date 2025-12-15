import { useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export type RecorderStatus = "idle" | "recording" | "error";

export function useMicrophoneRecorder() {
  const [status, setStatus] = useState<RecorderStatus>("idle");
  const [error, setError] = useState<string | null>(null);

  const mediaRecorderRef = useRef<MediaRecorder | null>(null);
  const streamRef = useRef<MediaStream | null>(null);

  const start = async () => {
    try {
      if (status === "recording") return;

      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      streamRef.current = stream;

      const recorder = new MediaRecorder(stream, {
        mimeType: "audio/webm;codecs=opus",
      });

      recorder.ondataavailable = async (e) => {
        if (e.data.size > 0) {
          const buf = await e.data.arrayBuffer();
          await invoke("send_audio_chunk", {
            chunk: Array.from(new Uint8Array(buf)),
          });
        }
      };

      recorder.start(250);
      mediaRecorderRef.current = recorder;
      setStatus("recording");
      setError(null);
    } catch (e: any) {
      console.error(e);
      setError(e.message ?? "Mic error");
      setStatus("error");
    }
  };

  const stop = () => {
    mediaRecorderRef.current?.stop();
    mediaRecorderRef.current = null;

    streamRef.current?.getTracks().forEach((t) => t.stop());
    streamRef.current = null;

    setStatus("idle");
  };

  return { start, stop, status, error };
}
