//! Long-form batch decoding: cut audio at pauses and decode chunks in order.

use std::time::Instant;

use log::info;
use transcribe_cpp::Transcript;

use super::{RunSetup, TranscriptionManager};
use crate::audio_toolkit::chunking::Chunker;
use crate::engine_supervisor::{CancelMark, EngineError};
use crate::managers::audio::create_detector;
use crate::settings::VadBackend;

/// Cuts a batch clip at pauses and decodes each chunk once its cut is certain.
pub(super) struct ChunkedDecode<'a> {
    tm: &'a TranscriptionManager,
    /// Every chunk's run stops with one cancel.
    mark: CancelMark,
    setup: RunSetup,
    model_id: String,
    chunker: Chunker,
    transcripts: Vec<Transcript>,
    /// The first run that failed or was cancelled; nothing more is decoded
    /// after it.
    failure: Option<EngineError>,
}

impl<'a> ChunkedDecode<'a> {
    pub(super) fn new(
        tm: &'a TranscriptionManager,
        mark: CancelMark,
        setup: RunSetup,
        model_id: &str,
        backend: VadBackend,
    ) -> Self {
        // The VAD is only loaded if the audio turns out long enough to cut.
        let app_handle = tm.app_handle.clone();
        Self {
            tm,
            mark,
            setup,
            model_id: model_id.to_string(),
            chunker: Chunker::new(Box::new(move || create_detector(backend, &app_handle))),
            transcripts: Vec::new(),
            failure: None,
        }
    }

    /// A whole clip's chunks, decoded in order.
    pub(super) fn decode_clip(
        tm: &'a TranscriptionManager,
        mark: CancelMark,
        setup: RunSetup,
        audio: Vec<f32>,
        model_id: &str,
        backend: VadBackend,
    ) -> Result<Vec<Transcript>, EngineError> {
        let mut decode = Self::new(tm, mark, setup, model_id, backend);
        decode.push(audio);
        decode.finish()
    }

    /// Samples pushed so far.
    pub(super) fn samples(&self) -> usize {
        self.chunker.samples()
    }

    /// Adds audio, decoding each chunk it makes certain.
    pub(super) fn push(&mut self, pcm: Vec<f32>) {
        if self.failure.is_some() {
            return;
        }
        let chunks = self.chunker.push(pcm);
        self.decode(chunks);
    }

    /// Decodes what is left. Every chunk's transcript in order, or the run
    /// that failed or was cancelled.
    pub(super) fn finish(mut self) -> Result<Vec<Transcript>, EngineError> {
        if self.failure.is_none() {
            let chunks = self.chunker.finish();
            self.decode(chunks);
        }
        if let Some(failure) = self.failure {
            return Err(failure);
        }
        if self.transcripts.len() > 1 {
            info!(
                "Decoded {:.1}s of audio in {} chunks",
                self.samples() as f64 / 16_000.0,
                self.transcripts.len()
            );
        }
        Ok(self.transcripts)
    }

    fn decode(&mut self, chunks: Vec<Vec<f32>>) {
        for chunk in chunks {
            if self.failure.is_some() {
                return;
            }
            let run_options = self.setup.run_options.clone();
            let audio_secs = chunk.len() as f64 / 16_000.0;
            let started = Instant::now();
            match self
                .tm
                .run_batch(self.mark, chunk, &run_options, &self.model_id)
            {
                Ok(transcript) => {
                    self.transcripts.push(transcript);
                    // Real-time factor: audio seconds decoded per second of
                    // wall clock. Floored so a near-instant decode reports a
                    // large factor instead of infinity.
                    let elapsed_secs = started.elapsed().as_secs_f64().max(1e-6);
                    info!(
                        "Decoded long-form chunk {} ({:.1}s of audio) in {:.2}s ({:.0}x real time)",
                        self.transcripts.len(),
                        audio_secs,
                        elapsed_secs,
                        audio_secs / elapsed_secs
                    );
                }
                Err(e) => {
                    info!("Chunk decoding stopped: {}", e);
                    self.failure = Some(e);
                }
            }
        }
    }
}
