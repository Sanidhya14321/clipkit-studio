use super::fixture::Fixture;
use crate::session::music::MusicAnalyzeRequest;

#[test]
fn music_candidates_are_cached_capped_and_never_modify_project() {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/generated/tones.wav");
    if !path.is_file() {
        eprintln!("skipped: run just fixtures");
        return;
    }
    let f = Fixture::new();
    let before = f.session.project().unwrap();
    let req = MusicAnalyzeRequest {
        path: path.to_string_lossy().into_owned(),
    };
    let first = f.session.music_analyze(&req).unwrap();
    assert!(!first.analysis.confirmed);
    assert!(serde_json::to_vec(&first).unwrap().len() <= crate::ops::caps::MUSIC);
    assert!(first.analysis.onsets.len() <= 16);
    let cache = f
        .session
        .cache_dir()
        .join("music")
        .join(format!("{}-energy-v1.json", first.hash));
    assert!(cache.is_file());
    let second = f.session.music_analyze(&req).unwrap();
    assert_eq!(
        serde_json::to_value(first).unwrap(),
        serde_json::to_value(second).unwrap()
    );
    assert_eq!(before.rev, f.session.project().unwrap().rev);
    assert!(
        f.session
            .music_analyze(&MusicAnalyzeRequest {
                path: "-bad".into()
            })
            .is_err()
    );
}
