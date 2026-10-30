---
id: rendercomp-free/racing-chart
name: RacingChart
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
tags: [racing-chart]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `RacingChart` from `src/components/racing-chart/RacingChart.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- data: RacingChartDatum[]
- frames: number
- title: string
- background?: string
- titleColor?: string

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 15-15)
- const `DURATION_FRAMES` (lines 16-16)
- const `SPACE` (lines 21-27)
- const `TYPE_SCALE` (lines 29-31)
- const `TEXT_BREAK_STYLES` (lines 34-38)
- const `FONT_STACK` (lines 40-40)
- type `RacingChartDatum` (lines 45-49)
- type `RacingChartProps` (lines 51-62)
- const `defaultRacingChartProps` (lines 64-76)
- type `RacingBarsProps` (lines 81-87)
- const `RacingBars` (lines 89-228)
- const `RacingChart` (lines 233-284)
- default `default` (lines 286-286)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, interpolate, Easing, AbsoluteFill.

## Adaptation presets



## Owner notes


