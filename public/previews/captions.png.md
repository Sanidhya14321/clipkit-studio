---
id: "preview-captions"
name: "Word captions preview"
kind: "image"
version: 1
source: "in-house render of captions; scripts/verify-assets.ts"
author: "ClipKit contributors (agent-generated preview)"
license: "MIT"
provenance: "Rendered from agent-authored native code; bundled dependency licenses also apply."
provenance-status: "agent-authored"
tags: [ "preview", "captions" ]
mood: "clear-accessible"
best-for: "Studio library thumbnails only."
avoid: "Using as flattened footage instead of the editable native clip."
perf: "Static PNG, 10032 bytes."
integration: "Preview URL: /previews/captions.png. Place the native clip, not this image."
media-path: "public/previews/captions.png"
sha256: "956117a67ba80dbd823d4f00f2a17b3fb69bc6eb9be75135f37db9b2eb5d710f"
source-files: [ "scripts/verify-assets.ts" ]
uses: []
tested:
  status: integrity-verified
  method: Preview regenerated with the current native render proof.
  evidence: public/packs/native/determinism.json
---

## What it looks like
A static preview of the editable word captions.
