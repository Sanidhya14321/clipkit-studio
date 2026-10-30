---
id: rendercomp-free/loop-grid-wave
name: LoopGridWave
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
tags: [loop-grid-wave]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `LoopGridWave` from `src/components/loop-grid-wave/LoopGridWave.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- backgroundColor: string
- dotColor: string
- accentColor: string
- amplitude: number
- speed: number

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 10-10)
- const `DURATION_FRAMES` (lines 11-11)
- type `LoopGridWaveProps` (lines 13-19)
- const `defaultLoopGridWaveProps` (lines 21-27)
- const `LoopGridWave` (lines 29-79)
- default `default` (lines 81-81)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, AbsoluteFill.

## Adaptation presets



## Owner notes


