#!/usr/bin/env bash
# Generate synthetic fixture media with known ground truth (spec 10.2).
# Usage: fixtures/generate.sh [OUT_DIR]   (default: fixtures/generated, which is gitignored)
#
# Every ffmpeg call uses an argument array, never a shell string.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT="${1:-$ROOT/fixtures/generated}"
FFMPEG="${FFMPEG:-ffmpeg}"
FFPROBE="${FFPROBE:-ffprobe}"

for bin in "$FFMPEG" "$FFPROBE"; do
  command -v "$bin" >/dev/null || { echo "fixtures: $bin not found (install ffmpeg)" >&2; exit 1; }
done

FPS=30
W=1280
H=720
DUR=12      # seconds, counter and av clips
SCENE_LEN=2 # seconds per solid-colour scene
SCENE_COLORS=(red green blue yellow magenta)
RATE=48000

mkdir -p "$OUT/images" "$OUT/brand"
ff() { "$FFMPEG" -hide_banner -loglevel error -nostdin -y "$@"; }
BITEXACT=(-map_metadata -1 -fflags +bitexact -flags:v +bitexact -flags:a +bitexact)
X264=(-c:v libx264 -preset veryfast -crf 23 -pix_fmt yuv420p -g "$FPS" -threads 1)

# 1. Frame counter and timecode video (overlay needs ffmpeg built with drawtext).
counter_filter="null"
overlay=false
# Capture first: with pipefail, `ffmpeg | grep -q` fails when grep exits early (SIGPIPE).
filters="$("$FFMPEG" -hide_banner -filters 2>/dev/null || true)"
if grep -q ' drawtext ' <<<"$filters"; then
  counter_filter="drawtext=text='frame %{frame_num}':x=40:y=40:fontsize=48:fontcolor=white:box=1:boxcolor=black,"
  counter_filter+="drawtext=timecode='00\:00\:00\:00':rate=$FPS:x=40:y=110:fontsize=48:fontcolor=white:box=1:boxcolor=black"
  overlay=true
fi
ff -f lavfi -i "testsrc2=size=${W}x${H}:rate=$FPS:duration=$DUR" \
  -vf "$counter_filter" "${X264[@]}" "${BITEXACT[@]}" "$OUT/counter.mp4"

# 2. Tone bursts (440 Hz) separated by silences at known times.
BURSTS=("1 2" "3 4.5" "6.5 7" "9 11")
SILENCES=("0 1" "2 3" "4.5 6.5" "7 9" "11 12")
gate=""
for b in "${BURSTS[@]}"; do
  read -r s e <<<"$b"
  gate+="${gate:++}between(t,$s,$e)"
done
ff -f lavfi -i "aevalsrc='0.5*sin(2*PI*440*t)*($gate)':s=$RATE:d=$DUR" \
  -ac 1 -c:a pcm_s16le "${BITEXACT[@]}" "$OUT/tones.wav"

# 3. Audio and video together, for sync checks.
ff -i "$OUT/counter.mp4" -i "$OUT/tones.wav" -map 0:v -map 1:a -c:v copy -c:a aac -b:a 128k \
  -shortest "${BITEXACT[@]}" "$OUT/av.mp4"

# 4. Hard scene changes every SCENE_LEN seconds.
scene_inputs=()
labels=""
for i in "${!SCENE_COLORS[@]}"; do
  scene_inputs+=(-f lavfi -i "color=c=${SCENE_COLORS[$i]}:s=${W}x${H}:r=$FPS:d=$SCENE_LEN")
  labels+="[$i:v]"
done
ff "${scene_inputs[@]}" -filter_complex "${labels}concat=n=${#SCENE_COLORS[@]}:v=1:a=0[v]" \
  -map "[v]" "${X264[@]}" "${BITEXACT[@]}" "$OUT/scenes.mp4"

# 5. Image set in three aspects.
declare -A IMAGES=([landscape]=1920x1080 [portrait]=1080x1920 [square]=1080x1080)
for name in "${!IMAGES[@]}"; do
  ff -f lavfi -i "testsrc2=size=${IMAGES[$name]}:rate=1:duration=1" -frames:v 1 \
    "${BITEXACT[@]}" "$OUT/images/$name.png"
done

