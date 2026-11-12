//! Custom-painted charts: calibrated display wells, evidence rails, inspection probes.

use eframe::egui::{self, pos2, vec2, Align2, Color32, CornerRadius, Painter, Rect, Shape, Stroke, StrokeKind};

use crate::gui_theme::*;
use crate::storage::StoredReading;
use crate::units::{GlucoseRange, GlucoseUnit, Thresholds};

// ============= Data =============

#[derive(Clone)]
pub struct Pt {
    pub epoch: i64,
    pub ts: String,
    pub day: String,
    pub hour: u8,
    pub mg: u16,
    pub mmol: f64,
}

impl Pt {
    pub fn from_reading(r: &StoredReading) -> Self {
        Self {
            epoch: r.epoch,
            ts: r.timestamp.replace('T', " "),
            day: r.timestamp.get(0..10).unwrap_or("").to_string(),
            hour: r.timestamp.get(11..13).and_then(|h| h.parse().ok()).unwrap_or(0),
            mg: r.mg_dl,
            mmol: r.mmol_l,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Cx {
    pub unit: GlucoseUnit,
    pub th: Thresholds,
}

impl Cx {
    pub fn val(&self, p: &Pt) -> f64 {
        match self.unit {
            GlucoseUnit::MgDl => p.mg as f64,
            GlucoseUnit::MmolL => p.mmol,
        }
    }
    pub fn low(&self) -> f64 {
        match self.unit {
            GlucoseUnit::MgDl => self.th.low_mgdl as f64,
            GlucoseUnit::MmolL => self.th.low_mmol,
        }
    }
    pub fn high(&self) -> f64 {
        match self.unit {
            GlucoseUnit::MgDl => self.th.high_mgdl as f64,
            GlucoseUnit::MmolL => self.th.high_mmol,
        }
    }
    pub fn fmt(&self, v: f64) -> String {
        match self.unit {
            GlucoseUnit::MgDl => format!("{:.0}", v),
            GlucoseUnit::MmolL => format!("{:.1}", v),
        }
    }
    pub fn unit_label(&self) -> &'static str {
        self.unit.label()
    }
    /// Classify a value expressed in the display unit.
    pub fn class_unit(&self, v: f64) -> GlucoseRange {
        let (vl, vh) = match self.unit {
            GlucoseUnit::MgDl => (Thresholds::VERY_LOW_MGDL as f64, Thresholds::VERY_HIGH_MGDL as f64),
            GlucoseUnit::MmolL => (Thresholds::VERY_LOW_MMOL, Thresholds::VERY_HIGH_MMOL),
        };
        if v < vl {
            GlucoseRange::VeryLow
        } else if v < self.low() {
            GlucoseRange::Low
        } else if v <= self.high() {
            GlucoseRange::InRange
        } else if v <= vh {
            GlucoseRange::High
        } else {
            GlucoseRange::VeryHigh
        }
    }
    pub fn class(&self, p: &Pt) -> GlucoseRange {
        self.th.classify(p.mg)
    }
}

pub const CLASSES: [GlucoseRange; 5] = [
    GlucoseRange::VeryLow,
    GlucoseRange::Low,
    GlucoseRange::InRange,
    GlucoseRange::High,
    GlucoseRange::VeryHigh,
];

fn class_idx(r: GlucoseRange) -> usize {
    CLASSES.iter().position(|c| *c == r).unwrap_or(2)
}

pub fn class_name(r: GlucoseRange) -> &'static str {
    match r {
        GlucoseRange::VeryLow => "VERY LOW",
        GlucoseRange::Low => "LOW",
        GlucoseRange::InRange => "IN RANGE",
        GlucoseRange::High => "HIGH",
        GlucoseRange::VeryHigh => "VERY HIGH",
    }
}

pub fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }
    let rank = p / 100.0 * (sorted.len() - 1) as f64;
    let lo = rank.floor() as usize;
    let hi = rank.ceil() as usize;
    sorted[lo] + (sorted[hi] - sorted[lo]) * (rank - lo as f64)
}

fn sorted_vals(pts: &[&Pt], cx: &Cx) -> Vec<f64> {
    let mut v: Vec<f64> = pts.iter().map(|p| cx.val(p)).collect();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    v
}

#[derive(Clone)]
pub struct Day {
    pub date: String,
    pub n: usize,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub counts: [usize; 5],
}

impl Day {
    pub fn in_range_pct(&self) -> f64 {
        if self.n == 0 { 0.0 } else { self.counts[2] as f64 / self.n as f64 * 100.0 }
    }
}

pub fn group_days(pts: &[Pt], cx: &Cx) -> Vec<Day> {
    let mut out: Vec<Day> = Vec::new();
    let mut cur: Vec<&Pt> = Vec::new();
    let mut flush = |cur: &mut Vec<&Pt>, out: &mut Vec<Day>| {
        if cur.is_empty() {
            return;
        }
        let vals: Vec<f64> = cur.iter().map(|p| cx.val(p)).collect();
        let mut counts = [0usize; 5];
        for p in cur.iter() {
            counts[class_idx(cx.class(p))] += 1;
        }
        out.push(Day {
            date: cur[0].day.clone(),
            n: cur.len(),
            mean: vals.iter().sum::<f64>() / vals.len() as f64,
            min: vals.iter().cloned().fold(f64::INFINITY, f64::min),
            max: vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
            counts,
        });
        cur.clear();
    };
    for p in pts {
        if let Some(first) = cur.first() {
            if first.day != p.day {
                flush(&mut cur, &mut out);
            }
        }
        cur.push(p);
    }
    flush(&mut cur, &mut out);
    out
}

pub struct Summary {
    pub n: usize,
    pub mean: f64,
    pub median: f64,
    pub sd: f64,
    pub cv: f64,
    pub min: f64,
    pub max: f64,
    pub q1: f64,
    pub q3: f64,
    pub counts: [usize; 5],
}

pub fn summarize(pts: &[Pt], cx: &Cx) -> Option<Summary> {
    if pts.is_empty() {
        return None;
    }
    let refs: Vec<&Pt> = pts.iter().collect();
    let s = sorted_vals(&refs, cx);
    let n = s.len();
    let mean = s.iter().sum::<f64>() / n as f64;
    let var = s.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
    let sd = var.sqrt();
    let mut counts = [0usize; 5];
    for p in pts {
        counts[class_idx(cx.class(p))] += 1;
    }
    Some(Summary {
        n,
        mean,
        median: percentile(&s, 50.0),
        sd,
        cv: if mean > 0.0 { sd / mean * 100.0 } else { 0.0 },
        min: s[0],
        max: s[n - 1],
        q1: percentile(&s, 25.0),
        q3: percentile(&s, 75.0),
        counts,
    })
}

