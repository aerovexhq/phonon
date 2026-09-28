//! Non-Reciprocal Acoustic Diode & 3-Port Circulator Solvers.
//!
//! Evaluates transmission spectra, isolation ratios, and S-parameters for
//! spatio-temporally modulated acoustic waveguides and circulating-fluid circulators.

use phonon_models::phononic_topological::{AcousticCirculatorParams, SpatioTemporalModulator};
use std::f64::consts::PI;

/// Single spectral evaluation point for an acoustic diode.
#[derive(Debug, Clone, PartialEq)]
pub struct DiodeSpectrumPoint {
    /// Angular frequency $\omega$ in rad/s.
    pub frequency_rad_per_s: f64,
    /// Frequency $f = \omega / (2\pi)$ in Hertz.
    pub frequency_hz: f64,
    /// Forward transmission power $T_f$.
    pub forward_transmission: f64,
    /// Reverse transmission power $T_r$.
    pub reverse_transmission: f64,
    /// Isolation ratio $\mathcal{I} = 10 \log_{10}(T_f / T_r)$ in dB.
    pub isolation_db: f64,
    /// Forward insertion loss $\mathcal{L} = -10 \log_{10}(T_f)$ in dB.
    pub insertion_loss_db: f64,
}

/// Solver for spatio-temporally modulated acoustic diodes.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NonReciprocalDiodeSolver {
    /// Modulator physical parameters.
    pub modulator: SpatioTemporalModulator,
}

impl NonReciprocalDiodeSolver {
    /// Creates a new diode solver with the specified modulator.
    pub fn new(modulator: SpatioTemporalModulator) -> Self {
        Self { modulator }
    }

    /// Evaluates the frequency spectrum across the range $[f_{start}, f_{end}]$ in Hertz.
    pub fn solve_spectral_response(
        &self,
        start_hz: f64,
        end_hz: f64,
        num_points: usize,
    ) -> Vec<DiodeSpectrumPoint> {
        let n = num_points.max(2);
        let df = (end_hz - start_hz) / ((n - 1) as f64);
        let mut spectrum = Vec::with_capacity(n);

        for i in 0..n {
            let f_hz = start_hz + (i as f64) * df;
            let omega = 2.0 * PI * f_hz;
            let tf = self.modulator.forward_transmission(omega);
            let tr = self.modulator.reverse_transmission(omega);
            let iso = self.modulator.isolation_db(omega);
            let il = self.modulator.insertion_loss_db(omega);

            spectrum.push(DiodeSpectrumPoint {
                frequency_rad_per_s: omega,
                frequency_hz: f_hz,
                forward_transmission: tf,
                reverse_transmission: tr,
                isolation_db: iso,
                insertion_loss_db: il,
            });
        }

        spectrum
    }

    /// Finds the peak isolation point in the specified frequency band.
    pub fn peak_isolation(&self, start_hz: f64, end_hz: f64) -> DiodeSpectrumPoint {
        let spectrum = self.solve_spectral_response(start_hz, end_hz, 50);
        spectrum
            .into_iter()
            .max_by(|a, b| {
                a.isolation_db
                    .partial_cmp(&b.isolation_db)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(DiodeSpectrumPoint {
                frequency_rad_per_s: 0.0,
                frequency_hz: 0.0,
                forward_transmission: 0.0,
                reverse_transmission: 0.0,
                isolation_db: 0.0,
                insertion_loss_db: 0.0,
            })
    }
}

/// Solver for circulating-fluid biased 3-port acoustic circulators.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AcousticCirculatorSolver {
    /// Circulator physical parameters.
    pub circulator: AcousticCirculatorParams,
}

impl AcousticCirculatorSolver {
    /// Creates a new acoustic circulator solver.
    pub fn new(circulator: AcousticCirculatorParams) -> Self {
        Self { circulator }
    }

    /// Evaluates the 3-port power scattering matrix at angular frequency $\omega$.
    pub fn solve_circulator_s_parameters(&self, omega: f64) -> [[f64; 3]; 3] {
        self.circulator.scattering_matrix_3port(omega)
    }

    /// Evaluates circulator frequency response around resonance: returns (omega, forward power |S21|^2, isolation dB).
    pub fn solve_circulator_bandwidth(&self, num_points: usize) -> Vec<(f64, f64, f64)> {
        let n = num_points.max(2);
        let w0 = self.circulator.unperturbed_resonance_rad_per_s();
        let gamma = self.circulator.cavity_decay_rate_rad_per_s();
        let span = 3.0 * gamma;

        let dw = 2.0 * span / ((n - 1) as f64);
        let mut results = Vec::with_capacity(n);

        for i in 0..n {
            let omega = (w0 - span) + (i as f64) * dw;
            let s21 = self.circulator.forward_transmission_power(omega);
            let iso = self.circulator.circulator_isolation_db(omega);
            results.push((omega, s21, iso));
        }

        results
    }

    /// Verifies that non-reciprocal circulation criteria are satisfied at center frequency:
    /// $|S_{21}|^2 \ge 0.80$ and isolation $\ge 20\text{ dB}$.
    pub fn verify_circulation(&self) -> bool {
        let w0 = self.circulator.unperturbed_resonance_rad_per_s();
        let s21 = self.circulator.forward_transmission_power(w0);
        let iso = self.circulator.circulator_isolation_db(w0);
        s21 >= 0.80 && iso >= 20.0
    }
}
