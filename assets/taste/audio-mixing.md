---
id: audio-mixing
title: Audio mixing and loudness
tags: [audio, loudness, mixing]
---
Levels are ClipKit targets; the quality gates measure them.

- Integrated loudness of the final mix: -14 LUFS for social and web, -16 LUFS for presentations;
  tolerance 1 LU.
- True peak at most -1.0 dBTP. No sample clipping.
- Voice sits at -16 to -18 LUFS short-term; music under voice is 12 to 18 dB below the voice.
- Ducking: music drops within 0.15 s of voice start and recovers over 0.4 s after voice ends.
- Music without voice: -18 to -20 LUFS short-term so it does not jump at the first word.
- Fades: 10 ms fade at every audio cut to avoid clicks; 1.0 to 2.0 s music fade out at the end.
- Room tone under cuts in dialogue at -50 to -60 dBFS so silence does not drop out.
- Sound effects peak 3 to 6 dB under the voice.