pub const BLOCKS: [(&str, u8, u8); 6] = [
    ("Overnight", 0, 6),
    ("Morning", 6, 10),
    ("Midday", 10, 14),
    ("Afternoon", 14, 18),
    ("Evening", 18, 22),
    ("Late", 22, 24),
];

const DOW: [&str; 7] = ["MON", "TUE", "WED", "THU", "FRI", "SAT", "SUN"];

fn dow_of(date: &str) -> Option<usize> {
    use chrono::Datelike;
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").ok().map(|d| d.weekday().num_days_from_monday() as usize)
}

// ============= Insights =============

pub struct Insight {
    pub head: String,
    pub evidence: String,
}

pub fn insights(pts: &[Pt], prior: &[Pt], period: &str, cx: &Cx) -> Vec<Insight> {
    let mut out = Vec::new();
    let Some(s) = summarize(pts, cx) else { return out };
    let u = cx.unit_label();

    // Variability
    let steady = s.cv <= 36.0;
    out.push(Insight {
        head: if steady { "READINGS LOOK STEADY".into() } else { "READINGS SWING WIDELY".into() },
        evidence: format!(
            "{}   N={}   MEDIAN {}   IQR {}-{}   CV {:.1}%   SD {}",
            period, s.n, cx.fmt(s.median), cx.fmt(s.q1), cx.fmt(s.q3), s.cv, cx.fmt(s.sd)
        ),
    });

    // Period change
    if let Some(ps) = summarize(prior, cx) {
        let d = s.mean - ps.mean;
        let pct_thresh = ps.mean * 0.03;
        let head = if d.abs() <= pct_thresh {
            "AVERAGE IS ABOUT THE SAME AS THE PRIOR PERIOD".to_string()
        } else if d > 0.0 {
            "AVERAGE IS HIGHER THAN THE PRIOR PERIOD".to_string()
        } else {
            "AVERAGE IS LOWER THAN THE PRIOR PERIOD".to_string()
        };
        let in_now = s.counts[2] as f64 / s.n as f64 * 100.0;
        let in_prev = ps.counts[2] as f64 / ps.n as f64 * 100.0;
        out.push(Insight {
            head,
            evidence: format!(
                "MEAN {} PRIOR {} DELTA {:+}   IN RANGE {:.0}% PRIOR {:.0}%   N={}/{}",
                cx.fmt(s.mean),
                cx.fmt(ps.mean),
                if cx.unit == GlucoseUnit::MgDl { d.round() } else { (d * 10.0).round() / 10.0 },
                in_now,
                in_prev,
                s.n,
                ps.n
            ),
        });
    }

    // Time of day
    let mut blocks: Vec<(&str, f64, usize, usize)> = Vec::new(); // name, mean, n, lows
    for (name, a, b) in BLOCKS {
        let sub: Vec<&Pt> = pts.iter().filter(|p| p.hour >= a && p.hour < b).collect();
        if sub.len() >= 3 {
            let m = sub.iter().map(|p| cx.val(p)).sum::<f64>() / sub.len() as f64;
            let lows = sub.iter().filter(|p| cx.val(p) < cx.low()).count();
            blocks.push((name, m, sub.len(), lows));
        }
    }
    if blocks.len() >= 2 {
        let hi = blocks.iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap();
        let lo = blocks.iter().min_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap();
        out.push(Insight {
            head: format!("READINGS RUN HIGHEST IN THE {}", hi.0.to_uppercase()),
            evidence: format!(
                "{} MEAN {} N={}   {} MEAN {} N={}   {}",
                hi.0.to_uppercase(), cx.fmt(hi.1), hi.2, lo.0.to_uppercase(), cx.fmt(lo.1), lo.2, u
            ),
        });
    }

    // Lows
    let lows: Vec<&Pt> = pts.iter().filter(|p| cx.val(p) < cx.low()).collect();
    if lows.is_empty() {
        out.push(Insight {
            head: format!("NO READINGS BELOW {}", cx.fmt(cx.low())),
            evidence: format!("{}   N={}   LOW LIMIT {} {}", period, s.n, cx.fmt(cx.low()), u),
        });
    } else {
        let worst = blocks.iter().max_by_key(|b| b.3);
        let when = worst.filter(|b| b.3 > 0).map(|b| format!("MOST IN THE {}", b.0.to_uppercase())).unwrap_or_default();
        let lowest = lows.iter().map(|p| cx.val(p)).fold(f64::INFINITY, f64::min);
        out.push(Insight {
            head: format!("{} READING{} BELOW {}", lows.len(), if lows.len() == 1 { "" } else { "S" }, cx.fmt(cx.low())),
            evidence: format!("{}   LOWEST {} {}   {}", period, cx.fmt(lowest), u, when),
        });
    }

    // Day of week
    let mut dows: Vec<(usize, usize, usize)> = vec![(0, 0, 0); 7]; // (n, in_range, _)
    for p in pts {
        if let Some(d) = dow_of(&p.day) {
            dows[d].0 += 1;
            if cx.class(p) == GlucoseRange::InRange {
                dows[d].1 += 1;
            }
        }
    }
    let ranked: Vec<(usize, f64)> = dows
        .iter()
        .enumerate()
        .filter(|(_, d)| d.0 >= 4)
        .map(|(i, d)| (i, d.1 as f64 / d.0 as f64 * 100.0))
        .collect();
    if ranked.len() >= 3 {
        let best = ranked.iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap();
        let worst = ranked.iter().min_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap();
        if best.0 != worst.0 {
            out.push(Insight {
                head: format!("MOST IN-RANGE ON {}, LEAST ON {}", DOW[best.0], DOW[worst.0]),
                evidence: format!(
                    "{} {:.0}% (N={})   {} {:.0}% (N={})",
                    DOW[best.0], best.1, dows[best.0].0, DOW[worst.0], worst.1, dows[worst.0].0
                ),
            });
        }
    }

    // Coverage
    let days = group_days(pts, cx);
    if !days.is_empty() {
        out.push(Insight {
            head: format!("{:.1} READINGS PER SAMPLED DAY", s.n as f64 / days.len() as f64),
            evidence: format!("DAYS WITH READINGS {}   N={}   SAMPLES ONLY, NOT CONTINUOUS", days.len(), s.n),
        });
    }
    out
}

// ============= Frame, axes, probe =============

struct Map {
    r: Rect,
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
}

impl Map {
    fn px(&self, x: f64) -> f32 {
        self.r.left() + ((x - self.x0) / (self.x1 - self.x0).max(1e-9)) as f32 * self.r.width()
    }
    fn py(&self, y: f64) -> f32 {
        self.r.bottom() - ((y - self.y0) / (self.y1 - self.y0).max(1e-9)) as f32 * self.r.height()
    }
}

