//! Custom-painted charts: calibrated display wells, evidence rails, inspection probes.
#![allow(dead_code)]

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
fn chart_well(ui: &mut egui::Ui, trace: &str, title: &str, evidence_txt: &str, height: f32) -> (Rect, Painter, Option<egui::Pos2>) {
    let _ = evidence_txt;
    ui.label(egui::RichText::new(short_title(trace, title)).font(mono(10.5)).color(ink_dim()));
    ui.add_space(-4.0);
    let width = ui.available_width();
    let (resp, painter) = ui.allocate_painter(vec2(width, height), egui::Sense::hover());
    painter.rect(resp.rect, CornerRadius::same(2), well(), Stroke::new(1.0, rule()), StrokeKind::Inside);
    // inset top shadow
    painter.line_segment([resp.rect.left_top() + vec2(1.0, 1.0), resp.rect.right_top() + vec2(-1.0, 1.0)], Stroke::new(2.0, Color32::from_black_alpha(if is_light() { 18 } else { 140 })));
    let inner = Rect::from_min_max(
        resp.rect.min + vec2(46.0, 14.0),
        resp.rect.max - vec2(16.0, 26.0),
    );
    (inner, painter, resp.hover_pos())
}

fn empty_msg(painter: &Painter, rect: Rect, msg: &str) {
    painter.text(rect.center(), Align2::CENTER_CENTER, msg, mono(11.5), ink_muted());
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
        painter.line_segment([pos2(m.r.left(), y), pos2(m.r.right(), y)], Stroke::new(1.0, rule_soft()));
        painter.text(pos2(m.r.left() - 6.0, y), Align2::RIGHT_CENTER, cx.fmt(t), mono(10.0), ink_muted());
    }
    if false && with_unit {
        painter.text(pos2(m.r.left() - 6.0, m.r.top() - 11.0), Align2::RIGHT_CENTER, cx.unit_label(), mono(9.0), ink_muted());
    }
}

fn target_band(painter: &Painter, m: &Map, cx: &Cx) {
    let top = m.py(cx.high()).max(m.r.top());
    let bot = m.py(cx.low()).min(m.r.bottom());
    painter.rect_filled(Rect::from_min_max(pos2(m.r.left(), top), pos2(m.r.right(), bot)), 0.0, alpha(g_ok(), 20));
    for (v, c) in [(cx.high(), g_high()), (cx.low(), g_low())] {
        let y = m.py(v);
        painter.extend(Shape::dashed_line(&[pos2(m.r.left(), y), pos2(m.r.right(), y)], Stroke::new(1.0, alpha(c, 170)), 5.0, 4.0));
        painter.text(pos2(m.r.right() - 3.0, y - 2.0), Align2::RIGHT_BOTTOM, cx.fmt(v), mono(9.5), alpha(c, 220));
    }
}

fn probe(painter: &Painter, bounds: Rect, anchor: egui::Pos2, lines: &[String], accent: Color32) {
    let font = mono(11.0);
    let galleys: Vec<_> = lines.iter().map(|l| painter.layout_no_wrap(l.clone(), font.clone(), paper())).collect();
    let w = galleys.iter().map(|g| g.size().x).fold(0.0, f32::max) + 16.0;
    let h = galleys.iter().map(|g| g.size().y).sum::<f32>() + 12.0;
    let mut x = anchor.x + 14.0;
    if x + w > bounds.right() {
        x = anchor.x - 14.0 - w;
    }
    let y = (anchor.y - h / 2.0).clamp(bounds.top() + 2.0, (bounds.bottom() - h - 2.0).max(bounds.top() + 2.0));
    let r = Rect::from_min_size(pos2(x, y), vec2(w, h));
    painter.line_segment([anchor, pos2(if x > anchor.x { r.left() } else { r.right() }, r.center().y)], Stroke::new(1.0, accent));
    painter.rect(r, CornerRadius::ZERO, night(), Stroke::new(1.0, accent), StrokeKind::Inside);
    let mut ty = r.top() + 6.0;
    for (i, g) in galleys.into_iter().enumerate() {
        let hh = g.size().y;
        let col = if i == 0 { accent } else { paper() };
        painter.galley(pos2(r.left() + 8.0, ty), g, col);
        ty += hh;
    }
}

fn x_label_dates(painter: &Painter, m: &Map, labels: &[(f32, String)]) {
    for (x, t) in labels {
        painter.line_segment([pos2(*x, m.r.bottom()), pos2(*x, m.r.bottom() + 4.0)], Stroke::new(1.0, rule()));
        painter.text(pos2(*x, m.r.bottom() + 6.0), Align2::CENTER_TOP, t, mono(9.5), ink_muted());
    }
}

// ============= Charts =============

/// Time trace of every reading with a rolling 7-day mean.
pub fn trend(ui: &mut egui::Ui, pts: &[Pt], period: &str, cx: &Cx, height: f32) {
    let ev = format!("{}  N={}  {}", period, pts.len(), cx.unit_label());
    let (inner, painter, hover) = chart_well(ui, "T01", "HOW READINGS HAVE MOVED", &ev, height);
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
    painter.add(Shape::line(line.clone(), Stroke::new(1.0, alpha(paper(), 38))));

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
    painter.add(Shape::line(roll.clone(), Stroke::new(2.5, tangerine())));
    if let Some(last) = roll.last() {
    }

    for (i, p) in pts.iter().enumerate() {
        let c = range_color(cx.class(p));
        painter.circle_filled(line[i], 3.2, c);
        painter.circle_stroke(line[i], 3.2, Stroke::new(1.0, well()));
    }

    if let Some(h) = hover {
        if inner.expand2(vec2(10.0, 10.0)).contains(h) {
            if let Some((i, _)) = line.iter().enumerate().min_by(|a, b| {
                let da = (a.1.x - h.x).powi(2) + ((a.1.y - h.y) * 0.5).powi(2);
                let db = (b.1.x - h.x).powi(2) + ((b.1.y - h.y) * 0.5).powi(2);
                da.partial_cmp(&db).unwrap()
            }) {
                let p = &pts[i];
                painter.line_segment([pos2(line[i].x, inner.top()), pos2(line[i].x, inner.bottom())], Stroke::new(1.0, alpha(tangerine(), 160)));
                painter.circle_stroke(line[i], 6.0, Stroke::new(2.0, tangerine()));
                probe(
                    &painter,
                    inner,
                    line[i],
                    &[p.ts.clone(), format!("{} {}", cx.fmt(vals[i]), cx.unit_label()), class_name(cx.class(p)).to_string()],
                    tangerine(),
                );
            }
        }
    }
}

/// Percentile bands by hour of day (ambulatory-glucose-profile style).
pub fn hourly_profile(ui: &mut egui::Ui, pts: &[Pt], period: &str, cx: &Cx, height: f32) {
    let ev = format!("{}  N={}  MEDIAN · IQR · P10-P90", period, pts.len());
    let (inner, painter, hover) = chart_well(ui, "A02", "WHEN VALUES RISE AND FALL", &ev, height);
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
        painter.line_segment([pos2(x, inner.top()), pos2(x, inner.bottom())], Stroke::new(1.0, rule_soft()));
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
            painter.add(Shape::convex_polygon(q, alpha(lavender(), al), Stroke::NONE));
        }
    }
    let med: Vec<_> = rows.iter().map(|r| pos2(m.px(r.0 as f64), m.py(r.1[2]))).collect();
    painter.add(Shape::line(med.clone(), Stroke::new(2.5, lavender())));
    for (p, r) in med.iter().zip(&rows) {
        painter.circle_filled(*p, 3.5, range_color(cx.class_unit(r.1[2])));
        painter.circle_stroke(*p, 3.5, Stroke::new(1.0, well()));
    }

    if let Some(h) = hover {
        if inner.expand2(vec2(10.0, 10.0)).contains(h) {
            if let Some((i, r)) = rows.iter().enumerate().min_by(|a, b| {
                (m.px(a.1 .0 as f64) - h.x).abs().partial_cmp(&(m.px(b.1 .0 as f64) - h.x).abs()).unwrap()
            }) {
                let ax = med[i];
                painter.line_segment([pos2(ax.x, inner.top()), pos2(ax.x, inner.bottom())], Stroke::new(1.0, alpha(lavender(), 170)));
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
                    lavender(),
                );
            }
        }
    }
}

/// Histogram of all values with threshold markers.
pub fn distribution(ui: &mut egui::Ui, pts: &[Pt], period: &str, cx: &Cx, height: f32) {
    let ev = format!("{}  N={}  {}", period, pts.len(), cx.unit_label());
    let (inner, painter, hover) = chart_well(ui, "D03", "READING DISTRIBUTION", &ev, height);
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
        painter.line_segment([pos2(inner.left(), y), pos2(inner.right(), y)], Stroke::new(1.0, rule_soft()));
        painter.text(pos2(inner.left() - 6.0, y), Align2::RIGHT_CENTER, format!("{:.0}", t), mono(10.0), ink_muted());
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
        painter.text(pos2(m.px(t), inner.bottom() + 6.0), Align2::CENTER_TOP, cx.fmt(t), mono(9.5), ink_muted());
    }
    for (v, c, name) in [(cx.low(), g_low(), "LOW"), (cx.high(), g_high(), "HIGH")] {
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
            tangerine(),
        );
    }
}

