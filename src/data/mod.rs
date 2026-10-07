#![allow(unused)]
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::io::{BufRead, Read};
use std::path::{Path, PathBuf};
use thiserror::Error;

// TODO: organize this into various fitting modules

struct ScryfallData {
    meta: BulkData,
    cards: String,
    otags: Vec<OracleTag>,
}

/// A data layer for the app which manages the underlying directory structure as well as access to
/// the required scryfall data.
pub struct DataLayer {
    dir: PathBuf,
    local_data: Option<ScryfallData>,
    tmp_data: Option<BulkData>, // maybe Cell/RefCell
}

#[derive(Debug, Deserialize)]
struct Tagging {
    oracle_id: String,
    weight: String,
}

#[derive(Debug, Deserialize)]
struct OracleTag {
    id: String,
    label: String,
    slug: String,
    #[serde(rename = "type")]
    kind: String,
    uri: String,
    description: Option<String>,
    parent_ids: Vec<String>,
    child_ids: Vec<String>,
    aliases: Vec<String>,
    taggings: Vec<Tagging>,
}

#[derive(Debug, Deserialize)]
struct OracleTags {
    tags: Vec<OracleTag>,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
struct BulkData {
    #[serde(rename = "data")]
    entries: Vec<BulkDataEntry>,
}

pub const APP_NAME: &str = "decktags";
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
const BULK_DATA_JSON: &str = "bulk-data.json";
const TMP_DIR: &str = "tmp";
const SCRYFALL_BULK_DATA_URL: &str = "https://api.scryfall.com/bulk-data";

/// A common error type for data directory operations
#[derive(Error, Debug)]
pub enum DataError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Could not determine XDG_DATA_HOME")]
    XdgDataHomeNotFound,

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),
}

impl ScryfallData {
    fn local(dir: &Path) -> Result<Self, DataError> {
        let meta = std::fs::read_to_string(dir.join(BULK_DATA_JSON))
            .and_then(|content| Ok(serde_json::from_str::<BulkData>(&content)?))?;
        // TODO: parse cards and tags data as well, if present
        Ok(ScryfallData {
            meta,
            cards: String::new(),
            otags: Vec::new(),
        })
    }
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

fn download_bulk_data_file() -> Result<BulkData, DataError> {
    let response = reqwest::blocking::Client::new()
        .get(SCRYFALL_BULK_DATA_URL)
        .header(
            reqwest::header::USER_AGENT,
            format!("{APP_NAME}/{APP_VERSION}"),
        )
        .header(reqwest::header::ACCEPT, "application/json")
        .send()?
        .error_for_status()?;
    Ok(response.json()?)
}

fn download_jsonl<T: reqwest::IntoUrl, O: for<'a> Deserialize<'a>>(
    uri: T,
) -> Result<Vec<O>, DataError> {
    let response = reqwest::blocking::Client::new()
        .get(uri)
        .header(
            reqwest::header::USER_AGENT,
            format!("{APP_NAME}/{APP_VERSION}"),
        )
        .header(reqwest::header::ACCEPT, "application/gzip")
        .send()?
        .error_for_status()?;

    let bytes = response.bytes()?;

    // debug: write bytes to local file:
    // let decoder = flate2::read::GzDecoder::new(bytes.as_ref());
    // let mut reader = std::io::BufReader::new(decoder);
    // let mut buffer = String::with_capacity(bytes.len());
    // reader.read_to_string(&mut buffer)?;
    // std::fs::write("oracle_cards.jsonl", buffer);

    let decoder = flate2::read::GzDecoder::new(bytes.as_ref());
    let reader = std::io::BufReader::new(decoder);
    reader
        .lines()
        .map(|l| match l {
            Ok(line) => serde_json::from_str(&line).map_err(DataError::from),
            Err(e) => Err(DataError::from(e)),
        })
        .collect::<Result<Vec<O>, _>>()
}

impl DataLayer {
    /// Creates a new data layer for the app.
    /// Ensures that the data directory for the application exists.
    /// Parses existing JSON data.
    pub fn new(app_name: &str) -> Result<DataLayer, DataError> {
        let dir = dirs::data_local_dir()
            .ok_or(DataError::XdgDataHomeNotFound)?
            .join(app_name);

        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }

        // FIXME: There are two distinct cases I want to handle for these JSON files:
        // 1. The data is not there -> not an issue, don't report this to user, just set to None
        // 2. The data is invalid / can not be deserialized -> for now return this as error, but
        //    might be better to simply warn the user and continue with None
        let local_data = ScryfallData::local(&dir);
        let local_data = match local_data {
            Ok(data) => Some(data),
            Err(e) => match e {
                DataError::Io(ref io_err) if io_err.kind() == std::io::ErrorKind::NotFound => None,
                _ => return Err(e),
            },
        };

        Ok(DataLayer {
            dir,
            local_data,
            tmp_data: None,
        })
    }

    /// Returns the `DateTime<Utc>` of the most recent relevant update, if present.
    pub fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.local_data.as_ref().and_then(|ld| ld.meta.updated_at())
    }

    fn update_meta_data(&mut self) -> Result<&BulkData, DataError> {
        self.tmp_data = Some(download_bulk_data_file()?);
        Ok(self
            .tmp_data
            .as_ref()
            .expect("tmp data was not set to Some()"))
    }

    /// Checks for updates to the Scryfall bulk data.
    /// Downloads the bulk data file and returns the `DateTime<Utc>` of the most
    /// recent relevant update, if present.
    pub fn check_for_updates(&mut self) -> Result<DateTime<Utc>, DataError> {
        let new_bulk_data = self.update_meta_data()?;
        Ok(new_bulk_data
            .updated_at()
            .expect("Expected at least one relevant update in bulk data"))
    }

    pub fn download_data(&mut self) -> Result<(), DataError> {
        let meta_data = if let Some(data) = &self.tmp_data {
            data
        } else {
            self.update_meta_data()?
        };

        dbg!(meta_data);

        // TODO: move into ScryfallData "remote" or whatever constructor
        let mut otags = Vec::new();
        // let mut oracle_cards = Vec::new();

        for entry in meta_data.entries.iter() {
            if matches!(entry.kind.as_str(), "oracle_cards" | "oracle_tags") {
                if entry.kind == "oracle_tags" {
                    otags = download_jsonl(&entry.jsonl_download_uri)?;
                } else {
                    // for line in lines {
                    // oracle_tags.push(serde_json::from_str::<OracleTag>(&line)?);
                    // }
                }
            }
        }
        self.local_data = Some(ScryfallData {
            meta: self
                .tmp_data
                .take()
                .expect("tmp_data was set at beginning of function"),
            cards: String::new(),
            otags,
        });

        dbg!(&self.tmp_data);

        for tag in self.local_data.as_ref().unwrap().otags.iter() {
            println!("{:?}", tag);
        }

        Ok(())
    }

    pub fn oracle_cards(&self) -> Result<(), DataError> {
        todo!()
    }
    pub fn oracle_tags(&self) -> Result<(), DataError> {
        todo!()
    }
}
