---
id: color
title: Color
tags: [color, palette, contrast]
---
Colors come from the look's design tokens; never hard-code a hex value in a layer.

- Roles: background, surface, text, mutedText, accent, accentText. Every layer uses roles.
- Proportion: about 60 percent background, 30 percent surface, 10 percent accent.
- One accent per scene. A second accent only for a before and after contrast.
- Text on background and accentText on accent must reach 4.5 to 1 contrast.
- Footage under text: add a scrim of the background color at 40 to 60 percent opacity, or a
  surface panel, until text contrast reaches 4.5 to 1.
- Saturation: keep large fills under 70 percent saturation; save pure accents for small areas.
- Re-theming swaps tokens only; layout and motion stay the same.
- Avoid: red and green as the only difference between two states; pure #000 under pure #fff for
  more than 5 s of body text (use the look's near-black and off-white).
