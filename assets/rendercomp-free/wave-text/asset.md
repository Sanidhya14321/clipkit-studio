---
id: rendercomp-free/wave-text
name: WaveText
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
tags: [wave-text]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `WaveText` from `src/components/wave-text/WaveText.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- text: string
- subText?: string
- background: string
- textColor: string
- subTextColor: string
- amplitude: number
- waveSpeed: number
- wavelength: number

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 12-12)
- const `DURATION_FRAMES` (lines 13-13)
- type `WaveTextProps` (lines 15-32)
- const `defaultWaveTextProps` (lines 34-43)
- const `FONT_FAMILY` (lines 45-45)
- function `shade` (lines 48-64)
- const `WaveWord` (lines 70-133)
- const `WaveText` (lines 135-200)
- default `default` (lines 202-202)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, interpolate, Easing, AbsoluteFill.

## Adaptation presets



## Owner notes


