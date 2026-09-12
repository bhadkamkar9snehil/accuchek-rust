impl Storage {
    pub fn upsert_device(&self, device: &DeviceMetadata) -> Result<()> {
        self.conn.execute(
            "INSERT INTO devices (
                device_key, vendor_id, product_id, manufacturer, usb_product, usb_serial,
                model_number, model_name, serial_number, meter_time
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(device_key) DO UPDATE SET
                vendor_id=excluded.vendor_id,
                product_id=excluded.product_id,
                manufacturer=excluded.manufacturer,
                usb_product=excluded.usb_product,
                usb_serial=excluded.usb_serial,
                model_number=excluded.model_number,
                model_name=excluded.model_name,
                serial_number=excluded.serial_number,
                meter_time=excluded.meter_time,
                last_seen_at=CURRENT_TIMESTAMP",
            params![
                device.device_key,
                device.vendor_id,
                device.product_id,
                device.manufacturer,
                device.usb_product,
                device.usb_serial,
                device.model_number,
                device.model_name,
                device.serial_number,
                device.meter_time,
            ],
        )?;
        Ok(())
    }

    fn insert_reading_with_occurrence(
        &self,
        reading: &GlucoseReading,
        occurrence: u16,
    ) -> Result<Option<i64>> {
        let result = self.conn.execute(
            "INSERT OR IGNORE INTO readings (
                epoch, timestamp, mg_dl, mmol_l, raw_value, status,
                range_state, device_key, occurrence
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                reading.epoch,
                reading.timestamp,
                reading.mg_dl,
                reading.mmol_l,
                reading.raw_value,
                reading.status,
                reading.range_state.as_str(),
                reading.device_key,
                occurrence,
            ],
        )?;

        if result > 0 {
            Ok(Some(self.conn.last_insert_rowid()))
        } else {
            Ok(None)
        }
    }

    /// Compatibility method for callers inserting one reading at a time.
    pub fn insert_reading(&self, reading: &GlucoseReading) -> Result<Option<i64>> {
        self.insert_reading_with_occurrence(reading, 0)
    }

    /// Bulk import with duplicate-occurrence accounting. This preserves two identical readings
    /// taken within the same meter minute without creating duplicates on subsequent syncs.
    pub fn import_readings(&self, readings: &[GlucoseReading]) -> Result<usize> {
        let mut count = 0usize;
        let mut occurrences: HashMap<(String, String, u16, u16), u16> = HashMap::new();

        for reading in readings {
            let key = (
                reading.device_key.clone(),
                reading.timestamp.clone(),
                reading.raw_value,
                reading.status,
            );
            let occurrence = occurrences.entry(key).or_insert(0);
            if self
                .insert_reading_with_occurrence(reading, *occurrence)?
                .is_some()
            {
                count += 1;
            }
            *occurrence = occurrence.saturating_add(1);
        }
        Ok(count)
    }

    pub fn update_note(&self, id: i64, note: &str) -> Result<usize> {
        self.conn
            .execute("UPDATE readings SET note = ?1 WHERE id = ?2", params![note, id])
    }

    pub fn update_tags(&self, id: i64, tags: &str) -> Result<usize> {
        self.conn
            .execute("UPDATE readings SET tags = ?1 WHERE id = ?2", params![tags, id])
    }

    pub fn get_all_readings(&self) -> Result<Vec<StoredReading>> {
        self.query_readings(false)
    }

    /// Readings used for numerical summaries. Unknown/nonzero source status is retained in the
    /// database and raw-data UI, but excluded from headline analysis until its semantics are known.
    pub fn get_analysis_readings(&self) -> Result<Vec<StoredReading>> {
        self.query_readings(true)
    }

    fn query_readings(&self, analysis_only: bool) -> Result<Vec<StoredReading>> {
        let sql = if analysis_only {
            "SELECT id, epoch, timestamp, mg_dl, mmol_l, raw_value, status, range_state,
                    device_key, occurrence, note, tags, imported_at
             FROM readings WHERE status = 0 ORDER BY epoch, id"
        } else {
            "SELECT id, epoch, timestamp, mg_dl, mmol_l, raw_value, status, range_state,
                    device_key, occurrence, note, tags, imported_at
             FROM readings ORDER BY epoch, id"
        };
        let mut stmt = self.conn.prepare(sql)?;
        stmt.query_map([], Self::row_to_stored_reading)?
            .collect::<Result<Vec<_>>>()
    }

    pub fn count(&self) -> Result<i64> {
        self.conn
            .query_row("SELECT COUNT(*) FROM readings", [], |row| row.get(0))
    }

    pub fn get_all_values(&self) -> Result<Vec<u16>> {
        let mut stmt = self
            .conn
            .prepare("SELECT mg_dl FROM readings WHERE status = 0 ORDER BY epoch, id")?;
        stmt.query_map([], |row| row.get::<_, u16>(0))?
            .collect::<Result<Vec<_>>>()
    }

    pub fn get_all_values_both(&self) -> Result<(Vec<u16>, Vec<f64>)> {
        let mut stmt = self.conn.prepare(
            "SELECT mg_dl, mmol_l FROM readings WHERE status = 0 ORDER BY epoch, id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, u16>(0)?, row.get::<_, f64>(1)?))
        })?;

        let mut mgdl = Vec::new();
        let mut mmol = Vec::new();
        for row in rows {
            let (mg, mm) = row?;
            mgdl.push(mg);
            mmol.push(mm);
        }
        Ok((mgdl, mmol))
    }

    pub fn get_basic_stats(&self) -> Result<Option<BasicStats>> {
        let (mgdl, mmol) = self.get_all_values_both()?;
        Ok(BasicStats::from_values(&mgdl, &mmol))
    }

    pub fn get_time_in_range(&self, thresholds: Thresholds) -> Result<TimeInRange> {
        Ok(TimeInRange::from_values(&self.get_all_values()?, thresholds))
    }

    pub fn get_daily_stats(&self, thresholds: Thresholds) -> Result<Vec<DailyStats>> {
        let readings = self.get_analysis_readings()?;
        let mut daily_readings: std::collections::BTreeMap<String, (Vec<u16>, Vec<f64>)> =
            std::collections::BTreeMap::new();

        for reading in &readings {
            if let Some(date) = reading.timestamp.get(0..10) {
                let entry = daily_readings.entry(date.to_string()).or_default();
                entry.0.push(reading.mg_dl);
                entry.1.push(reading.mmol_l);
            }
        }

        Ok(daily_readings
            .into_iter()
            .map(|(date, (mgdl, mmol))| DailyStats::new(date, &mgdl, &mmol, thresholds))
            .collect())
    }

    pub fn get_hourly_stats(&self) -> Result<Vec<HourlyStats>> {
        let readings = self.get_analysis_readings()?;
        let mut hourly_data: Vec<(Vec<u16>, Vec<f64>)> = vec![(Vec::new(), Vec::new()); 24];

        for reading in &readings {
            if let Some(hour_str) = reading.timestamp.get(11..13) {
                if let Ok(hour) = hour_str.parse::<usize>() {
                    if hour < 24 {
                        hourly_data[hour].0.push(reading.mg_dl);
                        hourly_data[hour].1.push(reading.mmol_l);
                    }
                }
            }
        }

        Ok(hourly_data
            .into_iter()
            .enumerate()
            .map(|(hour, (mgdl, mmol))| HourlyStats::new(hour as u8, mgdl, mmol))
            .collect())
    }

    pub fn get_time_bin_stats(&self) -> Result<Vec<TimeBinStats>> {
        let readings = self.get_analysis_readings()?;
        let bins = [
            ("Overnight", "12AM-6AM", 0u8, 6u8),
            ("Fasting/Morning", "6AM-9AM", 6, 9),
            ("Mid-Morning", "9AM-12PM", 9, 12),
            ("Afternoon", "12PM-6PM", 12, 18),
            ("Evening", "6PM-9PM", 18, 21),
            ("Night", "9PM-12AM", 21, 24),
        ];

        Ok(bins
            .iter()
            .map(|(name, desc, start, end)| {
                let filtered: Vec<_> = readings
                    .iter()
                    .filter(|r| {
                        r.timestamp
                            .get(11..13)
                            .and_then(|hour| hour.parse::<u8>().ok())
                            .is_some_and(|hour| hour >= *start && hour < *end)
                    })
                    .collect();
                let mgdl = filtered.iter().map(|r| r.mg_dl).collect::<Vec<_>>();
                let mmol = filtered.iter().map(|r| r.mmol_l).collect::<Vec<_>>();
                TimeBinStats::new(name, desc, *start, *end, mgdl, mmol)
            })
            .collect())
    }

    pub fn get_histogram(&self, bin_width: u16) -> Result<Vec<HistogramBin>> {
        let readings = self.get_analysis_readings()?;
        if readings.is_empty() {
            return Ok(Vec::new());
        }

        let mut bins = Vec::new();
        let mut start = 40u16;
        let total = readings.len();
        while start < 400 {
            let end = start + bin_width;
            let count = readings
                .iter()
                .filter(|r| r.mg_dl >= start && r.mg_dl < end)
                .count();
            bins.push(HistogramBin {
                range_start: start,
                range_end: end,
                count,
                percentage: (count as f64 / total as f64) * 100.0,
            });
            start = end;
        }
        Ok(bins)
    }

    pub fn get_calendar_data(&self, thresholds: Thresholds) -> Result<Vec<CalendarDay>> {
        let readings = self.get_analysis_readings()?;
        if readings.is_empty() {
            return Ok(Vec::new());
        }

        let mut daily_readings: std::collections::BTreeMap<String, Vec<(u8, u16, f64)>> =
            std::collections::BTreeMap::new();
        for reading in &readings {
            if let (Some(date), Some(hour_str)) =
                (reading.timestamp.get(0..10), reading.timestamp.get(11..13))
            {
                if let Ok(hour) = hour_str.parse::<u8>() {
                    daily_readings
                        .entry(date.to_string())
                        .or_default()
                        .push((hour, reading.mg_dl, reading.mmol_l));
                }
            }
        }

        Ok(daily_readings
            .into_iter()
            .map(|(date, readings)| {
                let (day_of_week, week_of_year) = parse_date_info(&date);
                CalendarDay::new(date, day_of_week, week_of_year, readings, thresholds)
            })
            .collect())
    }

    fn row_to_stored_reading(row: &rusqlite::Row) -> Result<StoredReading> {
        Ok(StoredReading {
            id: row.get(0)?,
            epoch: row.get(1)?,
            timestamp: row.get(2)?,
            mg_dl: row.get(3)?,
            mmol_l: row.get(4)?,
            raw_value: row.get(5)?,
            status: row.get(6)?,
            range_state: row.get(7)?,
            device_key: row.get(8)?,
            occurrence: row.get(9)?,
            note: row.get(10)?,
            tags: row.get(11)?,
            imported_at: row.get(12)?,
        })
    }
}
