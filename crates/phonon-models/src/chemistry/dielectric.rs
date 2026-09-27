//! Dielectric chemistry, relative permittivity, band offsets, breakdown fields, and interface traps.

use phonon_core::constants::{EPSILON_0, EPSILON_R_OX};

/// Chemical classification of dielectric materials.
#[derive(Debug, Clone, PartialEq)]
pub enum DielectricSpecies {
    /// Silicon Dioxide ($SiO_2$): Standard native/thermal gate and field oxide ($\kappa = 3.9$).
    SiliconDioxide,
    /// Hafnium Dioxide ($HfO_2$): High-$\kappa$ gate dielectric ($\kappa = 22.0$).
    HafniumDioxide,
    /// Aluminum Oxide ($Al_2O_3$): High-$\kappa$ dielectric and surface passivation ($\kappa = 9.0$).
    AluminumOxide,
    /// Silicon Nitride ($Si_3N_4$): Diffusion barrier, spacer, and passivation ($\kappa = 7.5$).
    SiliconNitride,
    /// Zirconium Dioxide ($ZrO_2$): High-$\kappa$ dielectric ($\kappa = 25.0$).
    ZirconiumDioxide,
    /// Titanium Dioxide ($TiO_2$): Ultra-high-$\kappa$ dielectric ($\kappa = 80.0$).
    TitaniumDioxide,
    /// Low-$\kappa$ porous organosilicate glass (SiCOH, $\kappa = 2.4$).
    LowKInterconnect,
    /// Custom user-defined dielectric material.
    Custom {
        name: String,
        permittivity: f64,
        bandgap_ev: f64,
        breakdown_field_v_m: f64,
        conduction_offset_si_ev: f64,
    },
}

/// Chemical and physical properties of a dielectric insulator layer.
#[derive(Debug, Clone, PartialEq)]
pub struct DielectricMaterial {
    pub species: DielectricSpecies,
    /// Relative static dielectric permittivity $\kappa_{static}$.
    pub relative_permittivity: f64,
    /// Optical dielectric permittivity $\kappa_\infty = n^2$.
    pub optical_permittivity: f64,
    /// Dielectric energy bandgap in electron-volts ($eV$).
    pub bandgap_ev: f64,
    /// Critical dielectric breakdown electric field in Volts per meter ($V/m$).
    pub breakdown_field_v_m: f64,
    /// Conduction band offset $\Delta E_c$ relative to Silicon in $eV$.
    pub conduction_offset_si_ev: f64,
    /// Valence band offset $\Delta E_v$ relative to Silicon in $eV$.
    pub valence_offset_si_ev: f64,
    /// Interface trap state density $D_{it}$ at semiconductor interface in $\text{m}^{-2} \text{eV}^{-1}$.
    pub interface_trap_density_m2_ev: f64,
    /// Fixed oxide charge area density $Q_f$ in Coulombs per square meter ($C/m^2$).
    pub fixed_charge_c_m2: f64,
}

impl DielectricMaterial {
    /// Computes the Equivalent Oxide Thickness (EOT) in meters for a given physical thickness $t_{phys}$:
    /// $$\text{EOT} = t_{phys} \cdot \left(\frac{\kappa_{SiO_2}}{\kappa_{dielectric}}\right)$$
    pub fn equivalent_oxide_thickness(&self, physical_thickness_m: f64) -> f64 {
        physical_thickness_m * (EPSILON_R_OX / self.relative_permittivity.max(1.0))
    }

    /// Computes the gate oxide areal capacitance $C_{ox}$ in Farads per square meter ($F/m^2$):
    /// $$C_{ox} = \frac{\epsilon_0 \kappa}{t_{phys}} = \frac{\epsilon_0 \kappa_{SiO_2}}{\text{EOT}}$$
    pub fn areal_capacitance(&self, physical_thickness_m: f64) -> f64 {
        let eps = EPSILON_0 * self.relative_permittivity;
        eps / physical_thickness_m.max(1e-12)
    }

    /// Computes the flatband voltage shift caused by fixed oxide charge and interface traps:
    /// $$\Delta V_{fb} = -\frac{Q_f}{C_{ox}}$$
    pub fn flatband_shift_v(&self, physical_thickness_m: f64) -> f64 {
        let c_ox = self.areal_capacitance(physical_thickness_m);
        -self.fixed_charge_c_m2 / c_ox.max(1e-12)
    }

    // =========================================================================
    // Standard Dielectric Presets
    // =========================================================================

    /// Silicon Dioxide ($SiO_2$): $\kappa = 3.9, E_g = 9.0 \text{ eV}, E_{bd} = 10^9 \text{ V/m}$.
    pub fn silicon_dioxide() -> Self {
        Self {
            species: DielectricSpecies::SiliconDioxide,
            relative_permittivity: 3.9,
            optical_permittivity: 2.13, // n ~ 1.46
            bandgap_ev: 9.0,
            breakdown_field_v_m: 1.0e9, // 10 MV/cm
            conduction_offset_si_ev: 3.15,
            valence_offset_si_ev: 4.73,
            interface_trap_density_m2_ev: 1.0e14, // 1e10 cm^-2 eV^-1
            fixed_charge_c_m2: 1.6e-4,            // ~1e11 charges/cm^2
        }
    }

    /// Hafnium Dioxide ($HfO_2$): $\kappa = 22.0, E_g = 5.7 \text{ eV}, E_{bd} = 5 \times 10^8 \text{ V/m}$.
    pub fn hafnium_dioxide() -> Self {
        Self {
            species: DielectricSpecies::HafniumDioxide,
            relative_permittivity: 22.0,
            optical_permittivity: 4.41, // n ~ 2.1
            bandgap_ev: 5.7,
            breakdown_field_v_m: 5.0e8, // 5 MV/cm
            conduction_offset_si_ev: 1.5,
            valence_offset_si_ev: 3.08,
            interface_trap_density_m2_ev: 5.0e14,
            fixed_charge_c_m2: 3.2e-4,
        }
    }

    /// Aluminum Oxide ($Al_2O_3$): $\kappa = 9.0, E_g = 8.8 \text{ eV}, E_{bd} = 8 \times 10^8 \text{ V/m}$.
    pub fn aluminum_oxide() -> Self {
        Self {
            species: DielectricSpecies::AluminumOxide,
            relative_permittivity: 9.0,
            optical_permittivity: 3.10, // n ~ 1.76
            bandgap_ev: 8.8,
            breakdown_field_v_m: 8.0e8,
            conduction_offset_si_ev: 2.8,
            valence_offset_si_ev: 4.88,
            interface_trap_density_m2_ev: 2.0e14,
            fixed_charge_c_m2: -8.0e-4, // negative fixed charge typical for Al2O3
        }
    }

    /// Silicon Nitride ($Si_3N_4$): $\kappa = 7.5, E_g = 5.3 \text{ eV}, E_{bd} = 10^9 \text{ V/m}$.
    pub fn silicon_nitride() -> Self {
        Self {
            species: DielectricSpecies::SiliconNitride,
            relative_permittivity: 7.5,
            optical_permittivity: 4.0, // n ~ 2.0
            bandgap_ev: 5.3,
            breakdown_field_v_m: 1.0e9,
            conduction_offset_si_ev: 2.0,
            valence_offset_si_ev: 2.18,
            interface_trap_density_m2_ev: 1.0e15,
            fixed_charge_c_m2: 8.0e-4,
        }
    }
}