/// Header rail + display well. Returns (plot rect, painter, hover position).
fn well(ui: &mut egui::Ui, trace: &str, title: &str, evidence_txt: &str, height: f32) -> (Rect, Painter, Option<egui::Pos2>) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(trace).font(mono(10.5)).color(TANGERINE));
        ui.label(egui::RichText::new(title).font(mono(11.5)).color(PAPER).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(evidence_txt).font(mono(10.5)).color(INK_DIM));
        });
    });
    let width = ui.available_width();
    let (resp, painter) = ui.allocate_painter(vec2(width, height), egui::Sense::hover());
    painter.rect(resp.rect, CornerRadius::same(2), WELL, Stroke::new(1.0, RULE), StrokeKind::Inside);
    // inset top shadow
    painter.line_segment([resp.rect.left_top() + vec2(1.0, 1.0), resp.rect.right_top() + vec2(-1.0, 1.0)], Stroke::new(2.0, Color32::from_black_alpha(140)));
    let inner = Rect::from_min_max(
        resp.rect.min + vec2(46.0, 14.0),
        resp.rect.max - vec2(16.0, 26.0),
    );
    (inner, painter, resp.hover_pos())
}

fn empty_msg(painter: &Painter, rect: Rect, msg: &str) {
    painter.text(rect.center(), Align2::CENTER_CENTER, msg, mono(11.5), INK_MUTED);
}

fn nice_ticks(lo: f64, hi: f64, target: usize) -> Vec<f64> {
    let raw = (hi - lo) / target.max(1) as f64;
    let mag = 10f64.powf(raw.max(1e-9).log10().floor());
    let step = [1.0, 2.0, 5.0, 10.0]
        .iter()
        .map(|m| m * mag)
        .find(|s| *s >= raw)
        .unwrap_or(10.0 * mag);
    let mut t = (lo / step).ceil() * step;
    let mut out = Vec::new();
    while t <= hi + step * 1e-6 {
        out.push(t);
        t += step;
    }
    out
}

fn y_range(cx: &Cx, lo: f64, hi: f64) -> (f64, f64) {
    match cx.unit {
        GlucoseUnit::MgDl => {
            let a = (((lo.min(cx.low()) - 10.0) / 20.0).floor() * 20.0).max(0.0);
            let b = ((hi.max(cx.high()) + 10.0) / 20.0).ceil() * 20.0;
            (a, b)
        }
        GlucoseUnit::MmolL => {
            let a = (lo.min(cx.low()) - 0.5).floor().max(0.0);
            let b = (hi.max(cx.high()) + 0.5).ceil();
            (a, b)
        }
    }
}

fn grid_y(painter: &Painter, m: &Map, cx: &Cx, with_unit: bool) {
    for t in nice_ticks(m.y0, m.y1, 5) {
        let y = m.py(t);
        painter.line_segment([pos2(m.r.left(), y), pos2(m.r.right(), y)], Stroke::new(1.0, RULE_SOFT));
        painter.text(pos2(m.r.left() - 6.0, y), Align2::RIGHT_CENTER, cx.fmt(t), mono(10.0), INK_MUTED);
    }
    if with_unit {
        painter.text(pos2(m.r.left() - 6.0, m.r.top() - 11.0), Align2::RIGHT_CENTER, cx.unit_label(), mono(9.0), INK_MUTED);
    }
}

fn target_band(painter: &Painter, m: &Map, cx: &Cx) {
    let top = m.py(cx.high()).max(m.r.top());
    let bot = m.py(cx.low()).min(m.r.bottom());
    painter.rect_filled(Rect::from_min_max(pos2(m.r.left(), top), pos2(m.r.right(), bot)), 0.0, alpha(G_OK, 20));
    for (v, c) in [(cx.high(), G_HIGH), (cx.low(), G_LOW)] {
        let y = m.py(v);
        painter.extend(Shape::dashed_line(&[pos2(m.r.left(), y), pos2(m.r.right(), y)], Stroke::new(1.0, alpha(c, 170)), 5.0, 4.0));
        painter.text(pos2(m.r.right() - 3.0, y - 2.0), Align2::RIGHT_BOTTOM, cx.fmt(v), mono(9.5), alpha(c, 220));
    }
}

fn probe(painter: &Painter, bounds: Rect, anchor: egui::Pos2, lines: &[String], accent: Color32) {
    let font = mono(11.0);
    let galleys: Vec<_> = lines.iter().map(|l| painter.layout_no_wrap(l.clone(), font.clone(), PAPER)).collect();
    let w = galleys.iter().map(|g| g.size().x).fold(0.0, f32::max) + 16.0;
    let h = galleys.iter().map(|g| g.size().y).sum::<f32>() + 12.0;
    let mut x = anchor.x + 14.0;
    if x + w > bounds.right() {
        x = anchor.x - 14.0 - w;
    }
    let y = (anchor.y - h / 2.0).clamp(bounds.top() + 2.0, (bounds.bottom() - h - 2.0).max(bounds.top() + 2.0));
    let r = Rect::from_min_size(pos2(x, y), vec2(w, h));
    painter.line_segment([anchor, pos2(if x > anchor.x { r.left() } else { r.right() }, r.center().y)], Stroke::new(1.0, accent));
    painter.rect(r, CornerRadius::ZERO, NIGHT, Stroke::new(1.0, accent), StrokeKind::Inside);
    let mut ty = r.top() + 6.0;
    for (i, g) in galleys.into_iter().enumerate() {
        let hh = g.size().y;
        let col = if i == 0 { accent } else { PAPER };
        painter.galley(pos2(r.left() + 8.0, ty), g, col);
        ty += hh;
    }
}

fn x_label_dates(painter: &Painter, m: &Map, labels: &[(f32, String)]) {
    for (x, t) in labels {
        painter.line_segment([pos2(*x, m.r.bottom()), pos2(*x, m.r.bottom() + 4.0)], Stroke::new(1.0, RULE));
        painter.text(pos2(*x, m.r.bottom() + 6.0), Align2::CENTER_TOP, t, mono(9.5), INK_MUTED);
    }
}

// ============= Charts =============

