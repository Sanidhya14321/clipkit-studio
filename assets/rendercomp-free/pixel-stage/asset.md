---
id: rendercomp-free/pixel-stage
name: PixelStage
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
tags: [pixel-stage]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `PixelStage` from `src/pixel-kit/render.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

None declared.

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `snap` (lines 18-18)
- const `PixelStage` (lines 24-58)
- type `BeatEvent` (lines 63-63)
- function `computePulse` (lines 66-77)
- function `idlePulse` (lines 80-85)
- function `idleBeats` (lines 88-93)
- const `GridFloor` (lines 99-139)
- const `SkyBands` (lines 142-161)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, AbsoluteFill.

## Adaptation presets



## Owner notes


