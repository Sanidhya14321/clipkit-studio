---
id: cliplib-scene-transition
name: Scene Transition
source: cliplibrary
kind: react-component
runtime: derive-only
license: MIT — Clip Library / clawwde
deps: []
cost: medium
duration: context-dependent
tags:
  - wipe
  - cover
  - minimal
  - animation
  - video
reviewed: false
category: wipe
summary: A solid colour covers the frame and opens again, to bridge two clips on
  the timeline.
useWhen: A motivated overlapping handoff between two already composed shots.
sourceHash: b402a18337e4bcff489a9dff563238f383f4f4b5a711f432e7620c9970d96e07
files:
  - assets/sources/cliplibrary/src/compositions/scenes/transitions/scene-transition.tsx
sourceBase: .
clipId: cliplib-scene-transition
repository: https://github.com/meetportal/cliplibrary/blob/c4585582f712294b82faccd4c9ed233308925f9b/src/compositions/scenes/transitions/scene-transition.tsx
revision: c4585582f712294b82faccd4c9ed233308925f9b
---
What it is: A solid colour covers the frame and opens again, to bridge two clips on the timeline.
Use when: A motivated overlapping handoff between two already composed shots.
Avoid when: the technique distracts from the subject or reading time.
Inputs: inspect the skeleton for declared controls.
Safe to adapt: brand colors, text, scale, frame timing and easing.
Structural (change with care): layout and animation sequencing.
Pairs well with: restrained typography and a quiet base.
Taste notes: choose purpose and reading time before decoration.
Adaptation examples: shorten reveal for a hook; replace palette for brand; remove secondary ornament.
Runtime notes: Retain actual frame-native choreography; derive all interaction/scroll progress from integral project frames. No wall-clock playback.
Rights: Prove the chosen composition in the three-second sample; component inclusion is optional.
<!-- owner-notes -->
