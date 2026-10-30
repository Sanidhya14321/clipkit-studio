---
id: rendercomp-free/chapter-title
name: ChapterTitle
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
tags: [chapter-title]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `ChapterTitle` from `src/components/chapter-title/ChapterTitle.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- chapterLabel: string
- title: string
- subtitle?: string
- background: string
- chapterColor: string
- titleColor: string

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 17-17)
- const `DURATION_FRAMES` (lines 18-18)
- const `FONT_FAMILY` (lines 20-20)
- type `ChapterTitleProps` (lines 22-35)
- const `defaultChapterTitleProps` (lines 37-44)
- const `FloatFadeIn` (lines 49-71)
- const `TitleReveal` (lines 74-136)
- const `ChapterTitle` (lines 138-237)
- default `default` (lines 239-239)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, Easing, AbsoluteFill.

## Adaptation presets



## Owner notes


