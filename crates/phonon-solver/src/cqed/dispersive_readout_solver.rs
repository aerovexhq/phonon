//! Dispersive cavity readout solver: cavity pointer state separation in the IQ plane,
//! signal-to-noise ratio (SNR), and single-shot state discrimination fidelity.

use phonon_models::cqed::DispersiveCqedSystem;
use std::f64::consts::PI;

/// Result of a dispersive cavity readout evaluation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DispersiveReadoutResult {
    /// In-phase component of cavity field for ground state $|0\rangle$.
    pub i0: f64,
    /// Quadrature component of cavity field for ground state $|0\rangle$.
    pub q0: f64,
    /// In-phase component of cavity field for excited state $|1\rangle$.
    pub i1: f64,
    /// Quadrature component of cavity field for excited state $|1\rangle$.
    pub q1: f64,
    /// Phase-space separation between pointer states $D_{01} = |\alpha_0 - \alpha_1|$.
    pub pointer_separation: f64,
    /// Signal-to-Noise Ratio (SNR) for the measurement.
    pub snr: f64,
    /// Theoretical single-shot state discrimination fidelity $F \in [0.5, 1.0]$.
    pub discrimination_fidelity: f64,
    /// Mean intra-cavity readout photon number $\bar{n}_{readout}$.
    pub mean_photon_number: f64,
}

/// Dispersive cavity readout simulation engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DispersiveReadoutSolver {
    /// Measurement integration time $\tau_{meas}$ in nanoseconds.
    pub measurement_time_ns: f64,
    /// Drive power parameter $\epsilon_d / (2\pi)$ in MHz.
    pub drive_amplitude_mhz: f64,
    /// Amplifier added noise photon number $n_{add}$ (e.g. 0.5 for TWPA, 15 for HEMT).
    pub amplifier_noise_photons: f64,
}

impl Default for DispersiveReadoutSolver {
    fn default() -> Self {
        Self {
            measurement_time_ns: 300.0,
            drive_amplitude_mhz: 3.5,
            amplifier_noise_photons: 2.0,
        }
    }
}

impl DispersiveReadoutSolver {
    /// Constructs a new dispersive readout solver.
    pub fn new(
        measurement_time_ns: f64,
        drive_amplitude_mhz: f64,
        amplifier_noise_photons: f64,
    ) -> Self {
        Self {
            measurement_time_ns,
            drive_amplitude_mhz,
            amplifier_noise_photons,
        }
    }

    /// Evaluates steady-state cavity pointer states and measurement fidelity:
    /// $$\alpha_{0,1} = \frac{-i \sqrt{\kappa_{ext}} \epsilon_d}{\kappa_r/2 + i(\omega_r \pm \chi - \omega_d)}$$
    pub fn evaluate_readout(
        &self,
        cqed: &DispersiveCqedSystem,
        drive_freq_ghz: f64,
    ) -> DispersiveReadoutResult {
        let kappa_tot = cqed.cavity.kappa_total_mhz();
        let kappa_ext = cqed.cavity.kappa_ext_mhz();
        let chi = cqed.dispersive_shift_chi_mhz();
        let f_r = cqed.cavity.resonance_frequency_ghz;

        // Drive detuning relative to bare cavity (in MHz)
        let delta_d_mhz = (f_r - drive_freq_ghz) * 1.0e3;

        // For ground state |0>: cavity resonance at f_r + chi
        // detuning = delta_d + chi
        let det_0 = delta_d_mhz + chi;
        // For excited state |1>: cavity resonance at f_r - chi
        // detuning = delta_d - chi
        let det_1 = delta_d_mhz - chi;

        // alpha = -i * sqrt(kappa_ext) * eps_d / (kappa/2 + i * det)
        // Multiply by (kappa/2 - i*det) / ((kappa/2)^2 + det^2)
        // Numerator: -i * A * (kappa/2 - i*det) = -A * det - i * A * (kappa/2)
        let num_amp = (kappa_ext.max(1e-3)).sqrt() * self.drive_amplitude_mhz;

        let denom_0 = (0.5 * kappa_tot).powi(2) + det_0.powi(2);
        let alpha0_re = (-num_amp * det_0) / denom_0.max(1e-6);
        let alpha0_im = (-num_amp * 0.5 * kappa_tot) / denom_0.max(1e-6);

        let denom_1 = (0.5 * kappa_tot).powi(2) + det_1.powi(2);
        let alpha1_re = (-num_amp * det_1) / denom_1.max(1e-6);
        let alpha1_im = (-num_amp * 0.5 * kappa_tot) / denom_1.max(1e-6);

        let d_re = alpha0_re - alpha1_re;
        let d_im = alpha0_im - alpha1_im;
        let pointer_sep = (d_re.powi(2) + d_im.powi(2)).sqrt();

        // Mean photon number n_bar
        let n0 = alpha0_re.powi(2) + alpha0_im.powi(2);
        let n1 = alpha1_re.powi(2) + alpha1_im.powi(2);
        let mean_n = 0.5 * (n0 + n1);

        // Measurement SNR: SNR = (kappa_ext * tau_meas * |alpha_0 - alpha_1|^2) / (1 + 2 * n_add)
        let tau_s = self.measurement_time_ns * 1.0e-9;
        let kappa_ext_rad_s = kappa_ext * 1.0e6 * 2.0 * PI;
        let noise_denom = 1.0 + 2.0 * self.amplifier_noise_photons;
        let snr = (kappa_ext_rad_s * tau_s * pointer_sep.powi(2)) / noise_denom.max(1.0);

        // Discrimination fidelity via complementary error function
        // F = 1 - 0.5 * erfc(sqrt(SNR) / (2 * sqrt(2)))
        let arg = (snr.max(0.0)).sqrt() / (2.0 * std::f64::consts::SQRT_2);
        let fidelity = 1.0 - 0.5 * erfc_approx(arg);

        DispersiveReadoutResult {
            i0: alpha0_re,
            q0: alpha0_im,
            i1: alpha1_re,
            q1: alpha1_im,
            pointer_separation: pointer_sep,
            snr,
            discrimination_fidelity: fidelity.clamp(0.5, 1.0),
            mean_photon_number: mean_n,
        }
    }
}

/// Fast Chebyshev approximation to the complementary error function erfc(x).
fn erfc_approx(x: f64) -> f64 {
    if x < 0.0 {
        return 2.0 - erfc_approx(-x);
    }
    // Rational Chebyshev approximation (Abramowitz & Stegun 7.1.26)
    let p = 0.3275911;
    let t = 1.0 / (1.0 + p * x);
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;

    let poly = t * (a1 + t * (a2 + t * (a3 + t * (a4 + t * a5))));
    poly * (-x.powi(2)).exp()
}
