// Lane B (spec 6.4): simple spoken commands run locally with zero agent tokens; anything else
// becomes a comment anchored to the selection or the playhead. Pure parsing, no side effects.

export type VoiceCommand =
  | { kind: "play" }
  | { kind: "pause" }
  | { kind: "undo" }
  | { kind: "redo" }
  | { kind: "seek"; seconds: number }
  | { kind: "split" }
  | { kind: "comment"; text: string };

/** "12", "12 seconds", "12.5s", "1:05", "1 minute 5 seconds" to seconds, or null. */
export function parseTime(raw: string): number | null {
  const s = raw.trim().toLowerCase();
  const clock = /^(\d+):([0-5]?\d(?:\.\d+)?)$/.exec(s);
  if (clock) return Number(clock[1]) * 60 + Number(clock[2]);
  const parts =
    /^(?:(\d+(?:\.\d+)?)\s*(?:m|min|mins|minute|minutes)\s*)?(?:(\d+(?:\.\d+)?)\s*(?:s|sec|secs|second|seconds)?)?$/.exec(
      s,
    );
  if (!parts || (parts[1] === undefined && parts[2] === undefined)) return null;
  return Number(parts[1] ?? 0) * 60 + Number(parts[2] ?? 0);
}

export function parseCommand(transcript: string): VoiceCommand | null {
  const text = transcript.trim();
  if (!text) return null;
  const s = text
    .toLowerCase()
    .replace(/[.!?,]+$/g, "")
    .trim();
  if (/^(play|start|resume)( it| the video)?$/.test(s)) return { kind: "play" };
  if (/^(pause|stop)( it| the video)?$/.test(s)) return { kind: "pause" };
  if (/^undo( that)?$/.test(s)) return { kind: "undo" };
  if (/^redo( that)?$/.test(s)) return { kind: "redo" };
  if (/^split( it)?( here)?$/.test(s)) return { kind: "split" };
  const seek = /^(?:go|jump|skip|seek) to (.+)$/.exec(s);
  if (seek?.[1]) {
    const seconds = parseTime(seek[1]);
    if (seconds !== null) return { kind: "seek", seconds };
  }
  return { kind: "comment", text };
}
