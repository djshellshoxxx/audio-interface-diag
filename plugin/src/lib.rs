//! Audio Interface Diag as a CLAP/VST3 plugin and standalone application.
//!
//! Live and DAW modes are strictly passive: audio passes through untouched (plugin hosts) and no
//! signal is generated. Engineer mode can emit test signals only after the user explicitly arms the
//! generator in the GUI. Arming is never persisted, so a restored session never resumes output.

use audio_interface_diag::{analyze_hum, apply_fade, measure_delay, noise_burst, Edition};
use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use shared::{Event, EventKind, Shared, MINUS_INF_DB};
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;
use std::time::Instant;

mod editor;
pub mod report;
pub mod shared;

const ANALYSIS_WINDOW_SECS: f32 = 0.3;
const SIGNAL_LOST_SECS: f32 = 2.0;
const LATENCY_REFERENCE_LEN: usize = 4096;
const LATENCY_PREROLL_SECS: f32 = 0.25;
const LATENCY_MAX_DELAY_SECS: f32 = 1.0;
const CLIP_EVENT_HOLDOFF_SECS: f64 = 0.5;
const PAUSE_THRESHOLD_SECS: f64 = 1.0;

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Live,
    #[name = "DAW"]
    Daw,
    Engineer,
}

impl Mode {
    pub fn edition(self) -> Edition {
        match self {
            Mode::Live => Edition::Live,
            Mode::Daw => Edition::Daw,
            Mode::Engineer => Edition::Engineer,
        }
    }
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenType {
    Off,
    Sine,
    #[name = "Pink noise"]
    Pink,
    #[name = "White noise"]
    White,
    #[name = "Impulse (1/s)"]
    Impulse,
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenChannel {
    Both,
    Left,
    Right,
}

#[derive(Enum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnSource {
    #[name = "Main input"]
    MainInput,
    #[name = "Sidechain"]
    Sidechain,
}

#[derive(Params)]
pub struct AidParams {
    #[persist = "editor-state"]
    pub editor_state: Arc<EguiState>,

    #[id = "mode"]
    pub mode: EnumParam<Mode>,
    #[id = "gen_type"]
    pub gen_type: EnumParam<GenType>,
    #[id = "gen_freq"]
    pub gen_freq: FloatParam,
    #[id = "gen_level"]
    pub gen_level: FloatParam,
    #[id = "gen_chan"]
    pub gen_channel: EnumParam<GenChannel>,
    #[id = "return"]
    pub return_source: EnumParam<ReturnSource>,
    #[id = "clip_thr"]
    pub clip_threshold: FloatParam,
    #[id = "sa_mon"]
    pub standalone_monitor: BoolParam,
}

impl Default for AidParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(980, 660),
            mode: EnumParam::new("Mode", Mode::Live),
            gen_type: EnumParam::new("Generator", GenType::Sine),
            gen_freq: FloatParam::new(
                "Frequency",
                1000.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20_000.0,
                    factor: FloatRange::skew_factor(-2.0),
                },
            )
            .with_unit(" Hz")
            .with_value_to_string(formatters::v2s_f32_rounded(1)),
            gen_level: FloatParam::new(
                "Level",
                audio_interface_diag::DEFAULT_GENERATOR_DBFS as f32,
                FloatRange::Linear {
                    min: -60.0,
                    max: 0.0,
                },
            )
            .with_unit(" dBFS")
            .with_step_size(0.5),
            gen_channel: EnumParam::new("Output channel", GenChannel::Both),
            return_source: EnumParam::new("Return source", ReturnSource::MainInput),
            clip_threshold: FloatParam::new(
                "Clip threshold",
                -0.1,
                FloatRange::Linear {
                    min: -6.0,
                    max: 0.0,
                },
            )
            .with_unit(" dBFS")
            .with_step_size(0.1),
            standalone_monitor: BoolParam::new("Standalone monitor", false),
        }
    }
}

pub enum Task {
    Latency,
    Hum,
}

#[derive(Clone, Copy, PartialEq)]
enum LatencyState {
    Idle,
    PreRoll(usize),
    Capture(usize),
}

pub struct AudioInterfaceDiag {
    params: Arc<AidParams>,
    shared: Arc<Shared>,
    events: rtrb::Producer<Event>,

