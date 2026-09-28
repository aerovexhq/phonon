//! Multi-Physics Physical Material Model & Cross-Domain Constants
//!
//! Formulates unified physical material representations covering:
//! 1. Electromagnetic / RF: complex permittivity, loss tangent, conductivity, permeability, skin depth.
//! 2. Optical: complex refractive index (n + jk), bandgap, BRDF diffuse/specular albedo and roughness.
//! 3. Acoustic: mass density, longitudinal/shear sound speeds, characteristic acoustic impedance, absorption.
//! 4. Thermal & Mechanical: thermal conductivity, heat capacity, Young's modulus, Poisson ratio, yield strength.

use phonon_core::constants::{EPSILON_0, MU_0};

/// Categorical classification for multi-physics materials.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaterialCategory {
    Conductor,
    Semiconductor,
    Dielectric,
    Polymer,
    StructuralAerospace,
    GeologicalCivil,
    AtmosphericBiological,
    Superconductor,
}

/// Electromagnetic and RF material properties.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElectromagneticProperties {
    /// Relative dielectric permittivity (real part epsilon_r' >= 1.0).
    pub relative_permittivity: f64,
    /// Dielectric loss tangent (tan delta = epsilon_r'' / epsilon_r').
    pub loss_tangent: f64,
    /// Relative magnetic permeability (mu_r >= 1.0 for non-magnetic).
    pub relative_permeability: f64,
    /// Bulk electrical DC conductivity in Siemens per meter (S/m).
    pub conductivity_s_per_m: f64,
}

impl ElectromagneticProperties {
    pub const fn new(
        relative_permittivity: f64,
        loss_tangent: f64,
        relative_permeability: f64,
        conductivity_s_per_m: f64,
    ) -> Self {
        Self {
            relative_permittivity,
            loss_tangent,
            relative_permeability,
            conductivity_s_per_m,
        }
    }

    /// Evaluates complex relative permittivity (eps_r' - j eps_r_eff'') at frequency f in Hz.
    pub fn complex_permittivity_at_freq(&self, freq_hz: f64) -> (f64, f64) {
        let f = freq_hz.max(1.0);
        let omega = std::f64::consts::TAU * f;
        let eps_r_real = self.relative_permittivity;
        let eps_r_diel_loss = eps_r_real * self.loss_tangent;
        let eps_r_cond_loss = self.conductivity_s_per_m / (omega * EPSILON_0);
        (eps_r_real, eps_r_diel_loss + eps_r_cond_loss)
    }

    /// Evaluates electromagnetic skin depth delta_s in meters at frequency f in Hz.
    pub fn skin_depth_m(&self, freq_hz: f64) -> f64 {
        if self.conductivity_s_per_m <= 1e-6 {
            return f64::INFINITY;
        }
        let f = freq_hz.max(1.0);
        let omega = std::f64::consts::TAU * f;
        let mu = self.relative_permeability * MU_0;
        (2.0 / (omega * mu * self.conductivity_s_per_m)).sqrt()
    }

    /// Power reflection coefficient R for normal incidence from free space (Z_0 = 376.73 Ohms).
    pub fn normal_power_reflection(&self, freq_hz: f64) -> f64 {
        let (eps_real, eps_imag) = self.complex_permittivity_at_freq(freq_hz);
        let mod_eps = (eps_real * eps_real + eps_imag * eps_imag).sqrt();
        let n_eff = ((mod_eps + eps_real) / 2.0).sqrt();
        let k_eff = ((mod_eps - eps_real) / 2.0).sqrt();
        let num = (n_eff - 1.0).powi(2) + k_eff * k_eff;
        let den = (n_eff + 1.0).powi(2) + k_eff * k_eff;
        (num / den.max(1e-12)).clamp(0.0, 1.0)
    }
}

/// Optical material and BRDF surface properties.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OpticalProperties {
    /// Real refractive index n (at 550 nm or central optical wavelength).
    pub refractive_index: f64,
    /// Extinction coefficient k.
    pub extinction_coefficient: f64,
    /// Optical electronic bandgap in electron-volts (eV).
    pub bandgap_ev: f64,
    /// Diffuse surface reflectance albedo (0.0 to 1.0).
    pub diffuse_albedo: f64,
    /// Specular surface reflectance albedo (0.0 to 1.0).
    pub specular_albedo: f64,
    /// Microfacet surface roughness (0.0 smooth to 1.0 fully diffuse).
    pub roughness: f64,
}

