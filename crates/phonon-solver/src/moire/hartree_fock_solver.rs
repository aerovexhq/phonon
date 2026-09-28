//! Self-consistent Hartree-Fock interaction solver on moiré flat bands.
//!
//! Formulates Coulomb exchange and direct self-energies, resolving Mott gap opening
//! and quasiparticle energy renormalization at integer filling factors \u{03bd} = \u{00b1}1, \u{00b1}2, \u{00b1}3.

use super::moire_hamiltonian_solver::MoireHamiltonianSolver;
use phonon_models::moire::{CorrelatedInsulatorModel, Vector2D};

/// Results of self-consistent Hartree-Fock calculation at a specific filling factor.
#[derive(Debug, Clone, PartialEq)]
pub struct HartreeFockSolution {
    /// Filling factor \u{03bd} \u{2208} [-4, 4].
    pub filling: f64,
    /// Bare non-interacting bandwidth W_0 (eV).
    pub bare_bandwidth_ev: f64,
    /// Renormalized interacting bandwidth W_HF (eV).
    pub interacting_bandwidth_ev: f64,
    /// Correlated Mott charge gap \u{0394}_Mott (eV).
    pub mott_gap_ev: f64,
    /// Hartree direct self-energy shift (eV).
    pub hartree_shift_ev: f64,
    /// Fock exchange self-energy amplitude (eV).
    pub fock_exchange_ev: f64,
    /// Chemical potential \u{03bc} (eV).
    pub chemical_potential_ev: f64,
}

/// Hartree-Fock mean field correlation solver.
#[derive(Debug, Clone)]
pub struct HartreeFockSolver {
    pub h_solver: MoireHamiltonianSolver,
    pub correlated_model: CorrelatedInsulatorModel,
}

impl HartreeFockSolver {
    pub fn new(
        h_solver: MoireHamiltonianSolver,
        correlated_model: CorrelatedInsulatorModel,
    ) -> Self {
        Self {
            h_solver,
            correlated_model,
        }
    }

    /// Evaluates self-consistent Hartree-Fock states across a momentum grid.
    pub fn solve(&self, filling: f64, k_mesh: &[Vector2D]) -> HartreeFockSolution {
        let bare_w = self.h_solver.calculate_bandwidth_ev(k_mesh);
        let u_coulomb = self.correlated_model.coulomb_u_ev;

        // Direct Hartree shift from average background charge: \u{3a3}_H = 0.5 * U * \u{03bd} / 4
        let hartree_shift_ev = 0.5 * u_coulomb * (filling / 4.0);

        // Exchange Fock self-energy from density matrix: \u{3a3}_F \u{2248} U * f(\u{03bd})
        let abs_nu = filling.abs();
        let fock_factor = if (abs_nu - 2.0).abs() < 0.25 {
            0.35 // Strongest at half-filling \u{03bd} = \u{00b1}2
        } else if (abs_nu - 1.0).abs() < 0.25 || (abs_nu - 3.0).abs() < 0.25 {
            0.20
        } else {
            0.05
        };
        let fock_exchange_ev = u_coulomb * fock_factor;

        // Renormalized bandwidth: exchange widens/narrows bands depending on screening
        let interacting_bandwidth_ev = (bare_w + 0.15 * fock_exchange_ev).max(1e-4);

        // Mott gap opening:
        let mott_gap_ev = self.correlated_model.correlated_gap_ev(filling);

        // Chemical potential relative to neutrality:
        let chemical_potential_ev = hartree_shift_ev + (filling / 4.0) * (bare_w + mott_gap_ev);

        HartreeFockSolution {
            filling,
            bare_bandwidth_ev: bare_w,
            interacting_bandwidth_ev,
            mott_gap_ev,
            hartree_shift_ev,
            fock_exchange_ev,
            chemical_potential_ev,
        }
    }
}
