//! Text to speech (spec 8.3): the speaker interface, the Fish Audio stub and the cache key.
//! Generated audio is cached by a hash of what produced it, so a regenerated slot with the same
//! text and voice costs nothing and plays offline.

use std::path::Path;

use ck_core::{CkError, CkResult, ErrorCode};

use crate::transcript::FISH_KEY_ENV;

/// Writes speech for `text` (in voice `voice_ref`, or the default voice) as audio to `out`.
pub trait Speaker: Send + Sync {
    fn speak(&self, text: &str, voice_ref: Option<&str>, out: &Path) -> CkResult<()>;

    /// Make a cloned voice from a consented studio recording; returns its voice ref.
    fn clone_voice(&self, audio: &Path) -> CkResult<String>;
}

/// Fish Audio TTS.
///
/// PLACEHOLDER: the API is unverified (spec 8.3 says verify it before coding), so this refuses
/// with `E_VOICE` even with a key; callers fall back to cached audio, then the studio's labeled
/// browser speech placeholder.
#[derive(Debug, Default)]
pub struct FishTts {
    configured: bool,
}

impl FishTts {
    pub fn from_env() -> Self {
        Self {
            configured: std::env::var_os(FISH_KEY_ENV).is_some_and(|v| !v.is_empty()),
        }
    }
}

impl FishTts {
    fn refuse(&self, what: &str) -> CkError {
        let message = if self.configured {
            format!("Fish Audio {what} is not built yet (API unverified)")
        } else {
            format!("no {what}: {FISH_KEY_ENV} is not set")
        };
        CkError::new(ErrorCode::Voice, message)
    }
}

impl Speaker for FishTts {
    fn speak(&self, _text: &str, _voice_ref: Option<&str>, _out: &Path) -> CkResult<()> {
        Err(self.refuse("TTS"))
    }

    fn clone_voice(&self, _audio: &Path) -> CkResult<String> {
        Err(self.refuse("voice cloning"))
    }
}

/// Cache key for generated speech: SHA-256 over the provider, voice and text.
pub fn cache_key(provider: &str, voice_ref: Option<&str>, text: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    for part in [provider, voice_ref.unwrap_or(""), text] {
        h.update((part.len() as u64).to_le_bytes());
        h.update(part.as_bytes());
    }
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_keys_separate_voice_and_text() {
        let a = cache_key("fish", None, "Hello");
        assert_eq!(a.len(), 64);
        assert_eq!(a, cache_key("fish", None, "Hello"));
        assert_ne!(a, cache_key("fish", Some("v"), "Hello"));
        assert_ne!(
            cache_key("fish", Some("ab"), "c"),
            cache_key("fish", Some("a"), "bc")
        );
    }

    #[test]
    fn the_stub_refuses_with_a_voice_error() {
        let err = FishTts::default()
            .speak("hi", None, Path::new("x.wav"))
            .unwrap_err();
        assert_eq!(err.code, ErrorCode::Voice);
    }
}
