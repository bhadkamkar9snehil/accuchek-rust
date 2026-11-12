//! GUI for Accu-Chek Data Manager using egui
//!
//! This module provides a modern, modular GUI organized into components:
//! - Settings management
//! - Dashboard view with summary statistics
//! - Readings list with search and editing
//! - Charts with multiple visualization types
//! - Sync management with device
//! - PDF export functionality

use eframe::egui;
use egui::{vec2, Color32, Rect, Stroke};
use crate::gui_charts::*;
use crate::gui_theme::*;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::fs;
use std::io::Write;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::device::find_and_download_accuchek;
use crate::storage::{Storage, StoredReading};
use crate::units::{GlucoseUnit, Thresholds, GlucoseRange};
use crate::stats::{BasicStats, TimeInRange, DailyStats, HourlyStats, TimeBinStats, HistogramBin, CalendarDay, ExportStatistics};
use crate::export::PdfExporter;

/// Type alias for reading list items (index, label, selected, note, tags)
type ReadingListItem = (usize, String, bool, Option<String>, Option<String>);

// ============= Settings =============

/// Persistent user settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub thresholds: Thresholds,
    pub glucose_unit: GlucoseUnit,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            thresholds: Thresholds::default(),
            glucose_unit: GlucoseUnit::MgDl,
        }
    }
}

impl AppSettings {
    pub fn load() -> Self {
        let path = crate::config::settings_file_path();
        if path.exists() {
            if let Ok(contents) = fs::read_to_string(&path) {
                if let Ok(settings) = serde_json::from_str(&contents) {
                    return settings;
                }
            }
        }
        Self::default()
    }
    
    pub fn save(&self) {
        let path = crate::config::settings_file_path();
        if let Ok(json) = serde_json::to_string_pretty(self) {
            if let Ok(mut file) = fs::File::create(&path) {
                let _ = file.write_all(json.as_bytes());
            }
        }
    }
}

// ============= Sync Management =============

pub enum SyncMessage {
    Started,
    Success { new_count: usize, total_from_device: usize },
    Error(String),
}

#[derive(PartialEq, Clone, Copy)]
enum SyncStatus {
    Idle,
    Syncing,
    Success,
    Error,
}

// ============= Notifications =============

#[derive(Clone)]
struct Notification {
    message: String,
    notification_type: NotificationType,
    created_at: std::time::Instant,
}

#[derive(Clone, Copy, PartialEq)]
enum NotificationType {
    Success,
    Error,
}

impl Notification {
    fn new(message: String, notification_type: NotificationType) -> Self {
        Self { message, notification_type, created_at: std::time::Instant::now() }
    }
    
    fn age(&self) -> f32 {
        self.created_at.elapsed().as_secs_f32()
    }
    
    fn should_dismiss(&self) -> bool {
        self.age() > 3.0
    }
}

// ============= UI State =============

#[derive(PartialEq, Clone, Copy)]
enum Tab {
    Overview,
    Readings,
    Analysis,
    Reports,
}

#[derive(PartialEq, Clone, Copy)]
enum RangeSel {
    D7,
    D14,
    D30,
    D90,
    All,
}

