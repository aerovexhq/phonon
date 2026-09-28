//! Optically Detected Magnetic Resonance (ODMR) Spectrum Synthesis.
//!
//! Models laser-induced spin polarization, intersystem crossing (ISC),
//! microwave resonance excitation, and photoluminescence contrast dips.

use crate::diamond_nv::nv_center::NvCenterConfig;

/// Parameters for generating an ODMR photoluminescence frequency sweep.
#[derive(Debug, Clone, PartialEq)]
pub struct OdmrSpectrumConfig {
    /// Minimum sweep frequency in Hertz (default 2.70 GHz).
    pub f_min_hz: f64,
    /// Maximum sweep frequency in Hertz (default 3.05 GHz).
    pub f_max_hz: f64,
    /// FWHM linewidth $\Gamma$ in Hertz (default 5.0 MHz).
    pub linewidth_hz: f64,
    /// Peak optical contrast $C \in [0.0, 1.0]$ (default 0.20).
    pub contrast: f64,
    /// Unperturbed baseline photoluminescence intensity (default 1.0).
    pub baseline_intensity: f64,
}

impl Default for OdmrSpectrumConfig {
    fn default() -> Self {
        Self {
            f_min_hz: 2.70e9,
            f_max_hz: 3.05e9,
            linewidth_hz: 5.0e6,
            contrast: 0.20,
            baseline_intensity: 1.0,
        }
    }
}

impl OdmrSpectrumConfig {
    /// Evaluates the normalized photoluminescence intensity at frequency $f$ under magnetic field $\mathbf{B}$.
    /// Sums Lorentzian resonance dips over the 4 crystallographic NV orientations.
    pub fn evaluate_point(&self, nv: &NvCenterConfig, b_vector: [f64; 3], freq_hz: f64) -> f64 {
        let hwhm = 0.5 * self.linewidth_hz;
        let hwhm_sq = hwhm * hwhm;
        let mut total_dip = 0.0;

        // Weight per orientation (4 axes)
        let weight = self.contrast / 8.0; // 4 axes * 2 branches = 8 transitions

        for axis_idx in 0..4 {
            let (f_plus, f_minus) = nv.transition_frequencies_hz(b_vector, axis_idx);
            let lorentz_plus = hwhm_sq / ((freq_hz - f_plus).powi(2) + hwhm_sq);
            let lorentz_minus = hwhm_sq / ((freq_hz - f_minus).powi(2) + hwhm_sq);
            total_dip += weight * (lorentz_plus + lorentz_minus);
        }

        self.baseline_intensity * (1.0 - total_dip).clamp(0.0, 1.0)
    }

    /// Generates a discrete ODMR spectrum across the configured frequency sweep range.
    pub fn generate_spectrum(
        &self,
        nv: &NvCenterConfig,
        b_vector: [f64; 3],
        num_points: usize,
    ) -> Vec<(f64, f64)> {
        let n = num_points.max(2);
        let df = (self.f_max_hz - self.f_min_hz) / ((n - 1) as f64);
        let mut points = Vec::with_capacity(n);

        for i in 0..n {
            let f = self.f_min_hz + (i as f64) * df;
            let pl = self.evaluate_point(nv, b_vector, f);
            points.push((f, pl));
        }

        points
    }
}