    sample_rate: f32,
    realtime: bool,
    epoch: Instant,
    last_call: Option<(Instant, usize)>,

    // Analysis window accumulators
    window_len: usize,
    window_pos: usize,
    sum_sq: [f64; 2],
    sum: [f64; 2],
    peak: [f32; 2],
    dc_smoothed: [f32; 2],
    corr_ab: f64,
    return_sum_sq: f64,
    silent_samples: usize,
    signal_lost_active: bool,
    in_clip: [bool; 2],
    last_clip_event: f64,

    // Hum capture
    hum_buffer: Vec<f32>,
    hum_pos: usize,

    // Generator
    gen_gain: f32,
    phase: f64,
    rng: u32,
    pink: [f32; 3],
    impulse_counter: usize,
    was_armed: bool,

    // Latency
    latency_state: LatencyState,
    latency_reference: Vec<f32>,
    latency_capture: Vec<f32>,
}

impl Default for AudioInterfaceDiag {
    fn default() -> Self {
        let (producer, consumer) = rtrb::RingBuffer::new(1024);
        Self {
            params: Arc::new(AidParams::default()),
            shared: Arc::new(Shared::new(consumer)),
            events: producer,
            sample_rate: 48_000.0,
            realtime: true,
            epoch: Instant::now(),
            last_call: None,
            window_len: 1,
            window_pos: 0,
            sum_sq: [0.0; 2],
            sum: [0.0; 2],
            peak: [0.0; 2],
            dc_smoothed: [0.0; 2],
            corr_ab: 0.0,
            return_sum_sq: 0.0,
            silent_samples: 0,
            signal_lost_active: false,
            in_clip: [false; 2],
            last_clip_event: f64::NEG_INFINITY,
            hum_buffer: Vec::new(),
            hum_pos: 0,
            gen_gain: 0.0,
            phase: 0.0,
            rng: 0x1234_5678,
            pink: [0.0; 3],
            impulse_counter: 0,
            was_armed: false,
            latency_state: LatencyState::Idle,
            latency_reference: Vec::new(),
            latency_capture: Vec::new(),
        }
    }
}

pub fn to_db(linear: f64) -> f32 {
    if linear > 0.0 {
        (20.0 * linear.log10()).max(MINUS_INF_DB as f64) as f32
    } else {
        MINUS_INF_DB
    }
}

impl AudioInterfaceDiag {
    fn now_secs(&self, now: Instant) -> f64 {
        now.duration_since(self.epoch).as_secs_f64()
    }

    fn emit(&mut self, t: f64, kind: EventKind, value: f32) {
        if kind.is_problem() {
            self.shared.last_problem_secs.store(t);
        }
        if self.events.push(Event { t, kind, value }).is_err() {
            self.shared.events_dropped.fetch_add(1, Relaxed);
        }
    }

    fn reset_session(&mut self, now: Instant) {
        self.shared.zero_counters();
        self.epoch = now;
        self.last_call = None;
        self.window_pos = 0;
        self.sum_sq = [0.0; 2];
        self.sum = [0.0; 2];
        self.peak = [0.0; 2];
        self.corr_ab = 0.0;
        self.return_sum_sq = 0.0;
        self.silent_samples = 0;
        self.signal_lost_active = false;
        self.last_clip_event = f64::NEG_INFINITY;
        self.emit(0.0, EventKind::SessionStart, self.sample_rate);
    }

    fn next_noise(&mut self) -> f32 {
        self.rng = self.rng.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        (self.rng >> 8) as f32 / (1u32 << 24) as f32 * 2.0 - 1.0
    }

