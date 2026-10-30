---
id: rendercomp-free/pourover-drip-fill-gauge
name: PouroverDripFillGauge
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
tags: [pourover-drip-fill-gauge]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `PouroverDripFillGauge` from `src/components/pourover-drip-fill-gauge/PouroverDripFillGauge.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- toneBg: string
- toneDark: string
- toneMid: string
- toneLight: string
- fillSpeed: number
- showDrips: boolean
- beats?: BeatEvent[]
- waveform?: number[]

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 16-16)
- const `DURATION_FRAMES` (lines 17-17)
- const `IW` (lines 19-19)
- const `IH` (lines 20-20)
- const `snap` (lines 21-21)
- const `seed` (lines 22-25)
- type `PouroverDripFillGaugeProps` (lines 27-36)
- const `defaultPouroverDripFillGaugeProps` (lines 38-45)
- const `hexToRgb` (lines 47-50)
- function `quantize4` (lines 53-60)
- const `PouroverDripFillGauge` (lines 62-218)
- function `boostLight` (lines 221-223)
- default `default` (lines 225-225)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, AbsoluteFill.

## Adaptation presets



## Owner notes


