//! High-Harmonic Generation (HHG) & Carrier Recollision Dynamics.
//!
//! Formulates non-perturbative solid-state high-harmonic generation, semiclassical
//! recollision cutoff energies, interband dipole matrix elements, and Semiconductor
//! Bloch Equations (SBE).

use super::floquet_lattice::{FloquetGrapheneLattice, FloquetLaserPulse};
use phonon_core::constants::{ELEMENTARY_CHARGE, H_BAR};

/// Electron rest mass $m_e$ in kg.
pub const ELECTRON_MASS_KG: f64 = 9.109_383_701_5e-31;

/// Semiclassical recollision cutoff model in solid-state systems.
#[derive(Debug, Clone, PartialEq)]
pub struct HhgCutoffModel {
    /// Effective carrier mass ratio $m^* / m_e$ (default 0.05 for Dirac carriers).
    pub effective_mass_ratio: f64,
}

impl Default for HhgCutoffModel {
    fn default() -> Self {
        Self {
            effective_mass_ratio: 0.05,
        }
    }
}

impl HhgCutoffModel {
    /// Effective carrier mass $m^*$ in kg.
    pub fn effective_mass_kg(&self) -> f64 {
        self.effective_mass_ratio * ELECTRON_MASS_KG
    }

    /// Ponderomotive energy $U_p = \frac{e^2 E_0^2}{4 m^* \Omega^2}$ in Joules.
    pub fn ponderomotive_energy_joules(&self, pulse: &FloquetLaserPulse) -> f64 {
        let e = ELEMENTARY_CHARGE;
        let e0 = pulse.peak_electric_field_v_per_m;
        let m_eff = self.effective_mass_kg();
        let omega = pulse.angular_frequency_rad();

        (e * e * e0 * e0) / (4.0 * m_eff * omega * omega)
    }

    /// Ponderomotive energy $U_p$ in eV.
    pub fn ponderomotive_energy_ev(&self, pulse: &FloquetLaserPulse) -> f64 {
        self.ponderomotive_energy_joules(pulse) / ELEMENTARY_CHARGE
    }

    /// Three-step semiclassical cutoff photon energy in eV:
    /// $$E_{cutoff} = E_g + 3.17 U_p$$
    pub fn cutoff_energy_ev(&self, bandgap_ev: f64, pulse: &FloquetLaserPulse) -> f64 {
        bandgap_ev + 3.17 * self.ponderomotive_energy_ev(pulse)
    }

    /// High-harmonic cutoff order $N_{cutoff} = \lfloor E_{cutoff} / (\hbar\Omega) \rfloor$.
    pub fn cutoff_harmonic_order(&self, bandgap_ev: f64, pulse: &FloquetLaserPulse) -> usize {
        let e_cut = self.cutoff_energy_ev(bandgap_ev, pulse);
        let hbar_omega = pulse.photon_energy_ev();
        if hbar_omega > 0.0 {
            (e_cut / hbar_omega).floor().max(1.0) as usize
        } else {
            1
        }
    }
}

/// Dynamic transition dipole matrix element $\mathbf{d}_{cv}(\mathbf{k})$ between valence and conduction bands.
#[derive(Debug, Clone, PartialEq)]
pub struct DipoleMatrixElement;

impl DipoleMatrixElement {
    /// Computes the interband transition dipole magnitude $\|\mathbf{d}_{cv}(\mathbf{k})\|$ in meters:
    /// $$d_{cv}(k) = \frac{e v_F}{2 \omega_{cv}(k)} = \frac{e v_F \hbar}{4 \epsilon_c(k)}$$
    pub fn magnitude_m(lattice: &FloquetGrapheneLattice, energy_ev: f64) -> f64 {
        let e_joules = energy_ev.max(0.01) * ELEMENTARY_CHARGE;
        let vf = lattice.fermi_velocity_m_per_s();
        // d_cv = hbar * v_F / (4 * energy)
        (H_BAR * vf) / (4.0 * e_joules)
    }

    /// Transition dipole vector components $(d_x, d_y)$ in meters at wavevector $(k_x, k_y)$.
    pub fn components_m(
        lattice: &FloquetGrapheneLattice,
        kx: f64,
        ky: f64,
        energy_ev: f64,
    ) -> (f64, f64) {
        let d_mag = Self::magnitude_m(lattice, energy_ev);
        let k_norm = (kx * kx + ky * ky).sqrt().max(1e-12);
        // Orthogonal to wavevector: (-ky/k, kx/k)
        (-d_mag * (ky / k_norm), d_mag * (kx / k_norm))
    }
}

/// Parameters for Semiconductor Bloch Equations (SBE) carrier dynamics.
#[derive(Debug, Clone, PartialEq)]
pub struct SemiconductorBlochParams {
    /// Interband polarization dephasing time $T_2$ in seconds (default 10 fs).
    pub dephasing_time_seconds: f64,
    /// Intraband carrier population relaxation time $T_1$ in seconds (default 100 fs).
    pub relaxation_time_seconds: f64,
}

impl Default for SemiconductorBlochParams {
    fn default() -> Self {
        Self {
            dephasing_time_seconds: 10.0e-15,
            relaxation_time_seconds: 100.0e-15,
        }
    }
}

impl SemiconductorBlochParams {
    /// Dephasing damping rate $\Gamma_2 = 1 / T_2$ in $\text{s}^{-1}$.
    pub fn dephasing_rate_s(&self) -> f64 {
        1.0 / self.dephasing_time_seconds
    }

    /// Relaxation damping rate $\Gamma_1 = 1 / T_1$ in $\text{s}^{-1}$.
    pub fn relaxation_rate_s(&self) -> f64 {
        1.0 / self.relaxation_time_seconds
    }
}