impl OpticalProperties {
    pub const fn new(
        refractive_index: f64,
        extinction_coefficient: f64,
        bandgap_ev: f64,
        diffuse_albedo: f64,
        specular_albedo: f64,
        roughness: f64,
    ) -> Self {
        Self {
            refractive_index,
            extinction_coefficient,
            bandgap_ev,
            diffuse_albedo,
            specular_albedo,
            roughness,
        }
    }

    /// Normal incidence optical power reflectance (Fresnel): R = ((n-1)^2 + k^2) / ((n+1)^2 + k^2).
    pub fn normal_reflectance(&self) -> f64 {
        let n = self.refractive_index;
        let k = self.extinction_coefficient;
        let num = (n - 1.0).powi(2) + k * k;
        let den = (n + 1.0).powi(2) + k * k;
        (num / den.max(1e-12)).clamp(0.0, 1.0)
    }
}

/// Acoustic wave propagation properties.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcousticProperties {
    /// Volumetric mass density in kg/m^3.
    pub density_kg_m3: f64,
    /// Longitudinal sound speed c_L in m/s.
    pub sound_speed_longitudinal_m_s: f64,
    /// Transverse / shear sound speed c_S in m/s (0.0 for fluids/gases).
    pub sound_speed_shear_m_s: f64,
    /// Characteristic acoustic impedance Z_0 = rho * c_L in Rayls (Pa*s/m).
    pub acoustic_impedance_rayls: f64,
    /// Acoustic attenuation / absorption coefficient in dB/m at 1 kHz.
    pub absorption_db_per_m: f64,
}

impl AcousticProperties {
    pub fn new(
        density_kg_m3: f64,
        sound_speed_longitudinal_m_s: f64,
        sound_speed_shear_m_s: f64,
        absorption_db_per_m: f64,
    ) -> Self {
        let acoustic_impedance_rayls = density_kg_m3 * sound_speed_longitudinal_m_s;
        Self {
            density_kg_m3,
            sound_speed_longitudinal_m_s,
            sound_speed_shear_m_s,
            acoustic_impedance_rayls,
            absorption_db_per_m,
        }
    }

    /// Evaluates normal acoustic pressure reflection coefficient R between this medium and medium 2:
    /// R = (Z_2 - Z_1) / (Z_2 + Z_1)
    pub fn reflection_coefficient(&self, other: &Self) -> f64 {
        let z1 = self.acoustic_impedance_rayls;
        let z2 = other.acoustic_impedance_rayls;
        (z2 - z1) / (z2 + z1).max(1e-6)
    }

    /// Acoustic power transmission coefficient T_p = 1 - R^2.
    pub fn power_transmission_coefficient(&self, other: &Self) -> f64 {
        let r = self.reflection_coefficient(other);
        (1.0 - r * r).clamp(0.0, 1.0)
    }
}

/// Thermal and mechanical structural properties.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThermalMechanicalProperties {
    /// Thermal conductivity in W/(m*K).
    pub thermal_conductivity_w_m_k: f64,
    /// Specific heat capacity in J/(kg*K).
    pub specific_heat_j_kg_k: f64,
    /// Young's modulus of elasticity in GPa.
    pub youngs_modulus_gpa: f64,
    /// Poisson's ratio (dimensionless, ~0.1 - 0.49).
    pub poissons_ratio: f64,
    /// Yield strength in MPa.
    pub yield_strength_mpa: f64,
}

impl ThermalMechanicalProperties {
    pub const fn new(
        thermal_conductivity_w_m_k: f64,
        specific_heat_j_kg_k: f64,
        youngs_modulus_gpa: f64,
        poissons_ratio: f64,
        yield_strength_mpa: f64,
    ) -> Self {
        Self {
            thermal_conductivity_w_m_k,
            specific_heat_j_kg_k,
            youngs_modulus_gpa,
            poissons_ratio,
            yield_strength_mpa,
        }
    }
}

/// Comprehensive physical material record uniting cross-domain physical properties.
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialRecord {
    /// Unique integer material identifier (0..N).
    pub id: u32,
    /// Canonical human-readable name of the material.
    pub name: &'static str,
    /// Categorical classification.
    pub category: MaterialCategory,
    /// Electromagnetic and RF parameters.
    pub em: ElectromagneticProperties,
    /// Optical and BRDF parameters.
    pub optical: OpticalProperties,
    /// Acoustic wave propagation parameters.
    pub acoustic: AcousticProperties,
    /// Thermal and structural mechanical parameters.
    pub thermal_mech: ThermalMechanicalProperties,
}
