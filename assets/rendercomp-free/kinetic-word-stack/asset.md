---
id: rendercomp-free/kinetic-word-stack
name: KineticWordStack
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
tags: [kinetic-word-stack]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `KineticWordStack` from `src/components/kinetic-word-stack/KineticWordStack.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- backgroundColor: string
- accentColor: string
- textColor: string
- words: string[]

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 12-12)
- const `DURATION_FRAMES` (lines 13-13)
- type `KineticWordStackProps` (lines 15-20)
- const `defaultKineticWordStackProps` (lines 22-27)
- const `FONT` (lines 29-29)
- const `KineticWordStack` (lines 31-169)
- default `default` (lines 171-171)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, spring, AbsoluteFill.

## Adaptation presets



## Owner notes


