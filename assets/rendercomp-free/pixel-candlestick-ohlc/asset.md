---
id: rendercomp-free/pixel-candlestick-ohlc
name: PixelCandlestickOhlc
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
tags: [pixel-candlestick-ohlc]
collections: [rendercomp-free]
reviewed: false
---

## What it is

Exported const `PixelCandlestickOhlc` from `src/components/pixel-candlestick-ohlc/PixelCandlestickOhlc.tsx` (no doc comment).

## Use when

- When a Remotion clip with these inputs fits the scene.

## Avoid when

- When the look calls for different motion than its own.

## Inputs

- backgroundColor: string
- gridColor: string
- bullColor: string
- bearColor: string
- accentColor: string
- candleCount: number
- volatility: number
- cycleSpeed: number
- ticker: string

## Safe to adapt

Text, colors and timing through props; deeper changes through derive.

## Structural parts

- const `FPS` (lines 15-15)
- const `DURATION_FRAMES` (lines 16-16)
- const `IW` (lines 18-18)
- const `IH` (lines 19-19)
- const `snap` (lines 20-20)
- const `PLOT_X0` (lines 23-23)
- const `PLOT_X1` (lines 24-24)
- const `PLOT_Y0` (lines 25-25)
- const `PLOT_Y1` (lines 26-26)
- const `PLOT_W` (lines 27-27)
- const `PLOT_H` (lines 28-28)
- const `hexToRgb` (lines 30-33)
- type `PixelCandlestickOhlcProps` (lines 35-45)
- const `defaultPixelCandlestickOhlcProps` (lines 47-59)
- type `Ohlc` (lines 61-61)
- function `buildSeries` (lines 64-88)
- const `seedFrac` (lines 90-90)
- const `seedSigned` (lines 92-92)
- const `priceToY` (lines 95-95)
- const `PixelCandlestickOhlc` (lines 97-245)
- default `default` (lines 247-247)

## Cannot do

Not reviewed by a person yet; nothing beyond its declared inputs.

## Pairs well with



## Taste notes

Animates with useCurrentFrame, AbsoluteFill.

## Adaptation presets



## Owner notes


