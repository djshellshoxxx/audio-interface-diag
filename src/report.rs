use crate::model::SessionSummary;

pub fn render_text(summary: &SessionSummary) -> String {
    let mut output = String::new();
    output.push_str(&format!("Audio Interface Diag - {:?}\n", summary.edition));

    for finding in &summary.findings {
        output.push_str(&format!(
            "[{}] {}: {}\n  Evidence: {}\n  Action: {}\n",
            finding.severity.label(),
            finding.id,
            finding.title,
            finding.evidence,
            finding.action
        ));
    }

    output
}

/// Escapes a string for inclusion in a JSON document.
pub fn json_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

pub fn render_json(summary: &SessionSummary) -> String {
    let findings = summary
        .findings
        .iter()
        .map(|f| {
            format!(
                "{{\"id\":\"{}\",\"status\":\"{}\",\"title\":\"{}\",\"evidence\":\"{}\",\"action\":\"{}\"}}",
                json_escape(f.id),
                f.severity.label(),
                json_escape(&f.title),
                json_escape(&f.evidence),
                json_escape(&f.action)
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "{{\"schema\":\"aid.report.v1\",\"edition\":\"{:?}\",\"findings\":[{}]}}",
        summary.edition, findings
    )
}

/// Escapes a CSV field per RFC 4180.
pub fn csv_field(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

pub fn render_csv(summary: &SessionSummary) -> String {
    let mut out = String::from("id,status,title,evidence,action\n");
    for f in &summary.findings {
        out.push_str(&format!(
            "{},{},{},{},{}\n",
            csv_field(f.id),
            f.severity.label(),
            csv_field(&f.title),
            csv_field(&f.evidence),
            csv_field(&f.action)
        ));
    }
    out
}
