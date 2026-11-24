use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub const HISTORY_FILE_NAME: &str = ".tidyfiles_history.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct MoveRecord {
    pub original_path: PathBuf,
    pub current_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub timestamp: u64,
    pub target_dir: PathBuf,
    pub created_dirs: Vec<PathBuf>,
    pub moves: Vec<MoveRecord>,
}

pub struct HistoryManager;

impl HistoryManager {
    pub fn save_transaction(target_dir: &Path, transaction: &Transaction) -> Result<()> {
        let history_path = target_dir.join(HISTORY_FILE_NAME);
        let mut history: Vec<Transaction> = Self::load_transactions(target_dir).unwrap_or_default();
        history.push(transaction.clone());

        let json = serde_json::to_string_pretty(&history)
            .context("Error serializando historial de operaciones")?;
        fs::write(&history_path, json)
            .with_context(|| format!("No se pudo guardar {}", history_path.display()))?;

        Ok(())
    }

    pub fn load_transactions(target_dir: &Path) -> Result<Vec<Transaction>> {
        let history_path = target_dir.join(HISTORY_FILE_NAME);
        if !history_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&history_path)
            .with_context(|| format!("No se pudo leer {}", history_path.display()))?;
        let list: Vec<Transaction> = serde_json::from_str(&content)
            .context("Error parseando el archivo de historial JSON")?;
        Ok(list)
    }

    pub fn pop_last_transaction(target_dir: &Path) -> Result<Option<Transaction>> {
        let mut history = Self::load_transactions(target_dir)?;
        let last = history.pop();

        let history_path = target_dir.join(HISTORY_FILE_NAME);
        if history.is_empty() {
            if history_path.exists() {
                let _ = fs::remove_file(&history_path);
            }
        } else {
            let json = serde_json::to_string_pretty(&history)?;
            fs::write(&history_path, json)?;
        }

        Ok(last)
    }

    pub fn new_transaction(
        target_dir: &Path,
        created_dirs: Vec<PathBuf>,
        moves: Vec<MoveRecord>,
    ) -> Transaction {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Transaction {
            timestamp,
            target_dir: target_dir.to_path_buf(),
            created_dirs,
            moves,
        }
    }
}

impl Clone for Transaction {
    fn clone(&self) -> Self {
        Self {
            timestamp: self.timestamp,
            target_dir: self.target_dir.clone(),
            created_dirs: self.created_dirs.clone(),
            moves: self
                .moves
                .iter()
                .map(|m| MoveRecord {
                    original_path: m.original_path.clone(),
                    current_path: m.current_path.clone(),
                })
                .collect(),
        }
    }
}
