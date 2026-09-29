//! Eigensolver for acoustic pseudomagnetic pseudo-Landau levels,
//! relativistic Dirac phonon dispersion, and valley-polarized zero-modes.

use phonon_core::constants::{ELEMENTARY_CHARGE, H_BAR};
use phonon_models::valley_acoustic::{StrainGaugeParams, ValleyIndex};

/// Eigensolver for acoustic pseudo-Landau levels in strained honeycomb phononic lattices.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValleyLandauLevelSolver {
    pub params: StrainGaugeParams,
}

impl ValleyLandauLevelSolver {
    /// Creates a new pseudo-Landau level solver.
    pub fn new(params: StrainGaugeParams) -> Self {
        Self { params }
    }

    /// Solves the discrete pseudo-Landau level spectrum for $n \in [-N, N]$.
    /// Returns a list of $(n, \omega_n\text{ [rad/s]}, E_n\text{ [meV]})$.
    pub fn solve_spectrum(&self, max_level: i32) -> Vec<(i32, f64, f64)> {
        let n_max = max_level.abs().max(1);
        (-n_max..=n_max)
            .map(|n| {
                let omega = self.params.landau_level_frequency_rad_s(n);
                let energy_mev = (H_BAR * omega / ELEMENTARY_CHARGE) * 1000.0;
                (n, omega, energy_mev)
            })
            .collect()
    }

    /// Solves the sublattice wavefunction probabilities $(P_A, P_B)$ for level $n$ and valley $\tau_z$.
    /// For the zero-energy mode ($n = 0$):
    /// - Valley $K$ is $100\%$ polarized on sublattice $A$ ($P_A = 1.0, P_B = 0.0$).
    /// - Valley $K'$ is $100\%$ polarized on sublattice $B$ ($P_A = 0.0, P_B = 1.0$).
    ///
    /// For excited modes ($n \ne 0$): equal distribution $P_A = P_B = 0.5$.
    pub fn solve_sublattice_polarization(&self, n: i32, valley: ValleyIndex) -> (f64, f64) {
        if n == 0 {
            match valley {
                ValleyIndex::ValleyK => (1.0, 0.0),
                ValleyIndex::ValleyKPrime => (0.0, 1.0),
            }
        } else {
            (0.5, 0.5)
        }
    }

    /// Computes the spectral gap between $n = 0$ and $n = 1$ pseudo-Landau levels in $\text{meV}$.
    pub fn solve_landau_gap_mev(&self) -> f64 {
        self.params.landau_level_gap_mev()
    }
}
