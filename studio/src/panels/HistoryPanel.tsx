// History and branches (spec 5.4): recent revisions with undo and redo shared with the agent's
// history, and named branches the user saves and switches to. A switch is one revision, so it
// can be undone like any edit.
import { useEffect, useState } from "react";
import { ApiError, request } from "../api";
import type {
  BranchRequest,
  HistoryRequest,
  Project,
} from "../generated/types";

type History = { rev: number; lines: string[] };
type Branches = { branches: string[] };

export function HistoryPanel({
  token,
  project,
  readOnly,
}: {
  token: string;
  project: Project;
  readOnly: boolean;
}) {
  const [history, setHistory] = useState<History | null>(null);
  const [branches, setBranches] = useState<string[]>([]);
  const [name, setName] = useState("");
  const [error, setError] = useState<string | null>(null);

  const load = () => {
    request<History>(token, "/api/history?limit=8").then(
      setHistory,
      () => undefined,
    );
    request<Branches>(token, "/api/branch/list").then(
      (b) => setBranches(b.branches),
      () => undefined,
    );
  };
  useEffect(load, [token, project.rev]);

  const post = async (path: string, body: unknown) => {
    try {
      await request(token, path, {
        method: "POST",
        body: JSON.stringify(body),
      });
      setError(null);
      load();
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
    }
  };
  const human: HistoryRequest = { author: "human" };

  return (
    <section className="panel history" aria-label="History" data-ck-history>
      <h2>History</h2>
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}
      <div className="row">
        <button
          type="button"
          data-ck-undo
          disabled={readOnly}
          onClick={() => void post("/api/undo", human)}
        >
          Undo
        </button>
        <button
          type="button"
          data-ck-redo
          disabled={readOnly}
          onClick={() => void post("/api/redo", human)}
        >
          Redo
        </button>
      </div>
      <ol className="history-lines" data-ck-history-lines>
        {history?.lines.map((l) => (
          <li key={l}>{l}</li>
        ))}
      </ol>
      <h3>Branches</h3>
      <ul className="branches">
        {branches.map((b) => {
          const branch = b.split(" ")[0] ?? b;
          const req: BranchRequest = { name: branch, author: "human" };
          return (
            <li key={b} data-ck-branch={branch}>
              {b}{" "}
              <button
                type="button"
                data-ck-branch-switch={branch}
                disabled={readOnly}
                onClick={() => void post("/api/branch/switch", req)}
              >
                Switch
              </button>
            </li>
          );
        })}
      </ul>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          const req: BranchRequest = { name: name.trim(), author: "human" };
          void post("/api/branch/save", req).then(() => setName(""));
        }}
      >
        <input
          aria-label="Branch name"
          data-ck-branch-name
          placeholder="branch-name"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
        <button type="submit" data-ck-branch-save disabled={!name.trim()}>
          Save branch
        </button>
      </form>
    </section>
  );
}
