//! Complete first-principles semiconductor materials and high-level presets.

use super::bandstructure::Bandstructure;
use super::contact::ContactMaterial;
use super::crystal::Crystallography;
use super::dielectric::DielectricMaterial;
use super::dopant::{DopantSpecies, DopantType};
use super::mobility::ChemicalMobility;
use phonon_core::constants::{EPSILON_0, T_REF};

/// Complete first-principles physical and chemical definition of a semiconductor material.
#[derive(Debug, Clone, PartialEq)]
pub struct ChemicalMaterial {
    pub name: String,
    pub crystallography: Crystallography,
    pub bandstructure: Bandstructure,
    pub mobility: ChemicalMobility,
    /// Relative static dielectric permittivity $\epsilon_r$.
    pub relative_permittivity: f64,
    /// Thermal conductivity at 300K in $\text{W} / (\text{m} \cdot \text{K})$.
    pub thermal_conductivity_300k: f64,
    /// Thermal conductivity temperature degradation exponent $\beta$ ($k(T) \propto T^{-\beta}$).
    pub thermal_conductivity_exponent: f64,
    /// Intentionally introduced chemical dopant and concentration in $\text{m}^{-3}$.
    pub dopant: Option<(DopantSpecies, f64)>,
    /// Gate/interfacial dielectric layer and physical thickness in meters.
    pub dielectric: Option<(DielectricMaterial, f64)>,
    /// Electrode contact / silicide interface.
    pub contact: Option<ContactMaterial>,
}

impl ChemicalMaterial {
    /// Computes the permittivity $\epsilon = \epsilon_r \epsilon_0$ in $F/m$.
    pub fn permittivity(&self) -> f64 {
        self.relative_permittivity * EPSILON_0
    }

    /// Computes the temperature-dependent thermal conductivity $k_{th}(T)$ in $\text{W} / (\text{m} \cdot \text{K})$:
    /// $$k_{th}(T) = k_{th, 300} \cdot \left(\frac{T}{300}\right)^{-\beta_{th}}$$
    pub fn thermal_conductivity(&self, temp_k: f64) -> f64 {
        let t = temp_k.max(10.0);
        self.thermal_conductivity_300k * (t / T_REF).powf(-self.thermal_conductivity_exponent)
    }

    /// Computes the electrically active ionized donor and acceptor concentrations $(N_D^+, N_A^-)$ in $\text{m}^{-3}$
    /// accounting for solid solubility limits and incomplete ionization freeze-out at temperature $T$:
    pub fn ionized_doping(&self, temp_k: f64) -> (f64, f64) {
        match &self.dopant {
            None => (0.0, 0.0),
            Some((species, total_conc)) => {
                let n_ion = species.ionized_concentration(&self.bandstructure, *total_conc, temp_k);
                match species.dopant_type() {
                    DopantType::Donor => (n_ion, 0.0),
                    DopantType::Acceptor => (0.0, n_ion),
                }
            }
        }
    }

    /// Returns the total active chemical doping density $N_{total} = N_D + N_A$ in $\text{m}^{-3}$.
    pub fn total_doping(&self) -> f64 {
        self.dopant.as_ref().map(|(_, n)| *n).unwrap_or(0.0)
    }

    /// Computes the effective intrinsic carrier density $n_{ie}(T)$ in $\text{m}^{-3}$
    /// accounting for heavy-doping bandgap narrowing.
    pub fn effective_intrinsic_carrier_density(&self, temp_k: f64) -> f64 {
        let total_n = self.total_doping();
        self.bandstructure
            .effective_intrinsic_carrier_density(temp_k, total_n)
    }

    /// Computes low-field carrier mobilities $(\mu_n, \mu_p)$ in $\text{m}^2 / (\text{V} \cdot \text{s})$
    /// accounting for ionized impurity scattering and temperature degradation.
    pub fn low_field_mobilities(&self, temp_k: f64) -> (f64, f64) {
        let total_n = self.total_doping();
        let mu_n = self.mobility.electron.low_field_mobility(total_n, temp_k);
        let mu_p = self.mobility.hole.low_field_mobility(total_n, temp_k);
        (mu_n, mu_p)
    }

    /// Computes carrier diffusion coefficients $(D_n, D_p)$ in $\text{m}^2/s$ via Einstein relation.
    pub fn diffusion_coefficients(&self, temp_k: f64) -> (f64, f64) {
        let total_n = self.total_doping();
        let dn = self
            .mobility
            .electron
            .diffusion_coefficient(total_n, temp_k);
        let dp = self.mobility.hole.diffusion_coefficient(total_n, temp_k);
        (dn, dp)
    }

    // =========================================================================
    // High-Level Standard Material Presets
    // =========================================================================

    /// Intrinsic pure Silicon ($Si$).
    pub fn silicon() -> Self {
        Self {
            name: "Silicon (Si)".to_string(),
            crystallography: Crystallography::silicon(),
            bandstructure: Bandstructure::silicon(),
            mobility: ChemicalMobility::silicon(),
            relative_permittivity: 11.7,
            thermal_conductivity_300k: 148.0,
            thermal_conductivity_exponent: 1.33,
            dopant: None,
            dielectric: Some((DielectricMaterial::silicon_dioxide(), 2.0e-9)),
            contact: Some(ContactMaterial::nickel_silicide()),
        }
    }

