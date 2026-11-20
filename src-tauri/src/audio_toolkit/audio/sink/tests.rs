use super::{AudioFrameCallback, CaptureSink, CaptureVad, LevelCallback, VadPolicy};
use crate::audio_toolkit::vad::{VadFrame, VoiceActivityDetector};
use handy_recorder::{AudioChunk, Sink};
use std::sync::{mpsc, Arc, Mutex};

/// Keeps frames whose first sample is non-zero, and records how the sink
/// configured it.
#[derive(Default)]
struct ScriptedVad {
    frame_samples: usize,
    hangover: Arc<Mutex<Option<usize>>>,
    resets: Arc<Mutex<usize>>,
}

impl VoiceActivityDetector for ScriptedVad {
    fn push_frame<'a>(&'a mut self, frame: &'a [f32]) -> anyhow::Result<VadFrame<'a>> {
        Ok(if frame[0] != 0.0 {
            VadFrame::Speech(frame)
        } else {
            VadFrame::Noise
        })
    }

    fn frame_samples(&self) -> usize {
        self.frame_samples
    }

    fn set_hangover_frames(&mut self, frames: usize) {
        *self.hangover.lock().unwrap() = Some(frames);
    }

    fn reset(&mut self) {
        *self.resets.lock().unwrap() += 1;
    }
}

const FRAME: usize = 4;

struct Harness {
    sink: CaptureSink,
    ready: mpsc::Receiver<()>,
    fed: Arc<Mutex<Vec<f32>>>,
    hangover: Arc<Mutex<Option<usize>>>,
    resets: Arc<Mutex<usize>>,
}

fn harness(policy: VadPolicy) -> Harness {
    let vad = ScriptedVad {
        frame_samples: FRAME,
        ..Default::default()
    };
    let hangover = Arc::clone(&vad.hangover);
    let resets = Arc::clone(&vad.resets);
    let fed = Arc::new(Mutex::new(Vec::new()));
    let on_audio: AudioFrameCallback = {
        let fed = Arc::clone(&fed);
        Arc::new(move |frame: &[f32]| fed.lock().unwrap().extend_from_slice(frame))
    };
    let on_levels: LevelCallback = Arc::new(|_| {});
    let (ready_tx, ready) = mpsc::channel();
    let sink = CaptureSink::new(
        CaptureVad::new(Box::new(vad), 10, 20),
        policy,
        on_levels,
        on_audio,
        ready_tx,
    );
    Harness {
        sink,
        ready,
        fed,
        hangover,
        resets,
    }
}

fn chunk(samples: &[f32], valid_frames: usize) -> AudioChunk<'_> {
    AudioChunk {
        samples,
        sample_rate: 16_000,
        channels: 1,
        valid_frames,
    }
}

#[test]
fn readiness_fires_once_on_the_first_chunk() {
    let mut h = harness(VadPolicy::Disabled);
    assert!(h.ready.try_recv().is_err());

    h.sink.process_chunk(chunk(&[0.0; FRAME], FRAME));
    h.sink.process_chunk(chunk(&[0.0; FRAME], FRAME));

    assert!(h.ready.try_recv().is_ok());
    // The sender is gone after the first chunk, so a second signal can't come.
    assert_eq!(h.ready.try_recv(), Err(mpsc::TryRecvError::Disconnected));
}

#[test]
fn readiness_waiter_wakes_when_a_recording_ends_without_audio() {
    let h = harness(VadPolicy::Offline);
    let _ = h.sink.finish();
    assert!(h.ready.recv().is_err());
}

#[test]
fn disabled_vad_keeps_every_real_sample_and_drops_padding() {
    let mut h = harness(VadPolicy::Disabled);
    h.sink.process_chunk(chunk(&[1.0, 0.0, 2.0, 0.0], FRAME));
    // Final chunk: one real frame, three of zero padding.
    h.sink.process_chunk(chunk(&[3.0, 0.0, 0.0, 0.0], 1));

    let expected = vec![1.0, 0.0, 2.0, 0.0, 3.0];
    assert_eq!(*h.fed.lock().unwrap(), expected);
    assert_eq!(h.sink.finish().samples, expected);
    assert_eq!(
        *h.resets.lock().unwrap(),
        0,
        "a disabled policy leaves VAD alone"
    );
}

#[test]
fn a_recording_of_only_zeros_is_silent_input() {
    let mut h = harness(VadPolicy::Offline);
    h.sink.process_chunk(chunk(&[0.0; FRAME], FRAME));
    h.sink.process_chunk(chunk(&[0.0; FRAME], FRAME));
    assert!(h.sink.finish().input_silent);
}

