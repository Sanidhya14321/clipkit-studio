// Transcript panel (spec 5): click a word to seek; Delete or Backspace on a word cuts it from
// the footage, and the same keys on a cut word restore it. Each change sends the whole deleted
// set to the server, which rebuilds the footage segments as one human revision. While the agent
// holds the video track every word is read-only, and the server refuses the edit anyway.
import { useEffect, useMemo, useState } from "react";
import { ApiError, request } from "../api";
import type { CutWordsRequest, Project, Transcript } from "../generated/types";
import { deletedWords, timelineTime, toggled } from "../transcript";

export function TranscriptPanel({
  token,
  project,
  readOnly,
  onSeek,
}: {
  token: string;
  project: Project;
  readOnly: boolean;
  onSeek: (seconds: number) => void;
}) {
  const source = project.sources.find((s) => s.transcriptPath);
  const [transcript, setTranscript] = useState<Transcript | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!source) return;
    const q = new URLSearchParams({ source: source.id });
    request<Transcript>(token, `/api/transcript/words?${q}`).then(
      setTranscript,
      (e: unknown) => setError(e instanceof ApiError ? e.message : String(e)),
    );
  }, [token, source?.id, source?.transcriptPath]);

  const deleted = useMemo(
    () =>
      source && transcript
        ? deletedWords(project, source.id, transcript.words)
        : new Set<number>(),
    [project, source, transcript],
  );

  if (!source) return null;

  const toggle = async (i: number) => {
    const req: CutWordsRequest = {
      source: source.id,
      deleted: toggled(deleted, i),
    };
    try {
      await request(token, "/api/cut/words", {
        method: "POST",
        body: JSON.stringify(req),
      });
      setError(null);
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
    }
  };

  return (
    <section
      className="panel transcript"
      aria-label="Transcript"
      data-ck-transcript={source.id}
      data-ck-readonly={readOnly}
    >
      <h2>Transcript</h2>
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}
      <p className="transcript-words">
        {transcript?.words.map((w) => {
          const cut = deleted.has(w.i);
          return (
            <button
              key={w.i}
              type="button"
              className={cut ? "word word-cut" : "word"}
              data-ck-word={w.i}
              data-ck-deleted={cut}
              aria-pressed={cut}
              title={`${w.start.toFixed(2)}s`}
              onClick={() => {
                const t = timelineTime(project, source.id, w.start);
                if (t !== null) onSeek(t);
              }}
              onKeyDown={(e) => {
                if (readOnly) return;
                if (e.key === "Delete" || e.key === "Backspace") {
                  e.preventDefault();
                  void toggle(w.i);
                }
              }}
            >
              {w.text}
            </button>
          );
        })}
      </p>
    </section>
  );
}
