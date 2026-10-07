#![deny(unsafe_code)]

//! Nonlinear Polariton Soliton Dynamics & Self-Trapping in Moiré Flat Bands.
//!
//! Models topological polariton solitons formed by the balance between quenched
//! kinetic energy (divergent effective mass m*) and enhanced Kerr acoustic nonlinearity.

/// Parameters for the nonlinear polariton soliton solver.
#[derive(Debug, Clone, PartialEq)]
pub struct PolaritonSolitonParams {
    /// Kerr acoustic nonlinearity coefficient g_nl in 1/(um * mW).
    pub kerr_nonlinearity_g_nl: f64,
    /// Effective acoustic mass ratio m* / m0 (diverges in flat bands).
    pub effective_mass_ratio: f64,
    /// Peak soliton power / amplitude in mW.
    pub peak_power_mw: f64,
    /// Propagation distance L in um.
    pub propagation_distance_um: f64,
    /// Linear acoustic propagation loss in dB/mm.
    pub loss_db_per_mm: f64,
}

impl Default for PolaritonSolitonParams {
    fn default() -> Self {
        Self {
            kerr_nonlinearity_g_nl: 0.0018,
            effective_mass_ratio: 15.0,
            peak_power_mw: 12.5,
            propagation_distance_um: 250.0,
            loss_db_per_mm: 0.8,
        }
    }
}

/// A spatial sample point along the soliton envelope.
#[derive(Debug, Clone, PartialEq)]
pub struct SolitonProfilePoint {
    /// Coordinate x in micrometers (um) relative to soliton center.
    pub pos_x_um: f64,
    /// Soliton intensity |psi(x)|^2 in mW/um.
    pub soliton_intensity: f64,
    /// Dispersive linear wavepacket intensity for comparison.
    pub linear_dispersive_intensity: f64,
}

/// Evaluated metrics for the polariton soliton.
#[derive(Debug, Clone, PartialEq)]
pub struct PolaritonSolitonMetrics {
    /// Spatial half-width of the soliton w_s in um (w_s <= 15.0 um).
    pub soliton_width_um: f64,
    /// Self-trapping energy confinement ratio (>= 85%).
    pub self_trapping_ratio: f64,
    /// Nonlinear phase shift acquired along propagation distance (radians).
    pub nonlinear_phase_shift_rad: f64,
    /// Dispersion suppression factor over linear packet.
    pub dispersion_suppression_ratio: f64,
    /// Whether stable soliton balance is sustained.
    pub is_soliton_stable: bool,
}

/// Solver for nonlinear polariton soliton dynamics.
#[derive(Debug, Clone)]
pub struct PolaritonSolitonSolver {
    pub params: PolaritonSolitonParams,
}

impl PolaritonSolitonSolver {
    /// Creates a new solver with specified parameters.
    pub fn new(params: PolaritonSolitonParams) -> Self {
        Self { params }
    }

    /// Evaluates operational metrics for the polariton soliton.
    pub fn evaluate_metrics(&self) -> PolaritonSolitonMetrics {
        let p = &self.params;
        let p_eff = p.peak_power_mw.max(0.1);
        let m_eff = p.effective_mass_ratio.max(1.0);
        let g_nl = p.kerr_nonlinearity_g_nl.max(1e-5);

        // Soliton width: w_s = 1 / sqrt(m* * g_nl * P_0) (um)
        let product = m_eff * g_nl * p_eff;
        let soliton_width_um = (1.0 / product.sqrt()).clamp(3.0, 25.0);

        // Self-trapping ratio: fraction of energy retained in the sech envelope over propagation L
        let dist_mm = p.propagation_distance_um / 1000.0;
        let linear_loss_factor = 10.0f64.powf(-p.loss_db_per_mm * dist_mm / 10.0);
        let self_trapping_ratio = (0.92 * linear_loss_factor).clamp(0.50, 0.98);

        // Nonlinear phase shift: phi_nl = g_nl * P_0 * L
        let nonlinear_phase_shift_rad = g_nl * p_eff * p.propagation_distance_um;

        // Dispersion suppression: comparison with diffraction broadening of a linear wavepacket
        let diffraction_length = m_eff * soliton_width_um.powi(2) * 0.1;
        let broadening = 1.0 + (p.propagation_distance_um / diffraction_length.max(10.0)).powi(2);
        let dispersion_suppression_ratio = broadening.sqrt();

        let is_soliton_stable = soliton_width_um <= 15.0 && self_trapping_ratio >= 0.85;

        PolaritonSolitonMetrics {
            soliton_width_um,
            self_trapping_ratio,
            nonlinear_phase_shift_rad,
            dispersion_suppression_ratio,
            is_soliton_stable,
        }
    }

    /// Computes spatial envelope profile comparing soliton against linear spreading wavepacket.
    pub fn compute_spatial_profile(&self, sample_count: usize, span_um: f64) -> Vec<SolitonProfilePoint> {
        let n = sample_count.max(20);
        let metrics = self.evaluate_metrics();
        let w_s = metrics.soliton_width_um;
        let mut profile = Vec::with_capacity(n);

        let half_span = span_um.max(20.0) * 0.5;

        for i in 0..=n {
            let x = -half_span + (i as f64 / n as f64) * span_um;

            // Soliton envelope: sech^2(x / w_s)
            let arg = (x / w_s).clamp(-10.0, 10.0);
            let sech = 1.0 / arg.cosh();
            let soliton_intensity = self.params.peak_power_mw * sech.powi(2);

            // Linear wavepacket spreading: broad Gaussian
            let w_lin = w_s * metrics.dispersion_suppression_ratio;
            let linear_dispersive_intensity = (self.params.peak_power_mw * (w_s / w_lin))
                * (-(x / w_lin).powi(2)).exp();

            profile.push(SolitonProfilePoint {
                pos_x_um: x,
                soliton_intensity,
                linear_dispersive_intensity,
            });
        }

        profile
    }
}
