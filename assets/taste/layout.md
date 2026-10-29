---
id: layout
title: Layout and grid
tags: [layout, grid, safe-area, composition]
---
- Grid: 12 columns for 16:9, 6 for 9:16 and 1:1; gutter 0.02 of frame width.
- Margins equal the platform safe area (see platform-specs); content never crosses them.
- Spacing scale from the look (multiples of its base unit); no ad hoc gaps.
- One focal point per scene; it sits on a third line or the center, not between.
- Text blocks align left for lists and body, center for hooks and single claims.
- Keep 0.03 of frame height between stacked text blocks; no two text layers overlap.
- Text over footage: keep the subject's face clear; move text to the opposite third.
- Text fitting: shrink by at most one step of the type scale, then wrap, then cut words; never
  below the label size.
- Avoid: more than 3 text blocks on screen at once; elements touching the frame edge unless they
  bleed off it on purpose by at least 0.05.
