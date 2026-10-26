# Shared editing architecture

```mermaid
flowchart LR
  Human[Human editor / voice command] --> Studio[React Studio]
  Studio -->|Authenticated HTTP + WebSocket| Server[Loopback Hono server]
  CLI[ck CLI] --> Server
  Agent[Codex or Claude subprocess] --> MCP[ClipKit MCP tools]
  MCP --> Server
  Server --> Lease[Owner lease + request scope]
  Lease --> Validate[Schema + source bounds + pack validation]
  Validate --> Transaction[Atomic forward/inverse transaction]
  Transaction --> Journal[(Append-only journal + snapshot)]
  Transaction --> Studio
  Server --> Review[Proposals / human acceptance]
  Review --> Validate
  Server --> Media[FFmpeg proxies, cuts, cached audio]
  Media --> Fish[Optional Fish STT / TTS / private clone]
  Media --> Whisper[Optional local whisper.cpp]
  Journal --> Render[Immutable snapshot / Remotion]
  Render --> MP4[Original-media H.264 + AAC export]
```

The server is the only project writer. A successful patch renews its lease; stale revisions and ended turns are rejected. Human selection constrains anchored agent edits. Three-way merges use stable IDs when reverting historical edits or accepting proposals after unrelated changes. Conflicting changes require review.

External text is untrusted data. Scope checks, protected source paths and explicit human executable-pack trust provide enforcement beyond prompt instructions. This is a local single-user application; it is not a sandbox for arbitrary third-party code. Trusted pack code runs locally only after explicit trust, validation and a pinned content receipt.

Cloud providers receive requested text/audio. Originals, cached takes and history remain in the local project directory. Browser speech synthesis is temporary preview playback and never silently becomes exported narration.
