//! AUDIO-104 acceptance: durable capture lifecycle across crashes.

use handy_app_lib::storage::audio_files::{scan_staged, stage_new, verify_finalized};

fn tmp_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("handy-audio-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn minimal_wav() -> Vec<u8> {
    // 44-byte canonical PCM header + a few samples.
    let mut wav = Vec::new();
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&36u32.to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&1u16.to_le_bytes()); // mono
    wav.extend_from_slice(&16000u32.to_le_bytes()); // sample rate
    wav.extend_from_slice(&32000u32.to_le_bytes()); // byte rate
    wav.extend_from_slice(&2u16.to_le_bytes()); // block align
    wav.extend_from_slice(&16u16.to_le_bytes()); // bits
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&8u32.to_le_bytes());
    wav.extend_from_slice(&[0u8, 0, 0, 0, 0, 0, 0, 0]);
    wav
}

#[test]
fn finalize_then_verify_roundtrip() {
    let dir = tmp_dir("roundtrip");
    let staged = stage_new(&dir).unwrap();
    let payload = minimal_wav();
    staged.append(&payload).unwrap();

    let finalized = staged.finalize(&dir).expect("valid wav finalizes");
    assert!(finalized.final_path.exists(), "final file exists");
    assert!(
        !dir.join(".staging").join("").exists() || scan_staged(&dir).is_empty(),
        "temp file must be gone after atomic rename"
    );
    assert_eq!(finalized.size_bytes, payload.len() as u64);
    assert!(finalized.file_name.starts_with("rec-"));
    assert!(finalized.file_name.ends_with(".wav"));

    assert!(
        verify_finalized(&finalized.final_path, &finalized.sha256).unwrap(),
        "hash must verify against stored value"
    );
}

#[test]
fn corrupt_or_tiny_payloads_are_rejected_and_kept_for_recovery() {
    let dir = tmp_dir("corrupt");

    // Not a WAV.
    let staged = stage_new(&dir).unwrap();
    staged.append(b"garbage bytes").unwrap();
    let temp_path = staged.temp_path.clone();
    let err = staged.finalize(&dir).expect_err("non-wav must fail");
    assert!(err.to_string().contains("RIFF"));
    assert!(
        temp_path.exists(),
        "failed temp stays for recovery inspection"
    );

    // Truncated/too-small WAV.
    let staged2 = stage_new(&dir).unwrap();
    staged2.append(b"RIFF----WAVE").unwrap();
    let temp2 = staged2.temp_path.clone();
    assert!(staged2.finalize(&dir).is_err());
    assert!(temp2.exists());

    // Recovery scan surfaces BOTH candidates without deleting anything.
    let mut orphans = scan_staged(&dir);
    orphans.sort();
    assert_eq!(orphans.len(), 2, "both unfinalized temps must be visible");
}

#[test]
fn staging_is_isolated_from_finalized_recordings() {
    let dir = tmp_dir("isolation");
    let staged = stage_new(&dir).unwrap();
    staged.append(&minimal_wav()).unwrap();
    staged.finalize(&dir).unwrap();

    // Final recordings live at the top level; staging dir stays but empty.
    let files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .collect();
    assert_eq!(files.len(), 1, "exactly one finalized recording");
    assert!(files[0].extension().unwrap() == "wav");
    assert!(
        scan_staged(&dir).is_empty(),
        "no unfinalized temps remain after finalize"
    );
}