/// Stacked sampled-range composition per day.
pub fn daily_composition(ui: &mut egui::Ui, days: &[Day], period: &str, _cx: &Cx, height: f32) {
    composition(ui, "R04", "DAILY SAMPLED RANGE", days, period, height);
}

pub fn weekly_composition(ui: &mut egui::Ui, days: &[Day], period: &str, height: f32) {
    composition(ui, "R05", "WEEK BY WEEK SAMPLED RANGE", days, period, height);
}

fn composition(ui: &mut egui::Ui, trace: &str, title: &str, days: &[Day], period: &str, height: f32) {
    let ev = format!("{}  {} BARS  % OF READINGS", period, days.len());
    let (inner, painter, hover) = chart_well(ui, trace, title, &ev, height);
    if days.is_empty() {
        empty_msg(&painter, inner, "NO DAYS IN THIS PERIOD");
        return;
    }
    let m = Map { r: inner, x0: 0.0, x1: 100.0, y0: 0.0, y1: 100.0 };
    for t in [0.0, 25.0, 50.0, 75.0, 100.0] {
        let y = m.py(t);
        painter.line_segment([pos2(inner.left(), y), pos2(inner.right(), y)], Stroke::new(1.0, rule_soft()));
        painter.text(pos2(inner.left() - 6.0, y), Align2::RIGHT_CENTER, format!("{:.0}%", t), mono(10.0), ink_muted());
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
            painter.text(pos2(cxp, inner.bottom() + 6.0), Align2::CENTER_TOP, d.date.get(5..).unwrap_or(""), mono(9.5), ink_muted());
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
        painter.rect_stroke(Rect::from_min_max(pos2(cxp - slot / 2.0, inner.top()), pos2(cxp + slot / 2.0, inner.bottom())), 0.0, Stroke::new(1.0, tangerine()), StrokeKind::Inside);
        let mut lines = vec![format!("{}  N={}", d.date, d.n)];
        for (ci, c) in CLASSES.iter().enumerate().rev() {
            if d.counts[ci] > 0 {
                lines.push(format!("{:<9} {:>3.0}%  ({})", class_name(*c), d.counts[ci] as f64 / d.n as f64 * 100.0, d.counts[ci]));
            }
        }
        probe(&painter, inner, pos2(cxp, inner.top() + inner.height() * 0.3), &lines, tangerine());
    }
}

/// Per-day min-max span with mean marker.
pub fn daily_spread(ui: &mut egui::Ui, days: &[Day], period: &str, cx: &Cx, height: f32) {
    let ev = format!("{}  {} DAYS  MIN · MEAN · MAX", period, days.len());
    let (inner, painter, hover) = chart_well(ui, "S05", "HOW WIDE EACH DAY RANGED", &ev, height);
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
        painter.line_segment([pos2(x, m.py(d.max)), pos2(x, m.py(d.min))], Stroke::new(3.0, alpha(paper(), 70)));
        for v in [d.min, d.max] {
            painter.line_segment([pos2(x - 3.0, m.py(v)), pos2(x + 3.0, m.py(v))], Stroke::new(1.5, alpha(paper(), 140)));
        }
        let c = range_color(cx.class_unit(d.mean));
        painter.circle_filled(pos2(x, m.py(d.mean)), 4.5, c);
        painter.circle_stroke(pos2(x, m.py(d.mean)), 4.5, Stroke::new(1.5, well()));
        if i % step == 0 {
            painter.text(pos2(x, inner.bottom() + 6.0), Align2::CENTER_TOP, d.date.get(5..).unwrap_or(""), mono(9.5), ink_muted());
        }
        if let Some(h) = hover {
            if (h.x - x).abs() <= slot / 2.0 && inner.expand2(vec2(0.0, 12.0)).contains(h) {
                hov = Some((i, x));
            }
        }
    }
    if let Some((i, x)) = hov {
        let d = &days[i];
        painter.line_segment([pos2(x, inner.top()), pos2(x, inner.bottom())], Stroke::new(1.0, alpha(tangerine(), 160)));
        probe(
            &painter,
            inner,
            pos2(x, m.py(d.mean)),
            &[
                format!("{}  N={}", d.date, d.n),
                format!("MEAN {} {}", cx.fmt(d.mean), cx.unit_label()),
                format!("MIN {}  MAX {}", cx.fmt(d.min), cx.fmt(d.max)),
            ],
            tangerine(),
        );
    }
}

/// Day-of-week x hour grid of mean values.
pub fn heatmap(ui: &mut egui::Ui, pts: &[Pt], period: &str, cx: &Cx, height: f32) {
    let ev = format!("{}  N={}  CELL = MEAN, OPACITY = SAMPLES", period, pts.len());
    let (inner, painter, hover) = chart_well(ui, "H06", "WEEK x HOUR", &ev, height);
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
        painter.text(pos2(inner.left() - 8.0, inner.top() + ch * (d as f32 + 0.5)), Align2::RIGHT_CENTER, DOW[d], mono(10.0), ink_muted());
        for h in 0..24 {
            let r = Rect::from_min_size(pos2(inner.left() + cw * h as f32, inner.top() + ch * d as f32), vec2(cw, ch)).shrink(1.0);
            if cnt[d][h] == 0 {
                painter.rect_filled(r, 0.0, rule_soft());
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
        painter.text(pos2(inner.left() + cw * (h as f32 + 0.5), inner.bottom() + 6.0), Align2::CENTER_TOP, format!("{:02}h", h), mono(9.5), ink_muted());
    }
    if let Some((d, h, r)) = hov {
        painter.rect_stroke(r, 0.0, Stroke::new(2.0, tangerine()), StrokeKind::Inside);
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
        probe(&painter, inner, r.center(), &lines, tangerine());
    }
}

/// Boxplots for time-of-day blocks.
pub fn time_blocks(ui: &mut egui::Ui, pts: &[Pt], period: &str, cx: &Cx, height: f32) {
    let ev = format!("{}  N={}  P10 · IQR · MEDIAN · P90", period, pts.len());
    let (inner, painter, hover) = chart_well(ui, "B07", "HOW EACH PART OF THE DAY COMPARES", &ev, height);
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
        painter.text(pos2(x, inner.bottom() + 5.0), Align2::CENTER_TOP, format!("{} {:02}-{:02}", name.to_uppercase(), a, b), mono(9.0), ink_muted());
        if v.len() < 3 {
            painter.text(pos2(x, inner.center().y), Align2::CENTER_CENTER, format!("N={}", v.len()), mono(9.5), ink_muted());
            continue;
        }
        let (p10, q1, med, q3, p90) = (percentile(v, 10.0), percentile(v, 25.0), percentile(v, 50.0), percentile(v, 75.0), percentile(v, 90.0));
        let col = range_color(cx.class_unit(med));
        painter.line_segment([pos2(x, m.py(p90)), pos2(x, m.py(p10))], Stroke::new(1.5, alpha(paper(), 150)));
        for yv in [p10, p90] {
            painter.line_segment([pos2(x - 8.0, m.py(yv)), pos2(x + 8.0, m.py(yv))], Stroke::new(1.5, alpha(paper(), 150)));
        }
        let bw = (slot * 0.5).min(60.0);
        let r = Rect::from_min_max(pos2(x - bw / 2.0, m.py(q3)), pos2(x + bw / 2.0, m.py(q1)));
        painter.rect(r, CornerRadius::same(2), alpha(col, 120), Stroke::new(1.5, col), StrokeKind::Inside);
        painter.line_segment([pos2(r.left(), m.py(med)), pos2(r.right(), m.py(med))], Stroke::new(3.0, paper()));
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
            tangerine(),
        );
    }
}