    /// One generator sample at unit amplitude.
    fn generator_sample(&mut self, kind: GenType, freq: f32) -> f32 {
        match kind {
            GenType::Off => 0.0,
            GenType::Sine => {
                let s = (self.phase * std::f64::consts::TAU).sin() as f32;
                self.phase = (self.phase + freq as f64 / self.sample_rate as f64).fract();
                s
            }
            GenType::White => self.next_noise(),
            GenType::Pink => {
                // Paul Kellet's economy pink filter, scaled to roughly unit peak.
                let w = self.next_noise();
                self.pink[0] = 0.99765 * self.pink[0] + w * 0.099_046;
                self.pink[1] = 0.96300 * self.pink[1] + w * 0.296_516_4;
                self.pink[2] = 0.57000 * self.pink[2] + w * 1.052_691_3;
                ((self.pink[0] + self.pink[1] + self.pink[2] + w * 0.1848) * 0.25).clamp(-1.0, 1.0)
            }
            GenType::Impulse => {
                let s = if self.impulse_counter == 0 { 1.0 } else { 0.0 };
                self.impulse_counter += 1;
                if self.impulse_counter >= self.sample_rate as usize {
                    self.impulse_counter = 0;
                }
                s
            }
        }
    }

    fn publish_window(&mut self, channels: usize, t: f64) {
        let n = self.window_pos.max(1) as f64;
        let mut any_signal = false;
        for ch in 0..channels.min(2) {
            let rms = (self.sum_sq[ch] / n).sqrt();
            let mean = (self.sum[ch] / n) as f32;
            self.dc_smoothed[ch] = 0.8 * self.dc_smoothed[ch] + 0.2 * mean;
            let rms_db = to_db(rms);
            let peak_db = to_db(self.peak[ch] as f64);
            any_signal |= self.peak[ch] > 0.0;
            self.shared.rms_db[ch].store(rms_db, Relaxed);
            self.shared.peak_db[ch].store(peak_db, Relaxed);
            self.shared.dc[ch].store(self.dc_smoothed[ch], Relaxed);
            self.shared.crest_db[ch].store(if rms > 0.0 { peak_db - rms_db } else { 0.0 }, Relaxed);
            if peak_db > self.shared.peak_hold_db[ch].load(Relaxed) {
                self.shared.peak_hold_db[ch].store(peak_db, Relaxed);
            }
        }

        if channels >= 2 {
            // Pearson correlation of the window (DC removed).
            let (ma, mb) = (self.sum[0] / n, self.sum[1] / n);
            let cov = self.corr_ab / n - ma * mb;
            let va = self.sum_sq[0] / n - ma * ma;
            let vb = self.sum_sq[1] / n - mb * mb;
            let valid = va > 1e-12 && vb > 1e-12;
            self.shared.correlation_valid.store(valid, Relaxed);
            if valid {
                self.shared
                    .correlation
                    .store((cov / (va * vb).sqrt()).clamp(-1.0, 1.0) as f32, Relaxed);
            }
        } else {
            self.shared.correlation_valid.store(false, Relaxed);
        }
        self.shared
            .return_rms_db
            .store(to_db((self.return_sum_sq / n).sqrt()), Relaxed);

        // Digital silence after signal was present => signal lost.
        if any_signal {
            self.shared.had_signal.store(true, Relaxed);
            self.silent_samples = 0;
            if self.signal_lost_active {
                self.signal_lost_active = false;
                self.emit(t, EventKind::SignalRestored, 0.0);
            }
        } else if self.shared.had_signal.load(Relaxed) && !self.signal_lost_active {
            self.silent_samples += self.window_pos;
            if self.silent_samples as f32 >= SIGNAL_LOST_SECS * self.sample_rate {
                self.signal_lost_active = true;
                self.shared.signal_lost.fetch_add(1, Relaxed);
                self.emit(t, EventKind::SignalLost, 0.0);
            }
        }

        self.window_pos = 0;
        self.sum_sq = [0.0; 2];
        self.sum = [0.0; 2];
        self.peak = [0.0; 2];
        self.corr_ab = 0.0;
        self.return_sum_sq = 0.0;
    }

    fn submit_hum(&mut self, context: &mut impl ProcessContext<Self>) {
        if let Ok(mut buffer) = self.shared.hum_buffer.try_lock() {
            if buffer.len() == self.hum_buffer.len() {
                std::mem::swap(&mut *buffer, &mut self.hum_buffer);
                drop(buffer);
                context.execute_background(Task::Hum);
            }
        }
        self.hum_pos = 0;
    }

