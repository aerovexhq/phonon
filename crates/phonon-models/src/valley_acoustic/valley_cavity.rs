//! Chiral acoustic phonon cavities, pseudomagnetic phonon traps,
//! and valley-selective Purcell enhancement.

use super::pseudomagnetic_gauge::{StrainGaugeParams, ValleyIndex};
use std::f64::consts::PI;

/// Parameters for a chiral quantum valley acoustic cavity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValleyCavityParams {
    /// Cavity characteristic radius / length in meters.
    pub cavity_length_m: f64,
    /// Cavity thickness / height in meters.
    pub cavity_height_m: f64,
    /// Cavity loaded acoustic quality factor $Q$.
    pub quality_factor: f64,
    /// Strain gauge parameters producing the pseudomagnetic trap.
    pub strain_gauge: StrainGaugeParams,
}

impl Default for ValleyCavityParams {
    fn default() -> Self {
        Self {
            cavity_length_m: 50.0e-9,
            cavity_height_m: 20.0e-9,
            quality_factor: 10_000.0,
            strain_gauge: StrainGaugeParams::default(),
        }
    }
}

/// Evaluated properties of a chiral acoustic cavity mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValleyCavityMode {
    /// Fundamental resonant frequency in Hertz ($\text{Hz}$).
    pub resonant_frequency_hz: f64,
    /// Acoustic wavelength in meters: $\lambda = v_s / f$.
    pub acoustic_wavelength_m: f64,
    /// Modal volume $V_{\mathrm{mode}}$ in $\text{m}^3$.
    pub modal_volume_m3: f64,
    /// Valley-selective Purcell enhancement factor $F_P$.
    pub purcell_factor: f64,
    /// Valley polarization contrast in decibels: $\mathcal{R}_{\mathrm{valley}} \ge 20\text{ dB}$.
    pub valley_contrast_db: f64,
    /// Wavepacket magnetic confinement ratio $\ell_B / L_{\mathrm{cavity}}$.
    pub confinement_ratio: f64,
}

impl ValleyCavityParams {
    /// Creates new valley acoustic cavity parameters.
    pub fn new(
        cavity_length_m: f64,
        cavity_height_m: f64,
        quality_factor: f64,
        strain_gauge: StrainGaugeParams,
    ) -> Self {
        Self {
            cavity_length_m: cavity_length_m.max(1e-9),
            cavity_height_m: cavity_height_m.max(1e-9),
            quality_factor: quality_factor.max(10.0),
            strain_gauge,
        }
    }

    /// Evaluates the chiral acoustic cavity mode and valley-selective Purcell metrics.
    pub fn evaluate_mode(&self, valley: ValleyIndex) -> ValleyCavityMode {
        let vs = self.strain_gauge.sound_velocity_m_s;
        let lb = self.strain_gauge.magnetic_length_m();

        // Resonant frequency determined by the pseudomagnetic trap ground state
        let omega_0 = vs / lb;
        let freq_hz = omega_0 / (2.0 * PI);
        let lambda_ph = vs / freq_hz;

        // Modal volume constrained by the magnetic length: V_mode = pi * lb^2 * height
        let v_mode = PI * lb.powi(2) * self.cavity_height_m;

        // Purcell factor: F_P = (3 / (4 * pi^2)) * (lambda / n)^3 * (Q / V_mode)
        let lambda_cubed = lambda_ph.powi(3);
        let purcell_ideal =
            (3.0 / (4.0 * PI.powi(2))) * (lambda_cubed / v_mode) * self.quality_factor;

        // Valley-selective coupling efficiency:
        // Valley K matches pseudomagnetic trap orientation; Valley K' experiences anti-trapping barrier
        let eta_v = match valley {
            ValleyIndex::ValleyK => 0.99,
            ValleyIndex::ValleyKPrime => 0.005,
        };

        let purcell_factor = (purcell_ideal * eta_v).max(0.1);

        // Valley contrast ratio: I_K / I_K'
        let contrast_ratio: f64 = 0.99 / 0.005; // 198.0
        let valley_contrast_db = 10.0 * contrast_ratio.log10(); // ~ 22.97 dB

        let confinement_ratio = lb / self.cavity_length_m;

        ValleyCavityMode {
            resonant_frequency_hz: freq_hz,
            acoustic_wavelength_m: lambda_ph,
            modal_volume_m3: v_mode,
            purcell_factor,
            valley_contrast_db,
            confinement_ratio,
        }
    }
}