    /// Intrinsic pure Germanium ($Ge$).
    pub fn germanium() -> Self {
        Self {
            name: "Germanium (Ge)".to_string(),
            crystallography: Crystallography::germanium(),
            bandstructure: Bandstructure::germanium(),
            mobility: ChemicalMobility::germanium(),
            relative_permittivity: 16.2,
            thermal_conductivity_300k: 60.0,
            thermal_conductivity_exponent: 1.25,
            dopant: None,
            dielectric: Some((DielectricMaterial::hafnium_dioxide(), 2.0e-9)),
            contact: Some(ContactMaterial::nickel_silicide()),
        }
    }

    /// Intrinsic pure Gallium Arsenide ($GaAs$).
    pub fn gallium_arsenide() -> Self {
        Self {
            name: "Gallium Arsenide (GaAs)".to_string(),
            crystallography: Crystallography::gallium_arsenide(),
            bandstructure: Bandstructure::gallium_arsenide(),
            mobility: ChemicalMobility::gallium_arsenide(),
            relative_permittivity: 12.9,
            thermal_conductivity_300k: 46.0,
            thermal_conductivity_exponent: 1.25,
            dopant: None,
            dielectric: Some((DielectricMaterial::aluminum_oxide(), 5.0e-9)),
            contact: Some(ContactMaterial::gold()),
        }
    }

    /// Intrinsic pure Gallium Nitride ($GaN$).
    pub fn gallium_nitride() -> Self {
        Self {
            name: "Gallium Nitride (GaN)".to_string(),
            crystallography: Crystallography::gallium_nitride(),
            bandstructure: Bandstructure::gallium_nitride(),
            mobility: ChemicalMobility::gallium_nitride(),
            relative_permittivity: 8.9,
            thermal_conductivity_300k: 130.0,
            thermal_conductivity_exponent: 1.4,
            dopant: None,
            dielectric: Some((DielectricMaterial::silicon_nitride(), 10.0e-9)),
            contact: Some(ContactMaterial::titanium_nitride_gate()),
        }
    }

    /// Intrinsic pure Silicon Carbide ($4H-SiC$).
    pub fn silicon_carbide_4h() -> Self {
        Self {
            name: "Silicon Carbide (4H-SiC)".to_string(),
            crystallography: Crystallography::silicon_carbide_4h(),
            bandstructure: Bandstructure::silicon_carbide_4h(),
            mobility: ChemicalMobility::silicon_carbide_4h(),
            relative_permittivity: 9.7,
            thermal_conductivity_300k: 370.0,
            thermal_conductivity_exponent: 1.49,
            dopant: None,
            dielectric: Some((DielectricMaterial::silicon_dioxide(), 5.0e-9)),
            contact: Some(ContactMaterial::nickel_silicide()),
        }
    }

    /// Intrinsic pure Indium Phosphide ($InP$).
    pub fn indium_phosphide() -> Self {
        Self {
            name: "Indium Phosphide (InP)".to_string(),
            crystallography: Crystallography::indium_phosphide(),
            bandstructure: Bandstructure::indium_phosphide(),
            mobility: ChemicalMobility::gallium_arsenide(), // close analog
            relative_permittivity: 12.5,
            thermal_conductivity_300k: 68.0,
            thermal_conductivity_exponent: 1.4,
            dopant: None,
            dielectric: Some((DielectricMaterial::aluminum_oxide(), 5.0e-9)),
            contact: Some(ContactMaterial::gold()),
        }
    }
}

// =========================================================================
// Ergonomic High-Level Presets: Silicon, Germanium, GaAs, GaN
// =========================================================================

/// Ergonomic high-level factory for Silicon semiconductor regions.
pub struct Silicon;

impl Silicon {
    /// P-type Silicon doped with Boron, Gallium, or Indium.
    pub fn p_type(species: DopantSpecies, concentration_m3: f64) -> ChemicalMaterial {
        let mut mat = ChemicalMaterial::silicon();
        mat.dopant = Some((species, concentration_m3));
        mat
    }

    /// N-type Silicon doped with Phosphorus, Arsenic, or Antimony.
    pub fn n_type(species: DopantSpecies, concentration_m3: f64) -> ChemicalMaterial {
        let mut mat = ChemicalMaterial::silicon();
        mat.dopant = Some((species, concentration_m3));
        mat
    }

    /// Standard Boron-doped p-substrate ($N_A = 10^{17} \text{ cm}^{-3}$).
    pub fn p_substrate_standard() -> ChemicalMaterial {
        Self::p_type(DopantSpecies::Boron, 1.0e23)
    }

    /// Standard Phosphorus-doped n-well ($N_D = 10^{17} \text{ cm}^{-3}$).
    pub fn n_well_standard() -> ChemicalMaterial {
        Self::n_type(DopantSpecies::Phosphorus, 1.0e23)
    }

    /// Heavy Arsenic-doped $N^+$ Source/Drain diffusion ($N_D = 10^{20} \text{ cm}^{-3}$).
    pub fn n_plus_diffusion() -> ChemicalMaterial {
        Self::n_type(DopantSpecies::Arsenic, 1.0e26)
    }

    /// Heavy Boron-doped $P^+$ Source/Drain diffusion ($N_A = 10^{20} \text{ cm}^{-3}$).
    pub fn p_plus_diffusion() -> ChemicalMaterial {
        Self::p_type(DopantSpecies::Boron, 1.0e26)
    }
}
