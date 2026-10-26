---
id: "lower-third"
name: "Lower third"
kind: "overlays"
version: 1
source: "in-house: packages/render/src/Composition.tsx (native ClipKit implementation)"
author: "ClipKit contributors (agent-assisted implementation)"
license: "MIT"
provenance: "Original agent-written ClipKit motion code; not certified as human-made. No third-party template code copied."
provenance-status: "agent-authored"
tags: [ "overlays", "identity", "lower-third", "a", "crisp", "card" ]
mood: "restrained-professional"
best-for: "speaker introductions and interview identifiers"
avoid: "Covering captions or important details near the bottom of footage."
duration: "150 frames at 30fps; timeline duration is editable"
perf: "Frame-driven DOM/SVG transforms; no live simulation."
integration: "Use asset_card('lower-third'), asset_schema('lower-third'), then assets_add with clipId 'lower-third'."
schema-path: "packages/assets/cards/overlays/lower-third/schema.ts"
source-files: [ "packages/assets/src/index.ts", "packages/render/src/Composition.tsx" ]
uses: [ "preview-lower-third", "space-grotesk-font", "inter-font" ]
tested:
  status: render-verified
  method: Start/middle/end rendered twice with equal PNG hashes on local Chromium.
  evidence: public/packs/native/determinism.json
---

## What it looks like
A crisp identity card with an animated accent rule.
