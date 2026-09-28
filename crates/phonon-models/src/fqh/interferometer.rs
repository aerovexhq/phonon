//! Fabry-P\u{00e9}rot and Mach-Zehnder electronic interferometers in fractional quantum Hall regimes.
//!
//! Models quantum point contact (QPC) tunneling, Aharonov-Bohm flux periods,
//! Coulomb charging diamonds, and non-Abelian even-odd visibility collapse.

use super::edge_luttinger::{LuttingerEdgeModel, ELEMENTARY_CHARGE, PLANCK_H};
use std::f64::consts::PI;

/// Interferometer operational regime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterferometerRegime {
    /// Aharonov-Bohm regime (weak Coulomb interaction, E_C << \u{0394}E_orb).
    AharonovBohm,
    /// Intermediate crossover regime.
    Intermediate,
    /// Coulomb-dominated charging regime (E_C >> \u{0394}E_orb).
    CoulombDominated,
}

/// Fabry-P\u{00e9}rot electronic quantum Hall interferometer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FabryPerotInterferometer {
    /// QPC 1 tunneling transmission amplitude t_1 \u{2208} [0, 1].
    pub t1_transmission: f64,
    /// QPC 2 tunneling transmission amplitude t_2 \u{2208} [0, 1].
    pub t2_transmission: f64,
    /// Enclosed interferometric cell area in \u{03bc}m^2 (typically 1.0 - 5.0 um^2).
    pub cell_area_um2: f64,
    /// Island charging energy E_C in meV (typically 0.01 - 0.20 meV).
    pub charging_energy_mev: f64,
    /// Single-particle orbital level spacing \u{0394}E_orb in meV (typically ~0.05 - 0.30 meV).
    pub orbital_spacing_mev: f64,
    /// Gate lever arm \u{03b1}_g = C_g / C_total \u{2208} (0, 1).
    pub gate_lever_arm: f64,
}

impl FabryPerotInterferometer {
    /// Standard Fabry-P\u{00e9}rot interferometer in the Aharonov-Bohm regime.
    pub fn standard_ab_regime() -> Self {
        Self {
            t1_transmission: 0.15,
            t2_transmission: 0.15,
            cell_area_um2: 2.0,
            charging_energy_mev: 0.02,
            orbital_spacing_mev: 0.12,
            gate_lever_arm: 0.25,
        }
    }

    /// Standard Fabry-P\u{00e9}rot interferometer in the Coulomb-dominated regime.
    pub fn standard_coulomb_regime() -> Self {
        Self {
            t1_transmission: 0.15,
            t2_transmission: 0.15,
            cell_area_um2: 0.8,
            charging_energy_mev: 0.18,
            orbital_spacing_mev: 0.06,
            gate_lever_arm: 0.25,
        }
    }

    /// Coupling ratio K_int = E_C / \u{0394}E_orb.
    pub fn coupling_ratio(&self) -> f64 {
        self.charging_energy_mev / self.orbital_spacing_mev.max(1e-4)
    }

    /// Classifies the operating regime of the interferometer.
    pub fn operating_regime(&self) -> InterferometerRegime {
        let k = self.coupling_ratio();
        if k < 0.5 {
            InterferometerRegime::AharonovBohm
        } else if k > 1.5 {
            InterferometerRegime::CoulombDominated
        } else {
            InterferometerRegime::Intermediate
        }
    }

    /// Cell area in m^2.
    pub fn cell_area_m2(&self) -> f64 {
        self.cell_area_um2 * 1e-12
    }

    /// Magnetic flux \u{03a6} = B * A_cell in Weber.
    pub fn magnetic_flux_weber(&self, b_tesla: f64) -> f64 {
        b_tesla * self.cell_area_m2()
    }

    /// Aharonov-Bohm oscillation period \u{0394}B in Tesla: \u{0394}B = \u{03a6}_0^* / A_cell.
    pub fn ab_period_tesla(&self, edge: &LuttingerEdgeModel) -> f64 {
        let phi_0_star = edge.effective_flux_quantum_weber();
        phi_0_star / self.cell_area_m2()
    }

    /// Gate voltage period \u{0394}V_g in milliVolts to add one quasiparticle e* into the island:
    /// \u{0394}V_g = e* / C_g = (e* / e) * (e / C_total) / \u{03b1}_g = (e* / e) * 2 E_C / \u{03b1}_g.
    pub fn gate_period_mv(&self, edge: &LuttingerEdgeModel) -> f64 {
        let q_ratio = edge.fractional_charge_ratio();
        // 2 * E_C (in meV) / lever_arm gives mV
        (q_ratio * 2.0 * self.charging_energy_mev) / self.gate_lever_arm.max(1e-4)
    }

