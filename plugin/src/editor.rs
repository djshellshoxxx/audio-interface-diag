//! egui editor: Live / DAW / Engineer / Devices / Report / Badges / Help screens.

use crate::report::{self, fmt_db};
use crate::shared::{Event, EventKind, Shared};
use crate::{AidParams, GenType, Mode, ReturnSource};
use audio_interface_diag::{
    enumerate_native_devices, plan, summarize_runs, Edition, NativeDeviceSummary, TestKind,
    ACK_REQUIRED_ABOVE_DBFS,
};
use nih_plug::prelude::*;
use nih_plug_egui::egui::{self, Color32, RichText};
use nih_plug_egui::{create_egui_editor, widgets::ParamSlider};
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;
use std::time::{Duration, Instant};

const GREEN: Color32 = Color32::from_rgb(64, 200, 120);
const AMBER: Color32 = Color32::from_rgb(240, 180, 40);
const RED: Color32 = Color32::from_rgb(235, 70, 70);
const GREY: Color32 = Color32::from_rgb(140, 140, 150);
const ACCENT: Color32 = Color32::from_rgb(90, 170, 255);
const RECENT_SECS: f64 = 30.0;
const LATENCY_RUNS: u32 = 5;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Live,
    Daw,
    Engineer,
    Devices,
    Report,
    Badges,
    Help,
}

struct Achievement {
    name: &'static str,
    desc: &'static str,
    check: fn(&Snapshot) -> bool,
}

const ACHIEVEMENTS: &[Achievement] = &[
    Achievement {
        name: "First Light",
        desc: "Receive your first audio callback.",
        check: |s| s.callbacks > 0,
    },
    Achievement {
        name: "Clean Minute",
        desc: "60 s glitch-free streak with audio running.",
        check: |s| s.streak >= 60.0,
    },
    Achievement {
        name: "Rock Solid",
        desc: "10 minute glitch-free streak.",
        check: |s| s.streak >= 600.0,
    },
    Achievement {
        name: "Hour of Power",
        desc: "1 hour glitch-free streak. Show-ready.",
        check: |s| s.streak >= 3600.0,
    },
    Achievement {
        name: "Headroom Hero",
        desc: "5 minutes of real signal with zero clips.",
        check: |s| s.had_signal && s.clip_runs == 0 && s.session >= 300.0,
    },
    Achievement {
        name: "Hum Buster",
        desc: "Hum analysed for 5 minutes and never dominant.",
        check: |s| s.hum_hz != 0 && !s.hum_ever && s.session >= 300.0,
    },
    Achievement {
        name: "Lab Rat",
        desc: "Complete a round-trip latency measurement.",
        check: |s| s.latency_runs > 0,
    },
    Achievement {
        name: "Sub-10 Club",
        desc: "Measure a median round trip under 10 ms.",
        check: |s| s.latency_median_ms.is_some_and(|ms| ms < 10.0),
    },
    Achievement {
        name: "Tight Ship",
        desc: "5 latency runs with zero spread.",
        check: |s| s.latency_runs >= 5 && s.latency_spread == Some(0.0),
    },
    Achievement {
        name: "Bookkeeper",
        desc: "Export your first report.",
        check: |s| s.exported,
    },
];

#[derive(Default)]
struct Snapshot {
    callbacks: u64,
    session: f64,
    streak: f64,
    had_signal: bool,
    clip_runs: u64,
    hum_hz: u32,
    hum_ever: bool,
    latency_runs: usize,
    latency_median_ms: Option<f64>,
    latency_spread: Option<f64>,
    exported: bool,
}

struct GuiState {
    tab: Tab,
    stage_view: bool,
    confirm_arm: bool,
    notes: String,
    status: String,
    best_streak: f64,
    unlocked: Vec<Option<f64>>,
    toast: Option<(String, Instant)>,
    exported: bool,
    hum_ever: bool,
    markers: u32,
    devices: Option<Result<Vec<NativeDeviceSummary>, String>>,
}

