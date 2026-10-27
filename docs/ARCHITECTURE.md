# Architecture

One project format, journal, owner lease and renderer serve human, CLI and MCP edits. The accepted project is durable JSON; agent drafts use the existing proposal store and immutable scene artifacts. Keeping a revision commits its three-way merge in one undoable transaction. Concurrent unrelated human edits are retained; conflicting changes require review.

`assets` contains the real component registry, selected source dependency closures and media indexing. `motion` supplies deterministic helpers and focused Video Forge guidance. `scenes` compiles validated TSX in a worker and retains last working versions. `server` coordinates sessions, leases and scoped operations. `studio` renders Components, Storyboard, Properties, Media and frame comments. `render` drives preview/export from the same composition.

Storyboard version 2 separates narrative beats, overlapping shots, persistent copy intervals, versioned component uses, transitions and audio cues. All boundaries are integral end-exclusive frames. Long work requires consecutive chapters of at most three minutes; later chapters reuse the accepted global direction and load bounded context. Hour-long performance is unverified.

Preparation makes one creative invocation, renders 90 real frames, releases the lease and waits for the human. Approval binds plan hash, component versions, sample bytes and base project. Continuation resumes an exact recorded session ID with a fresh lease. Approved prefix content is protected. No automatic aesthetic critic accepts a film.

A range edit preserves outside content and the source animation clock. Executable scene code is reviewed managed project code; static validation is not a security sandbox. Downloaded executable packs still require content-bound trust.
