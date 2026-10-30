---
id: rendercomp-free/camera-shake
name: CameraShake
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
tags: [camera-shake]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `CameraShake` from `src/components/camera-shake/CameraShake.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- title: string
- subtitle?: string
- background: string
- textColor: string
- accentColor: string
- maxIntensity?: number
- impactAt?: number
- shakeDuration?: number

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 14-14)
- const `DURATION_FRAMES` (lines 15-15)
- const `FONT_EN` (lines 17-17)
- type `CameraShakeProps` (lines 19-36)
- const `defaultCameraShakeProps` (lines 38-47)
- const `pseudoRand` (lines 50-53)
- const `ImpactRays` (lines 55-95)
- const `CameraShake` (lines 97-242)
- default `default` (lines 244-244)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, Easing, AbsoluteFill.

## Adaptation presets



## Owner notes


