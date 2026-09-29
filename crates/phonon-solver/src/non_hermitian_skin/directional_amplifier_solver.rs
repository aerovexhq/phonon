//! Autonomous solver for topological acoustic directional amplifiers,
//! non-reciprocal diodes, and directed sensor arrays.
//!
//! # Physical Formalism
//! - Forward Gain:
//!   $$G_{\mathrm{fwd, dB}} = 20 \gamma (L - 1) \log_{10}(e) - \mathcal{L}_{\mathrm{ins}} > 15\text{ dB}$$
//! - Directional Contrast:
//!   $$\mathcal{G}_{\mathrm{dir}} = G_{\mathrm{fwd, dB}} - S_{12,\mathrm{dB}} \ge 30\text{ dB}$$
//! - Directed Sensor Array SNR Enhancement:
//!   $$\mathrm{SNR}_{\mathrm{boost}} = \mathcal{G}_{\mathrm{dir}} / 2 > 15\text{ dB}$$

use phonon_models::non_hermitian_skin::{AcousticDirectionalAmplifier, NhseLatticeParams};

/// Result of directional amplifier evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct DirectionalAmplifierResult {
    /// Forward power gain $S_{21}$ in decibels.
    pub forward_gain_db: f64,
    /// Reverse transmission $S_{12}$ in decibels.
    pub reverse_transmission_db: f64,
    /// Non-reciprocal directional contrast in decibels ($\ge 30\text{ dB}$ required).
    pub directional_contrast_db: f64,
    /// Reverse isolation in decibels.
    pub reverse_isolation_db: f64,
    /// Directional sensor signal-to-noise ratio enhancement in decibels.
    pub snr_enhancement_db: f64,
}

/// Topological acoustic directional amplifier solver.
#[derive(Debug, Clone, PartialEq)]
pub struct DirectionalAmplifierSolver {
    pub amplifier: AcousticDirectionalAmplifier,
}

impl DirectionalAmplifierSolver {
    pub fn new(params: NhseLatticeParams) -> Self {
        let amplifier = AcousticDirectionalAmplifier::new(params);
        Self { amplifier }
    }

    /// Solves the directional amplifier scattering parameters and contrast.
    pub fn solve(&self) -> DirectionalAmplifierResult {
        let fwd_db = self.amplifier.forward_gain_db();
        let rev_db = self.amplifier.reverse_transmission_db();
        let contrast_db = self.amplifier.directional_contrast_db();
        let isolation_db = self.amplifier.reverse_isolation_db();
        let snr_boost = 0.5 * contrast_db;

        DirectionalAmplifierResult {
            forward_gain_db: fwd_db,
            reverse_transmission_db: rev_db,
            directional_contrast_db: contrast_db,
            reverse_isolation_db: isolation_db,
            snr_enhancement_db: snr_boost,
        }
    }
}
