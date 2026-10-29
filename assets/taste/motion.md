---
id: motion
title: Motion and easing
tags: [motion, easing, animation]
---
Durations in seconds. The look's motion personality scales them (calm 1.3x, snappy 0.8x).

| Element | Enter | Exit | Easing in | Easing out |
| --- | --- | --- | --- | --- |
| display text | 0.45 | 0.30 | expo out | quad in |
| body text | 0.35 | 0.25 | cubic out | quad in |
| image or card | 0.50 | 0.35 | cubic out | cubic in |
| icon | 0.30 | 0.20 | back out (1.4) | quad in |
| background | 0.80 | 0.60 | sine in-out | sine in-out |

- Hold at least 0.5 s after an element lands before it moves again; 1.0 s for text to be read.
- Hold 0.3 s at the start of a scene before the first entrance.
- At most 3 layers animating at the same time; stagger list items by 0.06 to 0.12 s.
- Keep one direction per scene (all enter from the left, or all rise); reverse only to exit.
- Distance: entrances travel 4 to 8 percent of frame width; more reads as sliding, not arriving.
- Avoid: linear easing on anything but progress bars and counters; spin over 180 degrees; bounce on
  body text.
