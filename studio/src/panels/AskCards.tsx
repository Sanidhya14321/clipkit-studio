// Lane C (spec 6.4): the agent's questions as cards. Pick an option, type an answer, or answer
// by voice when the browser can listen; the answer returns to the agent's `ck ask`.
import { useState } from "react";
import { ApiError, request } from "../api";
import type { AnswerRequest, Ask } from "../generated/types";

interface Recognition {
  lang: string;
  onresult:
    | ((e: { results: ArrayLike<ArrayLike<{ transcript: string }>> }) => void)
    | null;
  start: () => void;
}

function listen(onText: (t: string) => void): boolean {
  const w = window as unknown as {
    SpeechRecognition?: new () => Recognition;
    webkitSpeechRecognition?: new () => Recognition;
  };
  const R = w.SpeechRecognition ?? w.webkitSpeechRecognition;
  if (!R) return false;
  const r = new R();
  r.lang = navigator.language || "en-US";
  r.onresult = (e) => onText(e.results[0]?.[0]?.transcript ?? "");
  r.start();
  return true;
}

export function AskCards({ token, asks }: { token: string; asks: Ask[] }) {
  const [typed, setTyped] = useState<Record<string, string>>({});
  const [error, setError] = useState<string | null>(null);
  if (asks.length === 0) return null;
  const canListen =
    typeof window !== "undefined" &&
    ("SpeechRecognition" in window || "webkitSpeechRecognition" in window);

  const answer = async (id: string, text: string) => {
    if (!text.trim()) return;
    const req: AnswerRequest = { id, answer: text.trim() };
    try {
      await request(token, "/api/ask/answer", {
        method: "POST",
        body: JSON.stringify(req),
      });
      setError(null);
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
    }
  };

  return (
    <section className="panel asks" aria-label="Agent questions">
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}
      {asks.map((a) => (
        <div
          key={a.id}
          className="ask-card"
          data-ck-question={a.id}
          role="dialog"
        >
          <strong>{a.question}</strong>
          <div className="ask-options">
            {a.options.map((o) => (
              <button
                key={o}
                type="button"
                data-ck-ask-option={o}
                onClick={() => void answer(a.id, o)}
              >
                {o}
              </button>
            ))}
          </div>
          <form
            onSubmit={(e) => {
              e.preventDefault();
              void answer(a.id, typed[a.id] ?? "");
            }}
          >
            <input
              aria-label="Your answer"
              data-ck-ask-answer
              value={typed[a.id] ?? ""}
              onChange={(e) => {
                const v = e.target.value;
                setTyped((t) => ({ ...t, [a.id]: v }));
              }}
            />
            {canListen && (
              <button
                type="button"
                data-ck-ask-voice
                onClick={() => listen((text) => void answer(a.id, text))}
              >
                Answer by voice
              </button>
            )}
          </form>
        </div>
      ))}
    </section>
  );
}
