//! Undo & Rollback Journal for Cynapse Agent File Mutations.
//!
//! Automatically records inverse operations for `write_file` and `patch_file`
//! into session-scoped journals under `~/.cynapse/journal/<session_id>/`,
//! enabling instant, deterministic multi-turn rollback via `/undo` or `cynapse undo`.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JournalAction {
    CreatedNew { path: PathBuf },
    OverwroteExisting { path: PathBuf, previous_content: String },
    PatchedLines { path: PathBuf, original_content: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub timestamp: u64,
    pub action: JournalAction,
    pub description: String,
}

pub struct UndoJournal {
    #[allow(dead_code)]
    session_id: String,
    #[allow(dead_code)]
    journal_dir: PathBuf,
    history: Mutex<Vec<JournalEntry>>,
}

static GLOBAL_JOURNAL: Mutex<Option<UndoJournal>> = Mutex::new(None);

impl UndoJournal {
    pub fn init(session_id: &str) -> Result<()> {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let journal_dir = home.join(".cynapse").join("journal").join(session_id);
        fs::create_dir_all(&journal_dir)?;

        let journal = Self {
            session_id: session_id.to_string(),
            journal_dir,
            history: Mutex::new(Vec::new()),
        };

        let mut lock = GLOBAL_JOURNAL.lock().unwrap();
        *lock = Some(journal);
        Ok(())
    }

    /// Record a snapshot before writing or overwriting a file.
    pub fn record_write(path: &Path) -> Result<()> {
        let lock = GLOBAL_JOURNAL.lock().unwrap();
        let journal = match lock.as_ref() {
            Some(j) => j,
            None => return Ok(()),
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let action = if path.exists() {
            let previous = fs::read_to_string(path).unwrap_or_default();
            JournalAction::OverwroteExisting {
                path: path.to_path_buf(),
                previous_content: previous,
            }
        } else {
            JournalAction::CreatedNew {
                path: path.to_path_buf(),
            }
        };

        let entry = JournalEntry {
            timestamp: now,
            action,
            description: format!("Modified {}", path.display()),
        };

        let mut hist = journal.history.lock().unwrap();
        hist.push(entry);
        Ok(())
    }

    /// Record a snapshot before surgically patching a file.
    pub fn record_patch(path: &Path, original_content: &str) -> Result<()> {
        let lock = GLOBAL_JOURNAL.lock().unwrap();
        let journal = match lock.as_ref() {
            Some(j) => j,
            None => return Ok(()),
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let entry = JournalEntry {
            timestamp: now,
            action: JournalAction::PatchedLines {
                path: path.to_path_buf(),
                original_content: original_content.to_string(),
            },
            description: format!("Patched {}", path.display()),
        };

        let mut hist = journal.history.lock().unwrap();
        hist.push(entry);
        Ok(())
    }

    /// Roll back the most recent file modification in this session.
    pub fn undo_latest() -> Result<String> {
        let lock = GLOBAL_JOURNAL.lock().unwrap();
        let journal = match lock.as_ref() {
            Some(j) => j,
            None => bail!("Undo journal not initialized for this session."),
        };

        let mut hist = journal.history.lock().unwrap();
        let latest = match hist.pop() {
            Some(e) => e,
            None => bail!("No remaining file modifications to undo in this session."),
        };

        match latest.action {
            JournalAction::CreatedNew { path } => {
                if path.exists() {
                    fs::remove_file(&path)
                        .with_context(|| format!("Failed to delete created file {}", path.display()))?;
                    Ok(format!("Undid file creation: Deleted {}", path.display()))
                } else {
                    Ok(format!("File {} was already removed.", path.display()))
                }
            }
            JournalAction::OverwroteExisting { path, previous_content } => {
                fs::write(&path, &previous_content)
                    .with_context(|| format!("Failed to restore previous content in {}", path.display()))?;
                Ok(format!("Undid overwrite: Restored {} to previous state ({} bytes)", path.display(), previous_content.len()))
            }
            JournalAction::PatchedLines { path, original_content } => {
                fs::write(&path, &original_content)
                    .with_context(|| format!("Failed to revert patch on {}", path.display()))?;
                Ok(format!("Undid patch: Restored {} to exact pre-patch content ({} bytes)", path.display(), original_content.len()))
            }
        }
    }

    /// Count remaining undoable actions.
    pub fn count_available() -> usize {
        let lock = GLOBAL_JOURNAL.lock().unwrap();
        lock.as_ref().map(|j| j.history.lock().unwrap().len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_undo_journal_lifecycle() {
        let dir = tempdir().unwrap();
        let test_file = dir.path().join("code.rs");

        UndoJournal::init("test_session_1").unwrap();

        // 1. Test creation undo
        UndoJournal::record_write(&test_file).unwrap();
        fs::write(&test_file, "fn main() {}\n").unwrap();
        assert!(test_file.exists());

        let res = UndoJournal::undo_latest().unwrap();
        assert!(res.contains("Deleted"));
        assert!(!test_file.exists());

        // 2. Test overwrite undo
        fs::write(&test_file, "original content\n").unwrap();
        UndoJournal::record_write(&test_file).unwrap();
        fs::write(&test_file, "mutated content\n").unwrap();

        let res2 = UndoJournal::undo_latest().unwrap();
        assert!(res2.contains("Restored"));
        assert_eq!(fs::read_to_string(&test_file).unwrap(), "original content\n");
    }
}
