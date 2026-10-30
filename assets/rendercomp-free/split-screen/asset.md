---
id: rendercomp-free/split-screen
name: SplitScreen
source: rendercomp-free
sourceCommit: f648980e528defb7594e24c6447055f0540f340b
kind: remotion-clip
runtime: direct
determinism: native
license: MIT
spdx: MIT
licenseClass: permissive
grade: B
deps: [react, remotion, react]
cost: 2
aspects: []
tags: [split-screen]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `SplitScreen` from `src/components/split-screen/SplitScreen.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- direction: "horizontal" | "vertical"
- panelA: SplitScreenPanel
- panelB: SplitScreenPanel
- dividerColor: string
- dividerWidth: number
- backdropColor?: string

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 20-20)
- const `DURATION_FRAMES` (lines 21-21)
- const `FONT_LATIN` (lines 24-24)
- const `EASE_OUT` (lines 27-27)
- type `SplitScreenPanel` (lines 31-40)
- type `SplitScreenProps` (lines 42-59)
- const `DEFAULT_BACKDROP` (lines 61-61)
- const `defaultSplitScreenProps` (lines 63-80)
- type `SplitScreenRevealProps` (lines 87-95)
- const `SplitScreenReveal` (lines 97-211)
- const `Panel` (lines 216-285)
- const `SplitScreen` (lines 289-318)
- default `default` (lines 320-320)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, interpolate, Easing, AbsoluteFill.

## Adaptation presets



## Owner notes


