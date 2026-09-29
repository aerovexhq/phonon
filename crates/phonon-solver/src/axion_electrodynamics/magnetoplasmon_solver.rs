//! Solvers for chiral magnetic domain walls, topological solitons,
//! and non-reciprocal topological magnetoplasmons (TMP).

use phonon_models::axion_electrodynamics::{
    MagnetoplasmonMetrics, TopologicalMagnetoplasmonParams,
};

/// Solver for topological magnetoplasmons and chiral domain wall solitons.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologicalMagnetoplasmonSolver {
    pub params: TopologicalMagnetoplasmonParams,
}

impl TopologicalMagnetoplasmonSolver {
    /// Creates a new topological magnetoplasmon solver instance.
    pub fn new(params: TopologicalMagnetoplasmonParams) -> Self {
        Self { params }
    }

    /// Evaluates the chiral domain wall spatial profile $\theta(x) = 2 \arctan(\exp(x / \Delta_{\mathrm{DW}}))$ in radians.
    pub fn evaluate_soliton_profile(&self, x_nm: f64) -> f64 {
        let delta = self.params.domain_wall_width_nm.max(1.0);
        let arg = (x_nm / delta).clamp(-20.0, 20.0);
        2.0 * arg.exp().atan()
    }

    /// Evaluates the topological soliton charge $Q_{\mathrm{topo}} = \frac{1}{\pi}[\theta(+\infty) - \theta(-\infty)] = \pm 1$.
    pub fn compute_soliton_topological_charge(&self) -> i32 {
        if self.params.chiral_dmi_energy_mj_m2 >= 0.0 {
            1
        } else {
            -1
        }
    }

    /// Evaluates non-reciprocal forward transmission $T_+$ in decibels.
    pub fn compute_forward_transmission_db(&self) -> f64 {
        -self.params.forward_attenuation_db_per_um * self.params.waveguide_length_um
    }

    /// Evaluates non-reciprocal backward isolation attenuation $T_-$ in decibels.
    pub fn compute_backward_isolation_db(&self) -> f64 {
        -self.params.backward_attenuation_db_per_um * self.params.waveguide_length_um
    }

    /// Evaluates non-reciprocal forward-to-backward isolation contrast $\mathcal{G}_{\mathrm{dir}} = T_+ - T_-$ in decibels.
    pub fn compute_non_reciprocal_isolation_db(&self) -> f64 {
        let t_plus = self.compute_forward_transmission_db();
        let t_minus = self.compute_backward_isolation_db();
        (t_plus - t_minus).max(0.0)
    }

    /// Evaluates the chiral edge magnetoplasmon group velocity in $\text{m/s}$.
    pub fn compute_chiral_edge_velocity_m_s(&self) -> f64 {
        let dmi = self.params.chiral_dmi_energy_mj_m2;
        let v0 = self.params.plasmon_velocity_m_s;
        let dmi_mod = (dmi / 2.0).clamp(-0.5, 0.5);
        v0 * (1.0 + 0.15 * dmi_mod)
    }

    /// Evaluates the transverse confinement skin depth in nanometers.
    pub fn compute_confinement_depth_nm(&self) -> f64 {
        let delta = self.params.domain_wall_width_nm;
        let v = self.compute_chiral_edge_velocity_m_s();
        let freq_ghz = self.params.magnetoplasmon_frequency_ghz;

        // Effective confinement: lambda_conf ~ delta * sqrt(1 + (v * k / omega_p)^2)
        let confinement_factor = 1.0 + 0.08 * (freq_ghz / 30.0) * (v / 3.0e5);
        delta * confinement_factor
    }

    /// Solves the full topological magnetoplasmon metrics.
    pub fn solve(&self) -> MagnetoplasmonMetrics {
        let isolation_db = self.compute_non_reciprocal_isolation_db();
        let forward_db = self.compute_forward_transmission_db();
        let backward_db = self.compute_backward_isolation_db();
        let charge = self.compute_soliton_topological_charge();
        let edge_v = self.compute_chiral_edge_velocity_m_s();
        let conf_depth = self.compute_confinement_depth_nm();
        let handedness = if self.params.chiral_dmi_energy_mj_m2 >= 0.0 {
            1
        } else {
            -1
        };

        MagnetoplasmonMetrics {
            non_reciprocal_isolation_db: isolation_db,
            forward_transmission_db: forward_db,
            backward_isolation_db: backward_db,
            soliton_topological_charge: charge,
            chiral_edge_velocity_m_s: edge_v,
            plasmon_confinement_depth_nm: conf_depth,
            dmi_chirality_handedness: handedness,
        }
    }
}
