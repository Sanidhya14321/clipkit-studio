---
id: "terminal-typewriter"
name: "Terminal typewriter"
kind: "text"
version: 1
source: "in-house: packages/render/src/MotionClips.tsx"
author: "ClipKit contributors (agent-authored implementation)"
license: "MIT"
provenance: "Original agent-written motion code; no third-party template copied."
provenance-status: "agent-authored"
tags: [ "terminal-typewriter", "text", "technical" ]
mood: "technical"
best-for: "short demonstrations and technical status sequences"
avoid: "Executing commands: this is display-only text."
duration: "Editable timeline duration; motion uses project frames."
perf: "At most 240 DOM particles or four SVG bars; no live simulation."
integration: "Read asset_card('terminal-typewriter') and asset_schema('terminal-typewriter'), then assets_add."
schema-path: "packages/assets/cards/text/terminal-typewriter/schema.ts"
source-files: [ "packages/assets/src/index.ts", "packages/render/src/MotionClips.tsx", "packages/render/src/motion-math.ts" ]
uses: [ "preview-terminal-typewriter", "inter-font", "space-grotesk-font" ]
tested:
  status: render-verified
  method: Start/middle/end rendered twice with equal PNG hashes on local Chromium.
  evidence: public/packs/native/determinism.json
---

## What it looks like
A framed terminal-style panel reveals editable text.
