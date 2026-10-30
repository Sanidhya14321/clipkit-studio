---
id: rendercomp-free/bar-chart-anim
name: BarChartAnim
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
tags: [bar-chart-anim]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `BarChartAnim` from `src/components/bar-chart-anim/BarChartAnim.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- title: string
- subtitle?: string
- bars: BarItem[]
- background: string
- titleColor: string
- accentColor: string

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 13-13)
- const `DURATION_FRAMES` (lines 14-14)
- const `FONT` (lines 17-17)
- const `SPACE` (lines 20-20)
- type `BarItem` (lines 23-27)
- type `BarChartAnimProps` (lines 29-36)
- const `defaultBarChartAnimProps` (lines 38-50)
- const `BarChartAnim` (lines 52-191)
- default `default` (lines 193-193)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, spring, Easing, AbsoluteFill.

## Adaptation presets



## Owner notes