/// Calendar grid coloured by in-range share; one cell per day.
pub fn calendar(ui: &mut egui::Ui, days: &[Day], period: &str, cx: &Cx) {
    use chrono::{Datelike, NaiveDate};
    let ev = format!("{}  {} DAYS  FILL = % IN RANGE", period, days.len());
    let _ = &ev;
    ui.label(egui::RichText::new("CALENDAR").font(mono(10.5)).color(ink_dim()));
    let parsed: Vec<(NaiveDate, &Day)> = days
        .iter()
        .filter_map(|d| NaiveDate::parse_from_str(&d.date, "%Y-%m-%d").ok().map(|n| (n, d)))
        .collect();
    let width = ui.available_width();
    if parsed.is_empty() {
        let (resp, painter) = ui.allocate_painter(vec2(width, 80.0), egui::Sense::hover());
        painter.rect(resp.rect, CornerRadius::same(2), well(), Stroke::new(1.0, rule()), StrokeKind::Inside);
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
    painter.rect(resp.rect, CornerRadius::same(2), well(), Stroke::new(1.0, rule()), StrokeKind::Inside);
    let origin = resp.rect.min + vec2(left, top);
    for (i, n) in DOW.iter().enumerate() {
        painter.text(origin + vec2(cell_w * (i as f32 + 0.5), -11.0), Align2::CENTER_CENTER, *n, mono(9.5), ink_muted());
    }
    let mut hov = None;
    for w in 0..weeks {
        let wk = start + chrono::Duration::days(7 * w as i64);
        painter.text(origin + vec2(-8.0, cell_h * (w as f32 + 0.5)), Align2::RIGHT_CENTER, wk.format("%d %b").to_string().to_uppercase(), mono(9.0), ink_muted());
        for d in 0..7 {
            let date = wk + chrono::Duration::days(d);
            let r = Rect::from_min_size(origin + vec2(cell_w * d as f32, cell_h * w as f32), vec2(cell_w, cell_h)).shrink(1.5);
            if let Some((_, day)) = parsed.iter().find(|(n, _)| *n == date) {
                let pct = day.in_range_pct();
                let t = (pct / 100.0) as f32;
                let fill = lerp(rule_soft(), g_ok(), 0.18 + 0.82 * t);
                painter.rect_filled(r, CornerRadius::same(2), fill);
                let ink = if t > 0.5 { INK_ON } else { paper() };
                painter.text(r.left_top() + vec2(5.0, 4.0), Align2::LEFT_TOP, format!("{}", date.day()), mono(10.0), alpha(ink, 200));
                painter.text(r.right_bottom() - vec2(5.0, 4.0), Align2::RIGHT_BOTTOM, cx.fmt(day.mean), sans(15.0), ink);
                if hover_in(&resp, r) {
                    hov = Some((r, *day));
                }
            } else {
                painter.rect_filled(r, CornerRadius::same(2), alpha(rule_soft(), 120));
                painter.text(r.left_top() + vec2(5.0, 4.0), Align2::LEFT_TOP, format!("{}", date.day()), mono(10.0), alpha(ink_muted(), 120));
            }
        }
    }
    if let Some((r, d)) = hov {
        painter.rect_stroke(r, CornerRadius::same(2), Stroke::new(2.0, tangerine()), StrokeKind::Inside);
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
            tangerine(),
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
    painter.rect(resp.rect, CornerRadius::same(2), well(), Stroke::new(1.0, rule()), StrokeKind::Inside);
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
                    .color(ink_dim()),
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
    painter.rect(resp.rect, CornerRadius::same(2), well(), Stroke::new(1.0, rule()), StrokeKind::Inside);
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
            .color(ink_muted()),
    );
}

// ============= New visual forms =============

fn hour_frac(p: &Pt) -> f64 {
    p.hour as f64 + p.ts.get(14..16).and_then(|m| m.parse::<f64>().ok()).unwrap_or(0.0) / 60.0
}

pub fn group_weeks(days: &[Day]) -> Vec<Day> {
    use chrono::{Datelike, NaiveDate};
    let mut out: Vec<Day> = Vec::new();
    for d in days {
        let Ok(date) = NaiveDate::parse_from_str(&d.date, "%Y-%m-%d") else { continue };
        let ws = date - chrono::Duration::days(date.weekday().num_days_from_monday() as i64);
        let key = ws.format("%Y-%m-%d").to_string();
        let same = out.last().map(|w| w.date == key).unwrap_or(false);
        if same {
            let w = out.last_mut().unwrap();
            let tot = w.n + d.n;
            w.mean = (w.mean * w.n as f64 + d.mean * d.n as f64) / tot as f64;
            w.n = tot;
            w.min = w.min.min(d.min);
            w.max = w.max.max(d.max);
            for i in 0..5 {
                w.counts[i] += d.counts[i];
            }
        } else {
            out.push(Day { date: key, n: d.n, mean: d.mean, min: d.min, max: d.max, counts: d.counts });
        }
    }
    out
}

/// Tiny trace for metric tiles.
pub fn spark(ui: &mut egui::Ui, vals: &[f64], color: Color32, h: f32) {
    let w = ui.available_width();
    let (resp, painter) = ui.allocate_painter(vec2(w, h), egui::Sense::hover());
    if vals.len() < 2 {
        return;
    }
    let lo = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let hi = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let span = (hi - lo).max(1e-9);
    let r = resp.rect.shrink2(vec2(2.0, 5.0));
    let line: Vec<_> = vals
        .iter()
        .enumerate()
        .map(|(i, v)| pos2(r.left() + r.width() * i as f32 / (vals.len() - 1) as f32, r.bottom() - r.height() * ((v - lo) / span) as f32))
        .collect();
    painter.add(Shape::line(line.clone(), Stroke::new(1.5, color)));
    if let Some(l) = line.last() {
        painter.circle_filled(*l, 3.0, color);
    }
}

/// Last readings as coloured beads on a threshold band.
pub fn reading_strip(ui: &mut egui::Ui, items: &[(f64, GlucoseRange)], cx: &Cx, h: f32) {
    let w = ui.available_width();
    let (resp, painter) = ui.allocate_painter(vec2(w, h), egui::Sense::hover());
    painter.rect(resp.rect, CornerRadius::same(2), night(), Stroke::new(1.0, rule()), StrokeKind::Inside);
    if items.len() < 2 {
        return;
    }
    let lo = items.iter().map(|i| i.0).fold(f64::INFINITY, f64::min);
    let hi = items.iter().map(|i| i.0).fold(f64::NEG_INFINITY, f64::max);
    let (y0, y1) = y_range(cx, lo, hi);
    let r = resp.rect.shrink2(vec2(10.0, 8.0));
    let m = Map { r, x0: 0.0, x1: (items.len() - 1) as f64, y0, y1 };
    target_band(&painter, &m, cx);
    let line: Vec<_> = items.iter().enumerate().map(|(i, it)| pos2(m.px(i as f64), m.py(it.0))).collect();
    painter.add(Shape::line(line.clone(), Stroke::new(1.0, alpha(paper(), 60))));
    for (p, it) in line.iter().zip(items) {
        painter.circle_filled(*p, 3.5, range_color(it.1));
    }
    if let Some(l) = line.last() {
        painter.circle_stroke(*l, 6.0, Stroke::new(2.0, tangerine()));
    }
}

/// Donut of sampled-range composition with a legend.
pub fn range_ring(ui: &mut egui::Ui, pts: &[Pt], cx: &Cx, d: f32) {
    let mut counts = [0usize; 5];
    for p in pts {
        counts[class_idx(cx.class(p))] += 1;
    }
    let total = pts.len().max(1);
    ui.horizontal(|ui| {
        let (resp, painter) = ui.allocate_painter(vec2(d, d), egui::Sense::hover());
        let c = resp.rect.center();
        let th = d * 0.17;
        let rm = d / 2.0 - th / 2.0 - 2.0;
        thick_ring(&painter, c, rm, th, well());
        let nz = counts.iter().filter(|x| **x > 0).count();
        let gap = if nz > 1 { 0.035f32 } else { 0.0 };
        let mut a = -std::f32::consts::FRAC_PI_2;
        for (i, cl) in CLASSES.iter().enumerate() {
            if counts[i] == 0 {
                continue;
            }
            let sweep = std::f32::consts::TAU * counts[i] as f32 / total as f32;
            let (s, e) = (a + gap / 2.0, a + sweep - gap / 2.0);
            if e > s {
                let steps = (((e - s) / 0.05).ceil() as usize).max(2);
                let arc: Vec<_> = (0..=steps)
                    .map(|k| {
                        let t = s + (e - s) * k as f32 / steps as f32;
                        c + vec2(t.cos(), t.sin()) * rm
                    })
                    .collect();
                painter.add(Shape::line(arc, Stroke::new(th, range_color(*cl))));
            }
            a += sweep;
        }
        let pct = counts[2] as f64 / total as f64 * 100.0;
        painter.text(c + vec2(0.0, -4.0), Align2::CENTER_CENTER, format!("{:.0}%", pct), sans(d * 0.2), paper());
        painter.text(c + vec2(0.0, d * 0.13), Align2::CENTER_CENTER, "IN RANGE", mono(9.5), ink_dim());
        ui.add_space(12.0);
        ui.vertical(|ui| {
            ui.add_space(((d - 5.0 * 22.0) / 2.0).max(0.0));
            for (i, cl) in CLASSES.iter().enumerate().rev() {
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(vec2(10.0, 10.0), egui::Sense::hover());
                    ui.painter().rect_filled(r, 0.0, if counts[i] > 0 { range_color(*cl) } else { rule() });
                    ui.label(egui::RichText::new(format!("{:<9}", class_name(*cl))).font(mono(10.5)).color(ink_dim()));
                    ui.label(egui::RichText::new(format!("{:>3.0}%", counts[i] as f64 / total as f64 * 100.0)).font(sans(15.0)));
                    ui.label(egui::RichText::new(format!("N={}", counts[i])).font(mono(10.0)).color(ink_muted()));
                });
            }
        });
    });
}