pub fn create(params: Arc<AidParams>, shared: Arc<Shared>) -> Option<Box<dyn Editor>> {
    let state = GuiState {
        tab: Tab::Live,
        stage_view: false,
        confirm_arm: false,
        notes: String::new(),
        status: String::new(),
        best_streak: 0.0,
        unlocked: vec![None; ACHIEVEMENTS.len()],
        toast: None,
        exported: false,
        hum_ever: false,
        markers: 0,
        devices: None,
    };
    create_egui_editor(
        params.editor_state.clone(),
        state,
        |ctx, _| ctx.set_visuals(egui::Visuals::dark()),
        move |ctx, setter, state| {
            ctx.request_repaint_after(Duration::from_millis(33));
            let events = shared.events();
            let mode = params.mode.value();
            if state.tab == Tab::Live || state.tab == Tab::Daw || state.tab == Tab::Engineer {
                // Keep the screen in sync with automation/preset changes of the mode.
                state.tab = mode_tab(mode);
            }
            let snap = snapshot(&shared, state);
            update_achievements(state, &snap);

            top_bar(ctx, &params, &shared, state, &events, &snap);
            egui::SidePanel::left("nav")
                .resizable(false)
                .exact_width(120.0)
                .show(ctx, |ui| nav(ui, setter, &params, state));
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| match state.tab {
                    Tab::Live => live_screen(ui, &shared, state, &events, &snap),
                    Tab::Daw => daw_screen(ui, setter, &params, &shared, &events),
                    Tab::Engineer => engineer_screen(ui, setter, &params, &shared, state),
                    Tab::Devices => devices_screen(ui, &shared, state),
                    Tab::Report => report_screen(ui, &params, &shared, state, &events),
                    Tab::Badges => badges_screen(ui, state, &snap),
                    Tab::Help => help_screen(ui),
                });
            });
        },
    )
}

fn mode_tab(mode: Mode) -> Tab {
    match mode {
        Mode::Live => Tab::Live,
        Mode::Daw => Tab::Daw,
        Mode::Engineer => Tab::Engineer,
    }
}

fn snapshot(shared: &Shared, state: &mut GuiState) -> Snapshot {
    let session = shared.session_secs.load();
    let callbacks = shared.callbacks.load(Relaxed);
    let streak = if callbacks == 0 {
        0.0
    } else {
        (session - shared.last_problem_secs.load()).max(0.0)
    };
    state.best_streak = state.best_streak.max(streak);
    state.hum_ever |= shared.hum_present.load(Relaxed);
    let latency = shared.latency.lock().unwrap().clone();
    let summary = summarize_runs(&latency.runs_samples);
    Snapshot {
        callbacks,
        session,
        streak,
        had_signal: shared.had_signal.load(Relaxed),
        clip_runs: shared.clip_runs.load(Relaxed),
        hum_hz: shared.hum_hz.load(Relaxed),
        hum_ever: state.hum_ever,
        latency_runs: latency.runs_samples.len(),
        latency_median_ms: summary.map(|s| s.median / latency.sample_rate as f64 * 1000.0),
        latency_spread: summary.map(|s| s.spread),
        exported: state.exported,
    }
}

fn update_achievements(state: &mut GuiState, snap: &Snapshot) {
    for (i, a) in ACHIEVEMENTS.iter().enumerate() {
        if state.unlocked[i].is_none() && (a.check)(snap) {
            state.unlocked[i] = Some(snap.session);
            state.toast = Some((
                format!("🏆 Achievement unlocked: {}", a.name),
                Instant::now(),
            ));
        }
    }
}

fn set_enum<E: Enum + PartialEq + Copy + 'static>(
    setter: &ParamSetter,
    param: &EnumParam<E>,
    value: E,
) {
    setter.begin_set_parameter(param);
    setter.set_parameter(param, value);
    setter.end_set_parameter(param);
}

fn enum_combo<E: Enum + PartialEq + Copy + 'static>(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    param: &EnumParam<E>,
    label: &str,
    tip: &str,
) {
    let current = param.value();
    egui::ComboBox::from_label(label)
        .selected_text(E::variants()[current.to_index()])
        .show_ui(ui, |ui| {
            for (i, name) in E::variants().iter().enumerate() {
                let value = E::from_index(i);
                if ui.selectable_label(value == current, *name).clicked() {
                    set_enum(setter, param, value);
                }
            }
        })
        .response
        .on_hover_text(tip);
}

/// Health from recent events. Unknown is never shown as healthy.
fn health(shared: &Shared, events: &[Event]) -> (&'static str, Color32, String) {
    if shared.callbacks.load(Relaxed) == 0 {
        return ("WAITING", GREY, "No audio callbacks yet".into());
    }
    let now = shared.session_secs.load();
    let recent = |kinds: &[EventKind]| {
        events
            .iter()
            .rev()
            .take_while(|e| now - e.t <= RECENT_SECS)
            .find(|e| kinds.contains(&e.kind))
            .map(|e| e.kind)
    };
    if let Some(kind) = recent(&[
        EventKind::Dropout,
        EventKind::NonFinite,
        EventKind::DeadlineMiss,
        EventKind::SignalLost,
    ]) {
        return ("FAULT", RED, format!("{} in the last 30 s", kind.label()));
    }
    if let Some(kind) = recent(&[EventKind::Clip]) {
        return (
            "WARNING",
            AMBER,
            format!("{} in the last 30 s", kind.label()),
        );
    }
    if shared.hum_present.load(Relaxed) {
        return ("WARNING", AMBER, "Mains hum dominates a quiet input".into());
    }
    if shared.max_load.load(Relaxed) > 0.8 {
        return (
            "WARNING",
            AMBER,
            "AID callback used >80% of a period".into(),
        );
    }
    ("HEALTHY", GREEN, "No faults in the last 30 s".into())
}

