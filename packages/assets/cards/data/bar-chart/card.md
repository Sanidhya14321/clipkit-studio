---
id: "bar-chart"
name: "Animated bar chart"
kind: "data"
version: 1
source: "in-house: packages/render/src/MotionClips.tsx"
author: "ClipKit contributors (agent-authored implementation)"
license: "MIT"
provenance: "Original agent-written motion code; no third-party template copied."
provenance-status: "agent-authored"
tags: [ "bar-chart", "data", "editorial" ]
mood: "editorial"
best-for: "comparing four supplied nonnegative values"
avoid: "Presenting the default illustrative numbers as measured analytics."
duration: "Editable timeline duration; motion uses project frames."
perf: "At most 240 DOM particles or four SVG bars; no live simulation."
integration: "Read asset_card('bar-chart') and asset_schema('bar-chart'), then assets_add."
schema-path: "packages/assets/cards/data/bar-chart/schema.ts"
source-files: [ "packages/assets/src/index.ts", "packages/render/src/MotionClips.tsx", "packages/render/src/motion-math.ts" ]
uses: [ "preview-bar-chart", "inter-font", "space-grotesk-font" ]
tested:
  status: render-verified
  method: Start/middle/end rendered twice with equal PNG hashes on local Chromium.
  evidence: public/packs/native/determinism.json
---

## What it looks like
Four bars grow from a shared zero baseline with editable labels.
