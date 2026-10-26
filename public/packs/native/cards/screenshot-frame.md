---
id: "screenshot-frame"
name: "Screenshot frame"
kind: "overlays"
version: 1
source: "in-house: packages/render/src/Composition.tsx (native ClipKit implementation)"
author: "ClipKit contributors (agent-assisted implementation)"
license: "MIT"
provenance: "Original agent-written ClipKit motion code; not certified as human-made. No third-party template code copied."
provenance-status: "agent-authored"
tags: [ "overlays", "device", "screenshot-frame", "an", "editable", "native", "browser" ]
mood: "clean-product"
best-for: "product demonstrations and browser screenshots"
avoid: "Pretending the placeholder chart is actual product data; upload a real screenshot."
duration: "150 frames at 30fps; timeline duration is editable"
perf: "Frame-driven DOM/SVG transforms; no live simulation."
integration: "Use asset_card('screenshot-frame'), asset_schema('screenshot-frame'), then assets_add with clipId 'screenshot-frame'."
schema-path: "packages/assets/cards/overlays/screenshot-frame/schema.ts"
source-files: [ "packages/assets/src/index.ts", "packages/render/src/Composition.tsx" ]
uses: [ "preview-screenshot-frame", "space-grotesk-font", "inter-font" ]
tested:
  status: render-verified
  method: Start/middle/end rendered twice with equal PNG hashes on local Chromium.
  evidence: public/packs/native/determinism.json
---

## What it looks like
An editable native browser frame for product stories.