fn fmt_duration(secs: f64) -> String {
    let s = secs.max(0.0) as u64;
    format!("{:02}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60)
}

fn top_bar(
    ctx: &egui::Context,
    params: &AidParams,
    shared: &Shared,
    state: &mut GuiState,
    events: &[Event],
    snap: &Snapshot,
) {
    egui::TopBottomPanel::top("top").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(RichText::new("AID").strong().size(20.0).color(ACCENT));
            ui.label(format!("v{}", env!("CARGO_PKG_VERSION")));
            ui.separator();
            let (label, color, why) = health(shared, events);
            ui.label(RichText::new(format!("● {label}")).strong().color(color))
                .on_hover_text(why);
            ui.separator();
            let sr = shared.sample_rate.load(Relaxed);
            ui.label(format!(
                "{} Hz · block {} / max {} · {}",
                sr,
                shared.last_block.load(Relaxed),
                shared.max_buffer.load(Relaxed),
                shared.plugin_api.lock().unwrap()
            ))
            .on_hover_text("Host-reported sample rate and max buffer; observed block size.");
            ui.separator();
            ui.label(format!("Session {}", fmt_duration(snap.session)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if shared.generating.load(Relaxed) || shared.armed.load(Relaxed) {
                    let stop = ui.add(
                        egui::Button::new(RichText::new("■ STOP").strong().color(Color32::WHITE))
                            .fill(RED),
                    );
                    if stop
                        .on_hover_text("Emergency stop: silences generated output now.")
                        .clicked()
                    {
                        shared.armed.store(false, Relaxed);
                        shared.latency_remaining.store(0, Relaxed);
                    }
                }
                if params.mode.value() != Mode::Engineer {
                    ui.label(RichText::new("PASSIVE").color(GREEN))
                        .on_hover_text("Live/DAW modes never generate signal or alter audio.");
                } else if shared.armed.load(Relaxed) {
                    ui.label(RichText::new("ARMED").strong().color(RED));
                }
            });
        });
        if let Some((text, at)) = &state.toast {
            if at.elapsed() < Duration::from_secs(5) {
                ui.label(RichText::new(text).color(AMBER));
            } else {
                state.toast = None;
            }
        }
    });
}

fn nav(ui: &mut egui::Ui, setter: &ParamSetter, params: &AidParams, state: &mut GuiState) {
    ui.add_space(6.0);
    for (tab, label, tip) in [
        (Tab::Live, "Live", "Passive performance monitoring"),
        (Tab::Daw, "DAW", "Non-disruptive production diagnostics"),
        (
            Tab::Engineer,
            "Engineer",
            "Armed active tests and loopback measurement",
        ),
        (
            Tab::Devices,
            "Devices",
            "Host context and native device inventory",
        ),
        (Tab::Report, "Report", "Findings, notes, copy and export"),
        (
            Tab::Badges,
            "Badges",
            "Glitch-free streaks and achievements",
        ),
        (Tab::Help, "Help", "What each measurement means"),
    ] {
        let selected = state.tab == tab;
        if ui
            .add_sized([108.0, 28.0], egui::SelectableLabel::new(selected, label))
            .on_hover_text(tip)
            .clicked()
        {
            state.tab = tab;
            // Choosing a mode screen switches the mode; it never starts a test.
            let mode = match tab {
                Tab::Live => Some(Mode::Live),
                Tab::Daw => Some(Mode::Daw),
                Tab::Engineer => Some(Mode::Engineer),
                _ => None,
            };
            if let Some(mode) = mode {
                if params.mode.value() != mode {
                    set_enum(setter, &params.mode, mode);
                }
            }
        }
    }
}

fn meter(ui: &mut egui::Ui, label: &str, rms_db: f32, peak_db: f32, hold_db: f32) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).monospace());
        let norm = |db: f32| ((db + 60.0) / 60.0).clamp(0.0, 1.0);
        let color = if peak_db >= -0.5 {
            RED
        } else if peak_db >= -6.0 {
            AMBER
        } else {
            GREEN
        };
        ui.add(
            egui::ProgressBar::new(norm(peak_db))
                .desired_width(ui.available_width() - 4.0)
                .fill(color)
                .text(format!(
                    "peak {} · RMS {} · hold {} dBFS",
                    fmt_db(peak_db),
                    fmt_db(rms_db),
                    fmt_db(hold_db)
                )),
        );
    });
}