/// Polar chart: angle = time of day, distance from centre = glucose value.
pub fn day_dial(ui: &mut egui::Ui, pts: &[Pt], period: &str, cx: &Cx, height: f32) {
    let ev = format!("{}  N={}  DOT = READING  LINE = HOURLY MEDIAN", period, pts.len());
    let (inner, painter, hover) = chart_well(ui, "P09", "THE 24-HOUR DIAL", &ev, height);
    if pts.len() < 3 {
        empty_msg(&painter, inner, "NEED AT LEAST 3 READINGS");
        return;
    }
    let vals: Vec<f64> = pts.iter().map(|p| cx.val(p)).collect();
    let lo = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let hi = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let (y0, y1) = y_range(cx, lo, hi);
    let c = inner.center() + vec2(-14.0, 6.0);
    let big = inner.width().min(inner.height()) / 2.0 - 8.0 + 14.0;
    let r0 = big * 0.2;
    let rad = |v: f64| r0 + ((v - y0) / (y1 - y0)) as f32 * (big - r0);
    let ang = |h: f64| (h / 24.0) as f32 * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
    let at = |h: f64, v: f64| {
        let a = ang(h);
        c + vec2(a.cos(), a.sin()) * rad(v)
    };

    // target band + limit rings
    let (rl, rh) = (rad(cx.low()), rad(cx.high()));
    thick_ring(&painter, c, (rl + rh) / 2.0, rh - rl, alpha(g_ok(), 22));
    painter.circle_stroke(c, rl, Stroke::new(1.0, alpha(g_low(), 170)));
    painter.circle_stroke(c, rh, Stroke::new(1.0, alpha(g_high(), 170)));
    for t in nice_ticks(y0, y1, 4) {
        painter.circle_stroke(c, rad(t), Stroke::new(1.0, rule_soft()));
        painter.text(c + vec2(3.0, -rad(t)), Align2::LEFT_BOTTOM, cx.fmt(t), mono(8.5), ink_muted());
    }
    for h in 0..24 {
        let a = ang(h as f64);
        let (s, e) = (c + vec2(a.cos(), a.sin()) * r0, c + vec2(a.cos(), a.sin()) * big);
        painter.line_segment([s, e], Stroke::new(1.0, if h % 6 == 0 { rule() } else { rule_soft() }));
        if h % 3 == 0 {
            let lp = c + vec2(a.cos(), a.sin()) * (big + 11.0);
            painter.text(lp, Align2::CENTER_CENTER, format!("{:02}", h), mono(9.5), if h % 6 == 0 { ink_dim() } else { ink_muted() });
        }
    }
    painter.circle_filled(c, r0 - 2.0, night());
    painter.text(c, Align2::CENTER_CENTER, "24H", mono(9.5), ink_muted());

    // hourly medians
    let mut buckets: Vec<Vec<f64>> = vec![Vec::new(); 24];
    for p in pts {
        buckets[p.hour.min(23) as usize].push(cx.val(p));
    }
    let mut med: Vec<egui::Pos2> = Vec::new();
    for (h, b) in buckets.iter_mut().enumerate() {
        if b.len() >= 2 {
            b.sort_by(|a, c| a.partial_cmp(c).unwrap());
            med.push(at(h as f64 + 0.5, percentile(b, 50.0)));
        }
    }
    if med.len() >= 3 {
        painter.add(Shape::closed_line(med, Stroke::new(2.5, lavender())));
    }

    let dots: Vec<egui::Pos2> = pts.iter().map(|p| at(hour_frac(p), cx.val(p))).collect();
    for (d, p) in dots.iter().zip(pts) {
        painter.circle_filled(*d, 3.0, alpha(range_color(cx.class(p)), 215));
    }
    if let Some(h) = hover {
        if let Some((i, d)) = dots.iter().enumerate().min_by(|a, b| a.1.distance(h).partial_cmp(&b.1.distance(h)).unwrap()) {
            if d.distance(h) < 14.0 {
                let p = &pts[i];
                painter.circle_stroke(*d, 7.0, Stroke::new(2.0, tangerine()));
                probe(&painter, inner, *d, &[p.ts.clone(), format!("{} {}", cx.fmt(cx.val(p)), cx.unit_label()), class_name(cx.class(p)).to_string()], tangerine());
            }
        }
    }
}

/// One row per day, one dot per reading placed by time of day.
pub fn punch_card(ui: &mut egui::Ui, pts: &[Pt], period: &str, cx: &Cx) {
    let mut keys: Vec<&str> = Vec::new();
    for p in pts {
        if keys.last().copied() != Some(p.day.as_str()) {
            keys.push(&p.day);
        }
    }
    let shown: Vec<&str> = keys[keys.len().saturating_sub(31)..].to_vec();
    let rows = shown.len().max(1);
    let height = 44.0 + rows as f32 * 19.0;
    let ev = format!("{}  {} DAYS  N={}  NEWEST AT TOP", period, shown.len(), pts.len());
    let (inner, painter, hover) = chart_well(ui, "K10", "EVERY READING, DAY BY DAY", &ev, height);
    if shown.is_empty() {
        empty_msg(&painter, inner, "NO READINGS IN THIS PERIOD");
        return;
    }
    let rh = inner.height() / rows as f32;
    let m = Map { r: inner, x0: 0.0, x1: 24.0, y0: 0.0, y1: 1.0 };
    for (i, d) in shown.iter().enumerate() {
        let r = rows - 1 - i;
        let y = inner.top() + rh * (r as f32 + 0.5);
        if dow_of(d).map(|w| w >= 5).unwrap_or(false) {
            painter.rect_filled(Rect::from_min_max(pos2(inner.left(), y - rh / 2.0), pos2(inner.right(), y + rh / 2.0)), 0.0, alpha(paper(), 9));
        }
        painter.line_segment([pos2(inner.left(), y), pos2(inner.right(), y)], Stroke::new(1.0, rule_soft()));
        painter.text(pos2(inner.left() - 6.0, y), Align2::RIGHT_CENTER, d.get(5..).unwrap_or(""), mono(9.5), ink_muted());
    }
    for h in (0..=24).step_by(3) {
        let x = m.px(h as f64);
        painter.line_segment([pos2(x, inner.top()), pos2(x, inner.bottom())], Stroke::new(1.0, rule_soft()));
        painter.text(pos2(x, inner.bottom() + 6.0), Align2::CENTER_TOP, format!("{:02}h", h), mono(9.5), ink_muted());
    }
    let mut dots: Vec<(egui::Pos2, &Pt)> = Vec::new();
    for p in pts {
        if let Some(i) = shown.iter().position(|d| *d == p.day) {
            let r = rows - 1 - i;
            let pos = pos2(m.px(hour_frac(p)), inner.top() + rh * (r as f32 + 0.5));
            let rad = (rh * 0.36).clamp(3.0, 6.0);
            painter.circle_filled(pos, rad, range_color(cx.class(p)));
            painter.circle_stroke(pos, rad, Stroke::new(1.0, well()));
            dots.push((pos, p));
        }
    }
    if let Some(h) = hover {
        if let Some((pos, p)) = dots.iter().min_by(|a, b| a.0.distance(h).partial_cmp(&b.0.distance(h)).unwrap()) {
            if pos.distance(h) < 12.0 {
                painter.circle_stroke(*pos, 8.0, Stroke::new(2.0, tangerine()));
                probe(&painter, inner, *pos, &[p.ts.clone(), format!("{} {}", cx.fmt(cx.val(p)), cx.unit_label()), class_name(cx.class(p)).to_string()], tangerine());
            }
        }
    }
}

