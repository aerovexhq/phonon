#![deny(unsafe_code)]

//! Virtual multi-trace oscilloscope, FFT spectrum analyzer, and SI prefix engineering notation engine.

pub mod multi_graph;
pub mod trace;

pub use multi_graph::{FloatingScopeWindow, MultiGraphManager, ScopeRunMode};
pub use trace::{SignalDomain, WaveformTrace};

use egui::Ui;
use egui_plot::{Legend, Line, Plot, PlotBounds, PlotPoints};

/// Bounded zoom limits preventing runaway exponential zoom on WASM and desktop.
pub const MIN_TIME_SPAN: f64 = 1.0e-14; // 10 fs
pub const MAX_TIME_SPAN: f64 = 1.0e4; // 10,000 s
pub const MIN_AMPLITUDE_SPAN: f64 = 1.0e-8; // 10 nV
pub const MAX_AMPLITUDE_SPAN: f64 = 1.0e6; // 1 MV

/// Formats a time value with engineering SI prefixes (fs, ps, ns, us, ms, s).
pub fn format_time_si(t: f64) -> String {
    let abs_t = t.abs();
    if abs_t < 1.0e-18 {
        "0 s".to_string()
    } else if abs_t < 1.0e-12 {
        format!("{:.2} fs", t * 1.0e15)
    } else if abs_t < 1.0e-9 {
        format!("{:.2} ps", t * 1.0e12)
    } else if abs_t < 1.0e-6 {
        format!("{:.2} ns", t * 1.0e9)
    } else if abs_t < 1.0e-3 {
        format!("{:.2} us", t * 1.0e6)
    } else if abs_t < 1.0 {
        format!("{:.2} ms", t * 1.0e3)
    } else {
        format!("{:.3} s", t)
    }
}

/// Formats a voltage value with engineering SI prefixes (nV, uV, mV, V, kV).
pub fn format_voltage_si(v: f64) -> String {
    let abs_v = v.abs();
    if abs_v < 1.0e-12 {
        "0 V".to_string()
    } else if abs_v < 1.0e-6 {
        format!("{:.2} nV", v * 1.0e9)
    } else if abs_v < 1.0e-3 {
        format!("{:.2} uV", v * 1.0e6)
    } else if abs_v < 1.0 {
        format!("{:.2} mV", v * 1.0e3)
    } else if abs_v < 1000.0 {
        format!("{:.3} V", v)
    } else {
        format!("{:.2} kV", v * 1.0e-3)
    }
}

/// Formats an electrical current value with engineering SI prefixes (pA, nA, uA, mA, A).
pub fn format_current_si(i: f64) -> String {
    let abs_i = i.abs();
    if abs_i < 1.0e-15 {
        "0 A".to_string()
    } else if abs_i < 1.0e-9 {
        format!("{:.2} pA", i * 1.0e12)
    } else if abs_i < 1.0e-6 {
        format!("{:.2} nA", i * 1.0e9)
    } else if abs_i < 1.0e-3 {
        format!("{:.2} uA", i * 1.0e6)
    } else if abs_i < 1.0 {
        format!("{:.2} mA", i * 1.0e3)
    } else {
        format!("{:.3} A", i)
    }
}

/// Interactive dual-domain oscilloscope visualizer component.
#[derive(Debug, Clone, Default)]
pub struct OscilloscopePanel {
    pub traces: Vec<WaveformTrace>,
    pub fft_mode: bool,
    pub is_running: bool,
    pub auto_fit_requested: bool,
}

