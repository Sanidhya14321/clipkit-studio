---
id: "counter"
name: "Number counter"
kind: "data"
version: 1
source: "in-house: packages/render/src/Composition.tsx (native ClipKit implementation)"
author: "ClipKit contributors (agent-assisted implementation)"
license: "MIT"
provenance: "Original agent-written ClipKit motion code; not certified as human-made. No third-party template code copied."
provenance-status: "agent-authored"
tags: [ "data", "counter", "an", "eased", "numerical", "highlight" ]
mood: "confident-editorial"
best-for: "a numeric milestone with a readable label and unit"
avoid: "Unsupported statistics or more than one competing number."
duration: "150 frames at 30fps; timeline duration is editable"
perf: "Frame-driven DOM/SVG transforms; no live simulation."
integration: "Use asset_card('counter'), asset_schema('counter'), then assets_add with clipId 'counter'."
schema-path: "packages/assets/cards/data/counter/schema.ts"
source-files: [ "packages/assets/src/index.ts", "packages/render/src/Composition.tsx" ]
uses: [ "preview-counter", "space-grotesk-font", "inter-font" ]
tested:
  status: render-verified
  method: Start/middle/end rendered twice with equal PNG hashes on local Chromium.
  evidence: public/packs/native/determinism.json
---

## What it looks like
An eased numerical highlight with editable units.
