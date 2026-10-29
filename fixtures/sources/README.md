# Source fixtures

Tiny source packs written for ClipKit's tests (original, not copied from any library). They
stand in for third-party sources during vetting and ingestion tests:

- `pack-mit`: two Remotion components under the MIT license (permissive).
- `pack-restricted`: one component under a custom non-commercial license (BBC-style terms),
  blocked in commercial-safe mode.
- `pack-injection`: hostile content: prompt-injection text in a README and comments, an install
  script and a binary blob. Vetting must flag it; its text must never be followed.