/// Time trace of every reading with a rolling 7-day mean.
pub fn trend(ui: &mut egui::Ui, pts: &[Pt], period: &str, cx: &Cx, height: f32) {
    let ev = format!("{}  N={}  {}", period, pts.len(), cx.unit_label());
    let (inner, painter, hover) = well(ui, "T01", "HOW READINGS HAVE MOVED", &ev, height);
    if pts.len() < 2 {
        empty_msg(&painter, inner, "NEED AT LEAST 2 READINGS");
        return;
    }
    let painter = painter.with_clip_rect(inner.expand2(vec2(48.0, 22.0)));
    let vals: Vec<f64> = pts.iter().map(|p| cx.val(p)).collect();
    let (lo, hi) = (vals.iter().cloned().fold(f64::INFINITY, f64::min), vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max));
    let (y0, y1) = y_range(cx, lo, hi);
    let x0 = pts[0].epoch as f64;
    let mut x1 = pts[pts.len() - 1].epoch as f64;
    if x1 <= x0 {
        x1 = x0 + 1.0;
    }
    let m = Map { r: inner, x0, x1, y0, y1 };
    grid_y(&painter, &m, cx, true);
    target_band(&painter, &m, cx);

    // x labels from nearest actual readings
    let mut labels = Vec::new();
    for i in 0..=4 {
        let e = x0 + (x1 - x0) * i as f64 / 4.0;
        if let Some(p) = pts.iter().min_by_key(|p| ((p.epoch as f64 - e).abs()) as i64) {
            labels.push((m.px(e), p.day.get(5..).unwrap_or("").to_string()));
        }
    }
    x_label_dates(&painter, &m, &labels);

    // connecting trace
    let line: Vec<_> = pts.iter().zip(&vals).map(|(p, v)| pos2(m.px(p.epoch as f64), m.py(*v))).collect();
    painter.add(Shape::line(line.clone(), Stroke::new(1.0, alpha(PAPER, 38))));

    // rolling 7d mean
    let window = 7 * 86400;
    let mut roll = Vec::new();
    let (mut start, mut sum) = (0usize, 0.0f64);
    for (i, p) in pts.iter().enumerate() {
        sum += vals[i];
        while pts[start].epoch < p.epoch - window {
            sum -= vals[start];
            start += 1;
        }
        roll.push(pos2(m.px(p.epoch as f64), m.py(sum / (i - start + 1) as f64)));
    }
    painter.add(Shape::line(roll.clone(), Stroke::new(2.5, TANGERINE)));
    if let Some(last) = roll.last() {
        painter.text(*last + vec2(-6.0, -8.0), Align2::RIGHT_BOTTOM, "7D MEAN", mono(9.5), TANGERINE);
    }

    for (i, p) in pts.iter().enumerate() {
        let c = range_color(cx.class(p));
        painter.circle_filled(line[i], 3.2, c);
        painter.circle_stroke(line[i], 3.2, Stroke::new(1.0, WELL));
    }

    if let Some(h) = hover {
        if inner.expand2(vec2(10.0, 10.0)).contains(h) {
            if let Some((i, _)) = line.iter().enumerate().min_by(|a, b| {
                let da = (a.1.x - h.x).powi(2) + ((a.1.y - h.y) * 0.5).powi(2);
                let db = (b.1.x - h.x).powi(2) + ((b.1.y - h.y) * 0.5).powi(2);
                da.partial_cmp(&db).unwrap()
            }) {
                let p = &pts[i];
                painter.line_segment([pos2(line[i].x, inner.top()), pos2(line[i].x, inner.bottom())], Stroke::new(1.0, alpha(TANGERINE, 160)));
                painter.circle_stroke(line[i], 6.0, Stroke::new(2.0, TANGERINE));
                probe(
                    &painter,
                    inner,
                    line[i],
                    &[p.ts.clone(), format!("{} {}", cx.fmt(vals[i]), cx.unit_label()), class_name(cx.class(p)).to_string()],
                    TANGERINE,
                );
            }
        }
    }
}

/// Percentile bands by hour of day (ambulatory-glucose-profile style).
pub fn hourly_profile(ui: &mut egui::Ui, pts: &[Pt], period: &str, cx: &Cx, height: f32) {
    let ev = format!("{}  N={}  MEDIAN · IQR · P10-P90", period, pts.len());
    let (inner, painter, hover) = well(ui, "A02", "WHEN VALUES RISE AND FALL", &ev, height);
    let mut buckets: Vec<Vec<f64>> = vec![Vec::new(); 24];
    for p in pts {
        buckets[p.hour.min(23) as usize].push(cx.val(p));
    }
    // (hour, p10, p25, p50, p75, p90, n)
    let mut rows: Vec<(usize, [f64; 5], usize)> = Vec::new();
    for (h, b) in buckets.iter_mut().enumerate() {
        if b.len() >= 3 {
            b.sort_by(|a, c| a.partial_cmp(c).unwrap());
            rows.push((h, [percentile(b, 10.0), percentile(b, 25.0), percentile(b, 50.0), percentile(b, 75.0), percentile(b, 90.0)], b.len()));
        }
    }
    if rows.len() < 2 {
        empty_msg(&painter, inner, "NEED 3+ READINGS IN AT LEAST 2 HOURS");
        return;
    }
    let lo = rows.iter().map(|r| r.1[0]).fold(f64::INFINITY, f64::min);
    let hi = rows.iter().map(|r| r.1[4]).fold(f64::NEG_INFINITY, f64::max);
    let (y0, y1) = y_range(cx, lo, hi);
    let m = Map { r: inner, x0: 0.0, x1: 23.0, y0, y1 };
    grid_y(&painter, &m, cx, true);
    target_band(&painter, &m, cx);
    for h in (0..24).step_by(3) {
        let x = m.px(h as f64);
        painter.line_segment([pos2(x, inner.top()), pos2(x, inner.bottom())], Stroke::new(1.0, RULE_SOFT));
    }
    x_label_dates(&painter, &m, &(0..24).step_by(3).map(|h| (m.px(h as f64), format!("{:02}h", h))).collect::<Vec<_>>());

    for w in rows.windows(2) {
        if w[1].0 != w[0].0 + 1 {
            continue;
        }
        let (a, b) = (&w[0], &w[1]);
        for (lo_i, hi_i, al) in [(0, 4, 45u8), (1, 3, 95u8)] {
            let q = vec![
                pos2(m.px(a.0 as f64), m.py(a.1[hi_i])),
                pos2(m.px(b.0 as f64), m.py(b.1[hi_i])),
                pos2(m.px(b.0 as f64), m.py(b.1[lo_i])),
                pos2(m.px(a.0 as f64), m.py(a.1[lo_i])),
            ];
            painter.add(Shape::convex_polygon(q, alpha(LAVENDER, al), Stroke::NONE));
        }
    }
    let med: Vec<_> = rows.iter().map(|r| pos2(m.px(r.0 as f64), m.py(r.1[2]))).collect();
    painter.add(Shape::line(med.clone(), Stroke::new(2.5, LAVENDER)));
    for (p, r) in med.iter().zip(&rows) {
        painter.circle_filled(*p, 3.5, range_color(cx.class_unit(r.1[2])));
        painter.circle_stroke(*p, 3.5, Stroke::new(1.0, WELL));
    }

    if let Some(h) = hover {
        if inner.expand2(vec2(10.0, 10.0)).contains(h) {
            if let Some((i, r)) = rows.iter().enumerate().min_by(|a, b| {
                (m.px(a.1 .0 as f64) - h.x).abs().partial_cmp(&(m.px(b.1 .0 as f64) - h.x).abs()).unwrap()
            }) {
                let ax = med[i];
                painter.line_segment([pos2(ax.x, inner.top()), pos2(ax.x, inner.bottom())], Stroke::new(1.0, alpha(LAVENDER, 170)));
                probe(
                    &painter,
                    inner,
                    ax,
                    &[
                        format!("{:02}:00-{:02}:59  N={}", r.0, r.0, r.2),
                        format!("MEDIAN {} {}", cx.fmt(r.1[2]), cx.unit_label()),
                        format!("IQR {}-{}", cx.fmt(r.1[1]), cx.fmt(r.1[3])),
                        format!("P10-P90 {}-{}", cx.fmt(r.1[0]), cx.fmt(r.1[4])),
                    ],
                    LAVENDER,
                );
            }
        }
    }
}

