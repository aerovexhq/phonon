//! Magneto-Elastic Coupling & Chiral Phonon-Magnon Polariton Models.
//!
//! Formulates magneto-elastic coupling tensors, acoustic angular momentum conservation,
//! chiral acoustic selection rules, and anti-crossing polariton dispersion.

use std::f64::consts::{FRAC_1_SQRT_2, PI};

/// Electron gyromagnetic ratio in rad/(s·T).
pub const GAMMA_E: f64 = 1.760_859_630_23e11;
/// Vacuum permeability in H/m.
pub const MU_0: f64 = 4.0 * PI * 1.0e-7;
/// Reduced Planck constant in J·s.
pub const HBAR: f64 = 1.054_571_817e-34;

/// Polarization chirality of the transverse acoustic shear wave.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcousticPolarizationChirality {
    /// Right-handed circular polarization (co-rotating with electron spin precession).
    RightHandedCircular,
    /// Left-handed circular polarization (counter-rotating relative to spin precession).
    LeftHandedCircular,
    /// Linearly polarized shear wave (superposition of equal RH and LH components).
    Linear,
}

/// Magneto-elastic and acoustic properties of the ferromagnetic crystal.
#[derive(Debug, Clone, PartialEq)]
pub struct MagnetoElasticMedium {
    /// Mass density $\rho$ in kg/m$^3$ (default 5170 kg/m^3 for YIG).
    pub mass_density_kg_per_m3: f64,
    /// Transverse shear elastic stiffness $C_{44}$ in Pascals (default 7.64e10 Pa).
    pub shear_stiffness_c44_pa: f64,
    /// Saturation magnetization $M_s$ in A/m (default 1.4e5 A/m).
    pub saturation_magnetization_a_per_m: f64,
    /// Magneto-elastic shear coupling constant $B_2$ in J/m$^3$ (default 6.96e5 J/m^3).
    pub magneto_elastic_constant_b2_j_per_m3: f64,
    /// Static bias magnetic field $B_0$ in Tesla (default 0.10 T).
    pub bias_field_tesla: f64,
    /// Intrinsic Gilbert magnetic damping $\alpha$ (default 3.0e-5).
    pub gilbert_damping: f64,
}

impl Default for MagnetoElasticMedium {
    fn default() -> Self {
        Self {
            mass_density_kg_per_m3: 5170.0,
            shear_stiffness_c44_pa: 7.64e10,
            saturation_magnetization_a_per_m: 1.4e5,
            magneto_elastic_constant_b2_j_per_m3: 6.96e5,
            bias_field_tesla: 0.10,
            gilbert_damping: 3.0e-5,
        }
    }
}

impl MagnetoElasticMedium {
    /// Transverse shear acoustic phase velocity $v_t = \sqrt{C_{44} / \rho}$ in m/s.
    #[inline]
    pub fn transverse_sound_velocity_m_per_s(&self) -> f64 {
        (self.shear_stiffness_c44_pa / self.mass_density_kg_per_m3).sqrt()
    }

    /// Kittel magnon resonance angular frequency $\omega_m = \gamma B_0$ in rad/s.
    #[inline]
    pub fn kittel_magnon_frequency_rad_per_s(&self) -> f64 {
        GAMMA_E * self.bias_field_tesla
    }

    /// Wavevector $k_{res} = \omega_m / v_t$ in m$^{-1}$ where acoustic and magnon branches cross.
    pub fn resonance_wavevector_per_m(&self) -> f64 {
        let wm = self.kittel_magnon_frequency_rad_per_s();
        let vt = self.transverse_sound_velocity_m_per_s();
        wm / vt
    }

    /// Magneto-elastic anti-crossing splitting $\Delta\omega_{polariton} = \sqrt{\frac{2 \gamma B_2^2 k^2}{\omega_m \rho M_s}}$ in rad/s.
    pub fn polariton_splitting_rad_per_s(&self, k: f64) -> f64 {
        let b2 = self.magneto_elastic_constant_b2_j_per_m3;
        let rho = self.mass_density_kg_per_m3;
        let ms = self.saturation_magnetization_a_per_m;
        let wm = self.kittel_magnon_frequency_rad_per_s();
        ((2.0 * GAMMA_E * b2.powi(2) * k.powi(2)) / (wm * rho * ms)).sqrt()
    }

    /// Chiral coupling coefficient $g_{me}$ in rad/s depending on acoustic polarization.
    /// Right-handed mode couples strongly; left-handed mode is decoupled.
    pub fn chiral_coupling_coefficient_rad_per_s(
        &self,
        k: f64,
        chirality: AcousticPolarizationChirality,
    ) -> f64 {
        let full_split = self.polariton_splitting_rad_per_s(k);
        match chirality {
            AcousticPolarizationChirality::RightHandedCircular => 0.5 * full_split,
            AcousticPolarizationChirality::LeftHandedCircular => 0.0,
            AcousticPolarizationChirality::Linear => 0.5 * FRAC_1_SQRT_2 * full_split,
        }
    }

    /// Acoustic angular momentum volume density $J_{AM} = \rho u_0^2 \omega$ in J·s/m$^3$.
    pub fn acoustic_angular_momentum_density(
        &self,
        displacement_amplitude_m: f64,
        angular_freq_rad_per_s: f64,
        chirality: AcousticPolarizationChirality,
    ) -> f64 {
        let sign = match chirality {
            AcousticPolarizationChirality::RightHandedCircular => 1.0,
            AcousticPolarizationChirality::LeftHandedCircular => -1.0,
            AcousticPolarizationChirality::Linear => 0.0,
        };
        sign * self.mass_density_kg_per_m3
            * displacement_amplitude_m.powi(2)
            * angular_freq_rad_per_s
    }
}
