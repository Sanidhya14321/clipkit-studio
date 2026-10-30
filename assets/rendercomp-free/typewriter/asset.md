---
id: rendercomp-free/typewriter
name: Typewriter
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
tags: [typewriter]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `Typewriter` from `src/components/typewriter/Typewriter.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- backgroundColor: string
- machineColor: string
- machineDarkColor: string
- keyColor: string
- ribbonColor: string
- paperColor: string
- textColor: string
- subColor: string
- messageText: string
- subText: string

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 12-12)
- const `DURATION_FRAMES` (lines 13-13)
- type `TypewriterProps` (lines 15-26)
- const `defaultTypewriterProps` (lines 28-39)
- const `FONT` (lines 41-41)
- const `Typewriter` (lines 43-190)
- default `default` (lines 192-192)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, spring, AbsoluteFill.

## Adaptation presets



## Owner notes


