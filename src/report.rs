use crate::model::{SessionSummary, Severity};

pub fn render_text(summary: &SessionSummary) -> String {
    let mut output = String::new();
    output.push_str(&format!("Audio Interface Diag - {:?}\n", summary.edition));

    for finding in &summary.findings {
        let state = match finding.severity {
            Severity::Pass => "PASS",
            Severity::Info => "INFO",
            Severity::Warn => "WARN",
            Severity::Fail => "FAIL",
        };

        output.push_str(&format!(
            "[{state}] {}: {}\n  Evidence: {}\n  Action: {}\n",
            finding.id, finding.title, finding.evidence, finding.action
        ));
    }

    output
}
