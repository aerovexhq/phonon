//! Solvers for cascaded topological acoustic heat rectifiers,
//! thermal diodes, and non-reciprocal phononic waveguides.

use phonon_models::chiral_spin_seebeck::{
    ThermalRectifierMetrics, TopologicalThermalRectifierParams,
};

/// Solver for topological acoustic heat rectifiers and thermal diodes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologicalHeatRectifierSolver {
    pub params: TopologicalThermalRectifierParams,
}

impl TopologicalHeatRectifierSolver {
    /// Creates a new topological heat rectifier solver instance.
    pub fn new(params: TopologicalThermalRectifierParams) -> Self {
        Self { params }
    }

    /// Evaluates cascaded forward transmission $T_{\mathrm{fwd}, \mathrm{casc}} = T_{\mathrm{fwd}}^N$.
    pub fn compute_forward_total_transmission(&self) -> f64 {
        let n = self.params.diode_stages_count as i32;
        self.params.forward_phonon_transmission.powi(n)
    }

    /// Evaluates cascaded backward transmission $T_{\mathrm{bwd}, \mathrm{casc}} = T_{\mathrm{bwd}}^N$.
    pub fn compute_backward_total_transmission(&self) -> f64 {
        let n = self.params.diode_stages_count as i32;
        self.params
            .backward_phonon_transmission
            .powi(n)
            .max(1.0e-15)
    }

    /// Evaluates overall cascaded thermal rectification ratio $\mathcal{R}_{\mathrm{th}} = T_{\mathrm{fwd}, \mathrm{casc}} / T_{\mathrm{bwd}, \mathrm{casc}}$.
    pub fn compute_rectification_ratio(&self) -> f64 {
        let t_fwd = self.compute_forward_total_transmission();
        let t_bwd = self.compute_backward_total_transmission();
        (t_fwd / t_bwd).max(1.0)
    }

    /// Evaluates directional thermal contrast in decibels $10 \log_{10}(\mathcal{R}_{\mathrm{th}})$.
    pub fn compute_thermal_contrast_db(&self) -> f64 {
        10.0 * self.compute_rectification_ratio().log10()
    }

    /// Evaluates reverse thermal isolation in decibels $-10 \log_{10}(T_{\mathrm{bwd}, \mathrm{casc}})$.
    pub fn compute_reverse_isolation_db(&self) -> f64 {
        -10.0 * self.compute_backward_total_transmission().log10()
    }

    /// Solves the full topological thermal rectifier metrics.
    pub fn solve(&self) -> ThermalRectifierMetrics {
        let r_th = self.compute_rectification_ratio();
        let contrast_db = self.compute_thermal_contrast_db();
        let fwd_trans = self.compute_forward_total_transmission();
        let bwd_trans = self.compute_backward_total_transmission();
        let iso_db = self.compute_reverse_isolation_db();

        ThermalRectifierMetrics {
            rectification_ratio: r_th,
            thermal_contrast_db: contrast_db,
            forward_total_transmission: fwd_trans,
            backward_total_transmission: bwd_trans,
            reverse_isolation_db: iso_db,
        }
    }
}