impl OscilloscopePanel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_trace(&mut self, trace: WaveformTrace) {
        self.traces.push(trace);
    }

    pub fn clear(&mut self) {
        self.traces.clear();
    }

    /// Clears samples from all active traces without removing trace definitions.
    pub fn clear_samples(&mut self) {
        for t in &mut self.traces {
            t.clear();
        }
    }

    /// Triggers an automated viewport bounds recalculation on next draw.
    pub fn request_auto_fit(&mut self) {
        self.auto_fit_requested = true;
    }

    /// Computes the collective bounding box enclosing all active trace samples: `([t_min, t_max], [v_min, v_max])`.
    pub fn compute_data_bounds(&self) -> Option<([f64; 2], [f64; 2])> {
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        let mut has_points = false;

        for trace in &self.traces {
            for &[x, y] in &trace.samples {
                has_points = true;
                if x < min_x {
                    min_x = x;
                }
                if x > max_x {
                    max_x = x;
                }
                if y < min_y {
                    min_y = y;
                }
                if y > max_y {
                    max_y = y;
                }
            }
        }

        if !has_points {
            return None;
        }

        // Add 10% vertical padding and ensure non-zero span
        let y_span = (max_y - min_y).max(1.0e-6);
        let y_pad = y_span * 0.1;
        let x_span = (max_x - min_x).max(1.0e-9);

        let final_min_x = min_x.clamp(-MAX_TIME_SPAN, MAX_TIME_SPAN);
        let final_max_x = (min_x + x_span).clamp(-MAX_TIME_SPAN, MAX_TIME_SPAN);
        let final_min_y = (min_y - y_pad).clamp(-MAX_AMPLITUDE_SPAN, MAX_AMPLITUDE_SPAN);
        let final_max_y = (max_y + y_pad).clamp(-MAX_AMPLITUDE_SPAN, MAX_AMPLITUDE_SPAN);

        Some(([final_min_x, final_max_x], [final_min_y, final_max_y]))
    }

    /// Renders the oscilloscope UI and waveform plot.
    pub fn show(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading("Dual-Domain Virtual Oscilloscope");
            ui.separator();

            ui.checkbox(&mut self.fft_mode, "FFT Spectrum Mode");

            if ui.button("Clear Traces").clicked() {
                self.clear();
            }

            if ui.button("Auto-Fit / Reset View").clicked() {
                self.request_auto_fit();
            }

            ui.separator();

            // Display readouts for active traces with domain-specific metrics
            if let Some(first) = self.traces.first() {
                ui.label(format!("Trace: {}", first.name));
                match first.domain {
                    SignalDomain::Electrical => {
                        ui.label(format!("Vpp: {}", format_voltage_si(first.v_pp())));
                        ui.label(format!("Vrms: {}", format_voltage_si(first.v_rms())));
                    }
                    SignalDomain::AcousticPressure => {
                        ui.label(format!("Ppeak: {:.2} Pa", first.v_pp() * 0.5));
                        ui.label(format!("Prms: {:.2} Pa", first.v_rms()));
                        ui.label(format!("SPL: {:.1} dB SPL", first.peak_spl_db()));
                    }
                    SignalDomain::AcousticFlow => {
                        ui.label(format!("Flow: {:.1} cm³/s", first.flow_rate_cm3_s()));
                    }
                    SignalDomain::Mechanical => {
                        ui.label(format!("Disp: {:.3} mm", first.v_pp() * 1000.0));
                    }
                }
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
            let has_electrical = self.traces.iter().any(|t| t.domain == SignalDomain::Electrical);
            let has_acoustic = self.traces.iter().any(|t| {
                t.domain == SignalDomain::AcousticPressure || t.domain == SignalDomain::AcousticFlow
            });
            if has_electrical && has_acoustic {
                "Amplitude [V | Pa | cm³/s]"
            } else if has_acoustic {
                "Acoustic Pressure (Pa) / Volume Flow (cm³/s)"
            } else {
                "Amplitude (V)"
            }
        };

        let fft_mode = self.fft_mode;
        let mut plot = Plot::new("oscilloscope_plot")
            .legend(Legend::default())
            .x_axis_label(x_label)
            .y_axis_label(y_label)
            .allow_zoom(true)
            .allow_drag(true);

        if !fft_mode {
            plot = plot.x_axis_formatter(|mark, _range| format_time_si(mark.value));
        }

        let auto_fit = self.auto_fit_requested;
        self.auto_fit_requested = false;
        let bounds_opt = if auto_fit {
            self.compute_data_bounds()
        } else {
            None
        };

        plot.show(ui, |plot_ui| {
            if let Some(([min_x, max_x], [min_y, max_y])) = bounds_opt {
                plot_ui.set_plot_bounds(PlotBounds::from_min_max([min_x, min_y], [max_x, max_y]));
            }

            for trace in &self.traces {
                if trace.is_empty() {
                    continue;
                }

                let points: PlotPoints = if fft_mode {
                    let spectrum = trace.compute_spectrum(128);
                    PlotPoints::from(spectrum)
                } else {
                    let decimated = trace.min_max_decimate(2000);
                    PlotPoints::from(decimated)
                };

                let label = format!("{} [{}]", trace.name, trace.domain.unit_symbol());
                let line = Line::new(label, points).color(trace.color).width(1.8);

                plot_ui.line(line);
            }
        });
    }
}
