use std::f64::consts::PI;

pub fn dbfs_to_linear(dbfs: f64) -> Result<f64, &'static str> {
    if !dbfs.is_finite() || dbfs > 0.0 { return Err("dBFS amplitude must be finite and <= 0"); }
    Ok(10f64.powf(dbfs / 20.0))
}

pub fn sine(frequency_hz: f64, dbfs: f64, sample_rate: f64, frames: usize) -> Result<Vec<f32>, &'static str> {
    if frequency_hz <= 0.0 || sample_rate <= 0.0 || frequency_hz >= sample_rate / 2.0 {
        return Err("frequency must be above 0 and below Nyquist");
    }
    let amp = dbfs_to_linear(dbfs)?;
    Ok((0..frames).map(|i| (amp * (2.0 * PI * frequency_hz * i as f64 / sample_rate).sin()) as f32).collect())
}

pub fn linear_sweep(start_hz: f64, end_hz: f64, dbfs: f64, sample_rate: f64, frames: usize) -> Result<Vec<f32>, &'static str> {
    if frames == 0 || start_hz <= 0.0 || end_hz <= start_hz || end_hz >= sample_rate / 2.0 { return Err("invalid sweep parameters"); }
    let amp = dbfs_to_linear(dbfs)?;
    let duration = frames as f64 / sample_rate;
    let k = (end_hz - start_hz) / duration;
    Ok((0..frames).map(|i| {
        let t = i as f64 / sample_rate;
        let phase = 2.0 * PI * (start_hz * t + 0.5 * k * t * t);
        (amp * phase.sin()) as f32
    }).collect())
}

pub fn clipped_sample_count(samples: &[f32], threshold: f32) -> usize {
    samples.iter().filter(|x| x.abs() >= threshold).count()
}

pub fn crest_factor_db(samples: &[f32]) -> Option<f64> {
    let rms = crate::dbfs_rms(samples)?;
    let peak = crate::peak_dbfs(samples)?;
    if !rms.is_finite() || !peak.is_finite() { None } else { Some(peak - rms) }
}

pub fn gain_mismatch_db(a: &[f32], b: &[f32]) -> Option<f64> {
    let ar = crate::dbfs_rms(a)?;
    let br = crate::dbfs_rms(b)?;
    if !ar.is_finite() || !br.is_finite() { None } else { Some(ar - br) }
}
