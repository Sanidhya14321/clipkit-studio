---
id: rendercomp-free/four-tone-mono-titler
name: FourToneMonoTitler
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
tags: [four-tone-mono-titler]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `FourToneMonoTitler` from `src/components/four-tone-mono-titler/FourToneMonoTitler.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- title: string
- subtitle: string
- tone0: string
- tone1: string
- tone2: string
- tone3: string
- titleCell: number
- subtitleCell: number
- wipeFrames: number
- shimmerSpeed: number
- ditherDensity: number
- showAmbient: boolean

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 14-14)
- const `DURATION_FRAMES` (lines 15-15)
- const `IW` (lines 17-17)
- const `IH` (lines 18-18)
- const `snap` (lines 19-19)
- const `clamp01` (lines 20-20)
- const `easeInOutCubic` (lines 21-21)
- const `BAYER4` (lines 24-29)
- const `ditherOn` (lines 31-32)
- const `hexToRgb` (lines 34-37)
- const `rgbToHex` (lines 38-39)
- const `PANEL_TOP` (lines 42-42)
- const `PANEL_BOT` (lines 43-43)
- const `SHADE_Y0` (lines 44-44)
- const `SHADE_H` (lines 45-45)
- const `KICKER_X` (lines 46-46)
- const `TITLE_X0` (lines 47-47)
- const `TITLE_Y0` (lines 48-48)
- const `DIVIDER_Y` (lines 49-49)
- const `SUB_X0` (lines 50-50)
- const `SUB_Y0` (lines 51-51)
- const `DUST_COUNT` (lines 52-52)
- type `FourToneMonoTitlerProps` (lines 54-67)
- const `HANDHELD_4TONE` (lines 72-74)
- const `defaultFourToneMonoTitlerProps` (lines 77-91)
- const `FourToneMonoTitler` (lines 93-254)
- default `default` (lines 256-256)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame.

## Adaptation presets



## Owner notes


