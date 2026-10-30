---
id: rendercomp-free/neon-sign
name: NeonSign
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
tags: [neon-sign]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `NeonSign` from `src/components/neon-sign/NeonSign.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- backgroundColor: string
- neonColor: string
- glowColor: string
- flickerColor: string
- frameColor: string
- textColor: string
- subColor: string
- signText: string
- subText: string

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 11-11)
- const `DURATION_FRAMES` (lines 12-12)
- type `NeonSignProps` (lines 14-24)
- const `defaultNeonSignProps` (lines 26-36)
- const `FONT` (lines 38-38)
- const `NeonSign` (lines 40-153)
- default `default` (lines 155-155)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, AbsoluteFill.

## Adaptation presets



## Owner notes


