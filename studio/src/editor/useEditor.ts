import { useCallback, useEffect, useRef, useState } from "react";
import { request } from "../api";
import type { PatchRequest, PatchResult, Project } from "../generated/types";
import { optimisticProject } from "./edits";
export function useEditor(
  project: Project | null,
  token: string,
  refresh: () => Promise<void>,
) {
  const [preview, setPreview] = useState<Project | null>(null);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState("");
  const busy = useRef(false);
  const latest = useRef(project);
  latest.current = project;
  useEffect(() => {
    if (!busy.current) setPreview(null);
  }, [project]);
  const commit = useCallback(
    async (patch: PatchRequest | null): Promise<boolean> => {
      if (!patch || !latest.current) return false;
      if (busy.current) {
        setError("Finishing the previous edit. Try again in a moment.");
        return false;
      }
      busy.current = true;
      setPending(true);
      setError("");
      try {
        setPreview(optimisticProject(latest.current, patch));
        await request<PatchResult>(token, "/api/patch", {
          method: "POST",
          body: JSON.stringify(patch),
        });
        // A failed refresh does not undo an accepted server revision. Retrying
        // an add after that acknowledgement could create duplicate content.
        try {
          await refresh();
        } catch {
          setError(
            "Your edit was saved by ck, but the latest view could not be fetched. Reconnecting will refresh it; do not repeat the edit.",
          );
        }
        return true;
      } catch (e) {
        setError(
          e instanceof Error
            ? e.message
            : "The edit did not reach ck. Your change was rolled back.",
        );
        return false;
      } finally {
        busy.current = false;
        setPending(false);
        setPreview(null);
      }
    },
    [token, refresh],
  );
  return {
    project: preview ?? project,
    commit,
    pending,
    error,
    dismiss: () => setError(""),
  };
}
