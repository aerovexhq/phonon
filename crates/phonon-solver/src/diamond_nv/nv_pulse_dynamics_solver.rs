//! Pulse Sequence Dynamics, Hahn Echo & Nanoscale NMR Solvers.
//!
//! Simulates Ramsey free induction decay, Hahn echo refocusing, and XY8
//! dynamical decoupling for nanoscale nuclear magnetic resonance (NMR) spectroscopy.

use phonon_models::diamond_nv::NvCenterConfig;
use std::f64::consts::PI;

/// Result of an XY8 dynamical decoupling or nanoscale NMR spectral sweep.
#[derive(Debug, Clone, PartialEq)]
pub struct NmrSpectralPoint {
    /// Lock-in sensing frequency $f_{filter} = \frac{1}{4 \tau_0}$ in Hertz.
    pub filter_frequency_hz: f64,
    /// Inter-pulse delay $\tau_0$ in seconds.
    pub pulse_delay_s: f64,
    /// Total interaction time $\tau = 2 N \tau_0$ in seconds.
    pub total_time_s: f64,
    /// Coherence signal amplitude $W \in [0.0, 1.0]$.
    pub coherence: f64,
}

/// Solver for NV quantum pulse sequences and nanoscale NMR sensing.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NvPulseDynamicsSolver {
    /// NV center configuration.
    pub config: NvCenterConfig,
}

impl NvPulseDynamicsSolver {
    /// Creates a new pulse dynamics solver.
    pub fn new(config: NvCenterConfig) -> Self {
        Self { config }
    }

    /// Computes Ramsey free induction decay signal:
    /// $S(\tau) = \frac{1}{2} \left[ 1 + \exp\left(-\left(\frac{\tau}{T_2^*}\right)^2\right) \cos(2\pi \delta f \tau) \right]$.
    pub fn ramsey_coherence(&self, tau_s: f64, detuning_hz: f64) -> f64 {
        let t2_star = self.config.dephasing_time_t2_star_s.max(1e-12);
        let decay = (-((tau_s / t2_star).powi(2))).exp();
        let osc = (2.0 * PI * detuning_hz * tau_s).cos();
        0.5 * (1.0 + decay * osc)
    }

    /// Computes Hahn echo coherence signal:
    /// $S(\tau) = \frac{1}{2} \left[ 1 + \exp\left(-\left(\frac{\tau}{T_2}\right)^3\right) \right]$.
    pub fn hahn_echo_coherence(&self, tau_s: f64) -> f64 {
        let t2 = self.config.coherence_time_t2_echo_s.max(1e-12);
        let decay = (-((tau_s / t2).powi(3))).exp();
        0.5 * (1.0 + decay)
    }

    /// Dynamical decoupling coherence time $T_2^{(DD)} = T_2 \cdot N^{2/3}$ for $N$ pulses.
    pub fn dynamical_decoupling_t2(&self, num_pulses: usize) -> f64 {
        let n = num_pulses.max(1) as f64;
        self.config.coherence_time_t2_echo_s * n.powf(2.0 / 3.0)
    }

    /// Computes the NV spin coherence under an XY8-N sequence in the presence of
    /// a target nuclear spin species precessing at Larmor frequency $f_L$ with fluctuating field $B_{rms}$.
    pub fn nanoscale_nmr_coherence(
        &self,
        tau0_s: f64,
        num_pulses: usize,
        target_larmor_hz: f64,
        b_rms_tesla: f64,
    ) -> f64 {
        let n = num_pulses.max(8);
        let tau_total = 2.0 * (n as f64) * tau0_s;
        let t2_dd = self.dynamical_decoupling_t2(n);

        // Intrinsic decoherence envelope
        let intrinsic_decay = (-((tau_total / t2_dd).powi(3))).exp();

        // Lock-in filter resonance condition: f_filter = 1 / (4 * tau0)
        let f_filter = 1.0 / (4.0 * tau0_s.max(1e-15));
        let delta_f = (f_filter - target_larmor_hz).abs();

        let sinc_arg = PI * delta_f * tau_total;
        let filter_response = if sinc_arg.abs() < 1e-6 {
            1.0
        } else {
            (sinc_arg.sin() / sinc_arg).powi(2)
        };

        // Phase accumulation variance <phi^2>
        let gamma_rad = 2.0 * PI * self.config.electron_gyromagnetic_ratio_hz_per_t;
        let phase_var =
            (2.0 / PI.powi(2)) * (gamma_rad * b_rms_tesla * tau_total).powi(2) * filter_response;

        // Observed normalized coherence
        let nmr_decay = (-0.5 * phase_var).exp();
        (intrinsic_decay * nmr_decay).clamp(0.0, 1.0)
    }

    /// Sweeps the XY8 filter frequency across a target band to resolve nanoscale NMR spectra.
    pub fn solve_nmr_spectrum(
        &self,
        start_hz: f64,
        end_hz: f64,
        num_points: usize,
        num_pulses: usize,
        target_larmor_hz: f64,
        b_rms_tesla: f64,
    ) -> Vec<NmrSpectralPoint> {
        let n = num_points.max(2);
        let df = (end_hz - start_hz) / ((n - 1) as f64);
        let mut spectrum = Vec::with_capacity(n);

        for i in 0..n {
            let f = start_hz + (i as f64) * df;
            let tau0 = 1.0 / (4.0 * f.max(1.0));
            let total_time = 2.0 * (num_pulses as f64) * tau0;
            let w = self.nanoscale_nmr_coherence(tau0, num_pulses, target_larmor_hz, b_rms_tesla);

            spectrum.push(NmrSpectralPoint {
                filter_frequency_hz: f,
                pulse_delay_s: tau0,
                total_time_s: total_time,
                coherence: w,
            });
        }

        spectrum
    }
}
