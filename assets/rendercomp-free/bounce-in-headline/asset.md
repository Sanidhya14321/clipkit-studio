---
id: rendercomp-free/bounce-in-headline
name: BounceInHeadline
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
tags: [bounce-in-headline]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `BounceInHeadline` from `src/components/bounce-in-headline/BounceInHeadline.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- headline: string
- background: string
- color: string
- bounces: number
- height: number
- delay: number
- duration: number

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FONT_SANS` (lines 10-10)
- const `FPS` (lines 12-12)
- const `DURATION_FRAMES` (lines 13-13)
- type `BounceInHeadlineProps` (lines 15-27)
- const `defaultBounceInHeadlineProps` (lines 29-37)
- const `BounceInHeadline` (lines 39-92)
- default `default` (lines 94-94)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, interpolate, AbsoluteFill.

## Adaptation presets



## Owner notes