/// One 100% bar per part of the day: where readings are in or out of range.
pub fn block_bars(ui: &mut egui::Ui, pts: &[Pt], period: &str, cx: &Cx, height: f32) {
    let ev = format!("{}  N={}  SHARE OF READINGS", period, pts.len());
    let (inner, painter, hover) = chart_well(ui, "Q11", "WHERE EACH PART OF THE DAY LANDS", &ev, height);
    let full = Rect::from_min_max(inner.min - vec2(46.0, 14.0), inner.max + vec2(16.0, 26.0));
    let rows = BLOCKS.len() as f32;
    let rh = (full.height() - 20.0) / rows;
    let bx0 = full.left() + 130.0;
    let bx1 = full.right() - 150.0;
    let mut hov = None;
    for (i, (name, a, b)) in BLOCKS.iter().enumerate() {
        let sub: Vec<&Pt> = pts.iter().filter(|p| p.hour >= *a && p.hour < *b).collect();
        let y = full.top() + 10.0 + rh * (i as f32 + 0.5);
        painter.text(pos2(full.left() + 14.0, y - 6.0), Align2::LEFT_CENTER, name.to_uppercase(), mono(10.5), paper());
        painter.text(pos2(full.left() + 14.0, y + 7.0), Align2::LEFT_CENTER, format!("{:02}-{:02}h", a, b), mono(9.0), ink_muted());
        let bar = Rect::from_min_max(pos2(bx0, y - rh * 0.3), pos2(bx1, y + rh * 0.3));
        painter.rect_filled(bar, 0.0, rule_soft());
        if sub.is_empty() {
            painter.text(bar.center(), Align2::CENTER_CENTER, "NO READINGS", mono(9.0), ink_muted());
            continue;
        }
        let mut counts = [0usize; 5];
        for p in &sub {
            counts[class_idx(cx.class(p))] += 1;
        }
        let mut x = bar.left();
        for (ci, c) in CLASSES.iter().enumerate() {
            if counts[ci] == 0 {
                continue;
            }
            let w = bar.width() * counts[ci] as f32 / sub.len() as f32;
            painter.rect_filled(Rect::from_min_size(pos2(x, bar.top()), vec2((w - 1.0).max(1.0), bar.height())), 0.0, range_color(*c));
            if w > 30.0 {
                painter.text(pos2(x + w / 2.0, bar.center().y), Align2::CENTER_CENTER, format!("{:.0}%", counts[ci] as f64 / sub.len() as f64 * 100.0), mono(9.5), INK_ON);
            }
            x += w;
        }
        let mean = sub.iter().map(|p| cx.val(p)).sum::<f64>() / sub.len() as f64;
        painter.text(pos2(bx1 + 12.0, y - 6.0), Align2::LEFT_CENTER, format!("MEAN {}", cx.fmt(mean)), mono(10.0), ink_dim());
        painter.text(pos2(bx1 + 12.0, y + 7.0), Align2::LEFT_CENTER, format!("N={}", sub.len()), mono(9.0), ink_muted());
        if let Some(h) = hover {
            if bar.expand2(vec2(0.0, 4.0)).contains(h) {
                hov = Some((bar, counts, sub.len(), name));
            }
        }
    }
    if let Some((bar, counts, n, name)) = hov {
        let mut lines = vec![format!("{}  N={}", name.to_uppercase(), n)];
        for (ci, c) in CLASSES.iter().enumerate().rev() {
            if counts[ci] > 0 {
                lines.push(format!("{:<9} {:>3.0}%  ({})", class_name(*c), counts[ci] as f64 / n as f64 * 100.0, counts[ci]));
            }
        }
        probe(&painter, full, bar.center(), &lines, tangerine());
    }
}

/// Highest and lowest readings of the period.
pub fn extremes(ui: &mut egui::Ui, pts: &[Pt], period: &str, cx: &Cx) {
    let mut sorted: Vec<&Pt> = pts.iter().collect();
    sorted.sort_by(|a, b| cx.val(b).partial_cmp(&cx.val(a)).unwrap());
    let top: Vec<&Pt> = sorted.iter().take(5).cloned().collect();
    let bottom: Vec<&Pt> = sorted.iter().rev().take(5).cloned().collect();
    plate_frame().show(ui, |ui| {
        ui.set_width(ui.available_width());
        label(ui, &format!("Extremes  {}  N={}", period, pts.len()));
        ui.add_space(6.0);
        ui.columns(2, |cols| {
            for (col, (name, list)) in cols.iter_mut().zip([("Highest", &top), ("Lowest", &bottom)]) {
                label(col, name);
                for p in list.iter() {
                    col.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("{:>5}", cx.fmt(cx.val(p)))).font(sans(17.0)).color(range_color(cx.class(p))));
                        ui.label(egui::RichText::new(&p.ts).font(mono(10.5)).color(ink_dim()));
                    });
                }
            }
        });
    });
}

fn thick_ring(painter: &Painter, c: egui::Pos2, r: f32, th: f32, col: Color32) {
    let n = 120;
    let pts: Vec<_> = (0..n)
        .map(|k| {
            let t = k as f32 / n as f32 * std::f32::consts::TAU;
            c + vec2(t.cos(), t.sin()) * r
        })
        .collect();
    painter.add(Shape::closed_line(pts, Stroke::new(th, col)));
}

/// Min / IQR / median / max on one track for metric tiles.
pub fn bullet(ui: &mut egui::Ui, v: [f64; 5], color: Color32, h: f32) {
    let w = ui.available_width();
    let (resp, painter) = ui.allocate_painter(vec2(w, h), egui::Sense::hover());
    let r = resp.rect.shrink2(vec2(4.0, 6.0));
    let span = (v[4] - v[0]).max(1e-9);
    let x = |val: f64| r.left() + r.width() * ((val - v[0]) / span) as f32;
    let cy = r.center().y;
    painter.line_segment([pos2(x(v[0]), cy), pos2(x(v[4]), cy)], Stroke::new(2.0, rule()));
    for e in [v[0], v[4]] {
        painter.line_segment([pos2(x(e), cy - 6.0), pos2(x(e), cy + 6.0)], Stroke::new(2.0, ink_muted()));
    }
    let b = Rect::from_min_max(pos2(x(v[1]), cy - 9.0), pos2(x(v[3]).max(x(v[1]) + 2.0), cy + 9.0));
    painter.rect(b, CornerRadius::same(2), alpha(color, 90), Stroke::new(1.5, color), StrokeKind::Inside);
    painter.line_segment([pos2(x(v[2]), cy - 11.0), pos2(x(v[2]), cy + 11.0)], Stroke::new(3.0, paper()));
}

// ============= Compact KPI visuals =============

