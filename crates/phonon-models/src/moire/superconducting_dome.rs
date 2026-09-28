//! Unconventional superconductivity and pairing domes in electrostatic-doped moiré flat bands.
//!
//! Formulates doping-dependent critical temperature T_c(\u{03b4}), Ginzburg-Landau coherence length \u{03be}_GL,
//! and anisotropic upper critical magnetic fields B_c2.

use std::f64::consts::PI;

/// Unconventional superconducting pairing dome model adjacent to correlated insulator states.
#[derive(Debug, Clone, PartialEq)]
pub struct SuperconductingDomeModel {
    /// Maximum critical temperature T_c,max (K) at optimal doping.
    pub tc_max_k: f64,
    /// Center filling of the parent correlated insulator (typically \u{03bd} = -2 or +2).
    pub parent_filling: f64,
    /// Minimum doping \u{03b4}_min from parent state where superconductivity turns on.
    pub delta_min: f64,
    /// Maximum doping \u{03b4}_max where superconductivity is quenched.
    pub delta_max: f64,
    /// Ginzburg-Landau coherence length \u{03be}_GL,0 at T = 0 (nm).
    pub coherence_length_0_nm: f64,
}

impl SuperconductingDomeModel {
    /// Creates a typical superconducting dome for hole-doped TBG adjacent to \u{03bd} = -2:
    /// T_c,max = 1.7 K, optimal doping \u{03b4}_opt \u{2248} 0.25 (filling -2.25).
    pub fn new_hole_doped_tbg() -> Self {
        Self {
            tc_max_k: 1.7,
            parent_filling: -2.0,
            delta_min: 0.05,
            delta_max: 0.50,
            coherence_length_0_nm: 37.0, // 37 nm -> Bc2 ~ 0.24 T
        }
    }

    /// Creates an electron-doped superconducting dome adjacent to \u{03bd} = +2.
    pub fn new_electron_doped_tbg() -> Self {
        Self {
            tc_max_k: 1.2,
            parent_filling: 2.0,
            delta_min: 0.05,
            delta_max: 0.45,
            coherence_length_0_nm: 75.0,
        }
    }

    /// Evaluates critical temperature T_c(\u{03bd}) for a given moiré filling factor \u{03bd}:
    /// Dome parabolic profile: T_c(\u{03b4}) = 4 * T_c,max * (\u{03b4} - \u{03b4}_min)(\u{03b4}_max - \u{03b4}) / (\u{03b4}_max - \u{03b4}_min)^2.
    pub fn critical_temperature_k(&self, filling: f64) -> f64 {
        let delta = if self.parent_filling < 0.0 {
            self.parent_filling - filling // e.g. -2.0 - (-2.25) = 0.25
        } else {
            filling - self.parent_filling // e.g. 2.25 - 2.0 = 0.25
        };

        if delta < self.delta_min || delta > self.delta_max {
            return 0.0;
        }

        let span = self.delta_max - self.delta_min;
        let norm_d = (delta - self.delta_min) / span;
        4.0 * self.tc_max_k * norm_d * (1.0 - norm_d)
    }

    /// Evaluates the BCS superconducting gap \u{0394}_SC(0) = 1.764 * k_B * T_c (eV).
    pub fn superconducting_gap_0_ev(&self, filling: f64) -> f64 {
        let tc = self.critical_temperature_k(filling);
        let kb_ev_k = 8.617_333_262e-5;
        1.764 * kb_ev_k * tc
    }

    /// Evaluates the perpendicular upper critical field B_c2,\u{22a5}(0) = \u{03a6}_0 / (2\u{03c0} \u{03be}_GL^2) in Tesla.
    pub fn upper_critical_field_perp_tesla(&self, filling: f64) -> f64 {
        let tc = self.critical_temperature_k(filling);
        if tc <= 1e-4 {
            return 0.0;
        }
        // Magnetic flux quantum \u{03a6}_0 = h / (2e) \u{2248} 2.067833848e-15 Wb
        let phi_0 = 2.067_833_848e-15;
        let xi_m = self.coherence_length_0_nm * 1e-9;
        phi_0 / (2.0 * PI * xi_m * xi_m)
    }

    /// Evaluates the Pauli paramagnetic limit B_Pauli = \u{0394}_SC(0) / (\u{221a}2 \u{03bc}_B) \u{2248} 1.84 * T_c [Tesla].
    pub fn pauli_limit_tesla(&self, filling: f64) -> f64 {
        let tc = self.critical_temperature_k(filling);
        1.84 * tc
    }
}
