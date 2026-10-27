# Film direction rails — instruction first

ClipKit is the sole timeline/renderer. Begin from the viewer's experience, not a component gallery. Actual React/TSX components and original React/CSS/SVG are equally valid implementations. There is no component quota and no compulsory visual reference. Do not add a role or invocation merely to check a box.

## One preparation turn

1. Extract audience, purpose, duration/fps/aspect, exact versus editable copy, footage, measured speech, evidence and brand. Resolve conflicts explicitly; do not truncate dialogue or invent product proof.
2. State one visual thesis in a sentence: what does the viewer see changing, and why does it express the idea? Choose hierarchy, palette, type family and contrast. Limit simultaneous focal actions; this is a composition decision, not an effect count. Check subject occupancy and negative space at phone size: empty space must direct the eye, not expose an undersized centered poster. A camera zoom ending on the same static stack does not by itself demonstrate a new visual idea. The proof should show the defining transformation or a genuine handoff.
3. Write narrative beats independently from shots and copy. One sentence may persist across two cuts. Give essential copy an unobstructed reading interval. Fast pacing comes from contrast between travel/accents and stable reads, not constant shaking.
4. For each shot define 2–4 useful key poses: frame, subject, position, scale and camera orientation. Specify entry → anticipation → main movement → overshoot/settle → readable hold → exit as needed. Use integral end-exclusive ranges, not a list of every frame. For a 30fps three-second proof, explicitly plan the first image, strongest motion, readable pose and boundary at frame 90. Choose timing for this brief rather than applying these stages as a fixed preset.
5. Define each handoff physically: the outgoing object's edge, velocity, mask, baseline or shape becomes the incoming object's start. Specify coordinate correspondence at the boundary. Avoid a fade/reset between independent posters unless a deliberate cut serves the story.
6. Choose the implementation LAST. Study only the relevant reference mechanism. React Bits Blur Text suggests staggered soft-to-sharp resolution; Scroll Float suggests depth/parallax driven by normalized progress. Motion's scroll-linked pattern maps progress to several properties. Translate user scroll/pointer progress into authored frame progress; preserve mechanism, not website interaction dependencies. Clip Library provides real code when an existing component fits. New choreography may be authored freely. Do not label an original adaptation as copied upstream code.
7. Submit storyboard_set, then scene_draft for one actual three-second proof. Expose meaningful text, palette, camera/path and timing controls. Tag elements with data-ck-layer/item.id, data-ck-path and stable data-ck-element. Imports: React, Remotion, @clipkit/motion, optionally @clipkit/components, and local CSS/SVG. No external requests, wall clocks or unseeded randomness. Keep local animation phase continuous through scoped splits. Poll render completion. Check first/last frames and readable holds; a valid render proves technical behavior only.
8. STOP for the human. Report the exact plan/sample, what was authored versus adopted and measured usage. Release the lease. Do not continue a full film or pronounce aesthetic acceptance.

## After approval or feedback

Resume the exact creative session. Extend the approved composition itself and protect its first 90 frames. A rejected proof gets a bounded new preparation, not a full film. Preserve all attempts and feedback. Final revisions remain proposals until the human keeps them.

Direct exposed-property edits are local validated transactions: no agent call. Choreography changes load only affected source, timing and adjoining boundaries for one invocation. Story redesign returns to the storyboard/proof checkpoint. Long form approves global narrative/style before bounded chapters; never put an hour of source into one prompt.

## Reference mechanisms (not copied component packs)

- React Bits Blur Text: https://reactbits.dev/text-animations/blur-text
- React Bits Scroll Float: https://reactbits.dev/text-animations/scroll-float
- Motion scroll-linked choreography: https://motion.dev/docs/react-scroll-animations
- Actual frame-native Clip Library: https://github.com/meetportal/cliplibrary

Use source within its license. React Bits project application use and component redistribution are different; ClipKit does not redistribute a React Bits pack. Original Video Forge guidance remains available through focused motion_read guides. Do not assume its standalone renderer commands or browser refraction helpers exist here.
