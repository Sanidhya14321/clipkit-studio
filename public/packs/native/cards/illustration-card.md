---
id: "illustration-card"
name: "Illustration card"
kind: "media"
version: 1
source: "in-house: packages/render/src/Composition.tsx (native ClipKit implementation)"
author: "ClipKit contributors (agent-assisted implementation)"
license: "MIT"
provenance: "Original agent-written ClipKit motion code; not certified as human-made. No third-party template code copied."
provenance-status: "agent-authored"
tags: [ "media", "illustration", "illustration-card", "an", "open", "source" ]
mood: "friendly-editorial"
best-for: "fitness, activity or general explainer cutaways"
avoid: "Treating the bundled gym illustration as a different person, product or logo."
duration: "150 frames at 30fps; timeline duration is editable"
perf: "Frame-driven DOM/SVG transforms; no live simulation."
integration: "Use asset_card('illustration-card'), asset_schema('illustration-card'), then assets_add with clipId 'illustration-card'."
schema-path: "packages/assets/cards/media/illustration-card/schema.ts"
source-files: [ "packages/assets/src/index.ts", "packages/render/src/Composition.tsx" ]
uses: [ "preview-illustration-card", "space-grotesk-font", "inter-font", "gym-illustration" ]
tested:
  status: render-verified
  method: Start/middle/end rendered twice with equal PNG hashes on local Chromium.
  evidence: public/packs/native/determinism.json
---

## What it looks like
An open-source illustration in a layered editorial card.
