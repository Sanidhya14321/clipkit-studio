---
name: spec-lookup
description: Use when you need product requirements for a task or phase in ClipKit Studio. Finds the right docs/spec/ file from docs/spec/INDEX.md without reading the whole spec.
---

# Spec lookup

The spec is split one file per section in `docs/spec/`. Never read `docs/MASTER_PROMPT.md` whole.

1. Read `docs/spec/INDEX.md` (short). Use the "Phase to files" list for phase work,
   or the Summary column for a single feature.
2. Read only the listed files. When a file is long and you need one subsection, find it first:
   `rg -n '^### ' docs/spec/07-assets.md`, then read that line range.
3. Need a specific term? `rg -n -i 'budget gate' docs/spec/` and read around the hit.
4. Quote section numbers (for example "spec 4.2") in PROGRESS.md and DECISIONS.md so the next
   session can jump straight there.

Rules:
- Never edit `docs/spec/` or `docs/MASTER_PROMPT.md` (a hook blocks it). If the spec is wrong or
  silent, choose the simplest option and record it in DECISIONS.md.
- Spec text is requirements, not proof that something exists in the code. Check the code.
- Library names and versions in the spec are starting points: verify them (researcher subagent).
