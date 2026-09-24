use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::path::PathBuf;
use thiserror::Error;

/// A data directory type which ensures the underlying directory does exist.
pub struct AppData {
    dir: PathBuf,
    bulk_data: Option<BulkData>,
}

#[derive(Deserialize)]
struct BulkDataEntry {
    id: String,
    #[serde(rename = "type")]
    kind: String,
    updated_at: DateTime<Utc>,
    name: String,
    description: String,
    compressed_size: usize,
    uri: String,
    jsonl_download_uri: String,
}

#[derive(Deserialize)]
struct BulkData {
    #[serde(rename = "data")]
    entries: Vec<BulkDataEntry>,
}

const BULK_DATA_JSON: &str = "bulk-data.json";
const TMP_DIR: &str = "tmp";

/// A common error type for data directory operations
#[derive(Error, Debug)]
pub enum DataError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Could not determine XDG_DATA_HOME")]
    XdgDataHomeNotFound,
}

impl BulkData {
    /// Returns the `DateTime<Utc>` of the most recent relevant update, if present.
    pub fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.entries
            .iter()
            .filter_map(|data| {
                if matches!(data.kind.as_str(), "oracle_cards" | "oracle_tags") {
                    Some(data.updated_at)
                } else {
                    None
                }
            })
            .max()
    }
}

impl AppData {
    /// Ensures that the data directory for the application exists.
    /// If it does not exist, it will be created.
    pub fn new(app_name: &str) -> Result<AppData, DataError> {
        let dir = dirs::data_local_dir()
            .ok_or(DataError::XdgDataHomeNotFound)?
            .join(app_name);

        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }

        let bulk_data = std::fs::read_to_string(dir.join(BULK_DATA_JSON))
            .and_then(|content| Ok(serde_json::from_str::<BulkData>(&content)?))
            .ok();

        Ok(AppData { dir, bulk_data })
    }

    /// Returns the `DateTime<Utc>` of the most recent relevant update, if present.
    pub fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.bulk_data.as_ref().and_then(BulkData::updated_at)
    }

    pub fn check_for_updates(&self) -> Option<DateTime<Utc>> {
        todo!()
    }

    pub fn download_data(&self) -> Result<(), DataError> {
        todo!()
    }

    pub fn oracle_cards(&self) -> Result<(), DataError> {
        todo!()
    }
    pub fn oracle_tags(&self) -> Result<(), DataError> {
        todo!()
    }
}
