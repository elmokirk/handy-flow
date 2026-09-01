//! Notes manager (NOTE-301) — domain wrapper over the note repository.

use crate::storage::database::AppDatabase;
use crate::storage::repositories::notes as repo;

#[derive(Clone)]
pub struct NotesManager {
    db: AppDatabase,
}

impl NotesManager {
    pub fn new(db: AppDatabase) -> Self {
        Self { db }
    }

    pub fn create(
        &self,
        title: String,
        content: String,
    ) -> Result<(repo::NoteRecord, repo::NoteVersionRecord), rusqlite::Error> {
        repo::create_note(
            &self.db,
            &repo::NewNote {
                title,
                first_version_content: content,
                source: repo::SOURCE_MANUAL_EDIT,
            },
        )
    }

    /// Append a version with content-hash dedupe (autosave calls this
    /// constantly; identical content is a no-op).
    pub fn save_content(
        &self,
        note_id: &str,
        content: &str,
        source: &'static str,
    ) -> Result<Option<repo::NoteVersionRecord>, rusqlite::Error> {
        repo::append_version(&self.db, note_id, content, source)
    }

    pub fn list(&self) -> Result<Vec<repo::NoteRecord>, rusqlite::Error> {
        repo::list_active_notes(&self.db)
    }

    pub fn versions(&self, note_id: &str) -> Result<Vec<repo::NoteVersionRecord>, rusqlite::Error> {
        repo::note_versions(&self.db, note_id)
    }

    pub fn current_content(&self, note_id: &str) -> Result<Option<String>, rusqlite::Error> {
        Ok(repo::current_version(&self.db, note_id)?.map(|v| v.content))
    }

    /// Current version record (version_no/source metadata included).
    pub fn current_content_version(
        &self,
        note_id: &str,
    ) -> Result<Option<repo::NoteVersionRecord>, rusqlite::Error> {
        repo::current_version(&self.db, note_id)
    }

    /// Restore an older version's content as a NEW version (append-only;
    /// history is never rewritten).
    pub fn restore_version(
        &self,
        note_id: &str,
        version_no: i64,
    ) -> Result<Option<repo::NoteVersionRecord>, rusqlite::Error> {
        let target = repo::note_versions(&self.db, note_id)?
            .into_iter()
            .find(|v| v.version_no == version_no)
            .ok_or(rusqlite::Error::QueryReturnedNoRows)?;
        repo::append_version(&self.db, note_id, &target.content, repo::SOURCE_RESTORE)
    }

    pub fn set_title(&self, note_id: &str, title: &str) -> Result<(), rusqlite::Error> {
        repo::set_title(&self.db, note_id, title)
    }

    pub fn set_pinned(&self, note_id: &str, pinned: bool) -> Result<(), rusqlite::Error> {
        repo::set_pinned(&self.db, note_id, pinned)
    }

    pub fn trash(&self, note_id: &str) -> Result<bool, rusqlite::Error> {
        repo::trash_note(&self.db, note_id)
    }

    pub fn restore(&self, note_id: &str) -> Result<bool, rusqlite::Error> {
        repo::restore_note(&self.db, note_id)
    }
}
