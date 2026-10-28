// Live studio state. The server is the source of truth: the project comes only from `snapshot`
// events, locks from `locks` events, and `revision` events say what the last patch touched.
import { useEffect, useState } from "react";
import { subscribe } from "./api";
import type { Event, Lock, Project } from "./generated/types";

export type Revision = Extract<Event, { type: "revision" }>;

export interface StudioState {
  connected: boolean;
  project: Project | null;
  locks: Lock[];
  lastRevision: Revision | null;
}

const initial: StudioState = {
  connected: false,
  project: null,
  locks: [],
  lastRevision: null,
};

export function reduce(state: StudioState, event: Event): StudioState {
  switch (event.type) {
    case "snapshot":
      return { ...state, project: event.project };
    case "locks":
      return { ...state, locks: event.locks };
    case "revision":
      return { ...state, lastRevision: event };
  }
}

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
