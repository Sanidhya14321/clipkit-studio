---
id: rendercomp-free/pixel-mosaic-transition
name: PixelMosaicTransition
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
tags: [pixel-mosaic-transition]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `PixelMosaicTransition` from `src/components/pixel-mosaic-transition/PixelMosaicTransition.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- backgroundColor: string
- screenAColor: string
- screenAAccent: string
- screenBColor: string
- screenBAccent: string
- maxCell: number
- ditherOpacity: number

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 15-15)
- const `DURATION_FRAMES` (lines 16-16)
- const `IW` (lines 18-18)
- const `IH` (lines 19-19)
- const `snap` (lines 20-20)
- type `PixelMosaicTransitionProps` (lines 22-30)
- const `defaultPixelMosaicTransitionProps` (lines 32-40)
- const `hexToRgb` (lines 42-45)
- const `easeInOut` (lines 48-48)
- function `sceneField` (lines 55-103)
- const `PixelMosaicTransition` (lines 105-183)
- default `default` (lines 185-185)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, AbsoluteFill.

## Adaptation presets



## Owner notes