fn meters(ui: &mut egui::Ui, shared: &Shared) {
    let channels = shared.channels.load(Relaxed).clamp(1, 2) as usize;
    for ch in 0..channels {
        meter(
            ui,
            if channels == 1 {
                "M"
            } else if ch == 0 {
                "L"
            } else {
                "R"
            },
            shared.rms_db[ch].load(Relaxed),
            shared.peak_db[ch].load(Relaxed),
            shared.peak_hold_db[ch].load(Relaxed),
        );
    }
}

fn stat(ui: &mut egui::Ui, name: &str, value: String, tip: &str, bad: bool) {
    ui.label(name).on_hover_text(tip);
    ui.label(
        RichText::new(value)
            .strong()
            .color(if bad { RED } else { Color32::WHITE }),
    );
    ui.end_row();
}

fn counters(ui: &mut egui::Ui, shared: &Shared) {
    egui::Grid::new("counters")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            let n = |c: &std::sync::atomic::AtomicU64| c.load(Relaxed);
            stat(
                ui,
                "Suspected dropouts",
                n(&shared.dropouts).to_string(),
                "Callback gaps >2.5x the buffer period (inferred from timing).",
                n(&shared.dropouts) > 0,
            );
            stat(
                ui,
                "Clip runs / samples",
                format!("{} / {}", n(&shared.clip_runs), n(&shared.clipped_samples)),
                "Runs of samples at/above the clip threshold.",
                n(&shared.clip_runs) > 0,
            );
            stat(
                ui,
                "Signal lost",
                n(&shared.signal_lost).to_string(),
                "Transitions to >2 s digital silence after signal was present.",
                n(&shared.signal_lost) > 0,
            );
            stat(
                ui,
                "NaN/Inf blocks",
                n(&shared.non_finite).to_string(),
                "Blocks containing invalid samples.",
                n(&shared.non_finite) > 0,
            );
            stat(
                ui,
                "AID deadline misses",
                n(&shared.deadline_misses).to_string(),
                "AID's own processing took a full buffer period.",
                n(&shared.deadline_misses) > 0,
            );
            let max_load = shared.max_load.load(Relaxed);
            stat(
                ui,
                "AID callback margin",
                format!(
                    "{:.1}% avg · {:.1}% worst · {:.3} ms worst",
                    shared.avg_load.load(Relaxed) * 100.0,
                    max_load * 100.0,
                    shared.max_callback_ms.load(Relaxed)
                ),
                "Share of the buffer period used by AID's own processing (measured).",
                max_load > 0.8,
            );
            stat(
                ui,
                "Callbacks",
                n(&shared.callbacks).to_string(),
                "Audio blocks processed.",
                false,
            );
        });
}

fn timeline(ui: &mut egui::Ui, events: &[Event], height: f32) {
    egui::ScrollArea::vertical()
        .id_salt("timeline")
        .max_height(height)
        .stick_to_bottom(true)
        .show(ui, |ui| {
            if events.is_empty() {
                ui.label(RichText::new("No events yet.").color(GREY));
            }
            for e in events {
                let color = if e.kind.is_problem() {
                    RED
                } else if matches!(e.kind, EventKind::Marker | EventKind::LatencyResult) {
                    ACCENT
                } else {
                    GREY
                };
                ui.label(
                    RichText::new(format!(
                        "{:>10.2}s  {}  {:.2}",
                        e.t,
                        e.kind.label(),
                        e.value
                    ))
                    .monospace()
                    .color(color),
                );
            }
        });
}

fn live_screen(
    ui: &mut egui::Ui,
    shared: &Shared,
    state: &mut GuiState,
    events: &[Event],
    snap: &Snapshot,
) {
    let (label, color, why) = health(shared, events);
    ui.horizontal(|ui| {
        ui.checkbox(&mut state.stage_view, "Stage view")
            .on_hover_text("Huge status text readable from a distance.");
        if ui
            .button("⚑ Mark event")
            .on_hover_text("Drop a timestamped marker in the timeline.")
            .clicked()
        {
            state.markers += 1;
            shared.log_event(EventKind::Marker, state.markers as f32);
        }
        if ui
            .button("📸 Snapshot report")
            .on_hover_text("Export telemetry and timeline now.")
            .clicked()
        {
            do_export(shared, Mode::Live, state);
        }
        if ui
            .button("⟲ Reset session")
            .on_hover_text("Clear counters, timeline and results.")
            .clicked()
        {
            shared.clear_session();
        }
    });
    if !state.status.is_empty() {
        ui.label(RichText::new(&state.status).color(ACCENT));
    }
    let size = if state.stage_view { 96.0 } else { 40.0 };
    ui.vertical_centered(|ui| {
        ui.label(RichText::new(label).size(size).strong().color(color));
        ui.label(RichText::new(why).size(if state.stage_view { 24.0 } else { 14.0 }));
        ui.label(
            RichText::new(format!(
                "Glitch-free streak {}   ·   best {}",
                fmt_duration(snap.streak),
                fmt_duration(state.best_streak)
            ))
            .size(if state.stage_view { 28.0 } else { 16.0 })
            .color(ACCENT),
        );
    });
    ui.separator();
    meters(ui, shared);
    ui.add_space(6.0);
    ui.columns(2, |cols| {
        counters(&mut cols[0], shared);
        signal_details(&mut cols[1], shared);
    });
    ui.separator();
    ui.heading("Event timeline");
    timeline(ui, events, 180.0);
}

