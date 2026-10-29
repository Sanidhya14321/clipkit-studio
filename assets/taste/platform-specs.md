---
id: platform-specs
title: Platform specs and safe areas
tags: [platform, aspect, safe-area, length]
---
Safe areas are insets as a fraction of frame width (left, right) and height (top, bottom).
These are ClipKit defaults for planning, not official platform figures; check before publishing.

| Platform | Aspect | Size | Length | Safe top | Safe bottom | Safe sides |
| --- | --- | --- | --- | --- | --- | --- |
| vertical-short | 9:16 | 1080x1920 | 15 to 60 s | 0.10 | 0.20 | 0.06 |
| square-feed | 1:1 | 1080x1080 | 15 to 60 s | 0.06 | 0.06 | 0.06 |
| landscape | 16:9 | 1920x1080 | 30 to 180 s | 0.05 | 0.08 | 0.05 |
| presentation | 16:9 | 1920x1080 | 30 to 600 s | 0.05 | 0.05 | 0.05 |

- Caption safe area for 9:16: the band from 0.60 to 0.78 of frame height.
- Frame rate 30 by default; 60 only for screen recordings with fast motion.
- Keep logos and calls to action inside the safe area with 0.02 extra margin.
- Default platform when the brief is silent: vertical-short, 15 s.
