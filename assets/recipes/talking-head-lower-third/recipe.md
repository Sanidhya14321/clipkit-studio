---
{
  "id": "talking-head-lower-third",
  "name": "Talking head lower third",
  "purpose": "Identify a speaker while preserving the footage base.",
  "license": "MIT",
  "duration": [
    1,
    12
  ],
  "aspects": [
    "9:16",
    "1:1",
    "16:9"
  ],
  "slots": {
    "speaker": {
      "type": "text",
      "maxChars": 60
    },
    "role": {
      "type": "text",
      "maxChars": 100
    }
  },
  "assets": [],
  "briefDeps": [
    "goal",
    "audience",
    "aspect",
    "look",
    "brand",
    "pacing",
    "tone",
    "mustInclude"
  ],
  "layout": "Safe-area grid, noncolliding boxes, two type sizes; native authored primitives."
}
---
Original ClipKit recipe under the project MIT license.
Uses native authored typography and shape primitives; no third-party asset is claimed.
Fill factual slots from supplied evidence. Do not invent statistics or endorsements.
Cannot do: synthesize product proof, clone a voice, or fit unlimited copy.
