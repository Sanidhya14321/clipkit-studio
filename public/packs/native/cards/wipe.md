---
id: "wipe"
name: "Color wipe"
kind: "transitions"
version: 1
source: "in-house: packages/render/src/Composition.tsx (native ClipKit implementation)"
author: "ClipKit contributors (agent-assisted implementation)"
license: "MIT"
provenance: "Original agent-written ClipKit motion code; not certified as human-made. No third-party template code copied."
provenance-status: "agent-authored"
tags: [ "transitions", "transition", "wipe", "a", "sweeping", "color", "panel" ]
mood: "energetic-clean"
best-for: "short scene punctuation and color-matched transitions"
avoid: "Leaving it across readable dialogue; it is a color overlay, not a two-video blend."
duration: "36 frames at 30fps; timeline duration is editable"
perf: "Frame-driven DOM/SVG transforms; no live simulation."
integration: "Use asset_card('wipe'), asset_schema('wipe'), then assets_add with clipId 'wipe'."
schema-path: "packages/assets/cards/transitions/wipe/schema.ts"
source-files: [ "packages/assets/src/index.ts", "packages/render/src/Composition.tsx" ]
uses: [ "preview-wipe" ]
tested:
  status: render-verified
  method: Start/middle/end rendered twice with equal PNG hashes on local Chromium.
  evidence: public/packs/native/determinism.json
---

## What it looks like
A sweeping color panel that enters and leaves the frame.
