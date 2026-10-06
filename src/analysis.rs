use crate::model::{Finding, Severity, StreamStats};

pub fn dbfs_rms(samples: &[f32]) -> Option<f64> {
    if samples.is_empty() { return None; }
    let sum = samples.iter().map(|x| (*x as f64) * (*x as f64)).sum::<f64>();
    let rms = (sum / samples.len() as f64).sqrt();
    if rms == 0.0 { Some(f64::NEG_INFINITY) } else { Some(20.0 * rms.log10()) }
}

pub fn peak_dbfs(samples: &[f32]) -> Option<f64> {
    let peak = samples.iter().map(|x| x.abs() as f64).reduce(f64::max)?;
    if peak == 0.0 { Some(f64::NEG_INFINITY) } else { Some(20.0 * peak.log10()) }
}

pub fn dc_offset(samples: &[f32]) -> Option<f64> {
    if samples.is_empty() { return None; }
    Some(samples.iter().map(|x| *x as f64).sum::<f64>() / samples.len() as f64)
}

pub fn correlation(a: &[f32], b: &[f32]) -> Option<f64> {
    let n = a.len().min(b.len());
    if n < 2 { return None; }
    let (a,b)=(&a[..n],&b[..n]);
    let ma=a.iter().map(|x|*x as f64).sum::<f64>()/n as f64;
    let mb=b.iter().map(|x|*x as f64).sum::<f64>()/n as f64;
    let mut num=0.0; let mut da=0.0; let mut db=0.0;
    for i in 0..n {
        let xa=a[i] as f64-ma; let xb=b[i] as f64-mb;
        num+=xa*xb; da+=xa*xa; db+=xb*xb;
    }
    let den=(da*db).sqrt();
    if den==0.0 { None } else { Some((num/den).clamp(-1.0,1.0)) }
}

pub fn estimate_delay_samples(reference: &[f32], captured: &[f32], max_delay: usize) -> Option<usize> {
    if reference.is_empty() || captured.is_empty() { return None; }
    let limit=max_delay.min(captured.len().saturating_sub(1));
    let mut best=None;
    for delay in 0..=limit {
        let n=reference.len().min(captured.len()-delay);
        if n==0 { continue; }
        let score=(0..n).map(|i| reference[i] as f64 * captured[i+delay] as f64).sum::<f64>().abs();
        if best.map(|(_,s): (usize,f64)| score>s).unwrap_or(true) { best=Some((delay,score)); }
    }
    best.map(|x|x.0)
}

pub fn ppm_error(nominal_rate: f64, measured_rate: f64) -> Option<f64> {
    if nominal_rate <= 0.0 || measured_rate <= 0.0 { return None; }
    Some((measured_rate - nominal_rate) / nominal_rate * 1_000_000.0)
}

pub fn assess_stream(stats: StreamStats) -> Vec<Finding> {
    let mut out=Vec::new();
    if stats.xruns>0 {
        out.push(Finding{id:"stream.xruns",severity:Severity::Fail,title:"Buffer underrun/overrun detected".into(),evidence:format!("{} xruns observed",stats.xruns),action:"Increase buffer size, reduce competing load, and retest.".into()});
    }
    if stats.discontinuities>0 {
        out.push(Finding{id:"stream.discontinuity",severity:Severity::Fail,title:"Audio discontinuity detected".into(),evidence:format!("{} discontinuities observed",stats.discontinuities),action:"Inspect driver, transport, clocking and device-reset events.".into()});
    }
    let period_ms=stats.buffer_frames as f64 / stats.sample_rate * 1000.0;
    if stats.max_callback_ms > period_ms * 0.8 {
        out.push(Finding{id:"stream.callback_margin",severity:Severity::Warn,title:"Low callback timing margin".into(),evidence:format!("{:.3} ms worst callback vs {:.3} ms buffer period",stats.max_callback_ms,period_ms),action:"Reduce DSP/system load or increase buffer size before critical work.".into()});
    }
    if stats.cpu_load > 0.85 {
        out.push(Finding{id:"stream.cpu",severity:Severity::Warn,title:"High audio processing load".into(),evidence:format!("{:.0}% measured audio load",stats.cpu_load*100.0),action:"Create more CPU headroom before reducing latency.".into()});
    }
    if out.is_empty() {
        out.push(Finding{id:"stream.clean",severity:Severity::Pass,title:"Stream stable".into(),evidence:"No xruns or discontinuities and adequate callback margin.".into(),action:"No corrective action required.".into()});
    }
    out
}
