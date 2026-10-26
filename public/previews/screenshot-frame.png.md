---
id: "preview-screenshot-frame"
name: "Screenshot frame preview"
kind: "image"
version: 1
source: "in-house render of screenshot-frame; scripts/verify-assets.ts"
author: "ClipKit contributors (agent-generated preview)"
license: "MIT"
provenance: "Rendered from agent-authored native code; bundled dependency licenses also apply."
provenance-status: "agent-authored"
tags: [ "preview", "screenshot-frame" ]
mood: "clean-product"
best-for: "Studio library thumbnails only."
avoid: "Using as flattened footage instead of the editable native clip."
perf: "Static PNG, 18719 bytes."
integration: "Preview URL: /previews/screenshot-frame.png. Place the native clip, not this image."
media-path: "public/previews/screenshot-frame.png"
sha256: "44db8c9387651cef011dab9b05cae2866c450d33a03df066efcf801ddb2d0184"
source-files: [ "scripts/verify-assets.ts" ]
uses: []
tested:
  status: integrity-verified
  method: Preview regenerated with the current native render proof.
  evidence: public/packs/native/determinism.json
---

## What it looks like
A static preview of the editable screenshot frame.
