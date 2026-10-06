#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edition {
    Live,
    Daw,
    Engineer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Pass,
    Info,
    Warn,
    Fail,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceCapability {
    pub name: String,
    pub host_api: String,
    pub input_channels: usize,
    pub output_channels: usize,
    pub sample_rates: Vec<u32>,
    pub buffer_sizes: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StreamStats {
    pub sample_rate: f64,
    pub buffer_frames: u32,
    pub callback_count: u64,
    pub xruns: u64,
    pub discontinuities: u64,
    pub max_callback_ms: f64,
    pub cpu_load: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    pub id: &'static str,
    pub severity: Severity,
    pub title: String,
    pub evidence: String,
    pub action: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SessionSummary {
    pub edition: Edition,
    pub findings: Vec<Finding>,
}
