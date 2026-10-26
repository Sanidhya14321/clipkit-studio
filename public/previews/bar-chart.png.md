---
id: "preview-bar-chart"
name: "Animated bar chart preview"
kind: "image"
version: 1
source: "in-house render; scripts/verify-assets.ts"
author: "ClipKit contributors (agent-authored implementation)"
license: "MIT"
provenance: "Original agent-written motion code; no third-party template copied."
provenance-status: "agent-authored"
tags: [ "preview", "bar-chart" ]
mood: "editorial"
best-for: "Studio library thumbnail."
avoid: "Flattening the editable native clip."
duration: "Editable timeline duration; motion uses project frames."
perf: "Static PNG, 19375 bytes."
integration: "Preview URL: /previews/bar-chart.png"
source-files: [ "scripts/verify-assets.ts" ]
uses: []
tested:
  status: integrity-verified
  method: Preview regenerated with the current native render proof.
  evidence: public/packs/native/determinism.json
media-path: "public/previews/bar-chart.png"
sha256: "1a352c238f1d6fb37dec98d36e228262f681edbdbeb6a6838ba66ce2651c7bdc"
---

## What it looks like
Four bars grow from a shared zero baseline with editable labels.