fn signal_details(ui: &mut egui::Ui, shared: &Shared) {
    egui::Grid::new("signal")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            let channels = shared.channels.load(Relaxed).clamp(1, 2) as usize;
            let corr = if shared.correlation_valid.load(Relaxed) {
                format!("{:+.2}", shared.correlation.load(Relaxed))
            } else {
                "unavailable".into()
            };
            stat(
                ui,
                "Stereo correlation",
                corr,
                "+1 identical, 0 unrelated, -1 polarity inverted (Pearson, 300 ms).",
                shared.correlation_valid.load(Relaxed) && shared.correlation.load(Relaxed) < -0.5,
            );
            for ch in 0..channels {
                let dc = shared.dc[ch].load(Relaxed);
                stat(
                    ui,
                    &format!("DC offset ch{}", ch + 1),
                    format!("{dc:+.5}"),
                    "Average sample value; should be ~0.",
                    dc.abs() > 0.01,
                );
                stat(
                    ui,
                    &format!("Crest factor ch{}", ch + 1),
                    format!("{:.1} dB", shared.crest_db[ch].load(Relaxed)),
                    "Peak-to-RMS ratio. Very low values suggest heavy limiting or clipping.",
                    false,
                );
            }
            let hum_hz = shared.hum_hz.load(Relaxed);
            let hum = if hum_hz == 0 {
                "analysing…".into()
            } else {
                format!(
                    "{} Hz family {:.1} dB rel.",
                    hum_hz,
                    shared.hum_db_rel.load(Relaxed)
                )
            };
            stat(
                ui,
                "Mains hum",
                hum,
                "50/60 Hz + 2nd/3rd harmonic power relative to total input (Goertzel, 1 s).",
                shared.hum_present.load(Relaxed),
            );
        });
}

fn daw_screen(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    params: &AidParams,
    shared: &Shared,
    events: &[Event],
) {
    ui.heading("DAW production diagnostics");
    ui.label(
        "The DAW owns the audio interface. AID only observes the audio on the track/bus it is \
         inserted on and never changes sample rate, buffer, routing or device settings.",
    );
    ui.collapsing("Routing assistant (DAW Send)", |ui| {
        ui.label("1. Insert AID on the track, bus or master you want to watch.");
        ui.label("2. For a dedicated diagnostic path, create a send/bus feeding a track with AID.");
        ui.label("3. For loopback tests, route the hardware return input to AID's sidechain and set Return source = Sidechain on the Engineer screen.");
        ui.label("4. Leave AID in Live or DAW mode during sessions: audio passes through untouched.");
    });
    ui.separator();
    meters(ui, shared);
    ui.columns(2, |cols| {
        counters(&mut cols[0], shared);
        signal_details(&mut cols[1], shared);
    });
    ui.horizontal(|ui| {
        ui.add(ParamSlider::for_param(&params.clip_threshold, setter).with_width(160.0))
            .on_hover_text("Samples at or above this level count as clipping.");
        ui.label("Clip threshold");
    });
    ui.separator();
    ui.heading("Dropout markers (match against DAW logs)");
    timeline(ui, events, 160.0);
}

fn catalog_status(kind: TestKind) -> (&'static str, Color32) {
    use TestKind::*;
    match kind {
        RoundTripLatency | Polarity | DcOffset | NoiseFloor => ("available", GREEN),
        DeviceInventory | DriverCapability => ("host context + Devices scan", AMBER),
        _ => ("needs native backend (not in this build)", GREY),
    }
}

