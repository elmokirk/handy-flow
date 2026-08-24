//! Stable unique identifiers for durable custom records.
//!
//! Single source of truth for ID generation (DEP-100): UUIDv7 strings.
//! UUIDv7 is time-ordered, which keeps primary keys friendly to
//! `(created_at DESC, id DESC)` pagination and to B-tree locality.

use uuid::Uuid;

/// Generate a new UUIDv7 as a canonical lowercase hyphenated string.
pub fn new_id() -> String {
    Uuid::now_v7().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_ids_are_uuidv7() {
        let id = new_id();
        let parsed = Uuid::parse_str(&id).expect("id must parse as UUID");
        assert_eq!(parsed.get_version_num(), 7, "version nibble must be 7");
        // RFC 9562 variant: 10x in the highest bits of octet 8.
        let variant = parsed.get_variant();
        assert!(matches!(
            variant,
            uuid::Variant::RFC4122 | uuid::Variant::Microsoft
        ));
    }

    #[test]
    fn sequential_ids_are_lexicographically_ordered() {
        let mut previous = new_id();
        for _ in 0..50 {
            let next = new_id();
            // UUIDv7 embeds ms timestamps; identical-millisecond IDs may
            // tie, so ordering is non-strict. Strictly increasing across
            // millisecond boundaries must always hold.
            if next != previous {
                assert!(
                    next > previous,
                    "later UUIDv7 must sort after earlier one ({previous} !< {next})"
                );
            }
            previous = next;
            std::thread::sleep(std::time::Duration::from_micros(250));
        }
    }

    #[test]
    fn ids_are_unique_within_burst() {
        let mut seen = std::collections::HashSet::new();
        for _ in 0..1000 {
            assert!(seen.insert(new_id()), "duplicate UUIDv7 within burst");
        }
    }
}
