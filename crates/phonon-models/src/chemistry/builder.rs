//! Fluent builder for programmatic low-level and high-level chemical semiconductor synthesis.

use super::bandstructure::Bandstructure;
use super::contact::ContactMaterial;
use super::crystal::Crystallography;
use super::dielectric::DielectricMaterial;
use super::dopant::DopantSpecies;
use super::mobility::ChemicalMobility;
use super::presets::ChemicalMaterial;

/// Fluent builder for constructing customized physical chemical materials from scratch.
#[derive(Debug, Clone)]
pub struct ChemicalMaterialBuilder {
    name: String,
    crystallography: Option<Crystallography>,
    bandstructure: Option<Bandstructure>,
    mobility: Option<ChemicalMobility>,
    relative_permittivity: f64,
    thermal_conductivity_300k: f64,
    thermal_conductivity_exponent: f64,
    dopant: Option<(DopantSpecies, f64)>,
    dielectric: Option<(DielectricMaterial, f64)>,
    contact: Option<ContactMaterial>,
}

impl ChemicalMaterialBuilder {
    /// Creates a new builder initialized with default silicon properties.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            crystallography: None,
            bandstructure: None,
            mobility: None,
            relative_permittivity: 11.7,
            thermal_conductivity_300k: 148.0,
            thermal_conductivity_exponent: 1.33,
            dopant: None,
            dielectric: None,
            contact: None,
        }
    }

    /// Configures the crystal structure and lattice parameters.
    pub fn crystallography(mut self, crystal: Crystallography) -> Self {
        self.crystallography = Some(crystal);
        self
    }

    /// Configures the multi-valley electronic bandstructure.
    pub fn bandstructure(mut self, band: Bandstructure) -> Self {
        self.bandstructure = Some(band);
        self
    }

    /// Configures carrier mobilities and velocity saturation parameters.
    pub fn mobility(mut self, mob: ChemicalMobility) -> Self {
        self.mobility = Some(mob);
        self
    }

    /// Sets the static relative dielectric permittivity $\epsilon_r$.
    pub fn relative_permittivity(mut self, eps_r: f64) -> Self {
        self.relative_permittivity = eps_r;
        self
    }

    /// Sets thermal conductivity at 300K ($W/(m\cdot K)$) and temperature exponent $\beta$.
    pub fn thermal_conductivity(mut self, k_300k: f64, exponent: f64) -> Self {
        self.thermal_conductivity_300k = k_300k;
        self.thermal_conductivity_exponent = exponent;
        self
    }

    /// Configures chemical dopant species and concentration in $\text{m}^{-3}$.
    pub fn doping(mut self, species: DopantSpecies, concentration_m3: f64) -> Self {
        self.dopant = Some((species, concentration_m3));
        self
    }

    /// Configures an interfacial gate dielectric layer and its physical thickness in meters.
    pub fn dielectric(mut self, dielectric: DielectricMaterial, thickness_m: f64) -> Self {
        self.dielectric = Some((dielectric, thickness_m));
        self
    }

    /// Configures the electrode contact / silicide interface.
    pub fn contact(mut self, contact: ContactMaterial) -> Self {
        self.contact = Some(contact);
        self
    }

    /// Finalizes and constructs the strongly typed `ChemicalMaterial`.
    pub fn build(self) -> ChemicalMaterial {
        ChemicalMaterial {
            name: self.name,
            crystallography: self
                .crystallography
                .unwrap_or_else(Crystallography::silicon),
            bandstructure: self.bandstructure.unwrap_or_else(Bandstructure::silicon),
            mobility: self.mobility.unwrap_or_else(ChemicalMobility::silicon),
            relative_permittivity: self.relative_permittivity,
            thermal_conductivity_300k: self.thermal_conductivity_300k,
            thermal_conductivity_exponent: self.thermal_conductivity_exponent,
            dopant: self.dopant,
            dielectric: self.dielectric,
            contact: self.contact,
        }
    }
}
