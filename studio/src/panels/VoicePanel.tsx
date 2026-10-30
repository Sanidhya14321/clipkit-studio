// Voice slots (spec 8.3): each slot's mode badge (switchable), status and takes. Record-along:
// a teleprompter shows the line, a 3 s countdown runs, the microphone records one take for that
// slot (punch-in per line), and the upload becomes a new active take, trimmed, normalized and
// fit-checked by ck. A slot with no audio plays a labeled browser speech placeholder.
import { useEffect, useRef, useState } from "react";
import { ApiError, request } from "../api";
import type {
  ActiveTakeRequest,
  ConsentRequest,
  Project,
  VoiceMode,
  VoiceModeRequest,
  VoiceSlot,
} from "../generated/types";

const MODES: VoiceMode[] = ["ai", "recorded", "cloned"];
export const COUNTDOWN = 3;

type Recording = {
  slot: string;
  phase: "countdown" | "recording";
  left: number;
};

export function VoicePanel({
  token,
  project,
  readOnly,
}: {
  token: string;
  project: Project;
  readOnly: boolean;
}) {
  const [error, setError] = useState<string | null>(null);
  const [fit, setFit] = useState<Record<string, string>>({});
  const [rec, setRec] = useState<Recording | null>(null);
  const recorder = useRef<MediaRecorder | null>(null);
  useEffect(
    () => () => {
      const active = recorder.current;
      if (!active) return;
      active.onstop = null;
      if (active.state !== "inactive") active.stop();
      active.stream.getTracks().forEach((track) => track.stop());
    },
    [],
  );

  useEffect(() => {
    if (rec?.phase !== "countdown") return;
    if (rec.left === 0) {
      void start(rec.slot);
      return;
    }
    const t = setTimeout(() => setRec({ ...rec, left: rec.left - 1 }), 1000);
    return () => clearTimeout(t);
  }, [rec]);

  if (project.voiceSlots.length === 0) return null;

  const fail = (e: unknown) =>
    setError(e instanceof ApiError ? e.message : String(e));

  const post = async (path: string, body: unknown) => {
    try {
      await request(token, path, {
        method: "POST",
        body: JSON.stringify(body),
      });
      setError(null);
    } catch (e) {
      fail(e);
    }
  };

  async function start(slot: string) {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      const r = new MediaRecorder(stream);
      const chunks: Blob[] = [];
      r.ondataavailable = (e) => chunks.push(e.data);
      r.onstop = async () => {
        stream.getTracks().forEach((t) => t.stop());
        const blob = new Blob(chunks, { type: r.mimeType || "audio/webm" });
        await upload(slot, blob);
      };
      recorder.current = r;
      r.start();
      setRec({ slot, phase: "recording", left: 0 });
    } catch (e) {
      setRec(null);
      fail(e);
    }
  }

  async function upload(slot: string, blob: Blob) {
    setRec(null);
    const res = await fetch(
      `/api/voice/take?slot=${encodeURIComponent(slot)}`,
      {
        method: "POST",
        headers: {
          "x-ck-token": token,
          "content-type": blob.type.split(";")[0] || "audio/webm",
        },
        body: blob,
      },
    );
    const body = (await res.json()) as {
      message?: string;
      info?: { fit: string };
    };
    if (!res.ok) {
      setError(body.message ?? "upload failed");
      return;
    }
    setError(null);
    if (body.info) setFit((f) => ({ ...f, [slot]: body.info?.fit ?? "" }));
  }

  const speakPlaceholder = (slot: VoiceSlot) => {
    const u = new SpeechSynthesisUtterance(slot.text);
    window.speechSynthesis.speak(u);
  };

  return (
    <section className="panel voice" aria-label="Voice" data-ck-voice>
      <h2>Voice</h2>
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}
      {rec && (
        <div
          className="teleprompter"
          data-ck-teleprompter={rec.slot}
          data-ck-phase={rec.phase}
        >
          <p className="teleprompter-line">
            {project.voiceSlots.find((s) => s.id === rec.slot)?.text}
          </p>
          {rec.phase === "countdown" ? (
            <span className="countdown" data-ck-countdown>
              {rec.left}
            </span>
          ) : (
            <button
              type="button"
              data-ck-stop
              onClick={() => recorder.current?.stop()}
            >
              Stop
            </button>
          )}
        </div>
      )}
      <ul className="slots">
        {project.voiceSlots.map((slot) => (
          <li
            key={slot.id}
            data-ck-slot={slot.id}
            data-ck-slot-status={slot.status}
          >
            <select
              aria-label={`Mode of ${slot.id}`}
              data-ck-mode={slot.mode}
              value={slot.mode}
              disabled={readOnly}
              onChange={(e) => {
                const req: VoiceModeRequest = {
                  slot: slot.id,
                  mode: e.currentTarget.value as VoiceMode,
                  author: "human",
                };
                void post("/api/voice/mode", req);
              }}
            >
              {MODES.map((m) => (
                <option key={m} value={m}>
                  {m}
                </option>
              ))}
            </select>{" "}
            <span>{slot.text}</span>{" "}
            {slot.targetWindow && (
              <small className="muted">
                {slot.targetWindow.start.toFixed(1)}–
                {slot.targetWindow.end.toFixed(1)}s
              </small>
            )}
            {fit[slot.id] && (
              <small className="badge" data-ck-fit={fit[slot.id]}>
                {fit[slot.id]}
              </small>
            )}
            <div className="takes">
              {slot.takes.map((t) => (
                <label key={t.id} data-ck-take={t.id}>
                  <input
                    type="radio"
                    name={`take-${slot.id}`}
                    checked={slot.activeTake === t.id}
                    disabled={readOnly}
                    onChange={() => {
                      const req: ActiveTakeRequest = {
                        slot: slot.id,
                        take: t.id,
                      };
                      void post("/api/voice/active", req);
                    }}
                  />
                  {t.id} {t.durationSec.toFixed(1)}s
                  {t.recorded ? " (recorded)" : ""}
                </label>
              ))}
              {slot.takes
                .filter((t) => t.recorded)
                .map((t) => (
                  <label key={`consent-${t.id}`} className="consent">
                    <input
                      type="checkbox"
                      data-ck-consent={t.id}
                      checked={t.consented ?? false}
                      disabled={readOnly}
                      onChange={(e) => {
                        const req: ConsentRequest = {
                          slot: slot.id,
                          take: t.id,
                          consent: e.currentTarget.checked,
                        };
                        void post("/api/voice/consent", req);
                      }}
                    />
                    I am the speaker in take {t.id} and consent to cloning my
                    voice from it.
                  </label>
                ))}
            </div>
            <button
              type="button"
              data-ck-record={slot.id}
              disabled={readOnly || rec !== null}
              onClick={() =>
                setRec({ slot: slot.id, phase: "countdown", left: COUNTDOWN })
              }
            >
              Record
            </button>
            {slot.status === "failed" && (
              <button
                type="button"
                data-ck-placeholder={slot.id}
                onClick={() => speakPlaceholder(slot)}
              >
                Play browser voice (placeholder)
              </button>
            )}
          </li>
        ))}
      </ul>
    </section>
  );
}