/// Histogram of all values with threshold markers.
pub fn distribution(ui: &mut egui::Ui, pts: &[Pt], period: &str, cx: &Cx, height: f32) {
    let ev = format!("{}  N={}  {}", period, pts.len(), cx.unit_label());
    let (inner, painter, hover) = well(ui, "D03", "READING DISTRIBUTION", &ev, height);
    if pts.is_empty() {
        empty_msg(&painter, inner, "NO READINGS IN THIS PERIOD");
        return;
    }
    let bw = match cx.unit {
        GlucoseUnit::MgDl => 20.0,
        GlucoseUnit::MmolL => 1.0,
    };
    let vals: Vec<f64> = pts.iter().map(|p| cx.val(p)).collect();
    let lo = (vals.iter().cloned().fold(f64::INFINITY, f64::min).min(cx.low()) / bw).floor() * bw;
    let hi = (vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max).max(cx.high()) / bw).floor() * bw + bw;
    let nb = (((hi - lo) / bw).round() as usize).max(1);
    let mut counts = vec![0usize; nb];
    for v in &vals {
        let i = (((v - lo) / bw).floor() as usize).min(nb - 1);
        counts[i] += 1;
    }
    let maxc = *counts.iter().max().unwrap_or(&1) as f64;
    let m = Map { r: inner, x0: lo, x1: hi, y0: 0.0, y1: (maxc * 1.15).max(4.0) };
    for t in nice_ticks(0.0, m.y1, 4) {
        let y = m.py(t);
        painter.line_segment([pos2(inner.left(), y), pos2(inner.right(), y)], Stroke::new(1.0, RULE_SOFT));
        painter.text(pos2(inner.left() - 6.0, y), Align2::RIGHT_CENTER, format!("{:.0}", t), mono(10.0), INK_MUTED);
    }
    let mut hovered: Option<usize> = None;
    for (i, c) in counts.iter().enumerate() {
        let a = lo + i as f64 * bw;
        let r = Rect::from_min_max(pos2(m.px(a) + 1.5, m.py(*c as f64)), pos2(m.px(a + bw) - 1.5, inner.bottom()));
        let col = range_color(cx.class_unit(a + bw / 2.0));
        if hover.map(|h| h.x >= m.px(a) && h.x < m.px(a + bw) && inner.expand2(vec2(0.0, 12.0)).contains(h)).unwrap_or(false) {
            hovered = Some(i);
        }
        if *c > 0 {
            painter.rect_filled(r, CornerRadius { nw: 2, ne: 2, sw: 0, se: 0 }, if hovered == Some(i) { col } else { alpha(col, 215) });
        }
    }
    for t in nice_ticks(lo, hi, 8) {
        painter.text(pos2(m.px(t), inner.bottom() + 6.0), Align2::CENTER_TOP, cx.fmt(t), mono(9.5), INK_MUTED);
    }
    for (v, c, name) in [(cx.low(), G_LOW, "LOW"), (cx.high(), G_HIGH, "HIGH")] {
        let x = m.px(v);
        painter.extend(Shape::dashed_line(&[pos2(x, inner.top()), pos2(x, inner.bottom())], Stroke::new(1.0, alpha(c, 200)), 5.0, 4.0));
        painter.text(pos2(x + 4.0, inner.top() + 2.0), Align2::LEFT_TOP, format!("{} {}", name, cx.fmt(v)), mono(9.5), c);
    }
    if let Some(i) = hovered {
        let a = lo + i as f64 * bw;
        let anchor = pos2(m.px(a + bw / 2.0), m.py(counts[i] as f64));
        probe(
            &painter,
            inner,
            anchor,
            &[
                format!("{}-{} {}", cx.fmt(a), cx.fmt(a + bw), cx.unit_label()),
                format!("{} READINGS", counts[i]),
                format!("{:.1}% OF {}", counts[i] as f64 / vals.len() as f64 * 100.0, vals.len()),
            ],
            TANGERINE,
        );
    }
}

/// Stacked sampled-range composition per day.
pub fn daily_composition(ui: &mut egui::Ui, days: &[Day], period: &str, _cx: &Cx, height: f32) {
    let ev = format!("{}  {} DAYS  % OF READINGS", period, days.len());
    let (inner, painter, hover) = well(ui, "R04", "DAILY SAMPLED RANGE", &ev, height);
    if days.is_empty() {
        empty_msg(&painter, inner, "NO DAYS IN THIS PERIOD");
        return;
    }
    let m = Map { r: inner, x0: 0.0, x1: 100.0, y0: 0.0, y1: 100.0 };
    for t in [0.0, 25.0, 50.0, 75.0, 100.0] {
        let y = m.py(t);
        painter.line_segment([pos2(inner.left(), y), pos2(inner.right(), y)], Stroke::new(1.0, RULE_SOFT));
        painter.text(pos2(inner.left() - 6.0, y), Align2::RIGHT_CENTER, format!("{:.0}%", t), mono(10.0), INK_MUTED);
    }
    let n = days.len() as f32;
    let slot = inner.width() / n;
    let bar_w = (slot - 3.0).clamp(2.0, 28.0);
    let mut hov: Option<usize> = None;
    let step = ((n / 8.0).ceil() as usize).max(1);
    for (i, d) in days.iter().enumerate() {
        let cxp = inner.left() + slot * (i as f32 + 0.5);
        let mut y = inner.bottom();
        for (ci, c) in CLASSES.iter().enumerate() {
            let h = d.counts[ci] as f32 / d.n.max(1) as f32 * inner.height();
            if h <= 0.0 {
                continue;
            }
            let r = Rect::from_min_max(pos2(cxp - bar_w / 2.0, y - h + if h > 3.0 { 1.0 } else { 0.0 }), pos2(cxp + bar_w / 2.0, y));
            painter.rect_filled(r, 0.0, range_color(*c));
            y -= h;
        }
        if i % step == 0 {
            painter.text(pos2(cxp, inner.bottom() + 6.0), Align2::CENTER_TOP, d.date.get(5..).unwrap_or(""), mono(9.5), INK_MUTED);
        }
        if let Some(h) = hover {
            if (h.x - cxp).abs() <= slot / 2.0 && inner.expand2(vec2(0.0, 12.0)).contains(h) {
                hov = Some(i);
            }
        }
    }
    if let Some(i) = hov {
        let d = &days[i];
        let cxp = inner.left() + slot * (i as f32 + 0.5);
        painter.rect_stroke(Rect::from_min_max(pos2(cxp - slot / 2.0, inner.top()), pos2(cxp + slot / 2.0, inner.bottom())), 0.0, Stroke::new(1.0, TANGERINE), StrokeKind::Inside);
        let mut lines = vec![format!("{}  N={}", d.date, d.n)];
        for (ci, c) in CLASSES.iter().enumerate().rev() {
            if d.counts[ci] > 0 {
                lines.push(format!("{:<9} {:>3.0}%  ({})", class_name(*c), d.counts[ci] as f64 / d.n as f64 * 100.0, d.counts[ci]));
            }
        }
        probe(&painter, inner, pos2(cxp, inner.top() + inner.height() * 0.3), &lines, TANGERINE);
    }
}