#[test]
fn a_recording_stopped_before_any_audio_is_not_silent_input() {
    // A quick tap that ends before the device delivers its first chunk says
    // nothing about the microphone.
    let h = harness(VadPolicy::Offline);
    assert!(!h.sink.finish().input_silent);
}

#[test]
fn sound_counts_even_when_vad_drops_it() {
    let mut h = harness(VadPolicy::Offline);
    // Noise to the detector (its first sample is zero), but not silence.
    h.sink.process_chunk(chunk(&[0.0, 0.01, 0.0, 0.0], FRAME));
    let captured = h.sink.finish();
    assert!(captured.samples.is_empty());
    assert!(!captured.input_silent);
}

#[test]
fn vad_keeps_speech_frames_only() {
    let mut h = harness(VadPolicy::Offline);
    h.sink.process_chunk(chunk(&[1.0; FRAME], FRAME));
    h.sink.process_chunk(chunk(&[0.0; FRAME], FRAME));
    h.sink.process_chunk(chunk(&[2.0; FRAME], FRAME));

    let expected = [[1.0; FRAME], [2.0; FRAME]].concat();
    assert_eq!(*h.fed.lock().unwrap(), expected);
    assert_eq!(h.sink.finish().samples, expected);
}

#[test]
fn each_recording_resets_the_detector_with_its_policys_hangover() {
    let offline = harness(VadPolicy::Offline);
    assert_eq!(*offline.hangover.lock().unwrap(), Some(10));
    assert_eq!(*offline.resets.lock().unwrap(), 1);

    let streaming = harness(VadPolicy::Streaming);
    assert_eq!(*streaming.hangover.lock().unwrap(), Some(20));
    assert_eq!(*streaming.resets.lock().unwrap(), 1);
}

#[test]
fn finish_returns_the_detector_for_the_next_recording() {
    let h = harness(VadPolicy::Offline);
    let captured = h.sink.finish();
    assert_eq!(captured.vad.frame_samples(), FRAME);
    assert_eq!(captured.vad.hangover_for(VadPolicy::Streaming), 20);
}

#[test]
fn checkpoint_failure_stops_audio_forwarding_and_preserves_partial_wav() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("capture.wav");
    let errors = Arc::new(Mutex::new(Vec::new()));
    let observed = errors.clone();
    let mut h = harness(VadPolicy::Disabled);
    h.sink = h
        .sink
        .with_durable_wav(&path)
        .unwrap()
        .on_write_error(Arc::new(move |error| {
            observed.lock().unwrap().push(error);
        }));
    h.sink.process_chunk(chunk(&[0.5; FRAME], FRAME));
    h.sink.fail_next_checkpoint = true;
    h.sink.process_chunk(chunk(&[0.4; FRAME], FRAME));
    h.sink.process_chunk(chunk(&[0.3; FRAME], FRAME));
    assert_eq!(errors.lock().unwrap().len(), 1);
    assert_eq!(h.fed.lock().unwrap().len(), FRAME);
    assert!(h.sink.finish().write_failed);
    assert!(path.exists());
    let (db, _) =
        crate::storage::migrations::open_and_migrate(dir.path().join("history.db"), "test")
            .unwrap();
    let capture_id = crate::delivery::pipeline::complete_failed_prepared_dictation(
        &db,
        "Failed recording",
        Some("capture.wav"),
        None,
        None,
        "checkpoint failed",
        None,
        None,
    )
    .unwrap();
    let report = crate::storage::recovery::reconcile_startup(&db, dir.path()).unwrap();
    assert!(report.adopted_orphan_recordings.is_empty());
    let attempts =
        crate::storage::repositories::transcriptions::attempts_for_capture(&db, &capture_id)
            .unwrap();
    assert_eq!(attempts[0].status, "failed");
    assert!(hound::WavReader::open(path).unwrap().duration() >= FRAME as u32);
}

#[test]
fn checkpoint_failure_preserves_detail_even_when_the_input_is_silent() {
    let dir = tempfile::tempdir().unwrap();
    let mut h = harness(VadPolicy::Offline);
    h.sink = h
        .sink
        .with_durable_wav(&dir.path().join("failed.wav"))
        .unwrap();
    h.sink.fail_next_checkpoint = true;
    h.sink.process_chunk(chunk(&[0.0; FRAME], FRAME));
    let captured = h.sink.finish();
    assert!(captured.input_silent);
    assert!(captured.write_failed);
    assert!(captured
        .write_error
        .as_deref()
        .unwrap()
        .contains("injected disk write failure"));
}
