//! Storage layer.
//!
//! Owns SQLite persistence (database, migrations, IDs, models,
//! repositories) per the frozen implementation contracts.
//!
//! Ownership: Storage Lead only. Repositories own all SQL; managers and
//! services call repositories; connectors never touch SQL directly.
//! The physical database file keeps Handy's `history.db` name; it is
//! exposed internally as [`database::AppDatabase`].

pub mod database;
pub mod ids;
pub mod migrations;
pub mod models;
pub mod repositories;
