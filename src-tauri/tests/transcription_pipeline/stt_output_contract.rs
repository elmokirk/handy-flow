//! STT-103 public contract: the deterministic pipeline version and the
//! dual-output transcription surface stay stable for persistence layers.

use handy_app_lib::{TranscriptionOutput, NORMALIZER_VERSION};

#[test]
fn normalizer_version_is_public_and_pinned() {
    assert_eq!(NORMALIZER_VERSION, "1");
}

#[test]
fn transcription_output_is_cloneable_and_debuggable() {
    let out = TranscriptionOutput {
        engine_raw: "raw engine text".to_string(),
        normalized_stt: "normalized text".to_string(),
        model_id: None,
        language: None,
        normalizer_version: NORMALIZER_VERSION.to_string(),
    };

    // Persistence layers may pass the struct across threads/tasks.
    let cloned = out.clone();
    assert_eq!(cloned.engine_raw, "raw engine text");
    assert_eq!(cloned.normalized_stt, "normalized text");

    // Debug formatting must not panic (used in structured logging).
    let _ = format!("{out:?}");
}
