//! State shared between the audio thread, the background task executor and the GUI. The audio
//! thread only touches atomics, the lock-free event ring, and `try_lock`s the capture buffers.

use nih_plug::prelude::AtomicF32;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering::Relaxed};
use std::sync::Mutex;

pub const EVENT_LOG_CAPACITY: usize = 2000;
pub const MINUS_INF_DB: f32 = -150.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    SessionStart,
    Dropout,
    ProcessingPaused,
    Clip,
    NonFinite,
    SignalLost,
    SignalRestored,
    FormatChange,
    DeadlineMiss,
    Marker,
    GeneratorArmed,
    GeneratorStopped,
    LatencyResult,
    LatencyFailed,
    HumObserved,
    HumCleared,
}

impl EventKind {
    pub fn label(self) -> &'static str {
        match self {
            EventKind::SessionStart => "Session start",
            EventKind::Dropout => "Suspected dropout (callback gap)",
            EventKind::ProcessingPaused => "Host paused processing",
            EventKind::Clip => "Clipping",
            EventKind::NonFinite => "Non-finite samples (NaN/Inf)",
            EventKind::SignalLost => "Signal lost (digital silence)",
            EventKind::SignalRestored => "Signal restored",
            EventKind::FormatChange => "Sample rate / buffer changed",
            EventKind::DeadlineMiss => "AID callback exceeded period",
            EventKind::Marker => "User marker",
            EventKind::GeneratorArmed => "Generator armed",
            EventKind::GeneratorStopped => "Generator stopped",
            EventKind::LatencyResult => "Round-trip measured",
            EventKind::LatencyFailed => "Round-trip run invalid",
            EventKind::HumObserved => "Mains hum observed",
            EventKind::HumCleared => "Mains hum cleared",
        }
    }

    /// Problems break the glitch-free streak and count against health.
    pub fn is_problem(self) -> bool {
        matches!(
            self,
            EventKind::Dropout
                | EventKind::Clip
                | EventKind::NonFinite
                | EventKind::SignalLost
                | EventKind::DeadlineMiss
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Event {
    pub t: f64,
    pub kind: EventKind,
    /// Kind-specific value (gap ms, sample rate, latency ms, ...).
    pub value: f32,
}

#[derive(Debug, Default, Clone)]
pub struct LatencyResults {
    pub sample_rate: f32,
    pub runs_samples: Vec<f64>,
    pub failures: u32,
    pub last_confidence: f64,
}

pub struct AtomicF64(AtomicU64);

impl AtomicF64 {
    pub fn new(v: f64) -> Self {
        Self(AtomicU64::new(v.to_bits()))
    }
    pub fn load(&self) -> f64 {
        f64::from_bits(self.0.load(Relaxed))
    }
    pub fn store(&self, v: f64) {
        self.0.store(v.to_bits(), Relaxed)
    }
}

pub struct Shared {
    // Context
    pub sample_rate: AtomicF32,
    pub max_buffer: AtomicU32,
    pub last_block: AtomicU32,
    pub channels: AtomicU32,
    pub has_return_input: AtomicBool,
    pub is_standalone: AtomicBool,
    pub plugin_api: Mutex<String>,
    pub session_secs: AtomicF64,
    pub last_problem_secs: AtomicF64,

    // Levels (published per analysis window)
    pub peak_db: [AtomicF32; 2],
    pub rms_db: [AtomicF32; 2],
    pub peak_hold_db: [AtomicF32; 2],
    pub dc: [AtomicF32; 2],
    pub crest_db: [AtomicF32; 2],
    pub correlation: AtomicF32,
    pub correlation_valid: AtomicBool,
    pub return_rms_db: AtomicF32,

    // Counters
    pub callbacks: AtomicU64,
    pub dropouts: AtomicU64,
    pub deadline_misses: AtomicU64,
    pub clipped_samples: AtomicU64,
    pub clip_runs: AtomicU64,
    pub non_finite: AtomicU64,
    pub signal_lost: AtomicU64,
    pub max_callback_ms: AtomicF32,
    pub max_load: AtomicF32,
    pub avg_load: AtomicF32,
    pub had_signal: AtomicBool,

    // Hum observation (background task)
    pub hum_hz: AtomicU32,
    pub hum_db_rel: AtomicF32,
    pub hum_signal_db: AtomicF32,
    pub hum_present: AtomicBool,
    pub hum_buffer: Mutex<Vec<f32>>,
    pub hum_sample_rate: AtomicF32,

    // Generator / Engineer safety state. Never persisted so generation can't auto-resume.
    pub armed: AtomicBool,
    pub high_level_ack: AtomicBool,
    pub generating: AtomicBool,

    // Latency measurement
    pub latency_remaining: AtomicU32,
    pub latency_busy: AtomicBool,
    pub latency_reference: Mutex<Vec<f32>>,
    pub latency_capture: Mutex<Vec<f32>>,
    pub latency_max_delay: AtomicU32,
    pub latency: Mutex<LatencyResults>,

    // Requests from GUI
    pub reset_request: AtomicBool,

    // Events
    pub events_in: Mutex<rtrb::Consumer<Event>>,
    pub event_log: Mutex<VecDeque<Event>>,
    pub events_dropped: AtomicU64,
}

impl Shared {
    pub fn new(consumer: rtrb::Consumer<Event>) -> Self {
        let db = || AtomicF32::new(MINUS_INF_DB);
        Self {
            sample_rate: AtomicF32::new(0.0),
            max_buffer: AtomicU32::new(0),
            last_block: AtomicU32::new(0),
            channels: AtomicU32::new(0),
            has_return_input: AtomicBool::new(false),
            is_standalone: AtomicBool::new(false),
            plugin_api: Mutex::new(String::from("unknown")),
            session_secs: AtomicF64::new(0.0),
            last_problem_secs: AtomicF64::new(0.0),
            peak_db: [db(), db()],
            rms_db: [db(), db()],
            peak_hold_db: [db(), db()],
            dc: [AtomicF32::new(0.0), AtomicF32::new(0.0)],
            crest_db: [AtomicF32::new(0.0), AtomicF32::new(0.0)],
            correlation: AtomicF32::new(0.0),
            correlation_valid: AtomicBool::new(false),
            return_rms_db: db(),
            callbacks: AtomicU64::new(0),
            dropouts: AtomicU64::new(0),
            deadline_misses: AtomicU64::new(0),
            clipped_samples: AtomicU64::new(0),
            clip_runs: AtomicU64::new(0),
            non_finite: AtomicU64::new(0),
            signal_lost: AtomicU64::new(0),
            max_callback_ms: AtomicF32::new(0.0),
            max_load: AtomicF32::new(0.0),
            avg_load: AtomicF32::new(0.0),
            had_signal: AtomicBool::new(false),
            hum_hz: AtomicU32::new(0),
            hum_db_rel: AtomicF32::new(MINUS_INF_DB),
            hum_signal_db: AtomicF32::new(MINUS_INF_DB),
            hum_present: AtomicBool::new(false),
            hum_buffer: Mutex::new(Vec::new()),
            hum_sample_rate: AtomicF32::new(0.0),
            armed: AtomicBool::new(false),
            high_level_ack: AtomicBool::new(false),
            generating: AtomicBool::new(false),
            latency_remaining: AtomicU32::new(0),
            latency_busy: AtomicBool::new(false),
            latency_reference: Mutex::new(Vec::new()),
            latency_capture: Mutex::new(Vec::new()),
            latency_max_delay: AtomicU32::new(0),
            latency: Mutex::new(LatencyResults::default()),
            reset_request: AtomicBool::new(false),
            events_in: Mutex::new(consumer),
            event_log: Mutex::new(VecDeque::with_capacity(EVENT_LOG_CAPACITY)),
            events_dropped: AtomicU64::new(0),
        }
    }

    /// Moves events from the audio-thread ring into the log. Never called from the audio thread.
    pub fn drain_events(&self) {
        let mut log = self.event_log.lock().unwrap();
        if let Ok(mut consumer) = self.events_in.lock() {
            while let Ok(event) = consumer.pop() {
                Self::push_log(&mut log, event);
            }
        }
    }

    /// Adds an event from a non-realtime thread (GUI markers, background results).
    pub fn log_event(&self, kind: EventKind, value: f32) {
        self.drain_events();
        let t = self.session_secs.load();
        Self::push_log(
            &mut self.event_log.lock().unwrap(),
            Event { t, kind, value },
        );
    }

    fn push_log(log: &mut VecDeque<Event>, event: Event) {
        if log.len() >= EVENT_LOG_CAPACITY {
            log.pop_front();
        }
        log.push_back(event);
    }

    pub fn events(&self) -> Vec<Event> {
        self.drain_events();
        self.event_log.lock().unwrap().iter().copied().collect()
    }

    pub fn clear_session(&self) {
        self.drain_events();
        self.event_log.lock().unwrap().clear();
        *self.latency.lock().unwrap() = LatencyResults::default();
        self.zero_counters();
        self.reset_request.store(true, Relaxed);
    }

    /// Resets all session counters. Safe from any thread (atomics only).
    pub fn zero_counters(&self) {
        for c in [
            &self.callbacks,
            &self.dropouts,
            &self.deadline_misses,
            &self.clipped_samples,
            &self.clip_runs,
            &self.non_finite,
            &self.signal_lost,
        ] {
            c.store(0, Relaxed);
        }
        for ch in 0..2 {
            self.peak_hold_db[ch].store(MINUS_INF_DB, Relaxed);
        }
        self.max_callback_ms.store(0.0, Relaxed);
        self.max_load.store(0.0, Relaxed);
        self.had_signal.store(false, Relaxed);
        self.hum_present.store(false, Relaxed);
        self.last_problem_secs.store(0.0);
        self.session_secs.store(0.0);
    }
}
