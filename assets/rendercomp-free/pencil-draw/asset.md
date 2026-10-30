---
id: rendercomp-free/pencil-draw
name: PencilDraw
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
tags: [pencil-draw]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `PencilDraw` from `src/components/pencil-draw/PencilDraw.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- backgroundColor: string
- pencilColor: string
- lineColor: string
- eraserColor: string
- textColor: string
- subColor: string
- messageText: string
- subText: string

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 13-13)
- const `DURATION_FRAMES` (lines 14-14)
- type `PencilDrawProps` (lines 16-25)
- const `defaultPencilDrawProps` (lines 27-36)
- const `FONT` (lines 38-38)
- const `PencilDraw` (lines 40-162)
- default `default` (lines 164-164)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, spring, Easing, AbsoluteFill.

## Adaptation presets



## Owner notes


