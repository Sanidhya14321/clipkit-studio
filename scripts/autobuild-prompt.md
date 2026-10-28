AUTONOMOUS BUILD RUN. Nobody is watching; never ask questions and never wait for input.

1. Read CLAUDE.md, PROGRESS.md and docs/spec/INDEX.md. Read only the spec files the next task needs.
2. Pick the next unfinished task in the current phase. If the phase is finished and its checks pass, mark it done, advance to the next phase in section 12 order, and pick its first task.
3. Size the work to one session: one task, or a few small ones. Use the phase-runner skill. Send test runs, long logs and doc lookups to the test-runner and researcher subagents and use only their summaries.
4. Before writing code, write a 3 to 6 line plan for this run in PROGRESS.md.
5. Build in small steps with just check after each. Write the tests. Record evidence in VERIFICATION.md.
6. Stop at a clean point when the task is done or when your context is getting large: tests green, just check passing, PROGRESS.md updated (task ticked, 5-line handoff, next step), then commit with a message naming the phase and task. Then end the run.
7. If something is unclear, choose the simplest option that fits the spec, record it in DECISIONS.md and under Open questions, and continue. If one task is blocked, mark it blocked with the reason and move to another task in the same phase.
8. If the whole build is blocked (missing key, missing owner input that every remaining task needs, repeated failure you cannot fix), set STATUS: BLOCKED with the reason and stop. If everything in sections 12 and 12.1 is done, set STATUS: DONE.
9. Never edit docs/spec/ or docs/MASTER_PROMPT.md, never weaken a test to make it pass, never mark a check PASS without evidence, never push, never print secrets.

Loop contract (scripts/autobuild.sh reads these lines; keep them exact):
- The first lines of PROGRESS.md hold `STATUS: <RUNNING|BLOCKED|DONE|PAUSED>` and `PHASE: <number>`. Update `PHASE:` in the commit that marks a phase done; set `STATUS:` only as rule 8 says.
- This run replaces the "wait for approval" step of the phase-runner skill: write the plan and proceed.
- Start with `git status`. If a previous run left uncommitted changes, read the diff and either finish and commit them or restore them before new work.
- Bash runs only commands allowed in .claude/settings.json; a denied command does not prompt in this mode. Prefer the Read, Grep, Glob, Edit and Write tools, plus `just`, `cargo` and `git`.
