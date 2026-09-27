//! Virtual multi-trace oscilloscope and FFT spectrum analyzer panel.

pub mod trace;
pub use trace::WaveformTrace;

use egui::Ui;
use egui_plot::{Legend, Line, Plot, PlotPoints};

/// Interactive oscilloscope visualizer component.
#[derive(Debug, Clone, Default)]
pub struct OscilloscopePanel {
    pub traces: Vec<WaveformTrace>,
    pub fft_mode: bool,
    pub is_running: bool,
}

impl OscilloscopePanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_trace(&mut self, trace: WaveformTrace) {
        self.traces.push(trace);
    }

    pub fn clear(&mut self) {
        for t in &mut self.traces {
            t.clear();
        }
    }

    /// Renders the oscilloscope UI and waveform plot.
    pub fn show(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("Virtual Oscilloscope");
            ui.separator();

            ui.checkbox(&mut self.fft_mode, "FFT Spectrum Mode");

            if ui.button("Clear Traces").clicked() {
                self.clear();
            }

            ui.separator();

            // Display readouts for the first trace if available
            if let Some(first) = self.traces.first() {
                ui.label(format!("Trace: {}", first.name));
                ui.label(format!("Vpp: {:.3} V", first.v_pp()));
                ui.label(format!("Vrms: {:.3} V", first.v_rms()));
                if let Some(freq) = first.estimate_frequency() {
                    ui.label(format!("Freq: {:.1} Hz", freq));
                }
            }
        });

        ui.separator();

        let x_label = if self.fft_mode {
            "Frequency (Hz)"
        } else {
            "Time (s)"
        };
        let y_label = if self.fft_mode {
            "Magnitude (dB)"
        } else {
            "Amplitude (V)"
        };

        let plot = Plot::new("oscilloscope_plot")
            .legend(Legend::default())
            .x_axis_label(x_label)
            .y_axis_label(y_label)
            .allow_zoom(true)
            .allow_drag(true);

        plot.show(ui, |plot_ui| {
            for trace in &self.traces {
                if trace.is_empty() {
                    continue;
                }

                let points: PlotPoints = if self.fft_mode {
                    let spectrum = trace.compute_spectrum(128);
                    PlotPoints::from(spectrum)
                } else {
                    let decimated = trace.min_max_decimate(2000);
                    PlotPoints::from(decimated)
                };

                let line = Line::new(&trace.name, points).color(trace.color).width(1.8);

                plot_ui.line(line);
            }
        });
    }
}