fn engineer_screen(
    ui: &mut egui::Ui,
    setter: &ParamSetter,
    params: &AidParams,
    shared: &Shared,
    state: &mut GuiState,
) {
    let armed = shared.armed.load(Relaxed);
    let standalone = shared.is_standalone.load(Relaxed);
    ui.columns(2, |cols| {
        let ui = &mut cols[0];
        ui.heading("Test catalog");
        for test in plan(Edition::Engineer) {
            let (status, color) = catalog_status(test.kind);
            ui.horizontal(|ui| {
                ui.label(format!(
                    "{:?}{}",
                    test.kind,
                    if test.requires_loopback_cable { " 🔌" } else { "" }
                ));
                ui.label(RichText::new(status).small().color(color));
            });
        }
        ui.separator();
        ui.heading("Cabling");
        ui.label("Output L → cable → Input 1 (or the sidechain return). Start with the interface output and input gain low. 🔌 = loopback cable required.");

        let ui = &mut cols[1];
        ui.heading("Generator");
        enum_combo(ui, setter, &params.gen_type, "Signal", "Test signal emitted while armed.");
        ui.horizontal(|ui| {
            ui.add(ParamSlider::for_param(&params.gen_freq, setter).with_width(160.0));
            ui.label("Frequency");
        });
        ui.horizontal(|ui| {
            ui.add(ParamSlider::for_param(&params.gen_level, setter).with_width(160.0));
            ui.label("Level").on_hover_text("Defaults to -30 dBFS. Above -12 dBFS needs acknowledgement.");
        });
        let requested = params.gen_level.value() as f64;
        if requested > ACK_REQUIRED_ABOVE_DBFS {
            let mut ack = shared.high_level_ack.load(Relaxed);
            if ui.checkbox(&mut ack, RichText::new("I acknowledge a level above -12 dBFS").color(AMBER)).changed() {
                shared.high_level_ack.store(ack, Relaxed);
            }
            if !ack {
                ui.label(RichText::new("Output limited to -12 dBFS until acknowledged.").color(AMBER));
            }
        }
        enum_combo(ui, setter, &params.gen_channel, "Output channel", "Which output channel(s) carry the generator.");
        enum_combo(ui, setter, &params.return_source, "Return source", "Where the loopback return arrives.");
        if params.return_source.value() == ReturnSource::Sidechain && !shared.has_return_input.load(Relaxed) {
            ui.label(RichText::new("Host did not provide a sidechain; the main input is used.").color(AMBER));
        }
        if standalone {
            let mut monitor = params.standalone_monitor.value();
            if ui.checkbox(&mut monitor, "Monitor input to output (standalone)")
                .on_hover_text("Off by default to prevent feedback loops.").changed()
            {
                setter.begin_set_parameter(&params.standalone_monitor);
                setter.set_parameter(&params.standalone_monitor, monitor);
                setter.end_set_parameter(&params.standalone_monitor);
            }
        }
        ui.separator();
        if armed {
            ui.label(RichText::new("ARMED — output is generated").strong().color(RED));
            if ui.add(egui::Button::new(RichText::new("■ STOP").strong().size(22.0).color(Color32::WHITE)).fill(RED)).clicked() {
                shared.armed.store(false, Relaxed);
                shared.latency_remaining.store(0, Relaxed);
            }
        } else if state.confirm_arm {
            let level = audio_interface_diag::validate_generator_level(requested, shared.high_level_ack.load(Relaxed))
                .unwrap_or(ACK_REQUIRED_ABOVE_DBFS);
            ui.label(RichText::new("Confirm arming").strong().color(AMBER));
            ui.label(format!(
                "Device: {}\nSignal: {} · {:.1} dBFS · channel {}\nReturn: {}",
                if standalone { "standalone audio output" } else { "host track output" },
                GenType::variants()[params.gen_type.value().to_index()],
                level,
                crate::GenChannel::variants()[params.gen_channel.value().to_index()],
                crate::ReturnSource::variants()[params.return_source.value().to_index()],
            ));
            ui.horizontal(|ui| {
                if ui.button(RichText::new("ARM").strong().color(RED)).clicked() {
                    shared.armed.store(true, Relaxed);
                    state.confirm_arm = false;
                }
                if ui.button("Cancel").clicked() {
                    state.confirm_arm = false;
                }
            });
        } else if ui.button("Arm generator…").on_hover_text("Shows what will happen before any output.").clicked() {
            state.confirm_arm = true;
        }

        ui.separator();
        ui.heading("Round-trip latency");
        let remaining = shared.latency_remaining.load(Relaxed);
        let busy = shared.latency_busy.load(Relaxed);
        ui.horizontal(|ui| {
            if ui.add_enabled(armed && remaining == 0 && !busy, egui::Button::new(format!("Measure ({LATENCY_RUNS} runs)")))
                .on_hover_text("Emits a noise burst and cross-correlates the return. Arm first.")
                .on_disabled_hover_text("Arm the generator and wait for running measurements.")
                .clicked()
            {
                shared.latency_remaining.store(LATENCY_RUNS, Relaxed);
            }
            if ui.button("Clear").clicked() {
                *shared.latency.lock().unwrap() = Default::default();
            }
        });
        if remaining > 0 || busy {
            ui.label(RichText::new(format!("Measuring… {remaining} runs queued")).color(ACCENT));
        }
        ui.label(format!("Return level {} dBFS RMS", fmt_db(shared.return_rms_db.load(Relaxed))));
        let latency = shared.latency.lock().unwrap().clone();
        match summarize_runs(&latency.runs_samples) {
            Some(s) => {
                let ms = |v: f64| v / latency.sample_rate as f64 * 1000.0;
                ui.label(RichText::new(format!("{:.2} ms median ({:.0} samples)", ms(s.median), s.median)).size(22.0).strong());
                ui.label(format!(
                    "min {:.0} · max {:.0} · spread {:.0} samples · {} runs · {} invalid",
                    s.min, s.max, s.spread, s.runs, latency.failures
                ));
            }
            None if latency.failures > 0 => {
                ui.label(RichText::new(format!("{} runs invalid: no clear return. Check the loopback.", latency.failures)).color(AMBER));
            }
            None => {
                ui.label(RichText::new("Not measured").color(GREY));
            }
        }
    });
}

