---
id: rendercomp-free/pixel-waterfall-cycle
name: PixelWaterfallCycle
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
tags: [pixel-waterfall-cycle]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `PixelWaterfallCycle` from `src/components/pixel-waterfall-cycle/PixelWaterfallCycle.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- backgroundColor: string
- accentColor: string
- cliffColor: string
- cycleSpeed: number
- showMist: boolean

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 14-14)
- const `DURATION_FRAMES` (lines 15-15)
- const `IW` (lines 17-17)
- const `IH` (lines 18-18)
- const `HORIZON` (lines 19-19)
- const `snap` (lines 20-20)
- const `seed` (lines 21-24)
- type `PixelWaterfallCycleProps` (lines 26-32)
- const `defaultPixelWaterfallCycleProps` (lines 34-40)
- const `hexToRgb` (lines 42-45)
- function `cliffRects` (lines 48-66)
- const `PixelWaterfallCycle` (lines 68-139)
- default `default` (lines 141-141)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, AbsoluteFill.

## Adaptation presets



## Owner notes


