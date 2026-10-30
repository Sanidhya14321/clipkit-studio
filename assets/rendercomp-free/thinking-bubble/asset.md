---
id: rendercomp-free/thinking-bubble
name: ThinkingBubble
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
tags: [thinking-bubble]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `ThinkingBubble` from `src/components/thinking-bubble/ThinkingBubble.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- backgroundColor: string
- characterColor: string
- bubbleColor: string
- bubbleBorder: string
- dotColor: string
- thoughtText: string
- textColor: string
- showText: boolean

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 15-15)
- const `DURATION_FRAMES` (lines 16-16)
- type `ThinkingBubbleProps` (lines 18-27)
- const `defaultThinkingBubbleProps` (lines 29-38)
- const `FONT` (lines 40-40)
- const `ThinkingBubble` (lines 42-282)
- default `default` (lines 284-284)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, spring, Easing, AbsoluteFill.

## Adaptation presets



## Owner notes


