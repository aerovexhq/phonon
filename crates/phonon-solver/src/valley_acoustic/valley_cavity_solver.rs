//! Multi-physics solver for chiral acoustic cavities, pseudomagnetic phonon traps,
//! and valley-selective Purcell factor enhancement.

use phonon_models::valley_acoustic::{ValleyCavityMode, ValleyCavityParams, ValleyIndex};

/// Multi-physics solver for chiral quantum valley acoustic cavities.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValleyCavitySolver {
    pub params: ValleyCavityParams,
}

impl ValleyCavitySolver {
    /// Creates a new valley acoustic cavity solver.
    pub fn new(params: ValleyCavityParams) -> Self {
        Self { params }
    }

    /// Solves the chiral cavity mode for the specified valley.
    pub fn solve_mode(&self, valley: ValleyIndex) -> ValleyCavityMode {
        self.params.evaluate_mode(valley)
    }

    /// Solves the valley polarization contrast ratio in decibels: $\mathcal{R}_{\mathrm{valley}} = 10 \log_{10}(I_K / I_{K'})$.
    pub fn solve_valley_contrast_db(&self) -> f64 {
        let mode_k = self.params.evaluate_mode(ValleyIndex::ValleyK);
        mode_k.valley_contrast_db
    }

    /// Solves the valley-selective Purcell enhancement factor $F_P$.
    pub fn solve_purcell_enhancement(&self, valley: ValleyIndex) -> f64 {
        let mode = self.params.evaluate_mode(valley);
        mode.purcell_factor
    }
}
