//! Repository layer — the ONLY place SQL beyond pragmas is allowed.
//! Managers/services call repositories; connectors call QueryService.

pub mod captures;
pub mod deliveries;
pub mod dictionary;
pub mod notes;
pub mod prompt_profiles;
pub mod representations;
pub mod search;
pub mod snippets;
pub mod transcriptions;
