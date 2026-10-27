# Scene and component authoring

`library_list(kind="component")` returns actual placeable components. `library_inspect(assetId, source=true)` returns their APIs and adopted source. `library_place` inserts independent instances. Fonts and thumbnails do not count as components.

Managed scenes may import React, Remotion, `@clipkit/motion`, local CSS/SVG and `@clipkit/components`. Available adopted exports are WordPunch, LetterFlip, RibbonWords, ChunkyTypeStack, ComingSoonFlips, DotWordmarkReveal, SceneTransition, GooeyMorphingText, KineticCenterBuild, ApertureReveal, LiquidMetal and FlutedGlass. Pass actual inspected properties plus `item={item}` and a declared parent `bindPath`. Nested object and array paths such as `ribbon.width` and `lines[0].text` remain editable.

`scene_draft` requires Scene.tsx, an explicit object propsSchema with additionalProperties false, meaningful fields/defaults, durationFrames, and stable data-ck-layer/data-ck-path/data-ck-element bindings. Update an instance with its targetItemIds and exact baseVersion. Poll scene_status/job_status. Arbitrary imports, network calls, clocks and unseeded randomness are rejected. Invalid compilation retains the last good version.

Use local frame time. Retain upstream frame-native choreography. Seek paused CSS/WAAPI effects explicitly. Stateful simulations need deterministic replay or baking. Paper shaders use explicit setFrame with automatic speed disabled and cached textures. GPU cost is higher than typography; verify arbitrary seeks and output before production.

React Bits can be used as a user-owned project source adaptation when its specific license permits the intended application use. Import source for inspection with library_import, review its terms and dependencies, then author a deterministic scene adapter with explicit schema and selection bindings. Source registration does not execute it. ClipKit does not distribute a React Bits component pack or relabel another implementation as React Bits.
