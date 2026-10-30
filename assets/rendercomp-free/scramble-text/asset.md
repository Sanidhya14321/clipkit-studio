---
id: rendercomp-free/scramble-text
name: ScrambleText
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
tags: [scramble-text]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `ScrambleText` from `src/components/scramble-text/ScrambleText.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- text: string
- label?: string
- background: string
- textColor: string
- labelColor: string
- scrambleDuration: number
- charset: string

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 11-11)
- const `DURATION_FRAMES` (lines 12-12)
- type `ScrambleTextProps` (lines 14-29)
- const `defaultScrambleTextProps` (lines 31-39)
- const `FONT_DISPLAY` (lines 41-41)
- const `FONT_MONO` (lines 42-42)
- const `EASE_OUT` (lines 45-45)
- const `seededRandom` (lines 48-51)
- function `shiftLightness` (lines 54-60)
- type `ScrambleProps` (lines 64-72)
- const `Scramble` (lines 74-119)
- const `ScrambleText` (lines 121-212)
- default `default` (lines 214-214)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, interpolate, Easing, AbsoluteFill.

## Adaptation presets



## Owner notes


