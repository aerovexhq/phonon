//! Heterojunction band alignment, Anderson electron affinity rule, and polarization 2DEG physics.

use super::bandstructure::Bandstructure;
use phonon_core::constants::ELEMENTARY_CHARGE;

/// Classification of semiconductor heterojunction band alignments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BandAlignmentType {
    /// Type-I (Straddling gap): Both electrons and holes are confined within the narrower bandgap material (e.g. GaAs / AlGaAs, InP / InGaAs).
    TypeIStraddling,
    /// Type-II (Staggered gap): Conduction band minimum and valence band maximum reside in different materials (e.g. Si / SiGe, InAs / GaSb).
    TypeIIStaggered,
    /// Type-III (Broken gap): Conduction band of one material falls below the valence band of the second material.
    TypeIIIBroken,
}

/// Physical parameters and band alignment at an abrupt heterojunction interface.
#[derive(Debug, Clone, PartialEq)]
pub struct HeteroInterface {
    pub name: String,
    /// Conduction band offset $\Delta E_c = E_{c,wide} - E_{c,narrow}$ in $eV$.
    pub delta_ec_ev: f64,
    /// Valence band offset $\Delta E_v = E_{v,narrow} - E_{v,wide}$ in $eV$.
    pub delta_ev_ev: f64,
    /// Alignment classification.
    pub alignment: BandAlignmentType,
    /// Interface polarization sheet charge density $\sigma_{pol}$ in $\text{C/m}^2$.
    /// Positive charge induces a 2DEG (e.g. AlGaN/GaN HEMT).
    pub interface_polarization_c_m2: f64,
}

impl HeteroInterface {
    /// Computes the band alignment between two materials (Material 1 and Material 2)
    /// using Anderson's electron affinity rule:
    /// $$\Delta E_c = \chi_1 - \chi_2$$
    /// $$\Delta E_v = (E_{g,2} - E_{g,1}) - \Delta E_c$$
    pub fn from_anderson_rule(
        name: &str,
        band_1: &Bandstructure,
        band_2: &Bandstructure,
        temp_k: f64,
        polarization_charge_c_m2: f64,
    ) -> Self {
        let chi1 = band_1.electron_affinity_ev(temp_k);
        let chi2 = band_2.electron_affinity_ev(temp_k);
        let eg1 = band_1.bandgap_ev(temp_k);
        let eg2 = band_2.bandgap_ev(temp_k);

        let delta_ec = chi1 - chi2;
        let delta_ev = (eg2 - eg1) - delta_ec;

        let alignment = if delta_ec >= 0.0 && delta_ev >= 0.0 {
            BandAlignmentType::TypeIStraddling
        } else if delta_ec < 0.0 && delta_ev > delta_ec.abs() {
            BandAlignmentType::TypeIIStaggered
        } else {
            BandAlignmentType::TypeIIIBroken
        };

        Self {
            name: name.to_string(),
            delta_ec_ev: delta_ec,
            delta_ev_ev: delta_ev,
            alignment,
            interface_polarization_c_m2: polarization_charge_c_m2,
        }
    }

    /// Computes the sheet carrier density $n_s$ in $\text{m}^{-2}$ of a 2D electron gas (2DEG)
    /// induced by spontaneous + piezoelectric polarization:
    /// $$n_s = \frac{\sigma_{pol}}{q}$$
    pub fn two_dimensional_electron_gas_density(&self) -> f64 {
        if self.interface_polarization_c_m2 <= 0.0 {
            0.0
        } else {
            self.interface_polarization_c_m2 / ELEMENTARY_CHARGE
        }
    }

    // =========================================================================
    // Standard Hetero-Interface Presets
    // =========================================================================

    /// $\text{Al}_{0.3}\text{Ga}_{0.7}\text{As} / \text{GaAs}$ lattice-matched Type-I heterojunction:
    /// $\Delta E_c \approx 0.23 \text{ eV}, \Delta E_v \approx 0.15 \text{ eV}$.
    pub fn algaas_gaas() -> Self {
        Self {
            name: "Al0.3Ga0.7As/GaAs".to_string(),
            delta_ec_ev: 0.23,
            delta_ev_ev: 0.15,
            alignment: BandAlignmentType::TypeIStraddling,
            interface_polarization_c_m2: 0.0,
        }
    }

    /// $\text{Al}_{0.25}\text{Ga}_{0.75}\text{N} / \text{GaN}$ wurtzite HEMT heterojunction:
    /// Strong polarization inducing high-density 2DEG ($n_s \approx 10^{17} \text{ m}^{-2} = 10^{13} \text{ cm}^{-2}$).
    pub fn algan_gan_hemt() -> Self {
        // Polarization charge ~ 1.6e-2 C/m^2 -> ns ~ 1e13 cm^-2
        Self {
            name: "Al0.25Ga0.75N/GaN".to_string(),
            delta_ec_ev: 0.35,
            delta_ev_ev: 0.15,
            alignment: BandAlignmentType::TypeIStraddling,
            interface_polarization_c_m2: 1.6e-2, // ~1e13 cm^-2 2DEG
        }
    }

    /// $\text{Si}_{0.8}\text{Ge}_{0.2} / \text{Si}$ pseudomorphic HBT base heterojunction:
    /// Primarily valence band offset $\Delta E_v \approx 0.15 \text{ eV}$ boosting emitter injection efficiency.
    pub fn sige_si() -> Self {
        Self {
            name: "Si0.8Ge0.2/Si".to_string(),
            delta_ec_ev: 0.02,
            delta_ev_ev: 0.15,
            alignment: BandAlignmentType::TypeIStraddling,
            interface_polarization_c_m2: 0.0,
        }
    }
}
