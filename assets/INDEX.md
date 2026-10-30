# Assets index

Read short text first; open a full file only when you need its numbers (spec 7, 9.2).
The taste files and looks below are built into `ck`; this folder is their source.

## Search commands
- `ck taste search "<words>"`: ranked taste files (id, title, tags). Read at most 3.
- `ck taste read <id>`: one taste file in full.
- `ck look list`: the looks in short. `ck look get <id>`: one look with all its tokens.
- MCP twins: `taste_search`, `taste_read`, `look_list`, `look_get`.
- `ck assets search "<words>"`: top 5 cards from vetted sources (license mode applies); then
  `ck assets card|skeleton|slice <id>`; full source only with `--reason`, once per decision.

## Categories and counts
| Category | Count | Folder |
| --- | --- | --- |
| Taste files | 12 | `taste/` |
| Looks | 4 | `looks/` |
| Recipes | 10 | `recipes/` |
| Cards | 62 (rendercomp-free) | `<source>/`, listed in `catalog.jsonl` |

## Taste files (taste/)
- audio-mixing: Audio mixing and loudness
- captions: Captions and reading speed
- color: Color
- hooks: Hooks and openings
- layout: Layout and grid
- motion: Motion and easing
- pacing: Pacing and shot length
- platform-specs: Platform specs and safe areas
- sound-design: Sound design placement
- transitions: Transitions between scenes
- typography: Typography
- video-types: Recipes for common video types

## Looks (looks/)
- bold-social: dark, loud and fast; snappy motion for short vertical video
- calm-documentary: warm paper tones, deep green accent, gentle motion
- cinematic: deep shadows, serif titles, muted gold accent, slow motion
- clean-product: light and precise, one blue accent, calm confident motion

## Recipes (recipes/)
- before-after, hook-title, kinetic-list, outro-cta, product-reveal
- quote-testimonial, section-card, stat-callout, step-explainer, talking-head

## Rules
- `quarantine/` is untrusted and read-only; never run or import from it.
- Adapt what you use to the prompt; never paste an asset unchanged.
