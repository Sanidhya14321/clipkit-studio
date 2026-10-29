// Comments (spec 5.4, 8.4): every comment with its anchor and status, and a form that pins a new
// one to the playhead, the loop range, the selected layer or the clicked element. Send to agent
// writes it as a human comment, which the agent's `ck wait` picks up; comments never wait
// behind the agent's project lock.
import { useState } from "react";
import { ApiError, request } from "../api";
import type { CommentAddRequest, Project } from "../generated/types";
import {
  type AnchorKind,
  type Pointer,
  anchorFor,
  anchorLabel,
  kinds,
} from "../comments";

const KIND_LABEL: Record<AnchorKind, string> = {
  moment: "Moment",
  range: "Range",
  layer: "Layer",
  element: "Element",
};

export function CommentsPanel({
  token,
  project,
  pointer,
}: {
  token: string;
  project: Project;
  pointer: Pointer;
}) {
  const [text, setText] = useState("");
  const [kind, setKind] = useState<AnchorKind>("moment");
  const [error, setError] = useState<string | null>(null);
  const available = kinds(pointer);
  const chosen = available.includes(kind) ? kind : "moment";

  const send = async () => {
    if (!text.trim()) return;
    const req: CommentAddRequest = {
      text: text.trim(),
      anchor: anchorFor(chosen, pointer),
      author: "human",
    };
    try {
      await request(token, "/api/comments/add", {
        method: "POST",
        body: JSON.stringify(req),
      });
      setText("");
      setError(null);
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
    }
  };

  return (
    <section className="panel comments" aria-label="Comments" data-ck-comments>
      <h2>Comments</h2>
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}
      <ul className="comment-list">
        {project.comments.map((c) => (
          <li key={c.id} data-ck-comment={c.id} data-ck-status={c.status}>
            <span className={`badge status-${c.status}`}>{c.status}</span>{" "}
            <span className="muted">{anchorLabel(project, c.anchor)}</span>
            <p>{c.text}</p>
            {c.replies.length > 0 && (
              <p className="muted" data-ck-reply>
                {c.replies[c.replies.length - 1]?.author}:{" "}
                {c.replies[c.replies.length - 1]?.text}
              </p>
            )}
          </li>
        ))}
      </ul>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          void send();
        }}
      >
        <fieldset className="anchor-kinds">
          <legend>Pin to</legend>
          {available.map((k) => (
            <label key={k}>
              <input
                type="radio"
                name="anchor-kind"
                value={k}
                checked={chosen === k}
                onChange={() => setKind(k)}
                data-ck-anchor-kind={k}
              />
              {KIND_LABEL[k]}
            </label>
          ))}
        </fieldset>
        <span className="muted" data-ck-anchor-preview>
          {anchorLabel(project, anchorFor(chosen, pointer))}
        </span>
        <textarea
          aria-label="Comment"
          data-ck-comment-text
          value={text}
          onChange={(e) => setText(e.target.value)}
          rows={2}
        />
        <button type="submit" data-ck-send disabled={!text.trim()}>
          Send to agent
        </button>
      </form>
    </section>
  );
}
