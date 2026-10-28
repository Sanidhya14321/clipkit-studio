# Motion Principles And Easing

Animate a change in meaning, position or focus.
Use deterministic frame timing and seeded randomness.
Example: 8-frame ease-out entrance, 30-frame reading hold.
Limit simultaneous animation to 12 layers and 2 heavy effects.
Use ease-out for arrivals; ease-in-out for camera/shape handoffs; bounded spring only for playful accents.
Stagger by about 0.06 seconds; keep a 1.2-second default reading hold.
Use at most two type sizes per scene and maintain one consistent movement direction.
Read numeric pacing values from pacing.yaml; do not infer speech speed from decorative motion.
