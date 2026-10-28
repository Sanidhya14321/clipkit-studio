---
id: web-liquid-metal
name: Liquid Metal
source: paper
kind: react-component
runtime: derive-only
license: Apache-2.0 — Paper Design
deps: []
cost: high
duration: context-dependent
tags:
  - website
  - motion
  - paper design
  - animation
  - video
reviewed: false
category: website
summary: Actual Paper Design source adapted to seekable video frame time
useWhen: A short material accent, logo treatment or texture reveal supporting
  the thesis.
sourceHash: f00a9a31ded1f256c0f8fcbe1bf4f88abca1f5e30269281d412e76617c82e40a
files:
  - assets/adapters/web.tsx
sourceBase: .
clipId: web-liquid-metal
repository: https://github.com/paper-design/shaders
revision: npm:@paper-design/shaders@0.0.82
---
What it is: Actual Paper Design source adapted to seekable video frame time
Use when: A short material accent, logo treatment or texture reveal supporting the thesis.
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
