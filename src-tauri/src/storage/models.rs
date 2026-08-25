//! Canonical record models (storage-owned, transport-agnostic).
//!
//! These structs are the persistence-side truth for the canonical
//! tables. Public DTOs for UI/REST/MCP live behind QueryService later;
//! never expose these directly across the platform boundary.

/// Capture lifecycle per orchestration/DATA_STATE_MACHINES.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntegrityState {
    PendingAudio,
    AudioValid,
    AudioMissing,
    AudioCorrupt,
    RecoveredOrphan,
}

impl IntegrityState {
    pub fn as_str(&self) -> &'static str {
        match self {
            IntegrityState::PendingAudio => "pending_audio",
            IntegrityState::AudioValid => "audio_valid",
            IntegrityState::AudioMissing => "audio_missing",
            IntegrityState::AudioCorrupt => "audio_corrupt",
            IntegrityState::RecoveredOrphan => "recovered_orphan",
        }
    }
}

/// Attempt lifecycle: pending → running → success | failed.
/// Terminal payload is immutable; only canonical selection may change later.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttemptStatus {
    Pending,
    Running,
    Success,
    Failed,
}

impl AttemptStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AttemptStatus::Pending => "pending",
            AttemptStatus::Running => "running",
            AttemptStatus::Success => "success",
            AttemptStatus::Failed => "failed",
        }
    }
}

/// Where an attempt's text came from. `legacy_migration` marks rows that
/// predate durable provenance; their `engine_raw` is unknown and stays NULL.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttemptProvenance {
    Live,
    LegacyMigration,
}

impl AttemptProvenance {
    pub fn as_str(&self) -> &'static str {
        match self {
            AttemptProvenance::Live => "live",
            AttemptProvenance::LegacyMigration => "legacy_migration",
        }
    }
}

/// Representation lifecycle: success | failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepresentationStatus {
    Success,
    Failed,
}

impl RepresentationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RepresentationStatus::Success => "success",
            RepresentationStatus::Failed => "failed",
        }
    }
}

/// Which immutable text source a delivery event refers to (DATA-107).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeliverySourceKind {
    NormalizedStt,
    Representation,
}

impl DeliverySourceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeliverySourceKind::NormalizedStt => "normalized_stt",
            DeliverySourceKind::Representation => "representation",
        }
    }
}

/// Where delivered text went.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeliveryDestination {
    FocusedApp,
    Scratchpad,
    Clipboard,
}

impl DeliveryDestination {
    pub fn as_str(&self) -> &'static str {
        match self {
            DeliveryDestination::FocusedApp => "focused_app",
            DeliveryDestination::Scratchpad => "scratchpad",
            DeliveryDestination::Clipboard => "clipboard",
        }
    }
}
