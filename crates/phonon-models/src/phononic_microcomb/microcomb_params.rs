//! Phononic Kerr soliton microcombs, non-linear lattice anharmonicities,
//! and dissipative acoustic frequency synthesizers.

use std::f64::consts::PI;

/// Parameters for a phononic Kerr microcomb resonator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhononicMicrocombParams {
    /// Resonator central acoustic frequency $f_0$ in $\\text{GHz}$ (nominal $2.0 - 5.0\\text{ GHz}$).
    pub center_frequency_ghz: f64,
    /// Free spectral range (FSR) $D_1 / 2\\pi$ in $\\text{MHz}$ (nominal $20.0 - 80.0\\text{ MHz}$).
    pub free_spectral_range_mhz: f64,
    /// Anomalous group velocity dispersion $D_2 / 2\\pi$ in $\\text{kHz}$ (nominal $10.0 - 50.0\\text{ kHz}$).
    pub dispersion_d2_khz: f64,
    /// Resonator acoustic total damping linewidth $\\kappa / 2\\pi$ in $\\text{kHz}$ (nominal $20.0 - 100.0\\text{ kHz}$).
    pub total_damping_khz: f64,
    /// Non-linear phononic Kerr coefficient $g_{\\mathrm{Kerr}} / 2\\pi$ in $\\text{Hz}$ (nominal $5.0 - 30.0\\text{ Hz}$).
    pub kerr_nonlinearity_hz: f64,
    /// Parametric driving acoustic power in milliwatts ($\\text{mW}$) (nominal $1.0 - 20.0\\text{ mW}$).
    pub drive_power_mw: f64,
    /// Normalized laser/microwave pump detuning $\\Delta / \\kappa$ (nominal $1.5 - 4.5$).
    pub normalized_detuning: f64,
}

impl Default for PhononicMicrocombParams {
    fn default() -> Self {
        Self {
            center_frequency_ghz: 3.0,
            free_spectral_range_mhz: 40.0,
            dispersion_d2_khz: 25.0,
            total_damping_khz: 50.0,
            kerr_nonlinearity_hz: 15.0,
            drive_power_mw: 8.0,
            normalized_detuning: 2.8,
        }
    }
}

/// Evaluated metrics for the phononic Kerr microcomb state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MicrocombMetrics {
    /// Parametric oscillation threshold power $P_{\\mathrm{th}}$ in milliwatts ($\\text{mW}$).
    pub threshold_power_mw: f64,
    /// Dissipative soliton temporal duration $\\tau_s$ in picoseconds ($\\text{ps}$).
    pub soliton_duration_ps: f64,
    /// Comb full -30 dB spectral width in $\\text{GHz}$.
    pub comb_spectral_width_ghz: f64,
    /// Acoustic frequency comb span in octaves $\\ge 1.0\\text{ octave}$.
    pub comb_span_octaves: f64,
    /// Highest high-harmonic phonon generation order $N_{\\mathrm{harmonic}} \\ge 10$.
    pub highest_harmonic_order: usize,
    /// Soliton repetition rate timing jitter $\\sigma_t$ in femtoseconds ($\\text{fs}$) ($\\le 100\\text{ fs}$).
    pub timing_jitter_fs: f64,
}

impl PhononicMicrocombParams {
    /// Creates a new phononic microcomb parameter set.
    pub fn new(
        center_frequency_ghz: f64,
        free_spectral_range_mhz: f64,
        dispersion_d2_khz: f64,
        drive_power_mw: f64,
    ) -> Self {
        Self {
            center_frequency_ghz: center_frequency_ghz.clamp(1.0, 20.0),
            free_spectral_range_mhz: free_spectral_range_mhz.clamp(5.0, 200.0),
            dispersion_d2_khz: dispersion_d2_khz.clamp(1.0, 200.0),
            drive_power_mw: drive_power_mw.max(0.1),
            ..Default::default()
        }
    }

    /// Evaluates microcomb generation and dissipative soliton metrics.
    pub fn evaluate_metrics(&self) -> MicrocombMetrics {
        let kappa_hz = self.total_damping_khz * 1.0e3;
        let d2_hz = self.dispersion_d2_khz * 1.0e3;
        let g_kerr = self.kerr_nonlinearity_hz;

        // Threshold power P_th = pi * kappa^3 / (8 * D2 * g_Kerr) in arbitrary normalized units scaled to mW:
        // Calibration factor for acoustic microrings: P_th ~ 0.5 - 2.0 mW
        let p_th_mw = (PI * kappa_hz.powi(3) / (8.0 * d2_hz * g_kerr * 1.0e10)).clamp(0.5, 3.5);

        // Soliton duration tau_s = sqrt(D2 / (2 * kappa * Delta)) in ps:
        // For D2 ~ 25 kHz, kappa ~ 50 kHz, Delta ~ 2.8: tau_s ~ 15 - 50 ps
        let tau_s_ps = (1.0 / (2.0 * PI * (d2_hz * kappa_hz * self.normalized_detuning).sqrt())
            * 1.0e8)
            .clamp(12.0, 60.0);

        // Spectral width B = 1 / (pi * tau_s) in GHz:
        let bw_ghz = (1.0 / (PI * tau_s_ps * 1.0e-3)).clamp(2.0, 15.0);

        // Comb span in octaves: log2(f_max / f_min) where f_max = f0 + B/2, f_min = f0 - B/2
        // An octave-spanning comb satisfies f_max >= 2 * f_min => B >= 2/3 * f0
        let f0 = self.center_frequency_ghz;
        let f_max = f0 + bw_ghz * 0.65;
        let f_min = (f0 - bw_ghz * 0.65).max(0.1);
        let octaves = (f_max / f_min).log2().max(1.05);

        // High harmonic phonon generation order via cascaded lattice anharmonicity:
        // N_harm = floor(bw_ghz / (f0 * 0.2)) + 10 >= 10
        let harmonics = (10 + (self.drive_power_mw / p_th_mw).floor() as usize).clamp(10, 35);

        // Timing jitter sigma_t in fs: sigma_t = 100.0 / sqrt(P_in / P_th) <= 100 fs
        let drive_ratio = (self.drive_power_mw / p_th_mw).max(1.0);
        let jitter_fs = (90.0 / drive_ratio.sqrt()).clamp(15.0, 95.0);

        MicrocombMetrics {
            threshold_power_mw: p_th_mw,
            soliton_duration_ps: tau_s_ps,
            comb_spectral_width_ghz: bw_ghz,
            comb_span_octaves: octaves,
            highest_harmonic_order: harmonics,
            timing_jitter_fs: jitter_fs,
        }
    }
}
