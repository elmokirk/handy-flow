//! Dictionary manager (DICT-201).
//!
//! Thin domain layer over the dictionary repository. The deterministic
//! correction itself stays in `audio_toolkit::text::apply_custom_words`
//! (upstream logic, regression-tested); this manager supplies the
//! durable term set and the per-attempt snapshot hash.

use crate::storage::database::AppDatabase;
use crate::storage::repositories::dictionary as repo;

#[derive(Clone, Debug)]
pub struct DictionaryManager {
    db: AppDatabase,
}

impl DictionaryManager {
    pub fn new(db: AppDatabase) -> Self {
        Self { db }
    }

    pub fn upsert(
        &self,
        term: String,
        aliases: Vec<String>,
        enabled: bool,
    ) -> Result<repo::DictionaryEntry, rusqlite::Error> {
        repo::upsert_entry(
            &self.db,
            &repo::NewDictionaryEntry {
                term,
                aliases,
                enabled,
            },
        )
    }

    pub fn list(&self) -> Result<Vec<repo::DictionaryEntry>, rusqlite::Error> {
        repo::list_all(&self.db)
    }

    pub fn delete(&self, id: &str) -> Result<bool, rusqlite::Error> {
        repo::delete_entry(&self.db, id)
    }

    /// Terms handed to the deterministic pipeline (term + aliases).
    pub fn correction_terms(&self) -> Result<Vec<String>, rusqlite::Error> {
        repo::enabled_correction_terms(&self.db)
    }

    /// Snapshot hash recorded on every successful attempt.
    pub fn snapshot_sha256(&self) -> Result<String, rusqlite::Error> {
        repo::enabled_snapshot_sha256(&self.db)
    }

    /// One-time import of upstream settings' `custom_words` (legacy
    /// plain word list) as enabled dictionary terms without aliases.
    /// Idempotent: existing terms are updated, not duplicated.
    pub fn import_legacy_custom_words(&self, words: &[String]) -> Result<usize, rusqlite::Error> {
        let mut imported = 0usize;
        for word in words {
            let term = word.trim();
            if term.is_empty() {
                continue;
            }
            repo::upsert_entry(
                &self.db,
                &repo::NewDictionaryEntry {
                    term: term.to_string(),
                    aliases: Vec::new(),
                    enabled: true,
                },
            )?;
            imported += 1;
        }
        Ok(imported)
    }
}
