---
id: rendercomp-free/pixel-dither
name: PixelDither
source: rendercomp-free
sourceCommit: f648980e528defb7594e24c6447055f0540f340b
kind: remotion-clip
runtime: direct
determinism: native
license: MIT
spdx: MIT
licenseClass: permissive
grade: B
deps: [react, remotion]
cost: 2
aspects: []
tags: [pixel-dither]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `PixelDither` from `src/pixel-kit/fx.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

None declared.

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `snap` (lines 19-19)
- const `hseed` (lines 20-23)
- const `BitmapText` (lines 29-48)
- const `TEXT_H` (lines 50-50)
- const `PixelBeatFlash` (lines 55-80)
- const `PixelSparkle` (lines 85-112)
- const `BAYER4` (lines 118-123)
- const `PixelDither` (lines 124-142)
- function `fxSvgStyle` (lines 144-150)
- const `CrtScanlines` (lines 155-164)
- const `CrtVignette` (lines 166-173)
- const `CrtNoise` (lines 175-188)
- const `ChromaShift` (lines 190-199)
- const `ApertureGrille` (lines 202-212)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, AbsoluteFill.

## Adaptation presets



## Owner notes