impl RangeSel {
    fn days(self) -> Option<i64> {
        match self {
            RangeSel::D7 => Some(7),
            RangeSel::D14 => Some(14),
            RangeSel::D30 => Some(30),
            RangeSel::D90 => Some(90),
            RangeSel::All => None,
        }
    }
    fn label(self) -> &'static str {
        match self {
            RangeSel::D7 => "7 DAYS",
            RangeSel::D14 => "14 DAYS",
            RangeSel::D30 => "30 DAYS",
            RangeSel::D90 => "90 DAYS",
            RangeSel::All => "ALL TIME",
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum ChartView {
    Trend,
    Profile,
    Distribution,
    Daily,
    Calendar,
}

#[derive(PartialEq, Clone, Copy)]
enum ExportStatus {
    Idle,
    Success,
    Error,
}

// ============= Data Container =============

struct AppData {
    readings: Vec<StoredReading>,
    basic_stats: Option<BasicStats>,
    time_in_range: Option<TimeInRange>,
    daily_stats: Vec<DailyStats>,
    hourly_stats: Vec<HourlyStats>,
    time_bin_stats: Vec<TimeBinStats>,
    histogram_bins: Vec<HistogramBin>,
    calendar_data: Vec<CalendarDay>,
}

impl AppData {
    fn empty() -> Self {
        Self {
            readings: Vec::new(),
            basic_stats: None,
            time_in_range: None,
            daily_stats: Vec::new(),
            hourly_stats: Vec::new(),
            time_bin_stats: Vec::new(),
            histogram_bins: Vec::new(),
            calendar_data: Vec::new(),
        }
    }

    fn load(storage: &Storage, thresholds: Thresholds) -> Self {
        Self {
            readings: storage.get_all_readings().unwrap_or_default(),
            basic_stats: storage.get_basic_stats().ok().flatten(),
            time_in_range: storage.get_time_in_range(thresholds).ok(),
            daily_stats: storage.get_daily_stats(thresholds).unwrap_or_default(),
            hourly_stats: storage.get_hourly_stats().unwrap_or_default(),
            time_bin_stats: storage.get_time_bin_stats().unwrap_or_default(),
            histogram_bins: storage.get_histogram(20).unwrap_or_default(),
            calendar_data: storage.get_calendar_data(thresholds).unwrap_or_default(),
        }
    }
}

// ============= Main Application =============

pub struct AccuChekApp {
    db_path: String,
    data: AppData,
    settings: AppSettings,
    
    // UI state
    current_tab: Tab,
    selected_reading: Option<usize>,
    note_edit_buffer: String,
    tag_edit_buffer: String,
    search_query: String,
    current_chart_view: ChartView,
    show_settings: bool,
    range: RangeSel,
    pts: Vec<Pt>,
    prior: Vec<Pt>,
    days: Vec<Day>,
    status_note: String,
    status_error: bool,
    
    // Sync state
    sync_receiver: Option<Receiver<SyncMessage>>,
    sync_status: SyncStatus,
    last_sync_message: String,
    
    // Export state
    export_status: ExportStatus,
    exported_path: Option<std::path::PathBuf>,
    show_export_dialog: bool,
    
    // Notifications
    notifications: Vec<Notification>,
}

impl Default for AccuChekApp {
    fn default() -> Self {
        let settings = AppSettings::load();
        Self {
            db_path: "accuchek.db".to_string(),
            data: AppData::empty(),
            settings,
            current_tab: Tab::Overview,
            selected_reading: None,
            note_edit_buffer: String::new(),
            tag_edit_buffer: String::new(),
            search_query: String::new(),
            current_chart_view: ChartView::Trend,
            show_settings: false,
            range: RangeSel::D30,
            pts: Vec::new(),
            prior: Vec::new(),
            days: Vec::new(),
            status_note: String::new(),
            status_error: false,
            sync_receiver: None,
            sync_status: SyncStatus::Idle,
            last_sync_message: String::new(),
            export_status: ExportStatus::Idle,
            exported_path: None,
            show_export_dialog: false,
            notifications: Vec::new(),
        }
    }
}

impl AccuChekApp {
    pub fn new(cc: &eframe::CreationContext<'_>, db_path: String) -> Self {
        apply(&cc.egui_ctx);
        
        let settings = AppSettings::load();
        let mut app = Self {
            db_path,
            settings,
            ..Default::default()
        };
        
        app.refresh_data();
        app
    }
    
    fn refresh_data(&mut self) {
        if let Ok(storage) = Storage::new(&self.db_path) {
            self.data = AppData::load(&storage, self.settings.thresholds);
            self.rebuild_view();
        }
    }
    
    fn start_sync(&mut self) {
        if self.sync_status == SyncStatus::Syncing {
            return;
        }
        
        let (tx, rx): (Sender<SyncMessage>, Receiver<SyncMessage>) = channel();
        self.sync_receiver = Some(rx);
        self.sync_status = SyncStatus::Syncing;
        self.last_sync_message = "Connecting to device...".to_string();
        
        let db_path = self.db_path.clone();
        
        thread::spawn(move || {
            let _ = tx.send(SyncMessage::Started);
            
            let config = Config::load(crate::config::config_file_path())
                .or_else(|_| Config::load("config.txt"))
                .unwrap_or_default();
            
            match rusb::Context::new() {
                Ok(context) => {
                    match find_and_download_accuchek(&context, &config, None) {
                        Ok(download) => {
                            let total = download.readings.len();
                            match Storage::new(&db_path) {
                                Ok(storage) => {
                                    let import_result = storage
                                        .upsert_device(&download.device)
                                        .and_then(|_| storage.import_readings(&download.readings));
                                    match import_result {
                                        Ok(new_count) => {
                                            let _ = tx.send(SyncMessage::Success { new_count, total_from_device: total });
                                        }
                                        Err(e) => {
                                            let _ = tx.send(SyncMessage::Error(format!("Database error: {}", e)));
                                        }
                                    }
                                }
                                Err(e) => {
                                    let _ = tx.send(SyncMessage::Error(format!("Cannot open database: {}", e)));
                                }
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(SyncMessage::Error(format!("{}", e)));
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(SyncMessage::Error(format!("USB error: {}", e)));
                }
            }
        });
    }
    
    fn check_sync_status(&mut self) {
        let messages: Vec<SyncMessage> = if let Some(ref rx) = self.sync_receiver {
            let mut msgs = Vec::new();
            while let Ok(msg) = rx.try_recv() {
                msgs.push(msg);
            }
            msgs
        } else {
            Vec::new()
        };
        
        let mut should_refresh = false;
        let mut clear_receiver = false;
        
        for msg in messages {
            match msg {
                SyncMessage::Started => {
                    self.last_sync_message = "Syncing...".to_string();
                }
                SyncMessage::Success { new_count, total_from_device } => {
                    self.sync_status = SyncStatus::Success;
                    self.last_sync_message.clear();
                    self.status_error = false;
                    self.status_note = format!("SYNCED  +{} NEW  {} ON DEVICE", new_count, total_from_device);
                    self.notifications.push(Notification::new(
                        format!("✓ Synced! {} new readings ({} from device)", new_count, total_from_device),
                        NotificationType::Success
                    ));
                    should_refresh = true;
                    clear_receiver = true;
                }
                SyncMessage::Error(e) => {
                    self.sync_status = SyncStatus::Error;
                    self.last_sync_message.clear();
                    self.status_error = true;
                    self.status_note = format!("SYNC ! {}", e);
                    self.notifications.push(Notification::new(
                        format!("✗ Sync Error: {}", e),
                        NotificationType::Error
                    ));
                    clear_receiver = true;
                }
            }
        }
        
        if clear_receiver {
            self.sync_receiver = None;
        }
        if should_refresh {
            self.refresh_data();
        }
    }
    
    fn export_pdf(&mut self) {
        use crate::config::default_export_dir;
        
        let default_name = format!("glucose_report_{}.pdf", chrono::Local::now().format("%Y%m%d"));
        
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("PDF", &["pdf"])
            .set_directory(default_export_dir())
            .set_file_name(&default_name)
            .save_file()
        {
            let export_stats = ExportStatistics::generate(&self.data.readings, self.settings.thresholds);
            let exporter = PdfExporter::new(
                &self.data.readings,
                &export_stats,
                self.settings.thresholds,
                self.settings.glucose_unit,
            );
            
            match exporter.export(&path) {
                Ok(()) => {
                    self.export_status = ExportStatus::Success;
                    self.notifications.push(Notification::new(
                        format!("✓ PDF exported to {}", path.file_name().unwrap_or_default().to_string_lossy()),
                        NotificationType::Success
                    ));
                    self.exported_path = Some(path);
                    self.show_export_dialog = true;
                }
                Err(e) => {
                    self.export_status = ExportStatus::Error;
                    self.notifications.push(Notification::new(
                        format!("✗ Export failed: {}", e),
                        NotificationType::Error
                    ));
                    self.exported_path = None;
                }
            }
        }
    }
    
    fn save_note(&mut self, id: i64, note: &str) {
        if let Ok(storage) = Storage::new(&self.db_path) {
            let _ = storage.update_note(id, note);
            self.refresh_data();
        }
    }
    
    fn save_tags(&mut self, id: i64, tags: &str) {
        if let Ok(storage) = Storage::new(&self.db_path) {
            let _ = storage.update_tags(id, tags);
            self.refresh_data();
        }
    }
    
    fn get_reading_color(&self, mg_dl: u16) -> egui::Color32 {
        match self.settings.thresholds.classify(mg_dl) {
            GlucoseRange::VeryLow => egui::Color32::from_rgb(200, 50, 50),
            GlucoseRange::Low => egui::Color32::from_rgb(255, 100, 100),
            GlucoseRange::InRange => egui::Color32::from_rgb(100, 255, 100),
            GlucoseRange::High => egui::Color32::from_rgb(255, 180, 100),
            GlucoseRange::VeryHigh => egui::Color32::from_rgb(255, 100, 50),
        }
    }
    
    fn filtered_readings(&self) -> Vec<&StoredReading> {
        if self.search_query.is_empty() {
            self.data.readings.iter().collect()
        } else {
            let query = self.search_query.to_lowercase();
            self.data.readings.iter().filter(|r| {
                r.timestamp.to_lowercase().contains(&query) ||
                r.note.as_ref().map(|n| n.to_lowercase().contains(&query)).unwrap_or(false) ||
                r.tags.as_ref().map(|t| t.to_lowercase().contains(&query)).unwrap_or(false) ||
                r.mg_dl.to_string().contains(&query)
            }).collect()
        }
    }
    
    fn add_tag(&mut self, tag: &str) {
        if self.tag_edit_buffer.is_empty() {
            self.tag_edit_buffer = tag.to_string();
        } else if !self.tag_edit_buffer.contains(tag) {
            self.tag_edit_buffer.push(',');
            self.tag_edit_buffer.push_str(tag);
        }
    }
}

impl eframe::App for AccuChekApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.check_sync_status();

        if self.sync_status == SyncStatus::Syncing {
            ctx.request_repaint();
        }

        egui::TopBottomPanel::top("rail")
            .frame(egui::Frame::new().fill(NIGHT).stroke(Stroke::new(1.0, RULE)).inner_margin(egui::Margin::symmetric(16, 10)))
            .show(ctx, |ui| self.show_top_rail(ui));

        egui::TopBottomPanel::bottom("bus")
            .frame(egui::Frame::new().fill(NIGHT).stroke(Stroke::new(1.0, RULE)).inner_margin(egui::Margin::symmetric(16, 6)))
            .show(ctx, |ui| self.show_status_bus(ui));

        egui::SidePanel::left("nav")
            .exact_width(160.0)
            .resizable(false)
            .frame(egui::Frame::new().fill(Color32::from_rgb(0x1C, 0x1C, 0x1B)).stroke(Stroke::new(1.0, RULE)).inner_margin(egui::Margin::symmetric(0, 12)))
            .show(ctx, |ui| self.show_nav(ui));

        if self.current_tab == Tab::Readings {
            egui::SidePanel::right("detail")
                .exact_width(330.0)
                .resizable(false)
                .frame(egui::Frame::new().fill(PLATE).stroke(Stroke::new(1.0, RULE)).inner_margin(egui::Margin::same(16)))
                .show(ctx, |ui| self.show_reading_detail(ui));
        }

        if self.show_settings {
            self.show_settings_window(ctx);
        }
        if self.show_export_dialog {
            self.show_export_dialog_window(ctx);
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(CARBON).inner_margin(egui::Margin::same(20)))
            .show(ctx, |ui| match self.current_tab {
                Tab::Readings => self.show_readings(ui),
                tab => {
                    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| match tab {
                        Tab::Overview => self.show_overview(ui),
                        Tab::Analysis => self.show_analysis(ui),
                        _ => self.show_reports(ui),
                    });
                }
            });

        self.render_notifications(ctx);
    }
}

// ============= View data =============

impl AccuChekApp {
    fn cx(&self) -> Cx {
        Cx { unit: self.settings.glucose_unit, th: self.settings.thresholds }
    }

    fn rebuild_view(&mut self) {
        let all: Vec<Pt> = self.data.readings.iter().map(Pt::from_reading).collect();
        match (self.range.days(), all.last()) {
            (Some(d), Some(last)) => {
                let span = d * 86_400;
                let cutoff = last.epoch - span;
                self.pts = all.iter().filter(|p| p.epoch > cutoff).cloned().collect();
                self.prior = all.iter().filter(|p| p.epoch <= cutoff && p.epoch > cutoff - span).cloned().collect();
            }
            _ => {
                self.pts = all;
                self.prior = Vec::new();
            }
        }
        self.days = group_days(&self.pts, &self.cx());
    }

    fn show_top_rail(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("ACCU-CHEK").font(mono(15.0)).color(PAPER).strong());
            ui.label(egui::RichText::new("LOCAL  /  PERSONAL GLUCOSE INSTRUMENT").font(mono(10.0)).color(INK_MUTED));
            ui.add_space(12.0);
            let (c, t) = match self.sync_status {
                SyncStatus::Idle => (INK_MUTED, "USB IDLE"),
                SyncStatus::Syncing => (TANGERINE, "SYNCING"),
                SyncStatus::Success => (G_OK, "SYNC OK"),
                SyncStatus::Error => (G_LOW, "SYNC FAILED"),
            };
            lamp(ui, c, t);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let syncing = self.sync_status == SyncStatus::Syncing;
                if key(ui, if syncing { "SYNCING..." } else { "SYNC DEVICE" }, true, !syncing) {
                    self.start_sync();
                }
                ui.add_space(8.0);
                if matches!(self.current_tab, Tab::Overview | Tab::Analysis) {
                    let mut r = self.range;
                    let items = [(RangeSel::D7, "7D"), (RangeSel::D14, "14D"), (RangeSel::D30, "30D"), (RangeSel::D90, "90D"), (RangeSel::All, "ALL")];
                    if segmented(ui, &mut r, &items) {
                        self.range = r;
                        self.rebuild_view();
                    }
                    label(ui, "PERIOD");
                }
            });
        });
    }

    fn show_status_bus(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let has = !self.data.readings.is_empty();
            lamp(ui, if has { G_OK } else { INK_MUTED }, if has { "DB HEALTHY" } else { "DB EMPTY" });
            ui.add_space(10.0);
            let n = self.data.readings.len();
            ui.label(egui::RichText::new(format!("{} READINGS STORED", n)).font(mono(10.5)).color(INK_DIM));
            if let Some(l) = self.data.readings.last() {
                ui.add_space(10.0);
                ui.label(egui::RichText::new(format!("LATEST {}", l.timestamp.replace('T', " "))).font(mono(10.5)).color(INK_DIM));
            }
            ui.add_space(10.0);
            let c = if self.status_error { G_LOW } else { INK_DIM };
            let txt = if self.sync_status == SyncStatus::Syncing {
                self.last_sync_message.clone()
            } else {
                self.status_note.clone()
            };
            ui.label(egui::RichText::new(txt).font(mono(10.5)).color(c));
        });
    }

    fn show_nav(&mut self, ui: &mut egui::Ui) {
        for (tab, name, code) in [
            (Tab::Overview, "Overview", "01"),
            (Tab::Readings, "Readings", "02"),
            (Tab::Analysis, "Analysis", "03"),
            (Tab::Reports, "Reports", "04"),
        ] {
            let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 40.0), egui::Sense::click());
            let sel = self.current_tab == tab;
            if sel {
                ui.painter().rect_filled(rect, 0.0, Color32::from_rgb(0x3A, 0x36, 0x2E));
                ui.painter().rect_filled(Rect::from_min_size(rect.min, vec2(3.0, rect.height())), 0.0, TANGERINE);
            } else if resp.hovered() {
                ui.painter().rect_filled(rect, 0.0, Color32::from_rgb(0x26, 0x26, 0x25));
            }
            ui.painter().text(rect.left_center() + vec2(16.0, 0.0), egui::Align2::LEFT_CENTER, code, mono(10.0), if sel { TANGERINE } else { INK_MUTED });
            ui.painter().text(rect.left_center() + vec2(42.0, 0.0), egui::Align2::LEFT_CENTER, name.to_uppercase(), mono(12.0), if sel { PAPER } else { INK_DIM });
            if resp.clicked() {
                self.current_tab = tab;
            }
        }
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            ui.add_space(4.0);
            if key(ui, "SETTINGS", false, true) {
                self.show_settings = !self.show_settings;
            }
            ui.add_space(4.0);
            if key(ui, "REFRESH", false, true) {
                self.refresh_data();
            }
        });
    }

    fn show_empty(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(80.0);
            ui.label(egui::RichText::new("NO READINGS STORED").font(mono(14.0)).color(PAPER).strong());
            ui.add_space(6.0);
            ui.label(egui::RichText::new("Connect the meter by USB, then run a sync. Readings stay on this computer.").color(INK_DIM));
            ui.add_space(16.0);
            if key(ui, "SYNC DEVICE", true, self.sync_status != SyncStatus::Syncing) {
                self.start_sync();
            }
        });
    }

    // ----- Overview -----

    fn show_overview(&mut self, ui: &mut egui::Ui) {
        if self.data.readings.is_empty() {
            self.show_empty(ui);
            return;
        }
        let cx = self.cx();
        let period = self.range.label();
        let latest = self.data.readings.last().cloned().unwrap();
        let class = cx.th.classify(latest.mg_dl);
        let lc = range_color(class);
        let ins = insights(&self.pts, &self.prior, period, &cx);
        let summ = summarize(&self.pts, &cx);
        let psumm = summarize(&self.prior, &cx);
        let hero_val = cx.unit.format_value(latest.mg_dl, latest.mmol_l);

        ui.columns(2, |cols| {
            egui::Frame::new()
                .fill(WELL)
                .stroke(Stroke::new(1.0, RULE))
                .corner_radius(radius())
                .inner_margin(egui::Margin::same(20))
                .show(&mut cols[0], |ui| {
                    ui.set_min_height(150.0);
                    ui.set_width(ui.available_width());
                    label(ui, "Latest reading");
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(hero_val).font(sans(64.0)).color(lc));
                        ui.vertical(|ui| {
                            ui.add_space(30.0);
                            ui.label(egui::RichText::new(cx.unit_label()).font(mono(12.0)).color(INK_DIM));
                            ui.label(egui::RichText::new(class_name(class)).font(mono(11.0)).color(lc).strong());
                        });
                    });
                    evidence(ui, &format!("{}   METER TIME", latest.timestamp.replace('T', " ")));
                });
            plate().show(&mut cols[1], |ui| {
                ui.set_min_height(150.0);
                ui.set_width(ui.available_width());
                label(ui, "What the readings show");
                if let Some(i) = ins.first() {
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(&i.head).font(sans(22.0)).color(PAPER));
                    ui.add_space(4.0);
                    evidence(ui, &i.evidence);
                } else {
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(format!("NO READINGS IN THE LAST {}", period)).font(mono(12.0)).color(INK_DIM));
                    evidence(ui, "CHOOSE A LONGER PERIOD");
                }
            });
        });

        let Some(s) = summ else { return };
        ui.add_space(12.0);

        let (in_pct, in_n) = (s.counts[2] as f64 / s.n as f64 * 100.0, s.counts[2]);
        let delta = |cur: f64, prev: Option<f64>, dec: usize| -> String {
            match prev {
                Some(p) => format!("{:+.*} VS PRIOR {}", dec, cur - p, period),
                None => format!("N={}  {}", s.n, period),
            }
        };
        let dec = if cx.unit == GlucoseUnit::MgDl { 0 } else { 1 };
        ui.columns(4, |cols| {
            metric(&mut cols[0], "Mean", &cx.fmt(s.mean), cx.unit_label(), &delta(s.mean, psumm.as_ref().map(|p| p.mean), dec), CYAN);
            metric(&mut cols[1], "Median", &cx.fmt(s.median), cx.unit_label(), &format!("IQR {}-{}", cx.fmt(s.q1), cx.fmt(s.q3)), CYAN);
            metric(&mut cols[2], "Variability CV", &format!("{:.1}", s.cv), "%", &format!("SD {}  RANGE {}-{}", cx.fmt(s.sd), cx.fmt(s.min), cx.fmt(s.max)), LAVENDER);
            metric(&mut cols[3], "In sampled range", &format!("{:.0}", in_pct), "%", &format!("{} OF {}  {}", in_n, s.n, cx.th.format_range(cx.unit)), G_OK);
        });

        ui.add_space(12.0);
        plate().show(ui, |ui| {
            ui.set_width(ui.available_width());
            label(ui, &format!("Sampled range  {}  N={}", period, s.n));
            ui.add_space(4.0);
            sampled_range(ui, &self.pts, &cx);
        });

        ui.add_space(12.0);
        trend(ui, &self.pts, period, &cx, 280.0);

        ui.add_space(12.0);
        plate().show(ui, |ui| {
            ui.set_width(ui.available_width());
            label(ui, "Patterns found");
            for i in ins.iter().skip(1) {
                ui.add_space(6.0);
                ui.label(egui::RichText::new(&i.head).font(sans(15.0)).color(PAPER));
                evidence(ui, &i.evidence);
            }
            ui.add_space(6.0);
            evidence(ui, "DESCRIPTIVE SUMMARY OF STORED READINGS. NOT MEDICAL ADVICE.");
        });

        ui.add_space(12.0);
        plate().show(ui, |ui| {
            ui.set_width(ui.available_width());
            label(ui, "Recent readings");
            ui.add_space(4.0);
            egui::Grid::new("recent").num_columns(4).spacing([28.0, 6.0]).show(ui, |ui| {
                for h in ["TIME", cx.unit_label(), "STATUS", "NOTE"] {
                    label(ui, h);
                }
                ui.end_row();
                for r in self.data.readings.iter().rev().take(8) {
                    let c = range_color(cx.th.classify(r.mg_dl));
                    ui.label(egui::RichText::new(r.timestamp.replace('T', " ")).font(mono(12.0)).color(INK_DIM));
                    ui.label(egui::RichText::new(cx.unit.format_value(r.mg_dl, r.mmol_l)).font(sans(15.0)).color(c));
                    ui.label(egui::RichText::new(class_name(cx.th.classify(r.mg_dl))).font(mono(10.5)).color(c));
                    ui.label(egui::RichText::new(r.note.as_deref().unwrap_or("-")).color(RASPBERRY));
                    ui.end_row();
                }
            });
        });
    }

    // ----- Analysis -----

    fn show_analysis(&mut self, ui: &mut egui::Ui) {
        if self.data.readings.is_empty() {
            self.show_empty(ui);
            return;
        }
        let cx = self.cx();
        let period = self.range.label();
        let mut v = self.current_chart_view;
        segmented(
            ui,
            &mut v,
            &[
                (ChartView::Trend, "Trend"),
                (ChartView::Profile, "Time of day"),
                (ChartView::Distribution, "Distribution"),
                (ChartView::Daily, "Daily"),
                (ChartView::Calendar, "Calendar"),
            ],
        );
        self.current_chart_view = v;
        ui.add_space(12.0);
        let days_tail = &self.days[self.days.len().saturating_sub(60)..];

        match v {
            ChartView::Trend => {
                trend(ui, &self.pts, period, &cx, 360.0);
                ui.add_space(12.0);
                daily_spread(ui, days_tail, period, &cx, 240.0);
            }
            ChartView::Profile => {
                hourly_profile(ui, &self.pts, period, &cx, 300.0);
                ui.add_space(12.0);
                heatmap(ui, &self.pts, period, &cx);
                ui.add_space(12.0);
                time_blocks(ui, &self.pts, period, &cx, 280.0);
            }
            ChartView::Distribution => {
                distribution(ui, &self.pts, period, &cx, 320.0);
                ui.add_space(12.0);
                plate().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    label(ui, &format!("Sampled range  {}  N={}", period, self.pts.len()));
                    ui.add_space(4.0);
                    sampled_range(ui, &self.pts, &cx);
                });
                if let Some(s) = summarize(&self.pts, &cx) {
                    ui.add_space(8.0);
                    evidence(ui, &format!(
                        "{}   N={}   MEAN {}   MEDIAN {}   IQR {}-{}   SD {}   RANGE {}-{}   {}",
                        period, s.n, cx.fmt(s.mean), cx.fmt(s.median), cx.fmt(s.q1), cx.fmt(s.q3), cx.fmt(s.sd), cx.fmt(s.min), cx.fmt(s.max), cx.unit_label()
                    ));
                }
            }
            ChartView::Daily => {
                daily_composition(ui, days_tail, period, &cx, 300.0);
                ui.add_space(12.0);
                daily_spread(ui, days_tail, period, &cx, 280.0);
            }
            ChartView::Calendar => {
                calendar(ui, &self.days, period, &cx);
            }
        }
    }

    // ----- Reports -----

    fn show_reports(&mut self, ui: &mut egui::Ui) {
        label(ui, "Reports");
        ui.label(egui::RichText::new("Export a printable summary").font(sans(26.0)));
        ui.add_space(12.0);
        let cx = self.cx();
        plate().show(ui, |ui| {
            ui.set_width(ui.available_width());
            label(ui, "Configuration");
            ui.add_space(6.0);
            evidence(ui, &format!("UNIT {}   RANGE {}   N={} READINGS", cx.unit_label(), cx.th.format_range(cx.unit), self.data.readings.len()));
            if let (Some(a), Some(b)) = (self.data.readings.first(), self.data.readings.last()) {
                evidence(ui, &format!("SPAN {}  TO  {}", a.timestamp.get(0..10).unwrap_or(""), b.timestamp.get(0..10).unwrap_or("")));
            }
            ui.add_space(12.0);
            if key(ui, "EXPORT PDF", true, !self.data.readings.is_empty()) {
                self.export_pdf();
            }
        });
    }

    // ----- Readings -----

    fn show_readings(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            label(ui, "Search");
            ui.add(egui::TextEdit::singleline(&mut self.search_query).hint_text("date, value, note or tag").desired_width(280.0));
            if key(ui, "CLEAR", false, !self.search_query.is_empty()) {
                self.search_query.clear();
            }
        });
        ui.add_space(8.0);

        let th = self.settings.thresholds;
        let unit = self.settings.glucose_unit;
        let rows: Vec<(String, u16, f64, bool, bool)> = self
            .filtered_readings()
            .iter()
            .rev()
            .map(|r| (r.timestamp.replace('T', " "), r.mg_dl, r.mmol_l, r.note.is_some(), r.tags.is_some()))
            .collect();
        if rows.is_empty() {
            ui.label(egui::RichText::new("NO READINGS MATCH").font(mono(11.0)).color(INK_MUTED));
            return;
        }
        evidence(ui, &format!("{} READINGS  NEWEST FIRST", rows.len()));
        ui.add_space(4.0);

        let (hr, _) = ui.allocate_exact_size(vec2(ui.available_width(), 24.0), egui::Sense::hover());
        for (x, t) in [(12.0, "METER TIME"), (220.0, unit.label()), (330.0, "STATUS"), (460.0, "MARKS")] {
            ui.painter().text(hr.left_center() + vec2(x, 0.0), egui::Align2::LEFT_CENTER, t.to_uppercase(), mono(10.0), INK_MUTED);
        }
        ui.painter().line_segment([hr.left_bottom(), hr.right_bottom()], Stroke::new(1.0, RULE));

        let mut clicked: Option<usize> = None;
        let selected = self.selected_reading;
        egui::ScrollArea::vertical().auto_shrink([false, false]).show_rows(ui, 28.0, rows.len(), |ui, range| {
            for i in range {
                let (ts, mg, mmol, note, tags) = &rows[i];
                let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 28.0), egui::Sense::click());
                let sel = selected == Some(i);
                if sel {
                    ui.painter().rect_filled(rect, 0.0, Color32::from_rgb(0x3A, 0x36, 0x2E));
                    ui.painter().rect_filled(Rect::from_min_size(rect.min, vec2(3.0, rect.height())), 0.0, TANGERINE);
                } else if resp.hovered() {
                    ui.painter().rect_filled(rect, 0.0, Color32::from_rgb(0x30, 0x30, 0x2F));
                } else if i % 2 == 1 {
                    ui.painter().rect_filled(rect, 0.0, Color32::from_rgb(0x2F, 0x2F, 0x2E));
                }
                let cl = th.classify(*mg);
                let c = range_color(cl);
                let p = ui.painter();
                p.text(rect.left_center() + vec2(12.0, 0.0), egui::Align2::LEFT_CENTER, ts, mono(12.0), INK_DIM);
                p.text(rect.left_center() + vec2(220.0, 0.0), egui::Align2::LEFT_CENTER, unit.format_value(*mg, *mmol), sans(15.0), c);
                p.circle_filled(rect.left_center() + vec2(334.0, 0.0), 3.5, c);
                p.text(rect.left_center() + vec2(346.0, 0.0), egui::Align2::LEFT_CENTER, class_name(cl), mono(10.5), c);
                let mut marks = String::new();
                if *note {
                    marks.push_str("NOTE ");
                }
                if *tags {
                    marks.push_str("TAG");
                }
                p.text(rect.left_center() + vec2(460.0, 0.0), egui::Align2::LEFT_CENTER, marks, mono(10.5), RASPBERRY);
                if resp.clicked() {
                    clicked = Some(i);
                }
            }
        });
        if let Some(i) = clicked {
            self.selected_reading = Some(i);
            let (note, tags) = {
                let f = self.filtered_readings();
                match f.iter().rev().nth(i) {
                    Some(r) => (r.note.clone().unwrap_or_default(), r.tags.clone().unwrap_or_default()),
                    None => (String::new(), String::new()),
                }
            };
            self.note_edit_buffer = note;
            self.tag_edit_buffer = tags;
        }
    }

    fn show_reading_detail(&mut self, ui: &mut egui::Ui) {
        label(ui, "Reading detail");
        let sel = self.selected_reading.and_then(|idx| {
            self.filtered_readings().iter().rev().nth(idx).map(|r| (r.id, r.timestamp.clone(), r.mg_dl, r.mmol_l, r.imported_at.clone(), r.raw_value, r.status))
        });
        let Some((id, ts, mg, mmol, imported, raw, status)) = sel else {
            ui.add_space(40.0);
            ui.label(egui::RichText::new("SELECT A ROW").font(mono(11.0)).color(INK_MUTED));
            return;
        };
        let cl = self.settings.thresholds.classify(mg);
        let c = range_color(cl);
        ui.add_space(6.0);
        ui.label(egui::RichText::new(self.settings.glucose_unit.format_value(mg, mmol)).font(sans(48.0)).color(c));
        ui.label(egui::RichText::new(format!("{}  {}", self.settings.glucose_unit.label(), class_name(cl))).font(mono(11.0)).color(c));
        ui.add_space(8.0);
        evidence(ui, &ts);
        ui.add_space(10.0);
        label(ui, "Provenance");
        evidence(ui, &format!("IMPORTED {}", imported));
        evidence(ui, &format!("RAW {}   STATUS 0x{:04X}", raw, status));
        ui.add_space(12.0);
        label(ui, "Note");
        ui.add(egui::TextEdit::multiline(&mut self.note_edit_buffer).desired_rows(3).desired_width(f32::INFINITY));
        if key(ui, "SAVE NOTE", true, true) {
            let n = self.note_edit_buffer.clone();
            self.save_note(id, &n);
        }
        ui.add_space(10.0);
        label(ui, "Tags (comma-separated)");
        ui.add(egui::TextEdit::singleline(&mut self.tag_edit_buffer).desired_width(f32::INFINITY));
        ui.horizontal_wrapped(|ui| {
            if key(ui, "SAVE TAGS", true, true) {
                let t = self.tag_edit_buffer.clone();
                self.save_tags(id, &t);
            }
            for t in ["fasting", "before_meal", "after_meal"] {
                if key(ui, &format!("+ {}", t), false, true) {
                    self.add_tag(t);
                }
            }
        });
    }
}

