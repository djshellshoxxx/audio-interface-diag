//! Read-only, independently decimated signal plots. No measurement or audio I/O.
use crate::shared::Shared;
use nih_plug_egui::egui::{self, Color32};
use std::sync::atomic::Ordering::Relaxed;
use std::time::Instant;

pub(crate) fn draw(ui: &mut egui::Ui, shared: &Shared, accent: Color32) {
    // Never wait on analysis. Samples/FFT remain on the background worker.
    let plots = shared.plots.try_lock().ok().and_then(|plots| plots.clone());
    let Some(plots) = plots else {
        ui.label("Scope / spectrum unavailable — waiting for a valid capture.");
        return;
    };
    if !plots.current(shared.context_epoch.load(Relaxed), Instant::now()) {
        ui.label("Scope / spectrum unavailable — telemetry is stale or the host context changed.");
        return;
    }
    ui.collapsing("Scope and spectrum (mono input)", |ui| {
        ui.columns(2, |columns| {
            let scope_ui = &mut columns[0];
            scope_ui.label(format!("Min/max envelope · {:.2} s · ±1 FS", plots.scope_seconds));
            let (rect, _) = scope_ui.allocate_exact_size(egui::vec2(scope_ui.available_width(), 110.0), egui::Sense::hover());
            scope_ui.painter().rect_filled(rect, 0.0, Color32::from_gray(16));
            for (i, span) in plots.scope.iter().enumerate() {
                let x = rect.left() + (i as f32 + 0.5) / plots.scope.len() as f32 * rect.width();
                let y = |value: f32| rect.center().y - value.clamp(-1.0, 1.0) * rect.height() * 0.5;
                scope_ui.painter().line_segment([egui::pos2(x, y(span.min)), egui::pos2(x, y(span.max))], egui::Stroke::new(1.0_f32, accent));
            }
            let low = plots.scope.iter().map(|span| span.min).fold(f32::INFINITY, f32::min);
            let high = plots.scope.iter().map(|span| span.max).fold(f32::NEG_INFINITY, f32::max);
            scope_ui.label(format!("Capture min {low:.5}, max {high:.5} FS"));
            let spectrum_ui = &mut columns[1];
            let top_hz = (plots.sample_rate / 2.0).min(20_000.0);
            spectrum_ui.label(format!("Hann FFT · {} samples · 20–{top_hz:.0} Hz", plots.spectrum.frames));
            let (rect, _) = spectrum_ui.allocate_exact_size(egui::vec2(spectrum_ui.available_width(), 110.0), egui::Sense::hover());
            spectrum_ui.painter().rect_filled(rect, 0.0, Color32::from_gray(16));
            let points: Vec<_> = plots.spectrum.magnitudes_dbfs.iter().enumerate().filter_map(|(i, db)| {
                let hz = i as f64 * plots.spectrum.bin_hz;
                if hz < 20.0 || hz > top_hz { return None; }
                let x = ((hz / 20.0).ln() / (top_hz / 20.0).ln()) as f32;
                let y = (-db.clamp(-150.0, 0.0) / 150.0) as f32;
                Some(egui::pos2(rect.left() + x * rect.width(), rect.top() + y * rect.height()))
            }).collect();
            if points.len() > 1 {
                spectrum_ui.painter().add(egui::Shape::line(points, egui::Stroke::new(1.0_f32, accent)));
            }
            spectrum_ui.label("Peak amplitude 0 to −150 dBFS (display floor), logarithmic frequency.");
            let strongest = plots.spectrum.magnitudes_dbfs.iter().enumerate().skip(1)
                .filter(|(_, db)| **db > audio_interface_diag::SPECTRUM_FLOOR_DBFS)
                .max_by(|a, b| a.1.total_cmp(b.1));
            if let Some((bin, db)) = strongest {
                spectrum_ui.label(format!("Strongest non-DC bin {:.1} Hz: {db:.2} dBFS", bin as f64 * plots.spectrum.bin_hz));
            } else { spectrum_ui.label("No non-DC bin above the display floor."); }

        });
        ui.label("Unweighted digital magnitudes; not a calibrated noise floor or distortion test. Updates once per captured second; hidden when stale.");
    });
}
