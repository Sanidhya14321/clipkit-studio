---
id: rendercomp-free/transition-circle-wipe
name: TransitionCircleWipe
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
tags: [transition-circle-wipe]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `TransitionCircleWipe` from `src/components/transition-circle-wipe/TransitionCircleWipe.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- backgroundColor: string
- accentColor: string
- textColor: string
- fromLabel: string
- toLabel: string

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 21-21)
- const `DURATION_FRAMES` (lines 22-22)
- type `TransitionCircleWipeProps` (lines 24-30)
- const `defaultTransitionCircleWipeProps` (lines 32-38)
- const `FONT` (lines 40-40)
- const `IRIS_START` (lines 43-43)
- const `IRIS_END` (lines 44-44)
- const `RULE_START` (lines 45-45)
- const `RULE_END` (lines 46-46)
- const `parseHex` (lines 49-64)
- const `mixHex` (lines 67-75)
- const `TransitionCircleWipe` (lines 77-211)
- default `default` (lines 213-213)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, Easing, AbsoluteFill.

## Adaptation presets



## Owner notes


