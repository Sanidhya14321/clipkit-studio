---
id: "kinetic-title"
name: "Kinetic title"
kind: "text"
version: 1
source: "in-house: packages/render/src/Composition.tsx (native ClipKit implementation)"
author: "ClipKit contributors (agent-assisted implementation)"
license: "MIT"
provenance: "Original agent-written ClipKit motion code; not certified as human-made. No third-party template code copied."
provenance-status: "agent-authored"
tags: [ "text", "typography", "kinetic-title", "oversized", "editorial", "headline", "with" ]
mood: "confident-editorial"
best-for: "opening hooks and short section headlines"
avoid: "Long paragraphs; keep the title to two short lines."
duration: "150 frames at 30fps; timeline duration is editable"
perf: "Frame-driven DOM/SVG transforms; no live simulation."
integration: "Use asset_card('kinetic-title'), asset_schema('kinetic-title'), then assets_add with clipId 'kinetic-title'."
schema-path: "packages/assets/cards/text/kinetic-title/schema.ts"
source-files: [ "packages/assets/src/index.ts", "packages/render/src/Composition.tsx" ]
uses: [ "preview-kinetic-title", "space-grotesk-font", "inter-font" ]
tested:
  status: render-verified
  method: Start/middle/end rendered twice with equal PNG hashes on local Chromium.
  evidence: public/packs/native/determinism.json
---

## What it looks like
Oversized editorial headline with staggered word reveals.
