//! Does a prompt look like a video request (spec 5.1)? A word list, so an ordinary coding prompt
//! never starts the studio. Ambiguous words (clip, edit, render, timeline, captions) are left out.

const VIDEO_WORDS: &[&str] = &[
    "video",
    "videos",
    "footage",
    "reel",
    "reels",
    "trailer",
    "trailers",
    "montage",
    "voiceover",
    "voiceovers",
    "narration",
    "subtitles",
    "broll",
    "vlog",
    "youtube",
    "tiktok",
    "storyboard",
    "explainer",
    "teaser",
    "remotion",
    "clipkit",
    "mp4",
    "mov",
    "webm",
];

pub fn is_video_prompt(prompt: &str) -> bool {
    prompt
        .split(|c: char| !(c.is_alphanumeric() || c == '-'))
        .map(|w| w.replace('-', "").to_lowercase())
        .any(|w| VIDEO_WORDS.contains(&w.as_str()))
}

#[cfg(test)]
mod tests {
    use super::is_video_prompt;

    #[test]
    fn video_requests_are_detected() {
        for p in [
            "Make a 30 second video about our launch",
            "cut this footage into a reel",
            "add a voice-over and B-roll",
            "Trailer for the game, please",
            "export it as MP4",
            "make a YouTube intro",
        ] {
            assert!(is_video_prompt(p), "{p}");
        }
    }

    #[test]
    fn coding_prompts_are_left_alone() {
        for p in [
            "fix the failing test in parser.rs",
            "edit the README and render the docs",
            "clip the value to 0..1",
            "add captions to the image alt text",
            "videogame physics bug",
            "",
        ] {
            assert!(!is_video_prompt(p), "{p}");
        }
    }
}
