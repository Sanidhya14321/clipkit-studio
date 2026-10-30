---
id: rendercomp-free/community-chat
name: CommunityChat
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
tags: [community-chat]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `CommunityChat` from `src/components/community-chat/CommunityChat.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- workspaceName: string
- channelName: string
- messages: CommunityChatMessage[]

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 24-24)
- const `DURATION_FRAMES` (lines 25-25)
- const `SANS` (lines 27-27)
- const `C` (lines 30-45)
- type `CommunityChatMessage` (lines 47-54)
- type `CommunityChatProps` (lines 56-60)
- const `Message` (lines 62-155)
- const `WorkspaceIcon` (lines 157-194)
- const `ChatWindow` (lines 196-381)
- const `CommunityChat` (lines 383-398)
- const `defaultCommunityChatProps` (lines 400-438)
- default `default` (lines 440-440)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, useVideoConfig, interpolate, spring, Sequence, AbsoluteFill.

## Adaptation presets



## Owner notes


