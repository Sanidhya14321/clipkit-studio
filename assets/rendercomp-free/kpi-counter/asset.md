---
id: rendercomp-free/kpi-counter
name: KpiCounter
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
tags: [kpi-counter]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `KpiCounter` from `src/components/kpi-counter/KpiCounter.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- backgroundColor: string
- cardColor: string
- textColor: string
- subtextColor: string
- accentColor: string
- title: string
- subtitle: string
- metrics: KpiMetric[]

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 17-17)
- const `DURATION_FRAMES` (lines 18-18)
- type `KpiMetric` (lines 20-28)
- type `KpiCounterProps` (lines 30-39)
- const `defaultKpiCounterProps` (lines 41-55)
- const `FONT` (lines 57-57)
- type `NumberFormat` (lines 67-74)
- function `formatFor` (lines 76-84)
- const `DIGIT_CELL_RATIO` (lines 87-87)
- const `DIGIT_WIDTH_RATIO` (lines 88-88)
- const `DRUM_MASK` (lines 89-90)
- const `DigitDrum` (lines 96-143)
- function `wheelPos` (lines 151-157)
- const `RollingNumber` (lines 159-222)
- const `MetricCard` (lines 224-378)
- const `KpiCounter` (lines 380-508)
- default `default` (lines 510-510)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, spring, Easing, AbsoluteFill.

## Adaptation presets



## Owner notes


