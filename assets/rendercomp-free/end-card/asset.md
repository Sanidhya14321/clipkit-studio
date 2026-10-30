---
id: rendercomp-free/end-card
name: EndCard
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
tags: [end-card]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `EndCard` from `src/components/end-card/EndCard.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- title: string
- subtitle: string
- ctaText: string
- background: string
- titleColor: string
- accentColor: string

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 15-15)
- const `DURATION_FRAMES` (lines 16-16)
- const `FONT_EN` (lines 18-18)
- const `EASE_OUT` (lines 21-21)
- const `DUR_NORMAL` (lines 22-22)
- type `CtaButtonProps` (lines 30-36)
- const `CtaButton` (lines 38-132)
- type `EndCardProps` (lines 134-147)
- const `defaultEndCardProps` (lines 149-156)
- const `EndCard` (lines 158-312)
- default `default` (lines 314-314)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, spring, Easing, AbsoluteFill.

## Adaptation presets



## Owner notes


