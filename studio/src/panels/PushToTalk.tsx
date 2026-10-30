// Push-to-talk (spec 6.4, Lane B): hold the button and speak. Simple commands (play, pause,
// undo, redo, go to N seconds, split here) run here with zero agent tokens; other phrases are
// sent as voice comments anchored to the selection or the playhead. Uses the browser's speech
// recognition when it has one; the text field is the fallback (and how tests drive it).
import { useRef, useState } from "react";
import { ApiError, request } from "../api";
import type {
  CommentAddRequest,
  HistoryRequest,
  SplitRequest,
} from "../generated/types";
import { parseCommand, type VoiceCommand } from "../voiceCommands";

interface Recognition {
  lang: string;
  interimResults: boolean;
  onresult:
    | ((e: { results: ArrayLike<ArrayLike<{ transcript: string }>> }) => void)
    | null;
  onend: (() => void) | null;
  start: () => void;
  stop: () => void;
}

function recognizer(): Recognition | null {
  const w = window as unknown as {
    SpeechRecognition?: new () => Recognition;
    webkitSpeechRecognition?: new () => Recognition;
  };
  const R = w.SpeechRecognition ?? w.webkitSpeechRecognition;
  return R ? new R() : null;
}

export function PushToTalk({
  token,
  selection,
  playhead,
  getPlayhead,
  onPlay,
  onPause,
  onSeek,
}: {
  token: string;
  selection: string | null;
  playhead: number;
  getPlayhead?: () => number;
  onPlay: () => void;
  onPause: () => void;
  onSeek: (seconds: number) => void;
}) {
  const [last, setLast] = useState<string>("");
  const [listening, setListening] = useState(false);
  const [typed, setTyped] = useState("");
  const rec = useRef<Recognition | null>(null);
  const supported = typeof window !== "undefined" && recognizer() !== null;

  const post = async (path: string, body: unknown) => {
    await request(token, path, { method: "POST", body: JSON.stringify(body) });
  };

  const run = async (phrase: string) => {
    const cmd: VoiceCommand | null = parseCommand(phrase);
    if (!cmd) return;
    try {
      switch (cmd.kind) {
        case "play":
          onPlay();
          break;
        case "pause":
          onPause();
          break;
        case "seek":
          onSeek(cmd.seconds);
          break;
        case "undo":
        case "redo": {
          const req: HistoryRequest = { author: "voice" };
          await post(`/api/${cmd.kind}`, req);
          break;
        }
        case "split": {
          if (!selection) throw new Error("Select a layer to split first.");
          const req: SplitRequest = {
            layerId: selection,
            at: Math.round((getPlayhead?.() ?? playhead) * 1000) / 1000,
            author: "voice",
          };
          await post("/api/split", req);
          break;
        }
        case "comment": {
          const req: CommentAddRequest = {
            text: cmd.text,
            anchor: selection
              ? { layerId: selection }
              : {
                  tStart:
                    Math.round((getPlayhead?.() ?? playhead) * 1000) / 1000,
                },
            author: "voice",
          };
          await post("/api/comments/add", req);
          break;
        }
      }
      setLast(`${cmd.kind}: ${phrase}`);
    } catch (e) {
      setLast(e instanceof ApiError ? e.message : String(e));
    }
  };

  const startListening = () => {
    const r = recognizer();
    if (!r) return;
    r.lang = navigator.language || "en-US";
    r.interimResults = false;
    r.onresult = (e) => {
      const phrase = e.results[0]?.[0]?.transcript ?? "";
      void run(phrase);
    };
    r.onend = () => setListening(false);
    rec.current = r;
    setListening(true);
    r.start();
  };

  return (
    <section className="panel ptt" aria-label="Push to talk" data-ck-ptt>
      {supported && (
        <button
          type="button"
          data-ck-ptt-button
          aria-pressed={listening}
          onPointerDown={startListening}
          onPointerUp={() => rec.current?.stop()}
        >
          {listening ? "Listening…" : "Hold to talk"}
        </button>
      )}
      <form
        onSubmit={(e) => {
          e.preventDefault();
          void run(typed);
          setTyped("");
        }}
      >
        <input
          aria-label="Voice command"
          data-ck-ptt-text
          placeholder="Or type a command"
          value={typed}
          onChange={(e) => setTyped(e.target.value)}
        />
      </form>
      {last && (
        <small className="muted" data-ck-ptt-last>
          {last}
        </small>
      )}
    </section>
  );
}
