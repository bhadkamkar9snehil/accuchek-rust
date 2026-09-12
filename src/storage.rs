//! SQLite storage for glucose readings and device metadata.
//!
//! Source fidelity is prioritized: raw meter values and status words are preserved even when
//! they are excluded from headline analysis. Existing v1 databases are migrated in-place.

use std::collections::HashMap;
use std::path::Path;

use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

use crate::device::{DeviceMetadata, GlucoseReading};
use crate::stats::{
    BasicStats, CalendarDay, DailyStats, HistogramBin, HourlyStats, ReadingData, TimeBinStats,
    TimeInRange,
};
use crate::units::Thresholds;

const SCHEMA_VERSION: i64 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredReading {
    pub id: i64,
    pub epoch: i64,
    pub timestamp: String,
    #[serde(rename = "mg/dL")]
    pub mg_dl: u16,
    #[serde(rename = "mmol/L")]
    pub mmol_l: f64,
    pub raw_value: u16,
    pub status: u16,
    pub range_state: String,
    pub device_key: String,
    pub occurrence: u16,
    pub note: Option<String>,
    pub tags: Option<String>,
    pub imported_at: String,
}

impl ReadingData for StoredReading {
    fn mg_dl(&self) -> u16 {
        self.mg_dl
    }

    fn mmol_l(&self) -> f64 {
        self.mmol_l
    }

    fn timestamp(&self) -> &str {
        &self.timestamp
    }
}

pub struct Storage {
    conn: Connection,
}

include!("storage_parts/schema.rs");
include!("storage_parts/operations.rs");
include!("storage_parts/helpers_tests.rs");