    /// Non-Abelian interference visibility factor \u{03bd}_vis \u{2208} [0, 1] for Moore-Read \u{03bd} = 5/2 state:
    /// - If bulk contains an EVEN number of \u{03c3} anyons (N_\u{03c3} % 2 == 0): \u{03bd}_vis = 1.0.
    /// - If bulk contains an ODD number of \u{03c3} anyons (N_\u{03c3} % 2 == 1): \u{03bd}_vis = 0.0 (visibility collapse!).
    pub fn non_abelian_visibility(&self, edge: &LuttingerEdgeModel, num_bulk_anyons: usize) -> f64 {
        match edge.state {
            crate::fqh::edge_luttinger::FqhState::MooreReadFiveHalves => {
                if num_bulk_anyons.is_multiple_of(2) {
                    1.0
                } else {
                    0.0 // Non-Abelian braiding entangles with bulk anyon, erasing interference!
                }
            }
            _ => 1.0, // Abelian states retain visibility regardless of parity
        }
    }

    /// State-dependent topological phase shift in radians for \u{03bd} = 5/2:
    /// - |0\u{27e9} channel: phase = -\u{03c0}/4.
    /// - |1\u{27e9} channel: phase = +3\u{03c0}/4 (shifted by \u{03c0}!).
    pub fn topological_phase_shift_rad(
        &self,
        edge: &LuttingerEdgeModel,
        anyon_state_one: bool,
    ) -> f64 {
        match edge.state {
            crate::fqh::edge_luttinger::FqhState::MooreReadFiveHalves => {
                if anyon_state_one {
                    3.0 * PI / 4.0
                } else {
                    -PI / 4.0
                }
            }
            _ => 0.0,
        }
    }

    /// Evaluates effective two-path tunneling probability T_eff:
    /// T_eff = |t1|^2 + |t2|^2 + 2 |t1 t2| * \u{03bd}_vis * cos(\u{03b8}_total).
    pub fn effective_tunneling_probability(
        &self,
        edge: &LuttingerEdgeModel,
        b_tesla: f64,
        vg_mv: f64,
        num_bulk_anyons: usize,
        anyon_state_one: bool,
    ) -> f64 {
        let t1_sq = self.t1_transmission * self.t1_transmission;
        let t2_sq = self.t2_transmission * self.t2_transmission;
        let cross_term = 2.0 * self.t1_transmission * self.t2_transmission;

        let phi = self.magnetic_flux_weber(b_tesla);
        let phi_star = edge.effective_flux_quantum_weber();
        let theta_ab = 2.0 * PI * (phi / phi_star);

        let delta_vg = self.gate_period_mv(edge);
        let theta_gate = 2.0 * PI * (vg_mv / delta_vg.max(1e-4));

        let theta_topo = self.topological_phase_shift_rad(edge, anyon_state_one);
        let visibility = self.non_abelian_visibility(edge, num_bulk_anyons);

        // Phase coupling based on regime
        let theta_total = match self.operating_regime() {
            InterferometerRegime::AharonovBohm => theta_ab + theta_topo + 0.1 * theta_gate,
            InterferometerRegime::CoulombDominated => theta_ab - theta_gate + theta_topo,
            InterferometerRegime::Intermediate => theta_ab + theta_topo + 0.5 * theta_gate,
        };

        let t_eff = t1_sq + t2_sq + cross_term * visibility * theta_total.cos();
        t_eff.clamp(0.0, 1.0)
    }

    /// Evaluates tunneling conductance G in units of e^2 / h (microSiemens \u{2248} 38.74 uS):
    /// G = ((e* / e)^2 * (e^2 / h)) * T_eff.
    pub fn tunneling_conductance_siemens(
        &self,
        edge: &LuttingerEdgeModel,
        b_tesla: f64,
        vg_mv: f64,
        num_bulk_anyons: usize,
        anyon_state_one: bool,
    ) -> f64 {
        let t_eff = self.effective_tunneling_probability(
            edge,
            b_tesla,
            vg_mv,
            num_bulk_anyons,
            anyon_state_one,
        );
        let q_ratio = edge.fractional_charge_ratio();
        let quantum_conductance = (ELEMENTARY_CHARGE * ELEMENTARY_CHARGE) / PLANCK_H; // ~3.874e-5 S
        q_ratio * q_ratio * quantum_conductance * t_eff
    }
}
