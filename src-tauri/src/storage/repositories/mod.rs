//! Repository layer — the ONLY place SQL beyond pragmas is allowed.
//! Managers/services call repositories; connectors call QueryService.

pub mod captures;
pub mod deliveries;
pub mod dictionary;
pub mod representations;
pub mod transcriptions;