    fn submit_latency(&mut self, t: f64, context: &mut impl ProcessContext<Self>) {
        let submitted = match self.shared.latency_capture.try_lock() {
            Ok(mut capture) if capture.len() == self.latency_capture.len() => {
                std::mem::swap(&mut *capture, &mut self.latency_capture);
                true
            }
            _ => false,
        };
        if submitted {
            self.shared.latency_busy.store(true, Relaxed);
            context.execute_background(Task::Latency);
        } else {
            self.emit(t, EventKind::LatencyFailed, 0.0);
        }
        self.latency_state = LatencyState::Idle;
    }
}

fn run_hum_task(shared: &Shared) {
    let buffer = shared.hum_buffer.lock().unwrap();
    let sr = shared.hum_sample_rate.load(Relaxed) as f64;
    let rms = audio_interface_diag::dbfs_rms(&buffer).unwrap_or(f64::NEG_INFINITY);
    let hum = analyze_hum(&buffer, sr);
    drop(buffer);
    shared
        .hum_signal_db
        .store(rms.max(MINUS_INF_DB as f64) as f32, Relaxed);
    let present = match hum {
        Some(h) => {
            shared.hum_hz.store(h.mains_hz, Relaxed);
            shared
                .hum_db_rel
                .store(h.family_db_rel_total as f32, Relaxed);
            // Hum dominating a quiet input (noise-floor conditions) is worth reporting.
            h.family_db_rel_total > -10.0 && rms > -100.0 && rms < -30.0
        }
        None => {
            shared.hum_hz.store(0, Relaxed);
            false
        }
    };
    if present != shared.hum_present.swap(present, Relaxed) {
        let kind = if present {
            EventKind::HumObserved
        } else {
            EventKind::HumCleared
        };
        shared.log_event(kind, shared.hum_hz.load(Relaxed) as f32);
    }
}

fn run_latency_task(shared: &Shared) {
    let reference = shared.latency_reference.lock().unwrap();
    let capture = shared.latency_capture.lock().unwrap();
    let max_delay = shared.latency_max_delay.load(Relaxed) as usize;
    let measured = measure_delay(&reference, &capture, max_delay);
    drop((reference, capture));
    let sr = shared.sample_rate.load(Relaxed);
    let mut results = shared.latency.lock().unwrap();
    results.sample_rate = sr;
    match measured {
        Some(m) => {
            results.runs_samples.push(m.delay_samples as f64);
            results.last_confidence = m.confidence;
            drop(results);
            shared.log_event(
                EventKind::LatencyResult,
                m.delay_samples as f32 / sr * 1000.0,
            );
        }
        None => {
            results.failures += 1;
            drop(results);
            shared.log_event(EventKind::LatencyFailed, 0.0);
        }
    }
    shared.latency_busy.store(false, Relaxed);
}

impl Plugin for AudioInterfaceDiag {
    const NAME: &'static str = "Audio Interface Diag";
    const VENDOR: &'static str = "Circuit Drift Labs";
    const URL: &'static str = "https://github.com/djshellshoxxx/audio-interface-diag";
    const EMAIL: &'static str = "noreply@example.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            aux_input_ports: &[new_nonzero_u32(2)],
            names: PortNames {
                layout: Some("Stereo + loopback return"),
                main_input: Some("Input"),
                main_output: Some("Output"),
                aux_inputs: &["Loopback return"],
                aux_outputs: &[],
            },
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            aux_input_ports: &[new_nonzero_u32(1)],
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(1),
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

    type SysExMessage = ();
    type BackgroundTask = Task;

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn task_executor(&mut self) -> TaskExecutor<Self> {
        let shared = self.shared.clone();
        Box::new(move |task| match task {
            Task::Hum => run_hum_task(&shared),
            Task::Latency => run_latency_task(&shared),
        })
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(self.params.clone(), self.shared.clone())
    }

