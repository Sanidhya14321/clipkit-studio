// Live studio state: events from the server's WebSocket folded by the pure reducer in state.ts.
import { useEffect, useState } from "react";
import { subscribe } from "./api";
import { initial, reduce, type StudioState } from "./state";

export function useStudio(token: string): StudioState {
  const [state, setState] = useState<StudioState>(initial);
  useEffect(
    () =>
      subscribe(
        token,
        (event) => setState((s) => reduce(s, event)),
        (connected) => setState((s) => ({ ...s, connected })),
      ),
    [token],
  );
  return state;
}

/** Wall clock in ms, refreshed every `everyMs` so lock expiry shows without a new event. */
export function useNow(everyMs = 1000): number {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const id = setInterval(() => setNow(Date.now()), everyMs);
    return () => clearInterval(id);
  }, [everyMs]);
  return now;
}
