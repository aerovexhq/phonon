//! Fractional Chern insulator many-body states, anyonic quasiparticles,
//! and spectral gap thermodynamics.
//!
//! # Physical Formalism
//! - Fractional Filling Factor:
//!   $$\nu = \frac{p}{q}$$
//! - Fractional Quasiparticle Charge:
//!   $$e^* = \frac{e}{q}$$
//! - Abelian Anyon Exchange Phase:
//!   $$\theta_{\mathrm{exch}} = \frac{\pi p}{q}$$
//! - Topological Ground State Degeneracy on Torus:
//!   $$D_g = q$$
//! - Topological Neutral Spectral Gap:
//!   $$\Delta_{\mathrm{FCI}} = \alpha_\nu \frac{e^2}{4\pi \epsilon_0 \epsilon_r L_M} - W$$

use super::moire_flat_band::{MoireFlatBand, ELECTRON_CHARGE_C};

/// Filling factor fraction $\nu = p/q$.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FractionalFilling {
    /// Laughlin state at $\nu = 1/3$ ($e^* = e/3$, 3-fold degeneracy).
    OneThird,
    /// Complementary Laughlin state at $\nu = 2/3$ ($e^* = e/3$, 3-fold degeneracy).
    TwoThirds,
    /// Higher-order Laughlin state at $\nu = 1/5$ ($e^* = e/5$, 5-fold degeneracy).
    OneFifth,
    /// Jain hierarchy state at $\nu = 2/5$ ($e^* = e/5$, 5-fold degeneracy).
    TwoFifths,
}

impl FractionalFilling {
    pub fn numerator(&self) -> usize {
        match self {
            Self::OneThird => 1,
            Self::TwoThirds => 2,
            Self::OneFifth => 1,
            Self::TwoFifths => 2,
        }
    }

    pub fn denominator(&self) -> usize {
        match self {
            Self::OneThird | Self::TwoThirds => 3,
            Self::OneFifth | Self::TwoFifths => 5,
        }
    }

    pub fn as_f64(&self) -> f64 {
        (self.numerator() as f64) / (self.denominator() as f64)
    }

    /// Dimensionless gap prefactor $\alpha_\nu$ in $\Delta = \alpha_\nu E_C - W$.
    pub fn gap_coefficient(&self) -> f64 {
        match self {
            Self::OneThird => 0.16,
            Self::TwoThirds => 0.15,
            Self::OneFifth => 0.14,
            Self::TwoFifths => 0.13,
        }
    }
}

/// Fractional Chern Insulator (FCI) state.
#[derive(Debug, Clone, PartialEq)]
pub struct FractionalChernState {
    pub flat_band: MoireFlatBand,
    pub filling: FractionalFilling,
    /// Fractional quasiparticle charge $e^*$ in Coulombs.
    pub fractional_charge_c: f64,
    /// Quasiparticle statistical exchange angle in radians.
    pub statistical_exchange_angle_rad: f64,
    /// Many-body topological ground state degeneracy on torus.
    pub ground_state_degeneracy: usize,
    /// Neutral excitation many-body spectral gap in eV.
    pub spectral_gap_ev: f64,
    /// Many-body Chern number $C_{mb} = \nu \cdot \mathcal{C}$.
    pub many_body_chern_number: f64,
}

impl FractionalChernState {
    pub fn new(flat_band: MoireFlatBand, filling: FractionalFilling) -> Self {
        let q = filling.denominator();
        let p = filling.numerator();
        let e_star = ELECTRON_CHARGE_C / (q as f64);
        let theta_exch = std::f64::consts::PI * (p as f64) / (q as f64);
        let degeneracy = q;

        let ec = flat_band.params.characteristic_coulomb_ev();
        let alpha = filling.gap_coefficient();
        let spectral_gap = (alpha * ec - flat_band.bandwidth_ev).max(0.0005); // At least 0.5 meV

        let c_mb = filling.as_f64() * (flat_band.chern_number as f64);

        Self {
            flat_band,
            filling,
            fractional_charge_c: e_star,
            statistical_exchange_angle_rad: theta_exch,
            ground_state_degeneracy: degeneracy,
            spectral_gap_ev: spectral_gap,
            many_body_chern_number: c_mb,
        }
    }

    /// Evaluates thermal quasiparticle excitation error rate $p_{\mathrm{err}} = \exp(-\Delta / k_B T)$:
    pub fn thermal_quasiparticle_error(&self, temperature_k: f64) -> f64 {
        let kb_ev_k = 8.617_333_262e-5; // eV/K
        let kt = kb_ev_k * temperature_k.max(0.01);
        (-self.spectral_gap_ev / kt).exp().clamp(0.0, 1.0)
    }

    /// Evaluates Hall conductance $\sigma_{xy} = C_{mb} \frac{e^2}{h}$ in Siemens:
    pub fn hall_conductance_siemens(&self) -> f64 {
        let h_j_s = 6.626_070_15e-34;
        let conductance_quantum = ELECTRON_CHARGE_C.powi(2) / h_j_s;
        self.many_body_chern_number * conductance_quantum
    }
}
