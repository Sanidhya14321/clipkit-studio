---
id: rendercomp-free/logo-blur-reveal
name: LogoBlurReveal
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
tags: [logo-blur-reveal]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `LogoBlurReveal` from `src/components/logo-blur-reveal/LogoBlurReveal.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- logoText: string
- blurAmount: number
- revealDuration: number
- logoColor: string
- backgroundColor: string
- fontSize: number
- duration: number

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 17-17)
- const `DURATION_FRAMES` (lines 18-18)
- const `FONT_FAMILY` (lines 20-20)
- type `LogoBlurRevealProps` (lines 22-37)
- const `defaultLogoBlurRevealProps` (lines 39-47)
- const `LogoBlurReveal` (lines 49-114)
- default `default` (lines 116-116)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, Easing, AbsoluteFill.

## Adaptation presets



## Owner notes