/// Per-day min-max span with mean marker.
pub fn daily_spread(ui: &mut egui::Ui, days: &[Day], period: &str, cx: &Cx, height: f32) {
    let ev = format!("{}  {} DAYS  MIN · MEAN · MAX", period, days.len());
    let (inner, painter, hover) = well(ui, "S05", "HOW WIDE EACH DAY RANGED", &ev, height);
    if days.is_empty() {
        empty_msg(&painter, inner, "NO DAYS IN THIS PERIOD");
        return;
    }
    let lo = days.iter().map(|d| d.min).fold(f64::INFINITY, f64::min);
    let hi = days.iter().map(|d| d.max).fold(f64::NEG_INFINITY, f64::max);
    let (y0, y1) = y_range(cx, lo, hi);
    let m = Map { r: inner, x0: 0.0, x1: 1.0, y0, y1 };
    grid_y(&painter, &m, cx, true);
    target_band(&painter, &m, cx);
    let n = days.len() as f32;
    let slot = inner.width() / n;
    let step = ((n / 8.0).ceil() as usize).max(1);
    let mut hov = None;
    for (i, d) in days.iter().enumerate() {
        let x = inner.left() + slot * (i as f32 + 0.5);
        painter.line_segment([pos2(x, m.py(d.max)), pos2(x, m.py(d.min))], Stroke::new(3.0, alpha(PAPER, 70)));
        for v in [d.min, d.max] {
            painter.line_segment([pos2(x - 3.0, m.py(v)), pos2(x + 3.0, m.py(v))], Stroke::new(1.5, alpha(PAPER, 140)));
        }
        let c = range_color(cx.class_unit(d.mean));
        painter.circle_filled(pos2(x, m.py(d.mean)), 4.5, c);
        painter.circle_stroke(pos2(x, m.py(d.mean)), 4.5, Stroke::new(1.5, WELL));
        if i % step == 0 {
            painter.text(pos2(x, inner.bottom() + 6.0), Align2::CENTER_TOP, d.date.get(5..).unwrap_or(""), mono(9.5), INK_MUTED);
        }
        if let Some(h) = hover {
            if (h.x - x).abs() <= slot / 2.0 && inner.expand2(vec2(0.0, 12.0)).contains(h) {
                hov = Some((i, x));
            }
        }
    }
    if let Some((i, x)) = hov {
        let d = &days[i];
        painter.line_segment([pos2(x, inner.top()), pos2(x, inner.bottom())], Stroke::new(1.0, alpha(TANGERINE, 160)));
        probe(
            &painter,
            inner,
            pos2(x, m.py(d.mean)),
            &[
                format!("{}  N={}", d.date, d.n),
                format!("MEAN {} {}", cx.fmt(d.mean), cx.unit_label()),
                format!("MIN {}  MAX {}", cx.fmt(d.min), cx.fmt(d.max)),
            ],
            TANGERINE,
        );
    }
}

/// Day-of-week x hour grid of mean values.
pub fn heatmap(ui: &mut egui::Ui, pts: &[Pt], period: &str, cx: &Cx) {
    let ev = format!("{}  N={}  CELL = MEAN, OPACITY = SAMPLES", period, pts.len());
    let (inner, painter, hover) = well(ui, "H06", "WHEN IN THE WEEK READINGS SIT HIGH OR LOW", &ev, 250.0);
    let mut sum = [[0.0f64; 24]; 7];
    let mut cnt = [[0usize; 24]; 7];
    for p in pts {
        if let Some(d) = dow_of(&p.day) {
            sum[d][p.hour.min(23) as usize] += cx.val(p);
            cnt[d][p.hour.min(23) as usize] += 1;
        }
    }
    let inner = Rect::from_min_max(inner.min + vec2(0.0, 0.0), inner.max);
    let cw = inner.width() / 24.0;
    let ch = inner.height() / 7.0;
    let mut hov = None;
    for d in 0..7 {
        painter.text(pos2(inner.left() - 8.0, inner.top() + ch * (d as f32 + 0.5)), Align2::RIGHT_CENTER, DOW[d], mono(10.0), INK_MUTED);
        for h in 0..24 {
            let r = Rect::from_min_size(pos2(inner.left() + cw * h as f32, inner.top() + ch * d as f32), vec2(cw, ch)).shrink(1.0);
            if cnt[d][h] == 0 {
                painter.rect_filled(r, 0.0, RULE_SOFT);
            } else {
                let mean = sum[d][h] / cnt[d][h] as f64;
                let a = (80 + cnt[d][h].min(5) * 35).min(255) as u8;
                painter.rect_filled(r, 0.0, alpha(range_color(cx.class_unit(mean)), a));
            }
            if hover.map(|p: egui::Pos2| r.expand(1.0).contains(p)).unwrap_or(false) {
                hov = Some((d, h, r));
            }
        }
    }
    for h in (0..24).step_by(3) {
        painter.text(pos2(inner.left() + cw * (h as f32 + 0.5), inner.bottom() + 6.0), Align2::CENTER_TOP, format!("{:02}h", h), mono(9.5), INK_MUTED);
    }
    if let Some((d, h, r)) = hov {
        painter.rect_stroke(r, 0.0, Stroke::new(2.0, TANGERINE), StrokeKind::Inside);
        let lines = if cnt[d][h] == 0 {
            vec![format!("{} {:02}h", DOW[d], h), "NO READINGS".to_string()]
        } else {
            let mean = sum[d][h] / cnt[d][h] as f64;
            vec![
                format!("{} {:02}:00-{:02}:59", DOW[d], h, h),
                format!("MEAN {} {}", cx.fmt(mean), cx.unit_label()),
                format!("N={}  {}", cnt[d][h], class_name(cx.class_unit(mean))),
            ]
        };
        probe(&painter, inner, r.center(), &lines, TANGERINE);
    }
}