fn metric(ui: &mut egui::Ui, name: &str, value: &str, unit: &str, sub: &str, accent: Color32) {
    plate().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(vec2(3.0, 10.0), egui::Sense::hover());
            ui.painter().rect_filled(r, 0.0, accent);
            label(ui, name);
        });
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(value).font(sans(30.0)));
            ui.label(egui::RichText::new(unit).font(mono(11.0)).color(INK_DIM));
        });
        ui.label(egui::RichText::new(sub).font(mono(10.0)).color(INK_MUTED));
    });
}

// ============= UI Components =============

impl AccuChekApp {
    fn show_settings_window(&mut self, ctx: &egui::Context) {
        let mut save_settings = false;
        
        egui::Window::new("Settings")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Display Settings");
                ui.add_space(5.0);
                
                ui.horizontal(|ui| {
                    ui.label("Glucose unit:");
                    if ui.selectable_value(&mut self.settings.glucose_unit, GlucoseUnit::MgDl, "mg/dL").clicked() {
                        save_settings = true;
                    }
                    if ui.selectable_value(&mut self.settings.glucose_unit, GlucoseUnit::MmolL, "mmol/L").clicked() {
                        save_settings = true;
                    }
                });
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(5.0);
                
                ui.heading("Thresholds");
                ui.add_space(5.0);
                
                // Edit thresholds directly in the user's preferred unit
                match self.settings.glucose_unit {
                    GlucoseUnit::MgDl => {
                        let mut low = self.settings.thresholds.low_mgdl as f64;
                        let mut high = self.settings.thresholds.high_mgdl as f64;
                        
                        ui.horizontal(|ui| {
                            ui.label("Low threshold (mg/dL):");
                            if ui.add(egui::DragValue::new(&mut low).range(50.0..=100.0).speed(1.0)).changed() {
                                self.settings.thresholds.low_mgdl = low as u16;
                                save_settings = true;
                            }
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("High threshold (mg/dL):");
                            if ui.add(egui::DragValue::new(&mut high).range(140.0..=250.0).speed(1.0)).changed() {
                                self.settings.thresholds.high_mgdl = high as u16;
                                save_settings = true;
                            }
                        });
                    }
                    GlucoseUnit::MmolL => {
                        let mut low = self.settings.thresholds.low_mmol;
                        let mut high = self.settings.thresholds.high_mmol;
                        
                        ui.horizontal(|ui| {
                            ui.label("Low threshold (mmol/L):");
                            if ui.add(egui::DragValue::new(&mut low).range(2.8..=5.6).speed(0.1).max_decimals(1)).changed() {
                                self.settings.thresholds.low_mmol = low;
                                save_settings = true;
                            }
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("High threshold (mmol/L):");
                            if ui.add(egui::DragValue::new(&mut high).range(7.8..=13.9).speed(0.1).max_decimals(1)).changed() {
                                self.settings.thresholds.high_mmol = high;
                                save_settings = true;
                            }
                        });
                    }
                }
                
                ui.add_space(10.0);
                ui.separator();
                ui.add_space(5.0);
                
                ui.heading("Data Locations");
                ui.add_space(5.0);
                
                egui::Grid::new("paths_grid")
                    .num_columns(2)
                    .spacing([10.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Database:");
                        ui.monospace(&self.db_path);
                        ui.end_row();
                        
                        ui.label("Data folder:");
                        ui.monospace(crate::config::get_data_dir().to_string_lossy().to_string());
                        ui.end_row();
                    });
                
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    if ui.button("Open Data Folder").clicked() {
                        open_folder(&crate::config::get_data_dir());
                    }
                    
                    if ui.button("Close").clicked() {
                        self.show_settings = false;
                    }
                });
            });
        
        if save_settings {
            self.settings.save();
            self.refresh_data();
        }
    }
    
    fn show_export_dialog_window(&mut self, ctx: &egui::Context) {
        if let Some(ref path) = self.exported_path.clone() {
            egui::Window::new("Export Successful")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.label("PDF exported successfully!");
                    ui.add_space(5.0);
                    ui.label(format!("{}", path.display()));
                    ui.add_space(15.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("Open File").clicked() {
                            open_file(path);
                            self.show_export_dialog = false;
                        }
                        
                        if ui.button("Open Folder").clicked() {
                            if let Some(parent) = path.parent() {
                                open_folder(parent);
                            }
                            self.show_export_dialog = false;
                        }
                        
                        if ui.button("Close").clicked() {
                            self.show_export_dialog = false;
                        }
                    });
                });
        }
    }
    
    
    fn render_notifications(&mut self, ctx: &egui::Context) {
        let mut dismiss_indices = Vec::new();
        let available_rect = ctx.available_rect();
        let notification_width = 350.0;
        let notification_spacing = 10.0;
        let right_margin = 20.0;
        let top_margin = 60.0;
        
        for (idx, notification) in self.notifications.iter().enumerate() {
            let y_pos = top_margin + (idx as f32) * (70.0 + notification_spacing);
            let x_pos = available_rect.width() - notification_width - right_margin;
            
            let age = notification.age();
            let alpha = if age > 2.5 {
                ((3.0 - age) / 0.5).max(0.0)
            } else {
                1.0
            };
            
            if notification.should_dismiss() {
                dismiss_indices.push(idx);
                continue;
            }
            
            let (bg_color, text_color) = match notification.notification_type {
                NotificationType::Success => (
                    egui::Color32::from_rgba_unmultiplied(20, 100, 20, (200.0 * alpha) as u8),
                    egui::Color32::from_rgba_unmultiplied(150, 255, 150, (255.0 * alpha) as u8)
                ),
                NotificationType::Error => (
                    egui::Color32::from_rgba_unmultiplied(100, 20, 20, (200.0 * alpha) as u8),
                    egui::Color32::from_rgba_unmultiplied(255, 150, 150, (255.0 * alpha) as u8)
                ),
            };
            
            egui::Area::new(egui::Id::new(format!("notification_{}", idx)))
                .fixed_pos(egui::pos2(x_pos, y_pos))
                .show(ctx, |ui| {
                    egui::Frame::new()
                        .fill(bg_color)
                        .corner_radius(2.0)
                        .inner_margin(15.0)
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, (50.0 * alpha) as u8)))
                        .show(ui, |ui| {
                            ui.set_width(notification_width - 30.0);
                            ui.colored_label(text_color, &notification.message);
                        });
                });
        }
        
        for &idx in dismiss_indices.iter().rev() {
            self.notifications.remove(idx);
        }
        
        if !self.notifications.is_empty() {
            ctx.request_repaint();
        }
    }
}

// ============= Helper Functions =============

fn open_folder(path: &std::path::Path) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer").arg(path).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(path).spawn();
    }
}

fn open_file(path: &std::path::Path) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", &path.to_string_lossy()])
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(path).spawn();
    }
}

/// Run the GUI application
pub fn run_gui(db_path: String) -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 860.0])
            .with_min_inner_size([1000.0, 640.0])
            .with_transparent(false),
        vsync: true,
        multisampling: 0,
        depth_buffer: 0,
        ..Default::default()
    };
    
    eframe::run_native(
        "Accu-Chek Local",
        options,
        Box::new(|cc| {
            Ok(Box::new(AccuChekApp::new(cc, db_path)))
        }),
    )
}
