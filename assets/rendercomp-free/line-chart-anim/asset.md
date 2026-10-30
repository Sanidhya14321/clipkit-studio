---
id: rendercomp-free/line-chart-anim
name: LineChartAnim
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
tags: [line-chart-anim]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `LineChartAnim` from `src/components/line-chart-anim/LineChartAnim.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- title: string
- subtitle?: string
- points: LinePoint[]
- background: string
- titleColor: string
- lineColor: string
- gridColor: string

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 13-13)
- const `DURATION_FRAMES` (lines 14-14)
- const `FONT_STACK` (lines 16-16)
- type `LinePoint` (lines 19-22)
- type `LineChartAnimProps` (lines 25-33)
- const `defaultLineChartAnimProps` (lines 35-50)
- const `LineChartAnim` (lines 52-247)
- default `default` (lines 249-249)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, interpolate, Easing, AbsoluteFill.

## Adaptation presets



## Owner notes


