---
id: rendercomp-free/particle-snow
name: ParticleSnow
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
tags: [particle-snow]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `ParticleSnow` from `src/components/particle-snow/ParticleSnow.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- backgroundColor: string
- snowColor: string
- accentColor: string
- flakeCount: number
- title: string
- subtitle: string
- textColor: string

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 14-14)
- const `DURATION_FRAMES` (lines 15-15)
- type `ParticleSnowProps` (lines 17-25)
- const `defaultParticleSnowProps` (lines 27-35)
- const `FONT` (lines 37-37)
- type `Flake` (lines 39-49)
- function `buildFlakes` (lines 51-78)
- const `ParticleSnow` (lines 80-215)
- default `default` (lines 217-217)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, AbsoluteFill.

## Adaptation presets



## Owner notes


