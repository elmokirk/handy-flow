//! Snippet manager (SNIP-211) — deterministic spoken-phrase replacement.
//!
//! Frozen semantics (ADR-020): single pass, left-to-right,
//! longest-match-first, non-recursive; no fuzzy/regex/AI.
//!
//! Case semantics (explicit per ticket): triggers match
//! CASE-INSENSITIVE against whitespace tokens of `normalized_stt`;
//! the authored replacement text is emitted verbatim. Punctuation is
//! preserved because tokens are only replaced whole — punctuation
//! characters attached to a token stay attached to whatever replaces it.

use crate::storage::database::AppDatabase;
use crate::storage::repositories::snippets as repo;

#[derive(Clone, Debug)]
pub struct SnippetManager {
    db: AppDatabase,
}

impl SnippetManager {
    pub fn new(db: AppDatabase) -> Self {
        Self { db }
    }

    pub fn upsert(
        &self,
        trigger: String,
        replacement: String,
        priority: i64,
        enabled: bool,
    ) -> Result<repo::SnippetRecord, rusqlite::Error> {
        repo::upsert_snippet(
            &self.db,
            &repo::NewSnippet {
                trigger,
                replacement,
                priority,
                enabled,
            },
        )
    }

    pub fn list(&self) -> Result<Vec<repo::SnippetRecord>, rusqlite::Error> {
        repo::list_all(&self.db)
    }

    pub fn delete(&self, id: &str) -> Result<bool, rusqlite::Error> {
        repo::delete_snippet(&self.db, id)
    }

    /// Deterministic single-pass replacement over whitespace tokens.
    ///
    /// At each position the longest enabled matching trigger wins; among
    /// equal-length candidates the repository order decides
    /// (priority DESC, then id ASC). Replacements are never rescanned.
    pub fn apply_snippets(&self, text: &str) -> Result<String, rusqlite::Error> {
        let snippets = repo::list_enabled_for_match(&self.db)?;
        if snippets.is_empty() || text.is_empty() {
            return Ok(text.to_string());
        }

        // Pre-lowercase triggers once, grouped by token count so the
        // matcher can try longest phrases first (ADR-020).
        let mut by_len: std::collections::BTreeMap<usize, Vec<(String, &repo::SnippetRecord)>> =
            Default::default();
        for s in &snippets {
            let t = s.trigger.to_lowercase();
            let words = t.split_whitespace().count().max(1);
            by_len.entry(words).or_default().push((t, s));
        }
        let max_words = *by_len.keys().next_back().unwrap_or(&0);

        let tokens: Vec<&str> = text.split_whitespace().collect();
        let mut out: Vec<String> = Vec::with_capacity(tokens.len());
        let mut i = 0usize;

        while i < tokens.len() {
            let mut matched: Option<(&repo::SnippetRecord, usize)> = None;
            for n in (1..=max_words.min(tokens.len() - i)).rev() {
                if let Some(group) = by_len.get(&n) {
                    // Unicode-aware phrase boundaries: leading/trailing
                    // punctuation on the EDGE tokens is stripped for the
                    // comparison and reattached around the replacement.
                    let first = strip_edge_punct(tokens[i]);
                    let last = strip_edge_punct(tokens[i + n - 1]);
                    let mut cores: Vec<&str> = tokens[i..i + n].iter().copied().collect();
                    cores[0] = first.core;
                    cores[n - 1] = last.core;
                    let phrase = cores.join(" ").to_lowercase();
                    if phrase.is_empty() || cores.iter().any(|c| c.is_empty()) {
                        continue;
                    }
                    if let Some((_, rec)) = group.iter().find(|(t, _)| *t == phrase) {
                        matched = Some((rec, n));
                        break;
                    }
                }
            }
            match matched {
                Some((rec, n)) => {
                    let head = strip_edge_punct(tokens[i]);
                    let tail = strip_edge_punct(tokens[i + n - 1]);
                    out.push(format!("{}{}{}", head.prefix, rec.replacement, tail.suffix));
                    i += n;
                }
                None => {
                    out.push(tokens[i].to_string());
                    i += 1;
                }
            }
        }
        Ok(out.join(" "))
    }
}

/// Leading/trailing punctuation of one token (Unicode-aware: everything
/// that is not alphanumeric counts as punctuation).
struct TokenEdges<'a> {
    prefix: &'a str,
    core: &'a str,
    suffix: &'a str,
}

fn strip_edge_punct(token: &str) -> TokenEdges<'_> {
    let start = token
        .char_indices()
        .find(|(_, c)| c.is_alphanumeric())
        .map(|(i, _)| i)
        .unwrap_or(token.len());
    let end = token[start..]
        .char_indices()
        .rfind(|(_, c)| c.is_alphanumeric())
        .map(|(i, c)| start + i + c.len_utf8())
        .unwrap_or(start);
    TokenEdges {
        prefix: &token[..start],
        core: &token[start..end],
        suffix: &token[end..],
    }
}
