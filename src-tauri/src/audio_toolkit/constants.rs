pub const WHISPER_SAMPLE_RATE: u32 = 16000;
// ponytail: interim guard only; replace with measured, bounded worker chunks
// before AUDIO-240 is released. Six minutes is below the observed Vulkan crash.
pub const MAX_UNISOLATED_BATCH_SAMPLES: usize = 6 * 60 * WHISPER_SAMPLE_RATE as usize;
