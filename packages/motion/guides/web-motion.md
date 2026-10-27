# Expressive website mechanisms, authored as a film

Motion Primitives source is pinned at 120f64f6ca60348e251f929e9c81f11ccbe45eda (MIT). Selected components are adapted to frame time. React Bits and Codrops are visual references, not a copied component pack. Avoid adding browser Motion/GSAP autoplay to a video scene.

Import these from `@clipkit/motion`:

- `TextEffect({text,frame,start=0,duration=20,stagger=2,per='word',preset='blur-slide',style})`. `per`: word/char/line; preset: blur/blur-slide/slide/scale. Frame and duration are local scene frames. Put a stable reading hold after the stagger settles.
- `TextMorph({from,to,progress,style})`: progress 0–1. Shared glyphs travel between positions; outgoing/incoming glyphs dissolve. Use a monospace or near-monospace face for predictable spacing. It is a transition between related ideas, not constant scrambling.
- `AnimatedGroup({children,frame,start,duration,stagger,preset,style})`: preset flip/blur-slide/scale; deterministic perspective reveals. Use it for a related cluster, not a row of feature cards.
- `ScrollTunnel({children,progress,depth=900,roll=0})`: children are explicitly positioned 3D planes. Progress replaces scroll; land on a readable subject instead of flying forever.
- `GradientBlinds({progress,colors,stripes=12})`: frame-driven strip handoff. Connect outgoing/incoming palette and stage placement; don't leave it cycling behind unrelated text.
- `TrailType({text,progress,color,copies=5,style})`: offset outline trails catch up to a solid reading layer. Use briefly at entry/exit; settle the hero phrase.
- `webProgress(frame,start,duration)`: clamped linear 0–1. Use your own easing when a physical motion needs anticipation or overshoot.

Existing DepthStage, Plane, Wipe, SplitType and MaterialReveal remain composable. No network/RAF/timer imports. Scene source receives project, item and declared properties. Frame time is scene-local; the global shared score uses absolute timeline frames.

Contemporary mechanisms observed in current React Bits/Motion/Codrops material: coordinated stagger, shared-element morphs, outlined typography trails, controlled blur-to-sharp landing, dimensional scroll travel and shader/material reveals. These are selected visual mechanisms, not a claim about measured audience popularity.

A good shot has one dominant subject and one change. Carry a shape, word or motion vector across the cut. Contrast fast entries with quiet reading. Move supporting layers more aggressively than captions or evidence. Avoid repeating one headline box or circular backdrop across every scene.
