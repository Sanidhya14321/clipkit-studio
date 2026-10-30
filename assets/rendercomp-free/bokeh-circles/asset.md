---
id: rendercomp-free/bokeh-circles
name: BokehCircles
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
tags: [bokeh-circles]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `BokehCircles` from `src/components/bokeh-circles/BokehCircles.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- colors: string[]
- baseColor: string
- count: number
- label?: string
- labelColor?: string

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 7-7)
- const `DURATION_FRAMES` (lines 8-8)
- const `FONT_STACK` (lines 10-10)
- type `BokehCirclesProps` (lines 12-23)
- const `seededRandom` (lines 25-28)
- const `defaultBokehCirclesProps` (lines 30-36)
- const `BokehCircles` (lines 38-110)
- default `default` (lines 112-112)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, interpolate, AbsoluteFill.

## Adaptation presets



## Owner notes


