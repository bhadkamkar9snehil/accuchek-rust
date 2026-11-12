//! Synthetic demo data for screenshots and trials. Nothing here is real patient data.
//!
//! `accuchek demo` refuses to run unless `ACCUCHEK_DATA_DIR` points at a throwaway directory,
//! so it can never write into a real database.

use chrono::{Duration, NaiveDate};

use crate::device::{DeviceMetadata, GlucoseReading, ReadingRange};
use crate::error::AccuChekError;
use crate::storage::Storage;

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
    fn gauss(&mut self) -> f64 {
        (0..12).map(|_| self.next()).sum::<f64>() - 6.0
    }
}

/// (hour, hour jitter sd in minutes, base mg/dL, probability a reading is taken)
const SLOTS: [(f64, f64, f64, f64); 5] = [
    (7.0, 35.0, 104.0, 0.95),
    (12.5, 30.0, 116.0, 0.55),
    (14.5, 30.0, 158.0, 0.55),
    (19.5, 40.0, 130.0, 0.6),
    (21.5, 30.0, 168.0, 0.5),
];

pub fn generate(db_path: &str) -> Result<usize, AccuChekError> {
    if std::env::var_os("ACCUCHEK_DATA_DIR").is_none() {
        return Err(AccuChekError::Communication(
            "demo data is only written when ACCUCHEK_DATA_DIR points at a throwaway directory".into(),
        ));
    }
    crate::config::ensure_data_dir()?;
    let storage = Storage::new(db_path)?;
    let device = DeviceMetadata {
        vendor_id: 0x173a,
        product_id: 0x21d5,
        manufacturer: "Demo".into(),
        usb_product: "Demo Meter".into(),
        usb_serial: None,
        model_number: None,
        model_name: "Demo Meter".into(),
        serial_number: Some("DEMO-000000".into()),
        meter_time: None,
        device_key: "demo-0000".into(),
    };
    storage.upsert_device(&device)?;

    let mut rng = Rng(0xACC0_0C4E_5EED_0001);
    let end = NaiveDate::from_ymd_opt(2026, 11, 10).unwrap();
    let days = 92;
    let mut readings: Vec<GlucoseReading> = Vec::new();
    for d in (0..days).rev() {
        let date = end - Duration::days(d);
        let weekend = matches!(date.format("%a").to_string().as_str(), "Sat" | "Sun");
        let wave = 9.0 * ((days - d) as f64 / 11.0).sin();
        let mut day_vals: Vec<(f64, u16)> = Vec::new();
        for (i, (hour, sd, base, p)) in SLOTS.iter().enumerate() {
            if rng.next() > *p && !(i == 0 || day_vals.len() < 1 && i == 4) {
                continue;
            }
            let mut mg = base + wave + if weekend { 12.0 } else { 0.0 } + rng.gauss() * 20.0;
            let r = rng.next();
            if r < 0.035 {
                mg -= 52.0;
            } else if r > 0.965 {
                mg += 110.0;
            }
            let mg = mg.round().clamp(41.0, 400.0) as u16;
            let minutes = ((hour * 60.0) + rng.gauss() * sd).clamp(1.0, 1438.0) as i64;
            day_vals.push((minutes as f64, mg));
        }
        for (minutes, mg) in day_vals {
            let dt = date.and_hms_opt(0, 0, 0).unwrap() + Duration::minutes(minutes as i64);
            readings.push(GlucoseReading {
                id: readings.len(),
                epoch: dt.and_utc().timestamp(),
                timestamp: dt.format("%Y-%m-%dT%H:%M:%S").to_string(),
                mg_dl: mg,
                mmol_l: (mg as f64 / 18.0182 * 10.0).round() / 10.0,
                raw_value: mg,
                status: 0,
                range_state: if mg < 70 { ReadingRange::Low } else if mg > 180 { ReadingRange::High } else { ReadingRange::Normal },
                device_key: device.device_key.clone(),
            });
        }
    }
    let n = storage.import_readings(&readings)?;

    let notes = ["Late dinner", "Walk after lunch", "Skipped breakfast", "Stressful day", "Pizza night", "Felt tired"];
    for (i, r) in storage.get_all_readings()?.iter().enumerate() {
        let hour: u32 = r.timestamp.get(11..13).and_then(|h| h.parse().ok()).unwrap_or(0);
        match hour {
            11..=13 => {
                storage.update_tags(r.id, "before_meal")?;
            }
            14..=15 | 21..=22 => {
                storage.update_tags(r.id, "after_meal")?;
            }
            _ => {}
        }
        if i % 23 == 7 {
            storage.update_note(r.id, notes[(i / 23) % notes.len()])?;
        }
    }
    Ok(n)
}
