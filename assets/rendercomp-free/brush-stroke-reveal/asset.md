---
id: rendercomp-free/brush-stroke-reveal
name: BrushStrokeReveal
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
tags: [brush-stroke-reveal]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `BrushStrokeReveal` from `src/components/brush-stroke-reveal/BrushStrokeReveal.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- title: string
- description?: string
- strokes?: number
- direction?: "left" | "right"
- textColor?: string
- background?: string
- accentColor?: string

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 10-10)
- const `DURATION_FRAMES` (lines 11-11)
- const `FONT_STACK` (lines 13-13)
- const `EASE_OUT` (lines 16-16)
- type `BrushStrokeRevealProps` (lines 18-33)
- const `defaultBrushStrokeRevealProps` (lines 35-43)
- const `BrushReveal` (lines 51-113)
- const `BrushStrokeReveal` (lines 115-188)
- default `default` (lines 190-190)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, interpolate, Easing, AbsoluteFill.

## Adaptation presets



## Owner notes


