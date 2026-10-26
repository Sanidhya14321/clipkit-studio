---
id: "captions"
name: "Word captions"
kind: "text"
version: 1
source: "in-house: packages/render/src/Composition.tsx (native ClipKit implementation)"
author: "ClipKit contributors (agent-assisted implementation)"
license: "MIT"
provenance: "Original agent-written ClipKit motion code; not certified as human-made. No third-party template code copied."
provenance-status: "agent-authored"
tags: [ "text", "typography", "captions", "timestamp", "aware", "with" ]
mood: "clear-accessible"
best-for: "short-form dialogue and word-highlight narration"
avoid: "Inventing word times; use actual take or source transcript timing."
duration: "150 frames at 30fps; timeline duration is editable"
perf: "Frame-driven DOM/SVG transforms; no live simulation."
integration: "Use asset_card('captions'), asset_schema('captions'), then assets_add with clipId 'captions'."
schema-path: "packages/assets/cards/text/captions/schema.ts"
source-files: [ "packages/assets/src/index.ts", "packages/render/src/Composition.tsx" ]
uses: [ "preview-captions", "space-grotesk-font", "inter-font" ]
tested:
  status: render-verified
  method: Start/middle/end rendered twice with equal PNG hashes on local Chromium.
  evidence: public/packs/native/determinism.json
---

## What it looks like
Timestamp-aware captions with active word highlighting.
