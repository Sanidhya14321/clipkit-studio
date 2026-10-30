---
id: rendercomp-free/eye-reveal
name: EyeReveal
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
tags: [eye-reveal]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `EyeReveal` from `src/components/eye-reveal/EyeReveal.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- backgroundColor: string
- scleraColor: string
- irisColor: string
- pupilColor: string
- lidColor: string
- textColor: string
- subColor: string
- messageText: string
- subText: string

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 13-13)
- const `DURATION_FRAMES` (lines 14-14)
- type `EyeRevealProps` (lines 16-26)
- const `defaultEyeRevealProps` (lines 28-38)
- const `FONT` (lines 40-40)
- const `EyeReveal` (lines 42-170)
- default `default` (lines 172-172)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, spring, Easing, AbsoluteFill.

## Adaptation presets



## Owner notes


