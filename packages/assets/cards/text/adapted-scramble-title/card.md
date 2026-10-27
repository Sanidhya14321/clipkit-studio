---
id: "adapted-scramble-title"
name: "Scramble title"
kind: "text"
version: 1
source: "packages/assets/vendor/ibelick/components/core/text-morph.tsx"
author: "Original upstream authors; ClipKit frame adaptation"
license: "MIT"
provenance: "Upstream source and license retained under packages/assets/vendor. Adapter and schema are agent-authored; upstream human-only authorship is not certified."
provenance-status: "agent-authored"
tags: [ "text", "starter", "adapted-scramble-title" ]
mood: "editorial"
best-for: "Frame-driven adaptation with editable text and palette; original source attribution is preserved."
avoid: "Unsupported data shapes, baked text editing, and claims of independent human-only certification."
duration: "Editable bounded timeline duration."
perf: "Frame-driven local React/SVG composition; no provider or external media request."
integration: "Read asset_card and asset_schema for adapted-scramble-title; place with library_place or assets_add."
schema-path: "packages/assets/cards/text/adapted-scramble-title/schema.ts"
source-files: [ "packages/assets/src/starter-specs.ts", "packages/render/src/StarterClips.tsx", "packages/assets/vendor/ibelick/components/core/text-morph.tsx" ]
uses: [ "preview-adapted-scramble-title", "inter-font", "space-grotesk-font" ]
tested:
  status: render-verified
  method: Start/middle/end rendered twice with equal PNG hashes on local Chromium.
  evidence: public/packs/native/determinism.json
---

## Appearance
Frame-driven adaptation with editable text and palette; original source attribution is preserved.
