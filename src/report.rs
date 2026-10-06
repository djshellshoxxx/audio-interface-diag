use crate::model::{SessionSummary, Severity};

pub fn render_text(summary: &SessionSummary) -> String {
    let mut s=String::new();
    s.push_str(&format!("Audio Interface Diag - {:?}\n",summary.edition));
    for f in &summary.findings {
        let state=match f.severity { Severity::Pass=>"PASS",Severity::Info=>"INFO",Severity::Warn=>"WARN",Severity::Fail=>"FAIL" };
        s.push_str(&format!("[{state}] {}: {}\n  Evidence: {}\n  Action: {}\n",f.id,f.title,f.evidence,f.action));
    }
    s
}
