//! Passive host-path reliability tracking. No device is opened or output generated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BurnInState {
    Running,
    Complete,
    Aborted,
    Unavailable,
}

#[derive(Debug, Clone, Copy)]
pub struct BurnInTelemetry {
    pub epoch: u64,
    pub callbacks: u64,
    pub faults: u64,
    pub sample_rate: f64,
}

#[derive(Debug, Clone)]
pub struct BurnIn {
    pub state: BurnInState,
    pub target_secs: f64,
    pub elapsed_secs: f64,
    pub callbacks: u64,
    pub faults: u64,
    pub reason: &'static str,
    baseline: BurnInTelemetry,
    last_callbacks: u64,
    last_faults: u64,
    last_progress: f64,
}

impl BurnIn {
    pub fn new(target_secs: f64, baseline: BurnInTelemetry) -> Result<Self, &'static str> {
        if !target_secs.is_finite() || !(1.0..=604_800.0).contains(&target_secs) {
            return Err("duration must be between one second and seven days");
        }
        if !baseline.sample_rate.is_finite()
            || baseline.sample_rate <= 0.0
            || baseline.callbacks == 0
        {
            return Err("start audio processing before starting burn-in");
        }
        Ok(Self {
            state: BurnInState::Running,
            target_secs,
            elapsed_secs: 0.0,
            callbacks: 0,
            faults: 0,
            reason: "Monitoring host callbacks (not driver xrun qualification)",
            baseline,
            last_callbacks: baseline.callbacks,
            last_faults: baseline.faults,
            last_progress: 0.0,
        })
    }

    /// `elapsed` is monotonic wall time. Call from a background/UI thread. Gaps
    /// over two seconds cannot be retrospectively certified by counter totals.
    pub fn observe(&mut self, elapsed: f64, now: BurnInTelemetry) {
        if self.state != BurnInState::Running {
            return;
        }
        if !elapsed.is_finite()
            || elapsed < self.elapsed_secs
            || now.epoch != self.baseline.epoch
            || now.sample_rate != self.baseline.sample_rate
            || now.callbacks < self.last_callbacks
            || now.faults < self.last_faults
        {
            self.state = BurnInState::Unavailable;
            self.reason = "Host/session context changed or telemetry was reset";
            return;
        }
        if elapsed - self.elapsed_secs > 2.0 {
            self.state = BurnInState::Unavailable;
            self.reason = "Monitoring gap exceeded two seconds; continuity is unqualified";
            return;
        }
        self.elapsed_secs = elapsed;
        self.callbacks = now.callbacks - self.baseline.callbacks;
        self.faults = now.faults - self.baseline.faults;
        self.last_faults = now.faults;
        if now.callbacks > self.last_callbacks {
            self.last_progress = elapsed;
        }
        self.last_callbacks = now.callbacks;
        if elapsed - self.last_progress >= 2.0 {
            self.state = BurnInState::Unavailable;
            self.reason = "Audio callbacks stopped advancing";
        } else if elapsed >= self.target_secs && self.callbacks > 0 {
            self.state = BurnInState::Complete;
            self.reason = "Requested host-path observation completed";
        }
    }

    pub fn abort(&mut self) {
        if self.state == BurnInState::Running {
            self.state = BurnInState::Aborted;
            self.reason = "Stopped by user; incomplete observation";
        }
    }
}
