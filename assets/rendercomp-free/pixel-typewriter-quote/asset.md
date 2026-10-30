---
id: rendercomp-free/pixel-typewriter-quote
name: PixelTypewriterQuote
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
tags: [pixel-typewriter-quote]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `PixelTypewriterQuote` from `src/components/pixel-typewriter-quote/PixelTypewriterQuote.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- quote: string
- attribution: string
- backgroundColor: string
- inkColor: string
- accentColor: string
- cps: number

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 16-16)
- const `DURATION_FRAMES` (lines 17-17)
- const `IW` (lines 19-19)
- const `IH` (lines 20-20)
- const `snap` (lines 21-21)
- type `PixelTypewriterQuoteProps` (lines 23-30)
- const `defaultPixelTypewriterQuoteProps` (lines 32-39)
- const `hexToRgb` (lines 41-44)
- function `wrapWords` (lines 47-62)
- function `drawChar` (lines 65-81)
- const `PixelTypewriterQuote` (lines 83-236)
- default `default` (lines 238-238)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, AbsoluteFill.

## Adaptation presets



## Owner notes