fn devices_screen(ui: &mut egui::Ui, shared: &Shared, state: &mut GuiState) {
    ui.heading("Current audio path (host-provided)");
    egui::Grid::new("ctx")
        .num_columns(2)
        .striped(true)
        .show(ui, |ui| {
            stat(
                ui,
                "Wrapper / API",
                shared.plugin_api.lock().unwrap().clone(),
                "How AID is hosted.",
                false,
            );
            stat(
                ui,
                "Sample rate",
                format!("{} Hz", shared.sample_rate.load(Relaxed)),
                "Host-reported.",
                false,
            );
            stat(
                ui,
                "Max buffer",
                format!("{} frames", shared.max_buffer.load(Relaxed)),
                "Host-reported maximum block size.",
                false,
            );
            stat(
                ui,
                "Observed block",
                format!("{} frames", shared.last_block.load(Relaxed)),
                "Measured from the last callback.",
                false,
            );
            stat(
                ui,
                "Main channels",
                shared.channels.load(Relaxed).to_string(),
                "Channels on AID's main input.",
                false,
            );
            stat(
                ui,
                "Loopback return port",
                if shared.has_return_input.load(Relaxed) {
                    "available"
                } else {
                    "not provided"
                }
                .into(),
                "Whether the host gave AID a sidechain input.",
                false,
            );
        });
    ui.separator();
    ui.heading("Native device inventory");
    ui.label("Lists devices on this computer's default audio host. Runs only when you click; it does not open streams or change settings.");
    if ui.button("Scan devices").clicked() {
        state.devices = Some(enumerate_native_devices());
    }
    match &state.devices {
        Some(Ok(list)) if list.is_empty() => {
            ui.label("No devices reported.");
        }
        Some(Ok(list)) => {
            egui::Grid::new("devs")
                .num_columns(3)
                .striped(true)
                .show(ui, |ui| {
                    ui.strong("Host API");
                    ui.strong("Device");
                    ui.strong("Default");
                    ui.end_row();
                    for d in list {
                        ui.label(&d.host_api);
                        ui.label(&d.name);
                        ui.label(match (d.is_default_input, d.is_default_output) {
                            (true, true) => "in + out",
                            (true, false) => "input",
                            (false, true) => "output",
                            _ => "",
                        });
                        ui.end_row();
                    }
                });
        }
        Some(Err(e)) => {
            ui.label(RichText::new(format!("Unavailable: {e}")).color(AMBER));
        }
        None => {}
    }
}

fn do_export(shared: &Shared, mode: Mode, state: &mut GuiState) {
    state.status = match report::export(shared, mode, &state.notes) {
        Ok(path) => {
            state.exported = true;
            format!("Exported {}.txt/.json/.csv", path.display())
        }
        Err(e) => format!("Export failed: {e}"),
    };
}

