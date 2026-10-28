# Crash evidence: studio server dies during export

Two reproductions of the same S1 reliability finding (studio crash, data intact, jobs lost):

## Run 1 (18:12–18:23, log: /tmp/opencode/firstrun/dev5.log)
1. `ck export --preset vertical-720 --quality draft` → job started.
2. Job status → `failed, error: "Unknown system error -122 ... write"` (EDQUOT/quota on /tmp).
3. `ck export` again while the first job churned → server started a SECOND concurrent export job
   (no busy guard: job_c3c72ccc and job_6885a23c both "running").
4. Server process gone shortly after; `curl /api/bootstrap` → dead; vite log ends with webpack
   `SyncWriteStream` 'error' errno -122 → **unhandled crash kills the whole studio**.

## Run 2 (18:25–18:30, log: /tmp/opencode/firstrun/dev6.log)
1. Fresh server on 4311, export job started again; after ~90 s server dead again (bootstrap → 000).
2. Repeat diagnosis via dev6.log + discover adequate coverage bootstrap). A visible symptom in logs:
   `EMFILE: too many open files` file-watcher errors (ulimit consumed by the renderer/ffmpeg
   children + watchers), then webpack cache write failure again.
3. Remaining repo state: `.clipkit/exports/` EMPTY (partial render never placed evidence),
   project file intact on rev 8 — no data loss; but two jobs and ~4 minutes lost.

## Impact
- Studio must be manually restarted; editor workspace disconnects silently (no toast observed in
  logs, websocket just drops).
- Under normal machine conditions (no tmp quota), export probably succeeds; the crash-on-error
  path itself is the defect: ANY cache-write/IO failure during import/render kills the server.
- Additionally: the queued second export job demonstrates missing concurrency guard on export.

## Recommended fix directions
- Treat webpack/renderer cache write errors as warnings (they already 'warn' to console); the crash
  is only because the stream 'error' event is unhandled. Add an app-level unhandled 'error' guard
  around the export path.
- Block or coalesce a new export while one is running (the spec's queue exists but re-entry passes
  through `export_start` again).
