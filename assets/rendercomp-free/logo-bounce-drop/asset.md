---
id: rendercomp-free/logo-bounce-drop
name: LogoBounceDrop
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
tags: [logo-bounce-drop]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Logo drops in from above and bounces on landing. - Remotion spring() gives a physics-like bounce (damping controls rebound) - svgPath present => SVG mode, otherwise text mode - Pure useCurrentFrame() + spring + interpolate (no CSS transiti…

## Use when

- Logo drops in from above and bounces on landing

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- logoText: string
- svgPath: string
- viewBox: string
- dropHeight: number
- bounceStrength: number
- logoColor: string
- backgroundColor: string
- fontSize: number
- duration: number

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 12-12)
- const `DURATION_FRAMES` (lines 13-13)
- const `FONT` (lines 15-15)
- type `LogoBounceDropProps` (lines 17-36)
- const `defaultLogoBounceDropProps` (lines 38-48)
- const `LogoBounceDrop` (lines 58-166)
- default `default` (lines 168-168)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, spring, AbsoluteFill.

## Adaptation presets



## Owner notes


