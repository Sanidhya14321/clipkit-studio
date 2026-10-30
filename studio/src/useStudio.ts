// Live studio state: events from the server's WebSocket folded by the pure reducer in state.ts.
import { useCallback, useEffect, useState } from "react";
import { request, subscribe } from "./api";
import type { Ask, Project } from "./generated/types";
import { initial, reduce, type StudioState } from "./state";

export function useStudio(
  token: string,
): StudioState & { refresh: () => Promise<void> } {
  const [state, setState] = useState<StudioState>(initial);
  const refresh = useCallback(async () => {
    const project = await request<Project>(token, "/api/project?view=full");
    setState((s) =>
      s.project && s.project.rev > project.rev
        ? s
        : { ...s, project, locks: project.locks },
    );
  }, [token]);
  useEffect(
    () =>
      subscribe(
        token,
        (event) => setState((s) => reduce(s, event)),
        (connected) => {
          setState((s) => ({ ...s, connected }));
          // Questions asked before this studio connected (later ones arrive as events).
          if (connected) {
            request<Ask[]>(token, "/api/asks").then(
              (asks) => setState((s) => ({ ...s, asks })),
              () => undefined,
            );
          }
        },
      ),
    [token],
  );
  return { ...state, refresh };
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