    fn initialize(
        &mut self,
        audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        context: &mut impl InitContext<Self>,
    ) -> bool {
        let sr = buffer_config.sample_rate;
        let first = self.shared.sample_rate.load(Relaxed) == 0.0;
        let changed = self.shared.sample_rate.load(Relaxed) != sr
            || self.shared.max_buffer.load(Relaxed) != buffer_config.max_buffer_size;

        self.sample_rate = sr;
        self.realtime = buffer_config.process_mode != ProcessMode::Offline;
        self.window_len = ((ANALYSIS_WINDOW_SECS * sr) as usize).max(1);

        let api = context.plugin_api();
        self.shared
            .is_standalone
            .store(api == PluginApi::Standalone, Relaxed);
        *self.shared.plugin_api.lock().unwrap() = format!("{api:?}");
        self.shared.sample_rate.store(sr, Relaxed);
        self.shared
            .max_buffer
            .store(buffer_config.max_buffer_size, Relaxed);
        self.shared.channels.store(
            audio_io_layout.main_input_channels.map_or(0, |c| c.get()),
            Relaxed,
        );
        self.shared
            .has_return_input
            .store(!audio_io_layout.aux_input_ports.is_empty(), Relaxed);

        // Preallocate every realtime buffer here so process() never allocates.
        let hum_len = (sr as usize).max(1);
        self.hum_buffer = vec![0.0; hum_len];
        *self.shared.hum_buffer.lock().unwrap() = vec![0.0; hum_len];
        self.shared.hum_sample_rate.store(sr, Relaxed);
        self.hum_pos = 0;

        let mut reference =
            noise_burst(0.0, LATENCY_REFERENCE_LEN, 0xA1D).expect("valid burst parameters");
        apply_fade(&mut reference, 32);
        let max_delay = (LATENCY_MAX_DELAY_SECS * sr) as usize;
        let capture_len = LATENCY_REFERENCE_LEN + max_delay;
        self.latency_reference = reference.clone();
        *self.shared.latency_reference.lock().unwrap() = reference;
        self.latency_capture = vec![0.0; capture_len];
        *self.shared.latency_capture.lock().unwrap() = vec![0.0; capture_len];
        self.shared
            .latency_max_delay
            .store(max_delay as u32, Relaxed);
        self.latency_state = LatencyState::Idle;

        if first {
            self.reset_session(Instant::now());
        } else if changed {
            let t = self.now_secs(Instant::now());
            self.emit(t, EventKind::FormatChange, sr);
        }
        true
    }

    fn reset(&mut self) {
        // Re-activation: discard timing history so the gap is not reported as a dropout.
        self.last_call = None;
        self.latency_state = LatencyState::Idle;
        self.gen_gain = 0.0;
        self.phase = 0.0;
        self.pink = [0.0; 3];
        self.impulse_counter = 0;
    }

