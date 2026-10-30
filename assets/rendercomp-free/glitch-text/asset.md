---
id: rendercomp-free/glitch-text
name: GlitchText
source: rendercomp-free
sourceCommit: f648980e528defb7594e24c6447055f0540f340b
kind: remotion-clip
runtime: direct
determinism: native
license: MIT
spdx: MIT
licenseClass: permissive
grade: A
deps: [react, remotion]
cost: 2
aspects: []
tags: [glitch-text]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Glitch text reveal. - Stacks R/G/B layers and offsets each layer with frame-driven randomness - random(seed) gives reproducible random values (Remotion built-in) - Uses only useCurrentFrame() + interpolate() (no CSS transition / keyframes)…

## Use when

- Glitch text reveal

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- text: string
- glitchColor: string
- intensity: number
- backgroundColor: string
- fontSize: number
- duration: number

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 12-12)
- const `DURATION_FRAMES` (lines 13-13)
- const `FONT_EN` (lines 16-16)
- type `GlitchTextProps` (lines 18-31)
- const `defaultGlitchTextProps` (lines 33-40)
- const `GlitchText` (lines 50-228)
- default `default` (lines 230-230)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, AbsoluteFill.

## Adaptation presets



## Owner notes


