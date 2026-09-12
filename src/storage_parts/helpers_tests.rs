fn parse_date_info(date: &str) -> (u8, u32) {
    for format in ["%Y-%m-%d", "%Y/%m/%d"] {
        if let Ok(parsed) = chrono::NaiveDate::parse_from_str(date, format) {
            use chrono::Datelike;
            return (
                parsed.weekday().num_days_from_monday() as u8,
                parsed.iso_week().week(),
            );
        }
    }
    (0, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::ReadingRange;

    fn memory_storage() -> Storage {
        let conn = Connection::open_in_memory().expect("in-memory sqlite");
        Storage::initialize_schema(&conn).expect("schema init");
        Storage { conn }
    }

    fn reading(timestamp: &str, value: u16, status: u16) -> GlucoseReading {
        GlucoseReading {
            id: 0,
            epoch: 1_700_000_000,
            timestamp: timestamp.to_string(),
            mg_dl: value,
            mmol_l: value as f64 / 18.0,
            raw_value: value,
            status,
            range_state: ReadingRange::Normal,
            device_key: "roche-173a-21d7-test-meter".to_string(),
        }
    }

    #[test]
    fn repeat_sync_is_idempotent() {
        let storage = memory_storage();
        let rows = vec![reading("2026-09-13T08:00:00", 101, 0)];
        assert_eq!(storage.import_readings(&rows).unwrap(), 1);
        assert_eq!(storage.import_readings(&rows).unwrap(), 0);
        assert_eq!(storage.count().unwrap(), 1);
    }

    #[test]
    fn identical_same_minute_readings_are_preserved_as_occurrences() {
        let storage = memory_storage();
        let rows = vec![
            reading("2026-09-13T08:00:00", 101, 0),
            reading("2026-09-13T08:00:00", 101, 0),
        ];
        assert_eq!(storage.import_readings(&rows).unwrap(), 2);
        assert_eq!(storage.count().unwrap(), 2);
        assert_eq!(storage.import_readings(&rows).unwrap(), 0);
        assert_eq!(storage.count().unwrap(), 2);
    }

    #[test]
    fn nonzero_source_status_is_retained_and_included_until_semantics_are_known() {
        let storage = memory_storage();
        storage
            .import_readings(&[
                reading("2026-09-13T08:00:00", 101, 0),
                reading("2026-09-13T09:00:00", 222, 0x0040),
            ])
            .unwrap();

        let all = storage.get_all_readings().unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[1].status, 0x0040);

        // Non-zero status occurs on valid historical Accu-Chek records and may encode context.
        // Until a specific exclusion bit is positively identified, analysis must not drop it.
        let analysis = storage.get_analysis_readings().unwrap();
        assert_eq!(analysis.len(), 2);
        assert_eq!(analysis[1].mg_dl, 222);
    }

    #[test]
    fn legacy_schema_is_migrated_without_losing_rows_notes_or_tags() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE readings (
                id INTEGER PRIMARY KEY,
                epoch INTEGER NOT NULL UNIQUE,
                timestamp TEXT NOT NULL,
                mg_dl INTEGER NOT NULL,
                mmol_l REAL NOT NULL,
                note TEXT,
                tags TEXT,
                imported_at TEXT DEFAULT CURRENT_TIMESTAMP
             );
             INSERT INTO readings (id, epoch, timestamp, mg_dl, mmol_l, note, tags)
             VALUES (7, 1700000000, '2026/09/13 08:00', 123, 6.833333, 'keep me', 'fasting');",
        )
        .unwrap();

        Storage::initialize_schema(&conn).unwrap();
        let storage = Storage { conn };
        let rows = storage.get_all_readings().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, 7);
        assert_eq!(rows[0].raw_value, 123);
        assert_eq!(rows[0].device_key, "legacy");
        assert_eq!(rows[0].note.as_deref(), Some("keep me"));
        assert_eq!(rows[0].tags.as_deref(), Some("fasting"));
    }
}