pub enum KpiViz<'a> {
    None,
    Line(&'a [f64]),
    Bullet([f64; 5]),
    Strip(&'a [(f64, GlucoseRange)]),
}

fn spark_in(painter: &Painter, r: Rect, vals: &[f64], color: Color32) {
    if vals.len() < 2 {
        return;
    }
    let lo = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let hi = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let span = (hi - lo).max(1e-9);
    let line: Vec<_> = vals
        .iter()
        .enumerate()
        .map(|(i, v)| pos2(r.left() + r.width() * i as f32 / (vals.len() - 1) as f32, r.bottom() - r.height() * ((v - lo) / span) as f32))
        .collect();
    for w in line.windows(2) {
        let q = vec![w[0], w[1], pos2(w[1].x, r.bottom()), pos2(w[0].x, r.bottom())];
        painter.add(Shape::convex_polygon(q, alpha(color, 28), Stroke::NONE));
    }
    painter.add(Shape::line(line.clone(), Stroke::new(2.0, color)));
    if let Some(l) = line.last() {
        painter.circle_filled(*l, 3.5, color);
    }
}

fn bullet_in(painter: &Painter, r: Rect, v: [f64; 5], color: Color32) {
    let span = (v[4] - v[0]).max(1e-9);
    let x = |val: f64| r.left() + r.width() * ((val - v[0]) / span) as f32;
    let cy = r.center().y;
    painter.line_segment([pos2(x(v[0]), cy), pos2(x(v[4]), cy)], Stroke::new(2.0, rule()));
    for e in [v[0], v[4]] {
        painter.line_segment([pos2(x(e), cy - 7.0), pos2(x(e), cy + 7.0)], Stroke::new(2.0, ink_muted()));
    }
    let b = Rect::from_min_max(pos2(x(v[1]), cy - 11.0), pos2(x(v[3]).max(x(v[1]) + 2.0), cy + 11.0));
    painter.rect(b, CornerRadius::same(2), alpha(color, 90), Stroke::new(1.5, color), StrokeKind::Inside);
    painter.line_segment([pos2(x(v[2]), cy - 14.0), pos2(x(v[2]), cy + 14.0)], Stroke::new(3.0, paper()));
}

fn strip_in(painter: &Painter, r: Rect, items: &[(f64, GlucoseRange)], cx: &Cx) {
    if items.len() < 2 {
        return;
    }
    let lo = items.iter().map(|i| i.0).fold(f64::INFINITY, f64::min);
    let hi = items.iter().map(|i| i.0).fold(f64::NEG_INFINITY, f64::max);
    let (y0, y1) = y_range(cx, lo, hi);
    let m = Map { r, x0: 0.0, x1: (items.len() - 1) as f64, y0, y1 };
    let top = m.py(cx.high()).max(r.top());
    let bot = m.py(cx.low()).min(r.bottom());
    painter.rect_filled(Rect::from_min_max(pos2(r.left(), top), pos2(r.right(), bot)), 0.0, alpha(g_ok(), 24));
    let line: Vec<_> = items.iter().enumerate().map(|(i, it)| pos2(m.px(i as f64), m.py(it.0))).collect();
    painter.add(Shape::line(line.clone(), Stroke::new(1.0, alpha(paper(), 70))));
    for (p, it) in line.iter().zip(items) {
        painter.circle_filled(*p, 3.2, range_color(it.1));
    }
    if let Some(l) = line.last() {
        painter.circle_stroke(*l, 6.0, Stroke::new(2.0, tangerine()));
    }
}

/// Reduced-height KPI tile: small label, large number, trend visual.
pub fn kpi(ui: &mut egui::Ui, name: &str, value: &str, accent: Color32, value_color: Color32, viz: KpiViz, cx: &Cx, h: f32) {
    let w = ui.available_width();
    let (resp, painter) = ui.allocate_painter(vec2(w, h), egui::Sense::hover());
    let r = resp.rect;
    painter.rect(r, CornerRadius::same(2), plate(), Stroke::new(1.0, rule()), StrokeKind::Inside);
    painter.rect_filled(Rect::from_min_size(r.min, vec2(3.0, r.height())), 0.0, accent);
    painter.text(r.left_top() + vec2(14.0, 8.0), Align2::LEFT_TOP, name, mono(9.5), ink_muted());
    painter.text(r.left_bottom() + vec2(14.0, -6.0), Align2::LEFT_BOTTOM, value, sans(38.0), value_color);
    let vr = Rect::from_min_max(pos2(r.left() + w * 0.48, r.top() + 14.0), pos2(r.right() - 14.0, r.bottom() - 14.0));
    match viz {
        KpiViz::None => {}
        KpiViz::Line(v) => spark_in(&painter, vr, v, accent),
        KpiViz::Bullet(v) => bullet_in(&painter, vr, v, accent),
        KpiViz::Strip(v) => strip_in(&painter, vr, v, cx),
    }
}

/// 10x10 waffle: each cell is 1% of readings; low at the bottom, high at the top.
pub fn range_waffle(ui: &mut egui::Ui, pts: &[Pt], cx: &Cx, height: f32) {
    let (inner, painter, _) = chart_well(ui, "W12", "RANGE", "", height);
    if pts.is_empty() {
        empty_msg(&painter, inner, "-");
        return;
    }
    let total = pts.len();
    let mut counts = [0usize; 5];
    for p in pts {
        counts[class_idx(cx.class(p))] += 1;
    }
    let mut cells = [0usize; 5];
    let mut rem: Vec<(f64, usize)> = Vec::new();
    for i in 0..5 {
        let exact = counts[i] as f64 / total as f64 * 100.0;
        cells[i] = exact.floor() as usize;
        rem.push((exact - exact.floor(), i));
    }
    let mut left = 100usize.saturating_sub(cells.iter().sum::<usize>());
    rem.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    for (_, i) in rem {
        if left == 0 {
            break;
        }
        if counts[i] > 0 {
            cells[i] += 1;
            left -= 1;
        }
    }
    let side = inner.height().min(inner.width() * 0.6);
    let gap = 2.0;
    let cs = (side - gap * 9.0) / 10.0;
    let origin = pos2(inner.left(), inner.center().y - side / 2.0);
    let mut k = 0usize;
    for (ci, c) in CLASSES.iter().enumerate() {
        for _ in 0..cells[ci] {
            let (row, col) = (9 - k / 10, k % 10);
            let r = Rect::from_min_size(origin + vec2(col as f32 * (cs + gap), row as f32 * (cs + gap)), vec2(cs, cs));
            painter.rect_filled(r, CornerRadius::same(1), range_color(*c));
            k += 1;
        }
    }
    let x0 = origin.x + side + 22.0;
    if inner.right() - x0 > 70.0 {
        let pct = counts[2] as f64 / total as f64 * 100.0;
        painter.text(pos2(x0, origin.y - 4.0), Align2::LEFT_TOP, format!("{:.0}%", pct), sans(46.0), g_ok());
        let mut y = origin.y + 60.0;
        for (ci, c) in CLASSES.iter().enumerate().rev() {
            painter.rect_filled(Rect::from_min_size(pos2(x0, y + 2.0), vec2(10.0, 10.0)), 0.0, range_color(*c));
            painter.text(pos2(x0 + 18.0, y), Align2::LEFT_TOP, format!("{:.0}%", counts[ci] as f64 / total as f64 * 100.0), mono(12.0), if counts[ci] > 0 { paper() } else { ink_muted() });
            y += 19.0;
        }
    }
}

fn short_title(trace: &str, fallback: &str) -> String {
    match trace {
        "T01" => "TREND",
        "A02" => "HOURLY BAND",
        "D03" => "DISTRIBUTION",
        "R04" => "DAILY RANGE",
        "R05" => "WEEKLY RANGE",
        "S05" => "DAILY SPREAD",
        "H06" => "WEEK x HOUR",
        "B07" => "TIME BLOCKS",
        "P09" => "24H DIAL",
        "K10" => "DAY BY DAY",
        "Q11" => "SHARE BY TIME",
        "W12" => "RANGE",
        "L13" => "TIME BANDS",
        "B14" => "SPREAD BY BAND",
        "Q15" => "SHARE BY BAND",
        "H16" => "BAND x WEEKDAY",
        _ => fallback,
    }
    .to_string()
}

// ============= Inferred time bands =============

#[derive(Clone)]
pub struct Band {
    pub start: f64,
    pub end: f64,
    pub centre: f64,
    pub name: &'static str,
}

impl Band {
    pub fn contains(&self, h: f64) -> bool {
        if self.start < self.end {
            h >= self.start && h < self.end
        } else {
            h >= self.start || h < self.end
        }
    }
    pub fn range_label(&self) -> String {
        format!("{}-{}", hm(self.start), hm(self.end))
    }
}

fn hm(h: f64) -> String {
    let m = ((h * 60.0).round() as i64).rem_euclid(1440);
    format!("{:02}:{:02}", m / 60, m % 60)
}

fn circ(a: f64, b: f64) -> f64 {
    let d = (a - b).abs() % 24.0;
    d.min(24.0 - d)
}

fn circ_mean(v: &[f64]) -> f64 {
    let (s, c) = v.iter().fold((0.0, 0.0), |(s, c), x| {
        let a = x / 24.0 * std::f64::consts::TAU;
        (s + a.sin(), c + a.cos())
    });
    (s.atan2(c) / std::f64::consts::TAU * 24.0).rem_euclid(24.0)
}

fn kmeans_circ(x: &[f64], k: usize) -> (Vec<f64>, f64) {
    let n = x.len();
    let mut cen: Vec<f64> = (0..k).map(|i| x[((2 * i + 1) * n / (2 * k)).min(n - 1)]).collect();
    let mut assign = vec![0usize; n];
    for _ in 0..40 {
        for (i, v) in x.iter().enumerate() {
            assign[i] = (0..k)
                .min_by(|a, b| circ(*v, cen[*a]).partial_cmp(&circ(*v, cen[*b])).unwrap())
                .unwrap();
        }
        for c in 0..k {
            let m: Vec<f64> = x.iter().zip(&assign).filter(|(_, a)| **a == c).map(|(v, _)| *v).collect();
            if !m.is_empty() {
                cen[c] = circ_mean(&m);
            }
        }
    }
    let wss = x.iter().zip(&assign).map(|(v, a)| circ(*v, cen[*a]).powi(2)).sum();
    (cen, wss)
}

fn band_name(c: f64) -> &'static str {
    match c.floor() as i32 {
        4..=8 => "Morning",
        9..=11 => "Late morning",
        12..=14 => "Midday",
        15..=17 => "Afternoon",
        18..=21 => "Evening",
        _ => "Night",
    }
}

/// Infer parts of the day from when readings were actually taken.
pub fn infer_bands(pts: &[Pt]) -> Vec<Band> {
    let mut x: Vec<f64> = pts.iter().map(hour_frac).collect();
    x.sort_by(|a, b| a.partial_cmp(b).unwrap());
    if x.len() < 8 {
        return Vec::new();
    }
    let maxk = (x.len() / 4).clamp(2, 6);
    let wss: Vec<f64> = (1..=maxk + 1).map(|k| if k > x.len() { f64::INFINITY } else { kmeans_circ(&x, k).1 }).collect();
    // wss[k-1] is for k clusters
    let mut k = 2;
    while k < maxk && wss[k] <= 0.62 * wss[k - 1] {
        k += 1;
    }
    let (mut cen, _) = kmeans_circ(&x, k);
    cen.sort_by(|a, b| a.partial_cmp(b).unwrap());
    cen.dedup_by(|a, b| circ(*a, *b) < 0.25);
    let k = cen.len();
    if k < 2 {
        return Vec::new();
    }
    let snap = |v: f64| ((v * 2.0).round() / 2.0).rem_euclid(24.0);
    (0..k)
        .map(|i| {
            let (prev, c, next) = (cen[(i + k - 1) % k], cen[i], cen[(i + 1) % k]);
            let gp = (c - prev).rem_euclid(24.0);
            let gn = (next - c).rem_euclid(24.0);
            Band { start: snap(c - gp / 2.0), end: snap(c + gn / 2.0), centre: c, name: band_name(c) }
        })
        .collect()
}