/// Boxplots for time-of-day blocks.
pub fn time_blocks(ui: &mut egui::Ui, pts: &[Pt], period: &str, cx: &Cx, height: f32) {
    let ev = format!("{}  N={}  P10 · IQR · MEDIAN · P90", period, pts.len());
    let (inner, painter, hover) = well(ui, "B07", "HOW EACH PART OF THE DAY COMPARES", &ev, height);
    let mut data: Vec<(&str, u8, u8, Vec<f64>)> = Vec::new();
    for (name, a, b) in BLOCKS {
        let mut v: Vec<f64> = pts.iter().filter(|p| p.hour >= a && p.hour < b).map(|p| cx.val(p)).collect();
        v.sort_by(|x, y| x.partial_cmp(y).unwrap());
        data.push((name, a, b, v));
    }
    if data.iter().all(|d| d.3.len() < 3) {
        empty_msg(&painter, inner, "NEED 3+ READINGS IN A TIME BLOCK");
        return;
    }
    let lo = data.iter().filter(|d| d.3.len() >= 3).map(|d| percentile(&d.3, 10.0)).fold(f64::INFINITY, f64::min);
    let hi = data.iter().filter(|d| d.3.len() >= 3).map(|d| percentile(&d.3, 90.0)).fold(f64::NEG_INFINITY, f64::max);
    let (y0, y1) = y_range(cx, lo, hi);
    let m = Map { r: inner, x0: 0.0, x1: 1.0, y0, y1 };
    grid_y(&painter, &m, cx, true);
    target_band(&painter, &m, cx);
    let slot = inner.width() / data.len() as f32;
    let mut hov = None;
    for (i, (name, a, b, v)) in data.iter().enumerate() {
        let x = inner.left() + slot * (i as f32 + 0.5);
        painter.text(pos2(x, inner.bottom() + 5.0), Align2::CENTER_TOP, format!("{} {:02}-{:02}", name.to_uppercase(), a, b), mono(9.0), INK_MUTED);
        if v.len() < 3 {
            painter.text(pos2(x, inner.center().y), Align2::CENTER_CENTER, format!("N={}", v.len()), mono(9.5), INK_MUTED);
            continue;
        }
        let (p10, q1, med, q3, p90) = (percentile(v, 10.0), percentile(v, 25.0), percentile(v, 50.0), percentile(v, 75.0), percentile(v, 90.0));
        let col = range_color(cx.class_unit(med));
        painter.line_segment([pos2(x, m.py(p90)), pos2(x, m.py(p10))], Stroke::new(1.5, alpha(PAPER, 150)));
        for yv in [p10, p90] {
            painter.line_segment([pos2(x - 8.0, m.py(yv)), pos2(x + 8.0, m.py(yv))], Stroke::new(1.5, alpha(PAPER, 150)));
        }
        let bw = (slot * 0.5).min(60.0);
        let r = Rect::from_min_max(pos2(x - bw / 2.0, m.py(q3)), pos2(x + bw / 2.0, m.py(q1)));
        painter.rect(r, CornerRadius::same(2), alpha(col, 120), Stroke::new(1.5, col), StrokeKind::Inside);
        painter.line_segment([pos2(r.left(), m.py(med)), pos2(r.right(), m.py(med))], Stroke::new(3.0, PAPER));
        if let Some(h) = hover {
            if (h.x - x).abs() <= slot / 2.0 && inner.expand2(vec2(0.0, 12.0)).contains(h) {
                hov = Some((i, x, med, p10, q1, q3, p90));
            }
        }
    }
    if let Some((i, x, med, p10, q1, q3, p90)) = hov {
        let d = &data[i];
        let mean = d.3.iter().sum::<f64>() / d.3.len() as f64;
        probe(
            &painter,
            inner,
            pos2(x, m.py(med)),
            &[
                format!("{} {:02}:00-{:02}:00  N={}", d.0.to_uppercase(), d.1, d.2, d.3.len()),
                format!("MEDIAN {}  MEAN {}", cx.fmt(med), cx.fmt(mean)),
                format!("IQR {}-{}", cx.fmt(q1), cx.fmt(q3)),
                format!("P10-P90 {}-{}", cx.fmt(p10), cx.fmt(p90)),
            ],
            TANGERINE,
        );
    }
}