    fn deactivate(&mut self) {
        self.last_call = None;
        self.shared.generating.store(false, Relaxed);
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let start = Instant::now();
        let num_samples = buffer.samples();
        if num_samples == 0 {
            return ProcessStatus::Normal;
        }
        if self.shared.reset_request.swap(false, Relaxed) {
            self.reset_session(start);
        }
        let t = self.now_secs(start);
        self.shared.session_secs.store(t);
        self.shared.callbacks.fetch_add(1, Relaxed);
        self.shared.last_block.store(num_samples as u32, Relaxed);

        // Callback gap detection: an interval much longer than the previous block's period means
        // the host did not deliver audio in time (an inferred, "suspected" dropout).
        if let Some((prev, prev_len)) = self.last_call {
            let interval = start.duration_since(prev).as_secs_f64();
            let expected = prev_len as f64 / self.sample_rate as f64;
            if interval > PAUSE_THRESHOLD_SECS {
                self.emit(t, EventKind::ProcessingPaused, interval as f32);
            } else if self.realtime && interval > (expected * 2.5).max(expected + 0.005) {
                self.shared.dropouts.fetch_add(1, Relaxed);
                self.emit(t, EventKind::Dropout, (interval * 1000.0) as f32);
            }
        }
        self.last_call = Some((start, num_samples));

        let mode = self.params.mode.value();
        let gen_type = self.params.gen_type.value();
        let gen_freq = self.params.gen_freq.value().min(self.sample_rate * 0.45);
        let gen_channel = self.params.gen_channel.value();
        let use_sidechain = self.params.return_source.value() == ReturnSource::Sidechain;
        let clip_lin = util::db_to_gain(self.params.clip_threshold.value());
        let standalone = self.shared.is_standalone.load(Relaxed);
        let monitor = !standalone || self.params.standalone_monitor.value();

        // Safety: leaving Engineer mode disarms immediately.
        if mode != Mode::Engineer {
            self.shared.armed.store(false, Relaxed);
            self.shared.latency_remaining.store(0, Relaxed);
        }
        let armed = self.shared.armed.load(Relaxed);
        if armed != self.was_armed {
            self.was_armed = armed;
            let kind = if armed {
                EventKind::GeneratorArmed
            } else {
                EventKind::GeneratorStopped
            };
            self.emit(t, kind, 0.0);
        }
        if !armed && self.latency_state != LatencyState::Idle {
            self.latency_state = LatencyState::Idle;
            self.emit(t, EventKind::LatencyFailed, 0.0);
        }

        // Start the next latency run when requested and the previous analysis has finished.
        if armed
            && self.latency_state == LatencyState::Idle
            && !self.shared.latency_busy.load(Relaxed)
            && self.shared.latency_remaining.load(Relaxed) > 0
        {
            self.shared.latency_remaining.fetch_sub(1, Relaxed);
            self.latency_state =
                LatencyState::PreRoll(((LATENCY_PREROLL_SECS * self.sample_rate) as usize).max(1));
        }

        let level_db = audio_interface_diag::validate_generator_level(
            self.params.gen_level.value() as f64,
            self.shared.high_level_ack.load(Relaxed),
        )
        .unwrap_or(audio_interface_diag::ACK_REQUIRED_ABOVE_DBFS);
        let level = util::db_to_gain(level_db as f32);
        let ramp_in = 1.0 / (0.02 * self.sample_rate);
        let ramp_out = 1.0 / (0.002 * self.sample_rate);

        let channels = buffer.channels();
        let return_slices = if use_sidechain {
            aux.inputs
                .get_mut(0)
                .map(|b| b.as_slice())
                .filter(|s| !s.is_empty())
        } else {
            None
        };
        let main = buffer.as_slice();

        let mut non_finite_block = false;
        let mut latency_done = false;
        for i in 0..num_samples {
            // --- Passive analysis of the incoming signal (before any output is written) ---
            let mut input = [0.0f32; 2];
            for ch in 0..channels.min(2) {
                let x = main[ch][i];
                input[ch] = x;
                if !x.is_finite() {
                    non_finite_block = true;
                    continue;
                }
                let a = x.abs();
                self.sum_sq[ch] += (x as f64) * (x as f64);
                self.sum[ch] += x as f64;
                if a > self.peak[ch] {
                    self.peak[ch] = a;
                }
                if a >= clip_lin {
                    self.shared.clipped_samples.fetch_add(1, Relaxed);
                    if !self.in_clip[ch] {
                        self.in_clip[ch] = true;
                        self.shared.clip_runs.fetch_add(1, Relaxed);
                        if t - self.last_clip_event > CLIP_EVENT_HOLDOFF_SECS {
                            self.last_clip_event = t;
                            self.emit(t, EventKind::Clip, to_db(a as f64));
                        }
                    }
                } else {
                    self.in_clip[ch] = false;
                }
            }
            if channels >= 2 && input[0].is_finite() && input[1].is_finite() {
                self.corr_ab += input[0] as f64 * input[1] as f64;
            }

            let return_sample = match &return_slices {
                Some(r) => r[0][i],
                None => input[0],
            };
            if return_sample.is_finite() {
                self.return_sum_sq += (return_sample as f64).powi(2);
            }

            let mono = if channels >= 2 {
                (input[0] + input[1]) * 0.5
            } else {
                input[0]
            };
            if self.hum_pos < self.hum_buffer.len() {
                self.hum_buffer[self.hum_pos] = if mono.is_finite() { mono } else { 0.0 };
                self.hum_pos += 1;
                if self.hum_pos == self.hum_buffer.len() {
                    self.submit_hum(context);
                }
            }

            self.window_pos += 1;
            if self.window_pos >= self.window_len {
                self.publish_window(channels, t);
            }

            // --- Output ---
            let measuring = self.latency_state != LatencyState::Idle;
            let gen_active = armed && (gen_type != GenType::Off || measuring);
            let target = if gen_active { 1.0 } else { 0.0 };
            if self.gen_gain < target {
                self.gen_gain = (self.gen_gain + ramp_in).min(1.0);
            } else if self.gen_gain > target {
                self.gen_gain = (self.gen_gain - ramp_out).max(0.0);
            }

            let mut generated = 0.0;
            match self.latency_state {
                LatencyState::PreRoll(remaining) => {
                    self.latency_state = if remaining <= 1 {
                        LatencyState::Capture(0)
                    } else {
                        LatencyState::PreRoll(remaining - 1)
                    };
                }
                LatencyState::Capture(pos) => {
                    // The burst is emitted at full generator gain from the first sample so the
                    // reference shape is not distorted by the ramp (it carries its own fade).
                    if pos < self.latency_reference.len() {
                        generated = self.latency_reference[pos] * level;
                    }
                    self.latency_capture[pos] = if return_sample.is_finite() {
                        return_sample
                    } else {
                        0.0
                    };
                    if pos + 1 >= self.latency_capture.len() {
                        latency_done = true;
                        self.latency_state = LatencyState::Idle;
                    } else {
                        self.latency_state = LatencyState::Capture(pos + 1);
                    }
                }
                LatencyState::Idle => {
                    if self.gen_gain > 0.0 {
                        generated =
                            self.generator_sample(gen_type, gen_freq) * level * self.gen_gain;
                    }
                }
            }

            let generator_owns_output = self.gen_gain > 0.0 || measuring;
            for (ch, channel) in main.iter_mut().enumerate() {
                if generator_owns_output {
                    let routed = match gen_channel {
                        GenChannel::Both => true,
                        GenChannel::Left => ch == 0,
                        GenChannel::Right => ch == 1 || channels == 1,
                    };
                    channel[i] = if routed { generated } else { 0.0 };
                } else if !monitor {
                    channel[i] = 0.0;
                }
                // Otherwise: passive pass-through, the input is left untouched.
            }
        }
        self.shared.generating.store(self.gen_gain > 0.0, Relaxed);

        if non_finite_block {
            self.shared.non_finite.fetch_add(1, Relaxed);
            self.emit(t, EventKind::NonFinite, 0.0);
        }
        if latency_done {
            self.submit_latency(t, context);
        }

        // AID-owned callback timing (measured), relative to the buffer period.
        let elapsed = start.elapsed().as_secs_f64();
        let period = num_samples as f64 / self.sample_rate as f64;
        let load = (elapsed / period) as f32;
        let ms = (elapsed * 1000.0) as f32;
        if ms > self.shared.max_callback_ms.load(Relaxed) {
            self.shared.max_callback_ms.store(ms, Relaxed);
        }
        if load > self.shared.max_load.load(Relaxed) {
            self.shared.max_load.store(load, Relaxed);
        }
        let avg = self.shared.avg_load.load(Relaxed);
        self.shared
            .avg_load
            .store(avg * 0.99 + load * 0.01, Relaxed);
        if load >= 1.0 && self.realtime {
            self.shared.deadline_misses.fetch_add(1, Relaxed);
            self.emit(t, EventKind::DeadlineMiss, ms);
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for AudioInterfaceDiag {
    const CLAP_ID: &'static str = "com.circuitdriftlabs.audio-interface-diag";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Passive audio-path monitoring and armed loopback diagnostics");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Analyzer,
        ClapFeature::Utility,
        ClapFeature::Stereo,
        ClapFeature::Mono,
    ];
}

impl Vst3Plugin for AudioInterfaceDiag {
    const VST3_CLASS_ID: [u8; 16] = *b"CDLAudioIfaceDia";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,
        Vst3SubCategory::Analyzer,
        Vst3SubCategory::Tools,
    ];
}

nih_export_clap!(AudioInterfaceDiag);
// nih-plug's VST3 vtable macro ends in a semicolon in expression position, which newer
// rustc releases reject (rust-lang/rust#79813). The code is upstream; allow it here.
#[allow(
    unknown_lints,
    semicolon_in_expressions_from_macros,
    semicolon_in_expressions_from_non_local_macros
)]
mod vst3_export {
    use super::AudioInterfaceDiag;
    use nih_plug::prelude::*;
    nih_export_vst3!(AudioInterfaceDiag);
}
