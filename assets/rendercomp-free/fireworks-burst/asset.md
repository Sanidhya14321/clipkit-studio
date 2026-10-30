---
id: rendercomp-free/fireworks-burst
name: FireworksBurst
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
tags: [fireworks-burst]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `FireworksBurst` from `src/components/fireworks-burst/FireworksBurst.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- backgroundColor: string
- burstColors: string[]
- trailColor: string
- sparkColor: string
- textColor: string
- subColor: string
- messageText: string
- subText: string

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 12-12)
- const `DURATION_FRAMES` (lines 13-13)
- type `FireworksBurstProps` (lines 15-24)
- const `defaultFireworksBurstProps` (lines 26-35)
- const `FONT` (lines 37-37)
- const `FireworksBurst` (lines 39-158)
- default `default` (lines 160-160)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, spring, AbsoluteFill.

## Adaptation presets



## Owner notes


