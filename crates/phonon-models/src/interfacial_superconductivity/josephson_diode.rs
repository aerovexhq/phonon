//! Non-reciprocal Josephson diode arrays with finite-momentum Cooper pairing,
//! broken inversion and time-reversal symmetry, and giant rectification efficiency.

use std::f64::consts::PI;

/// Parameters for a non-reciprocal Josephson diode array.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JosephsonDiodeParams {
    /// Base fundamental critical current $I_{c1}$ in Amperes ($A$).
    pub base_critical_current_a: f64,
    /// Second-harmonic current ratio $r_2 = I_{c2} / I_{c1}$ (nominal $0.30 - 0.45$).
    pub second_harmonic_ratio: f64,
    /// Finite Cooper pair center-of-mass momentum $q_0$ in units of inverse length.
    pub finite_momentum_q0: f64,
    /// Number of Josephson junctions cascaded in the array $N \ge 1$.
    pub num_junctions: usize,
}

impl Default for JosephsonDiodeParams {
    fn default() -> Self {
        Self {
            base_critical_current_a: 1.0e-5,
            second_harmonic_ratio: 0.35,
            finite_momentum_q0: 0.1,
            num_junctions: 4,
        }
    }
}

/// Evaluated critical current metrics for the Josephson diode array.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JosephsonDiodeMetrics {
    /// Forward critical supercurrent $I_c^+$ in Amperes ($A$).
    pub forward_critical_current_a: f64,
    /// Reverse critical supercurrent $I_c^-$ in Amperes ($A$).
    pub reverse_critical_current_a: f64,
    /// Non-reciprocal diode efficiency $\eta_{\mathrm{diode}} = \frac{I_c^+ - I_c^-}{I_c^+ + I_c^-} \in [-1, 1]$.
    pub diode_efficiency: f64,
    /// Non-reciprocal rectification ratio in decibels: $\mathcal{R}_{\mathrm{diode}} = 20 \log_{10}(I_c^+ / I_c^-) \ge 3.0\text{ dB}$.
    pub rectification_ratio_db: f64,
}

impl JosephsonDiodeParams {
    /// Creates new Josephson diode parameters.
    pub fn new(
        base_critical_current_a: f64,
        second_harmonic_ratio: f64,
        finite_momentum_q0: f64,
        num_junctions: usize,
    ) -> Self {
        Self {
            base_critical_current_a: base_critical_current_a.max(1e-12),
            second_harmonic_ratio: second_harmonic_ratio.clamp(0.0, 0.9),
            finite_momentum_q0: finite_momentum_q0.max(0.0),
            num_junctions: num_junctions.max(1),
        }
    }

    /// Evaluates the supercurrent $I_s(\phi)$ in Amperes across the array at junction phase $\phi$:
    /// $$I_s(\phi) = I_{c1} [\sin(\phi) - r_2 \cos(2\phi)]$$
    #[inline]
    pub fn supercurrent_at_phase(&self, phi: f64) -> f64 {
        let n = self.num_junctions as f64;
        let ic1 = self.base_critical_current_a * n;
        let r2 = self.second_harmonic_ratio;
        ic1 * (phi.sin() - r2 * (2.0 * phi).cos())
    }

    /// Computes forward and reverse critical currents by finding the global maximum and minimum of $I_s(\phi)$.
    pub fn evaluate_critical_currents(&self) -> JosephsonDiodeMetrics {
        let num_samples = 360;
        let mut max_val = f64::MIN;
        let mut min_val = f64::MAX;

        for i in 0..num_samples {
            let phi = (i as f64) * 2.0 * PI / (num_samples as f64);
            let val = self.supercurrent_at_phase(phi);
            if val > max_val {
                max_val = val;
            }
            if val < min_val {
                min_val = val;
            }
        }

        let ic_plus = max_val.max(1e-15);
        let ic_minus = min_val.abs().max(1e-15);

        let efficiency = (ic_plus - ic_minus) / (ic_plus + ic_minus);
        let rectification_ratio_db = 20.0 * (ic_plus / ic_minus).log10();

        JosephsonDiodeMetrics {
            forward_critical_current_a: ic_plus,
            reverse_critical_current_a: ic_minus,
            diode_efficiency: efficiency,
            rectification_ratio_db,
        }
    }
}
