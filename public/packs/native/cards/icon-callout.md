---
id: "icon-callout"
name: "Icon callout"
kind: "overlays"
version: 1
source: "in-house: packages/render/src/Composition.tsx (native ClipKit implementation)"
author: "ClipKit contributors (agent-assisted implementation)"
license: "MIT"
provenance: "Original agent-written ClipKit motion code; not certified as human-made. No third-party template code copied."
provenance-status: "agent-authored"
tags: [ "overlays", "callout", "icon-callout", "a", "glowing", "native", "icon" ]
mood: "clean-positive"
best-for: "one feature benefit or an important detail"
avoid: "Stacks of callouts or unsupported third-party brand logos."
duration: "150 frames at 30fps; timeline duration is editable"
perf: "Frame-driven DOM/SVG transforms; no live simulation."
integration: "Use asset_card('icon-callout'), asset_schema('icon-callout'), then assets_add with clipId 'icon-callout'."
schema-path: "packages/assets/cards/overlays/icon-callout/schema.ts"
source-files: [ "packages/assets/src/index.ts", "packages/render/src/Composition.tsx" ]
uses: [ "preview-icon-callout", "space-grotesk-font", "inter-font", "lucide-icons" ]
tested:
  status: render-verified
  method: Start/middle/end rendered twice with equal PNG hashes on local Chromium.
  evidence: public/packs/native/determinism.json
---

## What it looks like
A glowing native icon, label and supporting detail.
