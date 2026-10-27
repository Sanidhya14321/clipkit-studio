---
id: "lottie-playpause"
name: "play Pause animation"
kind: "media"
version: 1
source: "packages/assets/vendor/useAnimations/icons/playPause.json"
author: "useAnimations; ClipKit integration"
license: "MIT"
provenance: "Upstream source and license retained under packages/assets/vendor. Adapter and schema are agent-authored; upstream human-only authorship is not certified."
provenance-status: "agent-authored"
tags: [ "media", "starter", "lottie-playpause" ]
mood: "editorial"
best-for: "Frame-driven useAnimations JSON with editable tint and optional label."
avoid: "Unsupported data shapes, baked text editing, and claims of independent human-only certification."
duration: "Editable bounded timeline duration."
perf: "Frame-driven local React/SVG composition; no provider or external media request."
integration: "Read asset_card and asset_schema for lottie-playpause; place with library_place or assets_add."
schema-path: "packages/assets/cards/media/lottie-playpause/schema.ts"
source-files: [ "packages/assets/src/starter-specs.ts", "packages/render/src/StarterClips.tsx", "packages/assets/vendor/useAnimations/icons/playPause.json" ]
uses: [ "preview-lottie-playpause", "inter-font", "space-grotesk-font" ]
tested:
  status: render-verified
  method: Start/middle/end rendered twice with equal PNG hashes on local Chromium.
  evidence: public/packs/native/determinism.json
---

## Appearance
Frame-driven useAnimations JSON with editable tint and optional label.