pub struct BandStat {
    pub n: usize,
    pub mean: f64,
    pub median: f64,
    pub cv: f64,
    pub min: f64,
    pub p10: f64,
    pub q1: f64,
    pub q3: f64,
    pub p90: f64,
    pub max: f64,
    pub counts: [usize; 5],
}

pub fn band_stat(pts: &[Pt], b: &Band, cx: &Cx) -> Option<BandStat> {
    let sub: Vec<&Pt> = pts.iter().filter(|p| b.contains(hour_frac(p))).collect();
    if sub.is_empty() {
        return None;
    }
    let v = sorted_vals(&sub, cx);
    let n = v.len();
    let mean = v.iter().sum::<f64>() / n as f64;
    let sd = (v.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64).sqrt();
    let mut counts = [0usize; 5];
    for p in &sub {
        counts[class_idx(cx.class(p))] += 1;
    }
    Some(BandStat {
        n,
        mean,
        median: percentile(&v, 50.0),
        cv: if mean > 0.0 { sd / mean * 100.0 } else { 0.0 },
        min: v[0],
        p10: percentile(&v, 10.0),
        q1: percentile(&v, 25.0),
        q3: percentile(&v, 75.0),
        p90: percentile(&v, 90.0),
        max: v[n - 1],
        counts,
    })
}

fn band_spans(b: &Band) -> Vec<(f64, f64)> {
    if b.start < b.end {
        vec![(b.start, b.end)]
    } else {
        vec![(b.start, 24.0), (0.0, b.end)]
    }
}

/// 24h axis: how many readings fall in each hour, with the inferred bands laid over it.
pub fn band_timeline(ui: &mut egui::Ui, pts: &[Pt], bands: &[Band], height: f32) {
    let (inner, painter, hover) = chart_well(ui, "L13", "TIME BANDS", "", height);
    let m = Map { r: inner, x0: 0.0, x1: 24.0, y0: 0.0, y1: 1.0 };
    let strip_h = 38.0f32.min(inner.height() * 0.4);
    let strip = Rect::from_min_max(inner.min, pos2(inner.right(), inner.top() + strip_h));
    let hist = Rect::from_min_max(pos2(inner.left(), strip.bottom() + 8.0), inner.max);
    let mut counts = [0usize; 24];
    for p in pts {
        counts[p.hour.min(23) as usize] += 1;
    }
    let maxc = (*counts.iter().max().unwrap_or(&1)).max(1) as f32;
    for (h, c) in counts.iter().enumerate() {
        let bi = bands.iter().position(|b| b.contains(h as f64 + 0.5)).unwrap_or(0);
        let col = band_color(bi);
        let x0 = m.px(h as f64) + 1.5;
        let x1 = m.px(h as f64 + 1.0) - 1.5;
        let bh = hist.height() * *c as f32 / maxc;
        if *c > 0 {
            painter.rect_filled(Rect::from_min_max(pos2(x0, hist.bottom() - bh), pos2(x1, hist.bottom())), CornerRadius { nw: 2, ne: 2, sw: 0, se: 0 }, alpha(col, 200));
        }
        if let Some(hp) = hover {
            if hp.x >= m.px(h as f64) && hp.x < m.px(h as f64 + 1.0) && hp.y > strip.bottom() {
                probe(&painter, inner, pos2((x0 + x1) / 2.0, hist.bottom() - bh), &[format!("{:02}:00-{:02}:59", h, h), format!("N={}", c)], tangerine());
            }
        }
    }
    painter.line_segment([pos2(hist.left(), hist.bottom()), pos2(hist.right(), hist.bottom())], Stroke::new(1.0, rule()));
    for (i, b) in bands.iter().enumerate() {
        let col = band_color(i);
        for (s, e) in band_spans(b) {
            let r = Rect::from_min_max(pos2(m.px(s) + 1.0, strip.top()), pos2(m.px(e) - 1.0, strip.bottom()));
            painter.rect(r, CornerRadius::same(2), alpha(col, 70), Stroke::new(1.5, col), StrokeKind::Inside);
        }
        let (s, e) = band_spans(b).into_iter().max_by(|a, b| (a.1 - a.0).partial_cmp(&(b.1 - b.0)).unwrap()).unwrap();
        let cxp = m.px((s + e) / 2.0);
        painter.text(pos2(cxp, strip.top() + strip_h * 0.32), Align2::CENTER_CENTER, b.name.to_uppercase(), mono(10.0), paper());
        painter.text(pos2(cxp, strip.top() + strip_h * 0.72), Align2::CENTER_CENTER, b.range_label(), mono(9.0), ink_dim());
    }
    for h in (0..=24).step_by(3) {
        painter.text(pos2(m.px(h as f64), inner.bottom() + 6.0), Align2::CENTER_TOP, format!("{:02}h", h % 24), mono(9.5), ink_muted());
    }
}

/// One card per band: name, range, big mean, median, CV, in-range, lows, mix bar.
pub fn band_cards(ui: &mut egui::Ui, pts: &[Pt], bands: &[Band], cx: &Cx, height: f32) {
    let w = ui.available_width();
    let (resp, painter) = ui.allocate_painter(vec2(w, height), egui::Sense::hover());
    let n = bands.len().max(1);
    let gap = 10.0;
    let cw = (w - gap * (n as f32 - 1.0)) / n as f32;
    for (i, b) in bands.iter().enumerate() {
        let col = band_color(i);
        let r = Rect::from_min_size(resp.rect.min + vec2(i as f32 * (cw + gap), 0.0), vec2(cw, height));
        painter.rect(r, CornerRadius::same(2), plate(), Stroke::new(1.0, rule()), StrokeKind::Inside);
        painter.rect_filled(Rect::from_min_size(r.min, vec2(r.width(), 4.0)), CornerRadius { nw: 2, ne: 2, sw: 0, se: 0 }, col);
        painter.text(r.left_top() + vec2(14.0, 16.0), Align2::LEFT_TOP, b.name.to_uppercase(), mono(11.0), paper());
        painter.text(r.left_top() + vec2(14.0, 32.0), Align2::LEFT_TOP, b.range_label(), mono(9.5), ink_muted());
        let st = band_stat(pts, b, cx);
        let Some(s) = st else {
            painter.text(r.center(), Align2::CENTER_CENTER, "-", sans(30.0), ink_muted());
            continue;
        };
        painter.text(r.right_top() + vec2(-14.0, 16.0), Align2::RIGHT_TOP, format!("N={}", s.n), mono(9.5), ink_muted());
        painter.text(r.left_top() + vec2(14.0, 54.0), Align2::LEFT_TOP, "MEAN", mono(9.0), ink_muted());
        painter.text(r.left_top() + vec2(14.0, 64.0), Align2::LEFT_TOP, cx.fmt(s.mean), sans(46.0), range_color(cx.class_unit(s.mean)));
        let in_pct = s.counts[2] as f64 / s.n as f64 * 100.0;
        let lows = s.counts[0] + s.counts[1];
        let cells = [("MEDIAN", cx.fmt(s.median)), ("CV %", format!("{:.0}", s.cv)), ("IN RANGE %", format!("{:.0}", in_pct)), ("LOWS", format!("{}", lows))];
        for (k, (lab, val)) in cells.iter().enumerate() {
            let (cx0, cy0) = (r.left() + 14.0 + (k % 2) as f32 * (cw - 28.0) / 2.0, r.top() + 128.0 + (k / 2) as f32 * 44.0);
            painter.text(pos2(cx0, cy0), Align2::LEFT_TOP, *lab, mono(9.0), ink_muted());
            let c = if k == 3 && lows > 0 { g_low() } else if k == 2 { g_ok() } else { paper() };
            painter.text(pos2(cx0, cy0 + 11.0), Align2::LEFT_TOP, val, sans(24.0), c);
        }
        // composition bar
        let bar = Rect::from_min_size(pos2(r.left() + 14.0, r.bottom() - 16.0), vec2(cw - 28.0, 8.0));
        painter.rect_filled(bar, 0.0, rule_soft());
        let mut x = bar.left();
        for (ci, c) in CLASSES.iter().enumerate() {
            if s.counts[ci] == 0 {
                continue;
            }
            let bw = bar.width() * s.counts[ci] as f32 / s.n as f32;
            painter.rect_filled(Rect::from_min_size(pos2(x, bar.top()), vec2((bw - 1.0).max(1.0), bar.height())), 0.0, range_color(*c));
            x += bw;
        }
    }
}