fn report_screen(
    ui: &mut egui::Ui,
    params: &AidParams,
    shared: &Shared,
    state: &mut GuiState,
    events: &[Event],
) {
    let mode = params.mode.value();
    ui.heading("Report");
    ui.horizontal(|ui| {
        if ui.button("📋 Copy to clipboard").clicked() {
            ui.ctx()
                .copy_text(report::render_full_text(shared, mode, events, &state.notes));
            state.status = "Report copied to clipboard.".into();
        }
        if ui
            .button("💾 Export TXT + JSON + CSV")
            .on_hover_text(report::report_dir().display().to_string())
            .clicked()
        {
            do_export(shared, mode, state);
        }
    });
    if !state.status.is_empty() {
        ui.label(RichText::new(&state.status).color(ACCENT));
    }
    ui.label("Technician notes (included in exports):");
    ui.add(
        egui::TextEdit::multiline(&mut state.notes)
            .desired_rows(3)
            .desired_width(f32::INFINITY),
    );
    ui.separator();
    for f in report::build_findings(shared) {
        let color = match f.severity {
            audio_interface_diag::Severity::Pass => GREEN,
            audio_interface_diag::Severity::Warn => AMBER,
            audio_interface_diag::Severity::Fail => RED,
            _ => GREY,
        };
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new(format!("[{}]", f.severity.label()))
                    .monospace()
                    .strong()
                    .color(color),
            );
            ui.label(RichText::new(&f.title).strong());
        });
        ui.label(RichText::new(format!("  {}", f.evidence)).small());
        ui.label(
            RichText::new(format!("  → {}", f.action))
                .small()
                .color(GREY),
        );
    }
}

fn badges_screen(ui: &mut egui::Ui, state: &GuiState, snap: &Snapshot) {
    ui.heading("Glitch-free streak");
    ui.label(
        RichText::new(fmt_duration(snap.streak))
            .size(56.0)
            .strong()
            .color(ACCENT),
    );
    ui.label(format!(
        "Best this session: {}",
        fmt_duration(state.best_streak)
    ));
    ui.separator();
    let unlocked = state.unlocked.iter().filter(|u| u.is_some()).count();
    ui.heading(format!("Achievements {unlocked}/{}", ACHIEVEMENTS.len()));
    for (a, at) in ACHIEVEMENTS.iter().zip(&state.unlocked) {
        ui.horizontal(|ui| match at {
            Some(t) => {
                ui.label(
                    RichText::new(format!("🏆 {}", a.name))
                        .strong()
                        .color(AMBER),
                );
                ui.label(format!("{} — unlocked at {}", a.desc, fmt_duration(*t)));
            }
            None => {
                ui.label(RichText::new(format!("🔒 {}", a.name)).color(GREY));
                ui.label(RichText::new(a.desc).color(GREY));
            }
        });
    }
}

fn help_screen(ui: &mut egui::Ui) {
    ui.heading("Help");
    for (title, body) in [
        ("Modes", "Live and DAW are passive: AID never emits signal or changes audio settings, and plugin audio passes through untouched. Engineer mode adds armed tests. Switching mode never starts a test, and leaving Engineer mode disarms instantly."),
        ("Suspected dropout", "AID measures the time between audio callbacks. A gap longer than 2.5x the buffer period means the host did not deliver audio in time. This is inferred from timing, not a driver-reported xrun. Invalidated by: offline rendering (ignored automatically), host pausing (shown as a separate event)."),
        ("Callback margin", "How much of each buffer period AID's own processing used. Above 80% is a warning, 100% is a missed deadline."),
        ("Clipping", "Samples at or above the clip threshold (default -0.1 dBFS). Repeated clipping (10+ runs) is a FAIL."),
        ("DC offset", "The average sample value. Large values waste headroom and can cause clicks."),
        ("Stereo correlation", "+1 means identical channels, 0 unrelated, -1 polarity-inverted. Strongly negative values suggest a reversed cable or polarity flip."),
        ("Mains hum", "AID measures 50/60 Hz and their 2nd/3rd harmonics relative to total input power each second. If hum dominates a quiet input it is flagged. This is an observation, not a diagnosis: check grounding, shielding and power."),
        ("Round-trip latency", "Wiring: AID output → interface output → cable → interface input → AID input (or sidechain). Arm the generator, click Measure. AID emits a noise burst and cross-correlates the return. Reported in samples and ms with median/min/max/spread over 5 runs. It includes host buffers. Invalidated by: no cable, too low a level, clipping, or a return routed through other processing."),
        ("Signal discipline", "Generator defaults to -30 dBFS; above -12 dBFS needs acknowledgement. Output ramps in over 20 ms and out over 2 ms. STOP, leaving Engineer mode or closing the session silences output. Arming is never saved, so generation never resumes automatically."),
        ("Reports", "Copy or export TXT, JSON and CSV with context (wrapper, sample rate, buffer, timestamps), findings, technician notes and the event timeline. Saved to Documents/AudioInterfaceDiag."),
        ("Badges", "Glitch-free streak counts time since the last dropout, clip, NaN, deadline miss or signal loss. Achievements unlock as you keep your rig clean."),
    ] {
        ui.collapsing(title, |ui| {
            ui.label(body);
        });
    }
}
