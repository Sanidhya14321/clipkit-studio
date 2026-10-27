---
id: "adapted-line-chart-anim"
name: "Line Chart Anim"
kind: "data"
version: 1
source: "packages/assets/vendor/RenderComp/src/components/line-chart-anim/LineChartAnim.tsx"
author: "Trimora Inc. / RenderComp; ClipKit adaptation"
license: "MIT"
provenance: "Upstream source and license retained under packages/assets/vendor. Adapter and schema are agent-authored; upstream human-only authorship is not certified."
provenance-status: "agent-authored"
tags: [ "data", "starter", "adapted-line-chart-anim" ]
mood: "editorial"
best-for: "Editable adaptation of RenderComp LineChartAnim. Source notices are preserved."
avoid: "Unsupported data shapes, baked text editing, and claims of independent human-only certification."
duration: "Editable bounded timeline duration."
perf: "Frame-driven local React/SVG composition; no provider or external media request."
integration: "Read asset_card and asset_schema for adapted-line-chart-anim; place with library_place or assets_add."
schema-path: "packages/assets/cards/data/adapted-line-chart-anim/schema.ts"
source-files: [ "packages/assets/src/starter-specs.ts", "packages/render/src/StarterClips.tsx", "packages/assets/vendor/RenderComp/src/components/line-chart-anim/LineChartAnim.tsx" ]
uses: [ "preview-adapted-line-chart-anim", "inter-font", "space-grotesk-font" ]
tested:
  status: render-verified
  method: Start/middle/end rendered twice with equal PNG hashes on local Chromium.
  evidence: public/packs/native/determinism.json
---

## Appearance
Editable adaptation of RenderComp LineChartAnim. Source notices are preserved.
