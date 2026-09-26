use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Default)]
pub struct ReadReceiptRegistry {
    receipts: HashMap<String, Receipt>, // key: canonical path string
    next_id: u64,
}

#[derive(Debug, Clone)]
struct Receipt {
    id: u64,
    content_hash: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadOutcome {
    Full {
        content: String,
        receipt_id: u64,
        content_hash: u64,
    },
    Unchanged {
        receipt_id: u64,
    },
}

pub fn receipt_checked_read(
    registry: &mut ReadReceiptRegistry,
    requested_path: &str,
    anchor_in_context: &dyn Fn(u64) -> bool,
) -> anyhow::Result<ReadOutcome> {
    let (safe_path, content) = crate::read_file_at(requested_path)?;

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    let content_hash = hasher.finish();

    let key = safe_path.to_string_lossy().to_string();

    if let Some(stored) = registry.receipts.get(&key) {
        if stored.content_hash == content_hash && anchor_in_context(stored.id) {
            return Ok(ReadOutcome::Unchanged {
                receipt_id: stored.id,
            });
        }
    }

    registry.next_id += 1;
    let receipt_id = registry.next_id;
    registry.receipts.insert(
        key,
        Receipt {
            id: receipt_id,
            content_hash,
        },
    );

    Ok(ReadOutcome::Full {
        content,
        receipt_id,
        content_hash,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_first_read_returns_full_with_anchor() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("cynapse_test_receipt_first.txt");
        fs::write(&file_path, "initial content 123").unwrap();

        let mut registry = ReadReceiptRegistry::default();
        let outcome = receipt_checked_read(&mut registry, &file_path.to_string_lossy(), &|_| true).unwrap();

        match outcome {
            ReadOutcome::Full { content, receipt_id, content_hash } => {
                assert_eq!(content, "initial content 123");
                assert_eq!(receipt_id, 1);
                assert_ne!(content_hash, 0);
            }
            ReadOutcome::Unchanged { .. } => panic!("Expected Full outcome on first read"),
        }

        let _ = fs::remove_file(&file_path);
    }

    #[test]
    fn test_unchanged_and_anchored_returns_stub() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("cynapse_test_receipt_unchanged.txt");
        fs::write(&file_path, "steady content").unwrap();

        let mut registry = ReadReceiptRegistry::default();
        let _ = receipt_checked_read(&mut registry, &file_path.to_string_lossy(), &|_| true).unwrap();

        // Second read with anchor present in context -> Unchanged
        let second = receipt_checked_read(&mut registry, &file_path.to_string_lossy(), &|id| id == 1).unwrap();
        assert_eq!(second, ReadOutcome::Unchanged { receipt_id: 1 });

        let _ = fs::remove_file(&file_path);
    }

    #[test]
    fn test_unchanged_but_anchor_evicted_returns_full() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("cynapse_test_receipt_evicted.txt");
        fs::write(&file_path, "steady content").unwrap();

        let mut registry = ReadReceiptRegistry::default();
        let _ = receipt_checked_read(&mut registry, &file_path.to_string_lossy(), &|_| true).unwrap();

        // Second read when anchor is evicted from model context (anchor_in_context returns false)
        let second = receipt_checked_read(&mut registry, &file_path.to_string_lossy(), &|_| false).unwrap();
        match second {
            ReadOutcome::Full { content, receipt_id, .. } => {
                assert_eq!(content, "steady content");
                assert_eq!(receipt_id, 2);
            }
            ReadOutcome::Unchanged { .. } => panic!("Expected Full outcome when anchor is evicted"),
        }

        let _ = fs::remove_file(&file_path);
    }

    #[test]
    fn test_changed_content_returns_full() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("cynapse_test_receipt_changed.txt");
        fs::write(&file_path, "version 1").unwrap();

        let mut registry = ReadReceiptRegistry::default();
        let first = receipt_checked_read(&mut registry, &file_path.to_string_lossy(), &|_| true).unwrap();
        let first_hash = match first {
            ReadOutcome::Full { content_hash, .. } => content_hash,
            _ => panic!("Expected Full"),
        };

        // Modify file
        fs::write(&file_path, "version 2 with modifications").unwrap();

        let second = receipt_checked_read(&mut registry, &file_path.to_string_lossy(), &|_| true).unwrap();
        match second {
            ReadOutcome::Full { content, receipt_id, content_hash } => {
                assert_eq!(content, "version 2 with modifications");
                assert_eq!(receipt_id, 2);
                assert_ne!(content_hash, first_hash);
            }
            ReadOutcome::Unchanged { .. } => panic!("Expected Full outcome when content changes"),
        }

        let _ = fs::remove_file(&file_path);
    }

    #[test]
    fn test_shared_helper_errors_propagate() {
        let mut registry = ReadReceiptRegistry::default();
        let res = receipt_checked_read(&mut registry, "/nonexistent/path/to/missing_file_12345.txt", &|_| true);
        assert!(res.is_err());
        assert_eq!(registry.next_id, 0);
        assert!(registry.receipts.is_empty());
    }
}