/// Calendar grid coloured by in-range share; one cell per day.
pub fn calendar(ui: &mut egui::Ui, days: &[Day], period: &str, cx: &Cx) {
    use chrono::{Datelike, NaiveDate};
    let ev = format!("{}  {} DAYS  FILL = % IN RANGE", period, days.len());
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("C08").font(mono(10.5)).color(TANGERINE));
        ui.label(egui::RichText::new("DAY BY DAY").font(mono(11.5)).color(PAPER).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new(&ev).font(mono(10.5)).color(INK_DIM));
        });
    });
    let parsed: Vec<(NaiveDate, &Day)> = days
        .iter()
        .filter_map(|d| NaiveDate::parse_from_str(&d.date, "%Y-%m-%d").ok().map(|n| (n, d)))
        .collect();
    let width = ui.available_width();
    if parsed.is_empty() {
        let (resp, painter) = ui.allocate_painter(vec2(width, 80.0), egui::Sense::hover());
        painter.rect(resp.rect, CornerRadius::same(2), WELL, Stroke::new(1.0, RULE), StrokeKind::Inside);
        empty_msg(&painter, resp.rect, "NO DAYS IN THIS PERIOD");
        return;
    }
    let first = parsed[0].0;
    let last = parsed[parsed.len() - 1].0;
    let start = first - chrono::Duration::days(first.weekday().num_days_from_monday() as i64);
    let weeks = (((last - start).num_days() / 7) + 1) as usize;
    let weeks = weeks.min(26);
    let start = last - chrono::Duration::days(last.weekday().num_days_from_monday() as i64) - chrono::Duration::days(7 * (weeks as i64 - 1));
    let left = 44.0;
    let top = 22.0;
    let cell_w = ((width - left - 8.0) / 7.0).clamp(40.0, 150.0);
    let cell_h = 44.0;
    let height = top + weeks as f32 * cell_h + 8.0;
    let (resp, painter) = ui.allocate_painter(vec2(width, height), egui::Sense::hover());
    painter.rect(resp.rect, CornerRadius::same(2), WELL, Stroke::new(1.0, RULE), StrokeKind::Inside);
    let origin = resp.rect.min + vec2(left, top);
    for (i, n) in DOW.iter().enumerate() {
        painter.text(origin + vec2(cell_w * (i as f32 + 0.5), -11.0), Align2::CENTER_CENTER, *n, mono(9.5), INK_MUTED);
    }
    let mut hov = None;
    for w in 0..weeks {
        let wk = start + chrono::Duration::days(7 * w as i64);
        painter.text(origin + vec2(-8.0, cell_h * (w as f32 + 0.5)), Align2::RIGHT_CENTER, wk.format("%d %b").to_string().to_uppercase(), mono(9.0), INK_MUTED);
        for d in 0..7 {
            let date = wk + chrono::Duration::days(d);
            let r = Rect::from_min_size(origin + vec2(cell_w * d as f32, cell_h * w as f32), vec2(cell_w, cell_h)).shrink(1.5);
            if let Some((_, day)) = parsed.iter().find(|(n, _)| *n == date) {
                let pct = day.in_range_pct();
                let t = (pct / 100.0) as f32;
                let fill = lerp(RULE_SOFT, G_OK, 0.18 + 0.82 * t);
                painter.rect_filled(r, CornerRadius::same(2), fill);
                let ink = if t > 0.5 { NIGHT } else { PAPER };
                painter.text(r.left_top() + vec2(5.0, 4.0), Align2::LEFT_TOP, format!("{}", date.day()), mono(10.0), alpha(ink, 200));
                painter.text(r.right_bottom() - vec2(5.0, 4.0), Align2::RIGHT_BOTTOM, cx.fmt(day.mean), sans(15.0), ink);
                if hover_in(&resp, r) {
                    hov = Some((r, *day));
                }
            } else {
                painter.rect_filled(r, CornerRadius::same(2), alpha(RULE_SOFT, 120));
                painter.text(r.left_top() + vec2(5.0, 4.0), Align2::LEFT_TOP, format!("{}", date.day()), mono(10.0), alpha(INK_MUTED, 120));
            }
        }
    }
    if let Some((r, d)) = hov {
        painter.rect_stroke(r, CornerRadius::same(2), Stroke::new(2.0, TANGERINE), StrokeKind::Inside);
        probe(
            &painter,
            resp.rect,
            r.center(),
            &[
                format!("{}  N={}", d.date, d.n),
                format!("MEAN {} {}", cx.fmt(d.mean), cx.unit_label()),
                format!("IN RANGE {:.0}%", d.in_range_pct()),
                format!("MIN {}  MAX {}", cx.fmt(d.min), cx.fmt(d.max)),
            ],
            TANGERINE,
        );
    }
}

fn hover_in(resp: &egui::Response, r: Rect) -> bool {
    resp.hover_pos().map(|p| r.contains(p)).unwrap_or(false)
}

fn lerp(a: Color32, b: Color32, t: f32) -> Color32 {
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    Color32::from_rgb(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()))
}

/// Reading beads (every bead is a real reading) + composition bar + legend.
pub fn sampled_range(ui: &mut egui::Ui, pts: &[Pt], cx: &Cx) {
    let mut counts = [0usize; 5];
    for p in pts {
        counts[class_idx(cx.class(p))] += 1;
    }
    let total = pts.len().max(1);
    let width = ui.available_width();

    // Composition bar
    let (resp, painter) = ui.allocate_painter(vec2(width, 20.0), egui::Sense::hover());
    painter.rect(resp.rect, CornerRadius::same(2), WELL, Stroke::new(1.0, RULE), StrokeKind::Inside);
    let inner = resp.rect.shrink(3.0);
    let mut x = inner.left();
    let nonzero = counts.iter().filter(|c| **c > 0).count();
    let gaps = nonzero.saturating_sub(1) as f32 * 2.0;
    for (i, c) in CLASSES.iter().enumerate() {
        if counts[i] == 0 {
            continue;
        }
        let w = (inner.width() - gaps) * counts[i] as f32 / total as f32;
        painter.rect_filled(Rect::from_min_size(pos2(x, inner.top()), vec2(w, inner.height())), 0.0, range_color(*c));
        x += w + 2.0;
    }

    // Legend
    ui.add_space(2.0);
    ui.horizontal_wrapped(|ui| {
        for (i, c) in CLASSES.iter().enumerate() {
            if counts[i] == 0 {
                continue;
            }
            let (r, _) = ui.allocate_exact_size(vec2(10.0, 10.0), egui::Sense::hover());
            ui.painter().rect_filled(r, 0.0, range_color(*c));
            ui.label(
                egui::RichText::new(format!("{} {:.1}%  N={}", class_name(*c), counts[i] as f64 / total as f64 * 100.0, counts[i]))
                    .font(mono(10.5))
                    .color(INK_DIM),
            );
            ui.add_space(8.0);
        }
    });

    // Beads
    let bead = 7.0;
    let gap = 2.0;
    let per_row = ((width - 8.0) / (bead + gap)).floor().max(1.0) as usize;
    let max_rows = 5usize;
    let k = ((pts.len() as f32 / (per_row * max_rows) as f32).ceil() as usize).max(1);
    let mut order: Vec<&Pt> = pts.iter().collect();
    order.sort_by(|a, b| (class_idx(cx.class(a)), a.mg).cmp(&(class_idx(cx.class(b)), b.mg)));
    let n_beads = (order.len() + k - 1) / k;
    let rows = ((n_beads + per_row - 1) / per_row).max(1);
    let h = rows as f32 * (bead + gap) + 8.0;
    let (resp, painter) = ui.allocate_painter(vec2(width, h), egui::Sense::hover());
    painter.rect(resp.rect, CornerRadius::same(2), WELL, Stroke::new(1.0, RULE), StrokeKind::Inside);
    for b in 0..n_beads {
        let (row, col) = (b / per_row, b % per_row);
        let p = order[(b * k).min(order.len() - 1)];
        let r = Rect::from_min_size(
            resp.rect.min + vec2(4.0 + col as f32 * (bead + gap), 4.0 + row as f32 * (bead + gap)),
            vec2(bead, bead),
        );
        painter.rect_filled(r, CornerRadius::same(1), range_color(cx.class(p)));
    }
    ui.label(
        egui::RichText::new(if k == 1 { "1 BEAD = 1 READING".to_string() } else { format!("1 BEAD = {} READINGS", k) })
            .font(mono(9.5))
            .color(INK_MUTED),
    );
}
