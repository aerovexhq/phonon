//! Non-reciprocal acoustic directional amplifier and acoustic diode model.
//!
//! # Physical Formalism
//! - Forward Transmission Gain:
//!   $$G_{\mathrm{fwd}} = e^{+\gamma (L - 1)}$$
//! - Reverse Transmission Gain:
//!   $$G_{\mathrm{rev}} = e^{-\gamma (L - 1)}$$
//! - Directional Contrast:
//!   $$\mathcal{G}_{\mathrm{dir}} = 20 \log_{10}\left( \frac{G_{\mathrm{fwd}}}{G_{\mathrm{rev}}} \right) = 40 \gamma (L - 1) \log_{10}(e) \approx 17.37178 \gamma (L - 1)\text{ dB}$$
//! - Reverse Isolation:
//!   $$\mathcal{I}_{\mathrm{rev}} = -20 \log_{10}(G_{\mathrm{rev}}) = 20 \gamma (L - 1) \log_{10}(e) > 30\text{ dB}$$

use super::nhse_lattice::NhseLatticeParams;

/// Topological acoustic directional amplifier and non-reciprocal diode.
#[derive(Debug, Clone, PartialEq)]
pub struct AcousticDirectionalAmplifier {
    pub params: NhseLatticeParams,
    /// Insertion loss factor (typically 0.85 to 0.95).
    pub insertion_efficiency: f64,
}

impl AcousticDirectionalAmplifier {
    pub fn new(params: NhseLatticeParams) -> Self {
        Self {
            params,
            insertion_efficiency: 0.90,
        }
    }

    /// Evaluates forward power gain in decibels:
    /// $$G_{\mathrm{fwd, dB}} = 20 \log_{10}(G_{\mathrm{fwd}} \cdot \eta)$$
    pub fn forward_gain_db(&self) -> f64 {
        let l = (self.params.num_sites - 1) as f64;
        let ideal_db = 20.0 * self.params.non_reciprocal_gamma * l * std::f64::consts::LOG10_E;
        ideal_db + 20.0 * self.insertion_efficiency.log10()
    }

    /// Evaluates reverse transmission in decibels:
    pub fn reverse_transmission_db(&self) -> f64 {
        let l = (self.params.num_sites - 1) as f64;
        let ideal_db = -20.0 * self.params.non_reciprocal_gamma * l * std::f64::consts::LOG10_E;
        ideal_db + 20.0 * self.insertion_efficiency.log10()
    }

    /// Evaluates non-reciprocal directional amplification contrast $\mathcal{G}_{\mathrm{dir}}$ in decibels:
    pub fn directional_contrast_db(&self) -> f64 {
        self.forward_gain_db() - self.reverse_transmission_db()
    }

    /// Evaluates reverse isolation in decibels:
    pub fn reverse_isolation_db(&self) -> f64 {
        -self.reverse_transmission_db()
    }
}
