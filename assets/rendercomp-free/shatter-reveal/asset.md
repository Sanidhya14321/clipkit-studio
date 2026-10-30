---
id: rendercomp-free/shatter-reveal
name: ShatterReveal
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
tags: [shatter-reveal]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `ShatterReveal` from `src/components/shatter-reveal/ShatterReveal.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- title: string
- subtitle: string
- pieces: number
- spread: number
- textColor: string
- background: string
- tileBackground: string
- accentColor: string

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 14-14)
- const `DURATION_FRAMES` (lines 15-15)
- const `FONT` (lines 17-17)
- function `seededRandom` (lines 20-26)
- type `ShatterRevealProps` (lines 28-45)
- const `defaultShatterRevealProps` (lines 47-57)
- const `ShatterReveal` (lines 59-227)
- default `default` (lines 229-229)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, Easing, AbsoluteFill.

## Adaptation presets



## Owner notes


