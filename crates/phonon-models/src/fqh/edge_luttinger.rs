//! Fractional quantum Hall (FQH) edge states and chiral Luttinger liquids.
//!
//! Formulates chiral bosonic charge modes, neutral Majorana fermion edge modes,
//! fractional quasiparticle charges e*, flux quanta \u{03a6}_0^*, and conformal field theory (CFT) weights.

use std::f64::consts::PI;

/// Fundamental physical constants.
pub const ELEMENTARY_CHARGE: f64 = 1.602_176_634e-19; // Coulombs
pub const PLANCK_H: f64 = 6.626_070_15e-34; // J*s
pub const HBAR: f64 = 1.054_571_817e-34; // J*s
pub const FLUX_QUANTUM_H_OVER_E: f64 = 4.135_667_696e-15; // Wb (h/e)

/// Fractional Quantum Hall topological state classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FqhState {
    /// Integer Quantum Hall state \u{03bd} = 1 (Fermi liquid edge).
    IntegerOne,
    /// Laughlin state \u{03bd} = 1/3 (Abelian chiral Luttinger liquid).
    LaughlinOneThird,
    /// Moore-Read Pfaffian state \u{03bd} = 5/2 (non-Abelian Ising anyons + neutral Majorana mode).
    MooreReadFiveHalves,
}

/// Parameters of a chiral Luttinger liquid edge channel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LuttingerEdgeModel {
    /// Topological state.
    pub state: FqhState,
    /// Magnetic field B in Tesla.
    pub magnetic_field_tesla: f64,
    /// Chiral charge mode velocity v_c in m/s (typically 1.0e4 - 1.0e5 m/s).
    pub charge_velocity_m_s: f64,
    /// Neutral Majorana mode velocity v_n in m/s (typically ~0.5 * v_c for Moore-Read).
    pub neutral_velocity_m_s: f64,
}

impl LuttingerEdgeModel {
    /// Constructs a standard \u{03bd} = 1/3 Laughlin edge channel.
    pub fn laughlin_one_third(magnetic_field_tesla: f64) -> Self {
        Self {
            state: FqhState::LaughlinOneThird,
            magnetic_field_tesla,
            charge_velocity_m_s: 5.0e4,
            neutral_velocity_m_s: 0.0, // No neutral mode
        }
    }

    /// Constructs a standard \u{03bd} = 5/2 Moore-Read non-Abelian edge channel.
    pub fn moore_read_five_halves(magnetic_field_tesla: f64) -> Self {
        Self {
            state: FqhState::MooreReadFiveHalves,
            magnetic_field_tesla,
            charge_velocity_m_s: 4.0e4,
            neutral_velocity_m_s: 2.0e4,
        }
    }

    /// Filling factor \u{03bd} of the active edge mode.
    pub fn filling_factor(&self) -> f64 {
        match self.state {
            FqhState::IntegerOne => 1.0,
            FqhState::LaughlinOneThird => 1.0 / 3.0,
            FqhState::MooreReadFiveHalves => 2.5,
        }
    }

    /// Fractional quasiparticle charge e* in units of fundamental electron charge e.
    /// \u{03bd} = 1/3 \u{2192} e* = e/3.
    /// \u{03bd} = 5/2 \u{2192} e* = e/4 (non-Abelian \u{03c3} anyon).
    pub fn fractional_charge_ratio(&self) -> f64 {
        match self.state {
            FqhState::IntegerOne => 1.0,
            FqhState::LaughlinOneThird => 1.0 / 3.0,
            FqhState::MooreReadFiveHalves => 0.25,
        }
    }

    /// Fractional quasiparticle charge in Coulombs.
    pub fn fractional_charge_coulombs(&self) -> f64 {
        self.fractional_charge_ratio() * ELEMENTARY_CHARGE
    }

    /// Magnetic length \u{2113}_B = sqrt(\u{210f} / (e B)) in nanometers.
    pub fn magnetic_length_nm(&self) -> f64 {
        let b = self.magnetic_field_tesla.abs().max(1e-4);
        let l_m = (HBAR / (ELEMENTARY_CHARGE * b)).sqrt();
        l_m * 1e9
    }

    /// Effective Aharonov-Bohm flux quantum \u{03a6}_0^* = h / e* in Weber (Tesla * m^2):
    /// \u{03bd} = 1/3 \u{2192} 3 * (h/e).
    /// \u{03bd} = 5/2 \u{2192} 4 * (h/e).
    pub fn effective_flux_quantum_weber(&self) -> f64 {
        FLUX_QUANTUM_H_OVER_E / self.fractional_charge_ratio()
    }

    /// Conformal central charge c of the edge theory.
    /// \u{03bd} = 1/3: c = 1 (chiral boson).
    /// \u{03bd} = 5/2: c = 1 (charge boson) + 1/2 (neutral Majorana fermion) = 1.5.
    pub fn central_charge(&self) -> f64 {
        match self.state {
            FqhState::IntegerOne => 1.0,
            FqhState::LaughlinOneThird => 1.0,
            FqhState::MooreReadFiveHalves => 1.5,
        }
    }

    /// Conformal scaling dimension / conformal weight h of quasiparticle operators:
    /// - Laughlin e/3: h = (1/3) / 2 = 1/6 \u{2248} 0.1667.
    /// - Moore-Read non-Abelian \u{03c3} anyon: h_\u{03c3} = h_charge + h_neutral = 1/32 + 1/16 = 3/32 = 0.09375.
    pub fn quasiparticle_conformal_weight(&self) -> f64 {
        match self.state {
            FqhState::IntegerOne => 0.5,
            FqhState::LaughlinOneThird => 1.0 / 6.0,
            FqhState::MooreReadFiveHalves => 3.0 / 32.0,
        }
    }

    /// Statistical exchange phase \u{03b8} in radians upon swapping two quasiparticles:
    /// - \u{03bd} = 1/3: \u{03b8} = \u{03c0} / 3.
    /// - \u{03bd} = 5/2: non-Abelian braid matrix R (phase factor e^{-i\u{03c0}/8}).
    pub fn statistical_exchange_phase_rad(&self) -> f64 {
        match self.state {
            FqhState::IntegerOne => PI,
            FqhState::LaughlinOneThird => PI / 3.0,
            FqhState::MooreReadFiveHalves => PI / 8.0,
        }
    }
}