# 6. Brand kit: logo plus tokens. Fonts are PLACEHOLDER until the owner supplies a real kit.
ff -f lavfi -i "color=c=0x6C5CE7:s=512x512:d=1" -frames:v 1 "${BITEXACT[@]}" "$OUT/brand/logo.png"
cat >"$OUT/brand/brand.json" <<'JSON'
{
  "name": "Fixture Brand",
  "colors": { "primary": "#6C5CE7", "secondary": "#00B894", "background": "#0F0F14", "text": "#F5F6FA" },
  "fonts": { "heading": "PLACEHOLDER", "body": "PLACEHOLDER" },
  "logo": "logo.png"
}
JSON

# 7. Fake word-level transcript for av.mp4 with known fillers and one retake.
cat >"$OUT/transcript.json" <<'JSON'
{
  "source": "av.mp4",
  "language": "en",
  "words": [
    { "i": 0,  "text": "Welcome", "start": 1.00, "end": 1.40 },
    { "i": 1,  "text": "um",      "start": 1.45, "end": 1.70 },
    { "i": 2,  "text": "to",      "start": 1.75, "end": 1.85 },
    { "i": 3,  "text": "ClipKit", "start": 1.85, "end": 2.00 },
    { "i": 4,  "text": "today",   "start": 3.00, "end": 3.30 },
    { "i": 5,  "text": "we",      "start": 3.30, "end": 3.45 },
    { "i": 6,  "text": "will",    "start": 3.45, "end": 3.60 },
    { "i": 7,  "text": "show",    "start": 3.60, "end": 3.90 },
    { "i": 8,  "text": "uh",      "start": 3.95, "end": 4.20 },
    { "i": 9,  "text": "today",   "start": 6.50, "end": 6.70 },
    { "i": 10, "text": "we'll",   "start": 6.70, "end": 6.85 },
    { "i": 11, "text": "show",    "start": 6.85, "end": 7.00 },
    { "i": 12, "text": "you",     "start": 9.00, "end": 9.30 },
    { "i": 13, "text": "like",    "start": 9.35, "end": 9.60 },
    { "i": 14, "text": "the",     "start": 9.65, "end": 9.80 },
    { "i": 15, "text": "editor",  "start": 9.80, "end": 10.40 },
    { "i": 16, "text": "live",    "start": 10.40, "end": 11.00 }
  ]
}
JSON

# 8. Ground truth.
json_ranges() { local out="" r s e; for r in "$@"; do read -r s e <<<"$r"; out+="${out:+, }[$s, $e]"; done; printf '[%s]' "$out"; }
scene_changes=""
for ((i = 1; i < ${#SCENE_COLORS[@]}; i++)); do scene_changes+="${scene_changes:+, }$((i * SCENE_LEN))"; done
cat >"$OUT/ground-truth.json" <<JSON
{
  "version": 1,
  "fps": $FPS,
  "counter": { "file": "counter.mp4", "width": $W, "height": $H, "durationSec": $DUR, "frames": $((DUR * FPS)), "overlay": $overlay },
  "tones": { "file": "tones.wav", "sampleRate": $RATE, "durationSec": $DUR, "freqHz": 440, "bursts": $(json_ranges "${BURSTS[@]}"), "silences": $(json_ranges "${SILENCES[@]}") },
  "av": { "file": "av.mp4", "durationSec": $DUR, "audioMatches": "tones" },
  "scenes": { "file": "scenes.mp4", "durationSec": $((SCENE_LEN * ${#SCENE_COLORS[@]})), "changesAtSec": [$scene_changes] },
  "images": { "landscape": "images/landscape.png", "portrait": "images/portrait.png", "square": "images/square.png" },
  "brand": "brand/brand.json",
  "transcript": { "file": "transcript.json", "fillerWordIndices": [1, 8, 13], "retakes": [{ "abandoned": [4, 8], "kept": [9, 11] }] }
}
JSON

# Verify durations with ffprobe so a broken ffmpeg build fails loudly.
check_dur() {
  local file="$1" want="$2" got
  got="$("$FFPROBE" -v error -show_entries format=duration -of csv=p=0 "$OUT/$file")"
  awk -v g="$got" -v w="$want" 'BEGIN { d = g - w; if (d < 0) d = -d; exit !(d < 0.1) }' \
    || { echo "fixtures: $file duration $got, expected $want" >&2; exit 1; }
}
check_dur counter.mp4 "$DUR"
check_dur tones.wav "$DUR"
check_dur av.mp4 "$DUR"
check_dur scenes.mp4 "$((SCENE_LEN * ${#SCENE_COLORS[@]}))"

echo "fixtures: wrote $(find "$OUT" -type f | wc -l) files to $OUT (counter overlay: $overlay)"
