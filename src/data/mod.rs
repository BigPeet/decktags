use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::path::PathBuf;

/// A data directory type which ensures the underlying directory does exist.
pub struct DataDirectory {
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

type DataError = std::io::Error;

impl DataDirectory {
    /// Ensures that the data directory for the application exists.
    /// If it does not exist, it will be created.
    pub fn ensure_exists(app_name: &str) -> Result<DataDirectory, DataError> {
        let dir = dirs::data_local_dir()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not determine XDG_DATA_HOME",
                )
            })?
            .join(app_name);

        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }

        Ok(DataDirectory {
            dir,
            bulk_data: None, // not yet initialized
        })
    }

    fn init_bulk_data(&self) -> Result<BulkData, DataError> {
        let bulk_data_path = self.dir.join(BULK_DATA_JSON);
        let bulk_data_json = std::fs::read_to_string(bulk_data_path)?;
        Ok(serde_json::from_str::<BulkData>(&bulk_data_json)?)
    }

    fn get_or_init_bulk_data(&mut self) -> Result<&BulkData, DataError> {
        match self.bulk_data {
            Some(ref data) => Ok(data),
            None => {
                self.bulk_data = Some(self.init_bulk_data()?);
                Ok(self.bulk_data.as_ref().unwrap()) // unwrap: known to be Some at this point
            }
        }
    }

    pub fn updated_at(&mut self) -> Option<DateTime<Utc>> {
        match self.get_or_init_bulk_data().ok() {
            Some(bulk_data) => bulk_data
                .entries
                .iter()
                .filter_map(|data| {
                    if matches!(data.kind.as_str(), "oracle_cards" | "oracle_tags") {
                        Some(data.updated_at)
                    } else {
                        None
                    }
                })
                .max(),
            None => None,
        }
        // if let Some(bulk_data) = self.get_or_init_bulk_data().ok() {

        // }

        // let bulk_data_path = self.dir.join(BULK_DATA_JSON);
        // if !bulk_data_path.exists() {
        // // The bulk data file does not exist (yet).
        // return None;
        // }
        // if let Ok(bulk_data_json) = std::fs::read_to_string(bulk_data_path)
        // && let Ok(bulk_data_list) = serde_json::from_str::<BulkData>(&bulk_data_json)
        // {
        // // TODO: Better logging for the case there is an error when reading the file, there
        // // are no entries, or there is no max.
        // return bulk_data_list
        // .entries
        // .iter()
        // .filter_map(|data| {
        // if matches!(data.kind.as_str(), "oracle_cards" | "oracle_tags") {
        // Some(data.updated_at)
        // } else {
        // None
        // }
        // })
        // .max();
        // }
        // None
    }
}
