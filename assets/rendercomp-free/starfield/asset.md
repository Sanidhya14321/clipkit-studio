---
id: rendercomp-free/starfield
name: Starfield
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
tags: [starfield]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `Starfield` from `src/components/starfield/Starfield.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- count: number
- starColor: string
- baseColor: string
- warpSpeed: number
- label?: string
- labelColor?: string

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 12-12)
- const `DURATION_FRAMES` (lines 13-13)
- const `FONT_SANS` (lines 15-15)
- const `seededRandom` (lines 18-21)
- type `StarfieldProps` (lines 23-36)
- const `defaultStarfieldProps` (lines 38-45)
- const `Starfield` (lines 47-126)
- default `default` (lines 128-128)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, AbsoluteFill.

## Adaptation presets



## Owner notes