/// Shared-axis boxplots per band.
pub fn band_boxes(ui: &mut egui::Ui, pts: &[Pt], bands: &[Band], cx: &Cx, height: f32) {
    let (inner, painter, hover) = chart_well(ui, "B14", "SPREAD BY BAND", "", height);
    let stats: Vec<Option<BandStat>> = bands.iter().map(|b| band_stat(pts, b, cx)).collect();
    let lo = stats.iter().flatten().map(|s| s.p10).fold(f64::INFINITY, f64::min);
    let hi = stats.iter().flatten().map(|s| s.p90).fold(f64::NEG_INFINITY, f64::max);
    if !lo.is_finite() {
        empty_msg(&painter, inner, "-");
        return;
    }
    let (y0, y1) = y_range(cx, lo, hi);
    let m = Map { r: inner, x0: 0.0, x1: 1.0, y0, y1 };
    grid_y(&painter, &m, cx, false);
    target_band(&painter, &m, cx);
    let slot = inner.width() / bands.len() as f32;
    let mut hov = None;
    for (i, (b, s)) in bands.iter().zip(&stats).enumerate() {
        let x = inner.left() + slot * (i as f32 + 0.5);
        let col = band_color(i);
        painter.text(pos2(x, inner.bottom() + 5.0), Align2::CENTER_TOP, b.name.to_uppercase(), mono(9.0), ink_muted());
        let Some(s) = s else { continue };
        painter.line_segment([pos2(x, m.py(s.p90)), pos2(x, m.py(s.p10))], Stroke::new(1.5, alpha(paper(), 150)));
        for v in [s.p10, s.p90] {
            painter.line_segment([pos2(x - 8.0, m.py(v)), pos2(x + 8.0, m.py(v))], Stroke::new(1.5, alpha(paper(), 150)));
        }
        let bw = (slot * 0.5).min(70.0);
        let r = Rect::from_min_max(pos2(x - bw / 2.0, m.py(s.q3)), pos2(x + bw / 2.0, m.py(s.q1)));
        painter.rect(r, CornerRadius::same(2), alpha(col, 110), Stroke::new(1.5, col), StrokeKind::Inside);
        painter.line_segment([pos2(r.left(), m.py(s.median)), pos2(r.right(), m.py(s.median))], Stroke::new(3.0, paper()));
        let c = pos2(x, m.py(s.mean));
        painter.add(Shape::convex_polygon(vec![c + vec2(0.0, -5.0), c + vec2(5.0, 0.0), c + vec2(0.0, 5.0), c + vec2(-5.0, 0.0)], tangerine(), Stroke::new(1.0, well())));
        if let Some(h) = hover {
            if (h.x - x).abs() <= slot / 2.0 && inner.expand2(vec2(0.0, 12.0)).contains(h) {
                hov = Some((i, x));
            }
        }
    }
    if let Some((i, x)) = hov {
        if let Some(s) = &stats[i] {
            probe(
                &painter,
                inner,
                pos2(x, m.py(s.median)),
                &[
                    format!("{} {}  N={}", bands[i].name.to_uppercase(), bands[i].range_label(), s.n),
                    format!("MEAN {}  MEDIAN {}", cx.fmt(s.mean), cx.fmt(s.median)),
                    format!("IQR {}-{}", cx.fmt(s.q1), cx.fmt(s.q3)),
                    format!("P10-P90 {}-{}", cx.fmt(s.p10), cx.fmt(s.p90)),
                    format!("MIN {}  MAX {}", cx.fmt(s.min), cx.fmt(s.max)),
                ],
                tangerine(),
            );
        }
    }
}

/// Range mix per band as 100% bars.
pub fn band_bars(ui: &mut egui::Ui, pts: &[Pt], bands: &[Band], cx: &Cx, height: f32) {
    let (inner, painter, hover) = chart_well(ui, "Q15", "SHARE BY BAND", "", height);
    let full = Rect::from_min_max(inner.min - vec2(46.0, 14.0), inner.max + vec2(16.0, 26.0));
    let rows = bands.len().max(1) as f32;
    let rh = (full.height() - 12.0) / rows;
    let bx0 = full.left() + 120.0;
    let bx1 = full.right() - 14.0;
    let mut hov = None;
    for (i, b) in bands.iter().enumerate() {
        let y = full.top() + 6.0 + rh * (i as f32 + 0.5);
        painter.rect_filled(Rect::from_min_size(pos2(full.left() + 10.0, y - 7.0), vec2(4.0, 14.0)), 0.0, band_color(i));
        painter.text(pos2(full.left() + 20.0, y - 6.0), Align2::LEFT_CENTER, b.name.to_uppercase(), mono(9.5), paper());
        painter.text(pos2(full.left() + 20.0, y + 7.0), Align2::LEFT_CENTER, b.range_label(), mono(8.5), ink_muted());
        let bar = Rect::from_min_max(pos2(bx0, y - rh * 0.3), pos2(bx1, y + rh * 0.3));
        painter.rect_filled(bar, 0.0, rule_soft());
        let Some(s) = band_stat(pts, b, cx) else { continue };
        let mut x = bar.left();
        for (ci, c) in CLASSES.iter().enumerate() {
            if s.counts[ci] == 0 {
                continue;
            }
            let w = bar.width() * s.counts[ci] as f32 / s.n as f32;
            painter.rect_filled(Rect::from_min_size(pos2(x, bar.top()), vec2((w - 1.0).max(1.0), bar.height())), 0.0, range_color(*c));
            if w > 30.0 {
                painter.text(pos2(x + w / 2.0, bar.center().y), Align2::CENTER_CENTER, format!("{:.0}%", s.counts[ci] as f64 / s.n as f64 * 100.0), mono(9.5), INK_ON);
            }
            x += w;
        }
        if let Some(h) = hover {
            if bar.expand2(vec2(0.0, 4.0)).contains(h) {
                hov = Some((bar, s.counts, s.n, i));
            }
        }
    }
    if let Some((bar, counts, n, i)) = hov {
        let mut lines = vec![format!("{}  N={}", bands[i].name.to_uppercase(), n)];
        for (ci, c) in CLASSES.iter().enumerate().rev() {
            if counts[ci] > 0 {
                lines.push(format!("{:<9} {:>3.0}%  ({})", class_name(*c), counts[ci] as f64 / n as f64 * 100.0, counts[ci]));
            }
        }
        probe(&painter, full, bar.center(), &lines, tangerine());
    }
}

/// Band x weekday grid of mean values.
pub fn band_week(ui: &mut egui::Ui, pts: &[Pt], bands: &[Band], cx: &Cx, height: f32) {
    let (inner, painter, hover) = chart_well(ui, "H16", "BAND x WEEKDAY", "", height);
    let rows = bands.len().max(1);
    let left = 70.0;
    let grid = Rect::from_min_max(pos2(inner.left() - 46.0 + left, inner.top()), inner.max);
    let cw = grid.width() / 7.0;
    let ch = grid.height() / rows as f32;
    let mut sum = vec![[0.0f64; 7]; rows];
    let mut cnt = vec![[0usize; 7]; rows];
    for p in pts {
        let Some(d) = dow_of(&p.day) else { continue };
        let h = hour_frac(p);
        if let Some(bi) = bands.iter().position(|b| b.contains(h)) {
            sum[bi][d] += cx.val(p);
            cnt[bi][d] += 1;
        }
    }
    for (d, name) in DOW.iter().enumerate() {
        painter.text(pos2(grid.left() + cw * (d as f32 + 0.5), inner.bottom() + 6.0), Align2::CENTER_TOP, *name, mono(9.5), ink_muted());
    }
    let mut hov = None;
    for (bi, b) in bands.iter().enumerate() {
        let y = grid.top() + ch * (bi as f32 + 0.5);
        painter.rect_filled(Rect::from_min_size(pos2(inner.left() - 38.0, y - 7.0), vec2(4.0, 14.0)), 0.0, band_color(bi));
        painter.text(pos2(inner.left() - 28.0, y), Align2::LEFT_CENTER, b.name.to_uppercase().chars().take(9).collect::<String>(), mono(8.5), ink_dim());
        for d in 0..7 {
            let r = Rect::from_min_size(pos2(grid.left() + cw * d as f32, grid.top() + ch * bi as f32), vec2(cw, ch)).shrink(1.5);
            if cnt[bi][d] == 0 {
                painter.rect_filled(r, CornerRadius::same(1), rule_soft());
                continue;
            }
            let mean = sum[bi][d] / cnt[bi][d] as f64;
            let a = (110 + cnt[bi][d].min(4) * 36).min(255) as u8;
            painter.rect_filled(r, CornerRadius::same(1), alpha(range_color(cx.class_unit(mean)), a));
            painter.text(r.center(), Align2::CENTER_CENTER, cx.fmt(mean), mono(11.0), INK_ON);
            if hover.map(|p: egui::Pos2| r.contains(p)).unwrap_or(false) {
                hov = Some((bi, d, r, mean));
            }
        }
    }
    if let Some((bi, d, r, mean)) = hov {
        painter.rect_stroke(r, CornerRadius::same(1), Stroke::new(2.0, tangerine()), StrokeKind::Inside);
        probe(
            &painter,
            inner,
            r.center(),
            &[format!("{} {}", DOW[d], bands[bi].name.to_uppercase()), format!("MEAN {} {}", cx.fmt(mean), cx.unit_label()), format!("N={}", cnt[bi][d])],
            tangerine(),
        );
    }
}
