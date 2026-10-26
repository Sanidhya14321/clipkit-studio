---
id: "gradient"
name: "Living gradient"
kind: "backgrounds"
version: 1
source: "in-house: packages/render/src/Composition.tsx (native ClipKit implementation)"
author: "ClipKit contributors (agent-assisted implementation)"
license: "MIT"
provenance: "Original agent-written ClipKit motion code; not certified as human-made. No third-party template code copied."
provenance-status: "agent-authored"
tags: [ "backgrounds", "background", "gradient", "a", "deterministic", "soft", "light" ]
mood: "calm-ambient"
best-for: "ambient title backdrops and low-detail section intros"
avoid: "Claiming this CSS light-field is a 3D or fluid simulation."
duration: "150 frames at 30fps; timeline duration is editable"
perf: "Frame-driven DOM/SVG transforms; no live simulation."
integration: "Use asset_card('gradient'), asset_schema('gradient'), then assets_add with clipId 'gradient'."
schema-path: "packages/assets/cards/backgrounds/gradient/schema.ts"
source-files: [ "packages/assets/src/index.ts", "packages/render/src/Composition.tsx" ]
uses: [ "preview-gradient" ]
tested:
  status: render-verified
  method: Start/middle/end rendered twice with equal PNG hashes on local Chromium.
  evidence: public/packs/native/determinism.json
---

## What it looks like
A deterministic soft light background with orbital forms.
