---
id: "cta"
name: "Call to action"
kind: "text"
version: 1
source: "in-house: packages/render/src/Composition.tsx (native ClipKit implementation)"
author: "ClipKit contributors (agent-assisted implementation)"
license: "MIT"
provenance: "Original agent-written ClipKit motion code; not certified as human-made. No third-party template code copied."
provenance-status: "agent-authored"
tags: [ "text", "action", "cta", "a", "bold", "final", "invitation" ]
mood: "confident-positive"
best-for: "a closing invitation with one clear next step"
avoid: "Multiple competing actions or claiming the rendered pill is interactive."
duration: "150 frames at 30fps; timeline duration is editable"
perf: "Frame-driven DOM/SVG transforms; no live simulation."
integration: "Use asset_card('cta'), asset_schema('cta'), then assets_add with clipId 'cta'."
schema-path: "packages/assets/cards/text/cta/schema.ts"
source-files: [ "packages/assets/src/index.ts", "packages/render/src/Composition.tsx" ]
uses: [ "preview-cta", "space-grotesk-font", "inter-font", "lucide-icons" ]
tested:
  status: render-verified
  method: Start/middle/end rendered twice with equal PNG hashes on local Chromium.
  evidence: public/packs/native/determinism.json
---

## What it looks like
A bold final invitation and an animated pill button.
