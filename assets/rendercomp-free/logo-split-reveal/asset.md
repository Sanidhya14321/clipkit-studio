---
id: rendercomp-free/logo-split-reveal
name: LogoSplitReveal
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
tags: [logo-split-reveal]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `LogoSplitReveal` from `src/components/logo-split-reveal/LogoSplitReveal.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- logoText: string
- splitOffset: number
- revealDuration: number
- logoColor: string
- backgroundColor: string
- fontSize: number
- duration: number

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 30-30)
- const `DURATION_FRAMES` (lines 31-31)
- const `FONT_STACK` (lines 34-34)
- type `LogoSplitRevealProps` (lines 36-51)
- const `defaultLogoSplitRevealProps` (lines 53-61)
- const `LogoSplitReveal` (lines 63-186)
- default `default` (lines 188-188)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, Easing, AbsoluteFill.

## Adaptation presets



## Owner notes


