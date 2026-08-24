//! Repository layer — the ONLY place SQL beyond pragmas is allowed.
//! Managers/services call repositories; connectors call QueryService.

pub mod deliveries;
pub mod representations;
pub mod transcriptions;
