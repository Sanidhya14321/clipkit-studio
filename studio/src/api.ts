// Client for the local ck server. The session token arrives in the URL fragment
// (`#token=...`), which browsers never send to the server; it is kept in sessionStorage and
// removed from the address bar so it does not end up in history or screenshots.
import type { CkError, Event } from "./generated/types";

const TOKEN_KEY = "ck-token";
const TOKEN_HEADER = "x-ck-token";

export function takeToken(): string | null {
  const hash = new URLSearchParams(window.location.hash.slice(1));
  const fromHash = hash.get("token");
  if (fromHash) {
    // Private browsing and storage quotas must not prevent a live session.
    try {
      sessionStorage.setItem(TOKEN_KEY, fromHash);
    } catch {
      /* token stays in memory */
    }
    history.replaceState(
      null,
      "",
      window.location.pathname + window.location.search,
    );
    return fromHash;
  }
  try {
    return sessionStorage.getItem(TOKEN_KEY);
  } catch {
    return null;
  }
}

export class ApiError extends Error {
  constructor(readonly body: CkError) {
    super(`${body.code}: ${body.message}`);
  }
}

/** Token-authenticated JSON call; a non-2xx reply throws the server's CkError. */
export async function request<T>(
  token: string,
  path: string,
  init?: RequestInit,
): Promise<T> {
  const res = await fetch(path, {
    ...init,
    headers: {
      ...init?.headers,
      [TOKEN_HEADER]: token,
      "content-type": "application/json",
    },
  });
  const body: unknown = await res.json();
  if (!res.ok) throw new ApiError(body as CkError);
  return body as T;
}

/** Open the event stream; returns a function that closes it. Reconnects after drops. */
export function subscribe(
  token: string,
  onEvent: (event: Event) => void,
  onOpen: (open: boolean) => void,
): () => void {
  let socket: WebSocket | null = null;
  let closed = false;
  let retry: ReturnType<typeof setTimeout> | undefined;

  const connect = () => {
    const scheme = window.location.protocol === "https:" ? "wss" : "ws";
    const url = `${scheme}://${window.location.host}/ws?token=${encodeURIComponent(token)}`;
    socket = new WebSocket(url);
    socket.onopen = () => onOpen(true);
    socket.onmessage = (msg) => {
      if (typeof msg.data === "string") onEvent(JSON.parse(msg.data) as Event);
    };
    socket.onclose = () => {
      onOpen(false);
      if (!closed) retry = setTimeout(connect, 1000);
    };
  };
  connect();

  return () => {
    closed = true;
    clearTimeout(retry);
    socket?.close();
  };
}
