---
id: rendercomp-free/wave-hello
name: WaveHello
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
tags: [wave-hello]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `WaveHello` from `src/components/wave-hello/WaveHello.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- backgroundColor: string
- skinColor: string
- bodyColor: string
- accentColor: string
- textColor: string
- title: string
- subtitle: string

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 14-14)
- const `DURATION_FRAMES` (lines 15-15)
- type `WaveHelloProps` (lines 17-25)
- const `defaultWaveHelloProps` (lines 27-35)
- const `FONT` (lines 37-37)
- const `WaveHello` (lines 39-254)
- default `default` (lines 256-256)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, spring, AbsoluteFill.

## Adaptation presets



## Owner notes


