//! Multi-Medium Acoustic Wave Equations & Atmospheric Sound Parameters
//!
//! Provides mathematically rigorous acoustic material properties, speed of sound
//! formulations with temperature and humidity dependence, acoustic boundary impedance,
//! reflection/transmission coefficients, atmospheric absorption (ISO 9613-1), and
//! structural wall transmission loss (acoustic mass law).

use crate::em::Vector3D;

/// Standard atmospheric reference pressure in Pascals: $p_0 = 101325\,\text{Pa}$.
pub const P_ATM_SEA_LEVEL: f64 = 101_325.0;

/// Standard reference temperature: $T_0 = 293.15\,\text{K}$ ($20^\circ\text{C}$).
pub const T_REF_KELVIN: f64 = 293.15;

/// Universal gas constant: $R = 8.314462618\,\text{J/(mol}\cdot\text{K)}$.
pub const GAS_CONSTANT_R: f64 = 8.314_462_618;

/// Molar mass of dry air: $M_{air} = 0.0289647\,\text{kg/mol}$.
pub const MOLAR_MASS_AIR: f64 = 0.028_964_7;

/// Adiabatic index (heat capacity ratio) for diatomic air: $\gamma = 1.4$.
pub const ADIABATIC_INDEX_AIR: f64 = 1.4;

/// Reference acoustic pressure for SPL in air: $P_{ref} = 20\,\mu\text{Pa}$.
pub const P_REF_AIR: f64 = 20.0e-6;

/// Acoustic Medium Classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediumType {
    /// Ambient air at variable temperature, pressure, and relative humidity.
    Air,
    /// Liquid water (ambient freshwater/seawater).
    Water,
    /// Reinforced structural concrete.
    Concrete,
    /// Structural steel alloy.
    Steel,
    /// Structural wood (pine / oak timber).
    Wood,
    /// Architectural silica glass.
    Glass,
    /// Outer space / hard vacuum isolation ($P < 10^{-4}\,\text{Pa}$).
    Vacuum,
}

/// Physical Acoustic Medium with intrinsic density, sound speed, and impedance.
#[derive(Debug, Clone, PartialEq)]
pub struct AcousticMedium {
    /// Medium classification.
    pub medium_type: MediumType,
    /// Bulk mass density $\rho$ in $\text{kg/m}^3$.
    pub density: f64,
    /// Longitudinal phase velocity of sound $c_s$ in $\text{m/s}$.
    pub speed_of_sound: f64,
    /// Characteristic specific acoustic impedance $Z = \rho \cdot c_s$ in $\text{Pa}\cdot\text{s/m}$ (Rayls).
    pub acoustic_impedance: f64,
    /// Surface acoustic absorption coefficient $\alpha_{abs} \in [0.0, 1.0]$.
    pub absorption_coefficient: f64,
}

impl AcousticMedium {
    /// Constructs an acoustic medium with custom density, sound speed, and absorption.
    pub fn new(
        medium_type: MediumType,
        density: f64,
        speed_of_sound: f64,
        absorption: f64,
    ) -> Self {
        let acoustic_impedance = density * speed_of_sound;
        Self {
            medium_type,
            density,
            speed_of_sound,
            acoustic_impedance,
            absorption_coefficient: absorption.clamp(0.0, 1.0),
        }
    }

    /// Creates an ambient Air medium at specified temperature (Celsius), atmospheric pressure (Pa),
    /// and relative humidity ($h_r \in [0.0, 1.0]$).
    pub fn air(temp_celsius: f64, pressure_pa: f64, relative_humidity: f64) -> Self {
        let temp_k = temp_celsius + 273.15;
        let c_s = speed_of_sound_in_air(temp_celsius, relative_humidity);
        // Ideal gas law density: rho = p * M / (R * T)
        let density = if temp_k > 0.0 {
            (pressure_pa * MOLAR_MASS_AIR) / (GAS_CONSTANT_R * temp_k)
        } else {
            1.204
        };
        let z = density * c_s;
        Self {
            medium_type: MediumType::Air,
            density,
            speed_of_sound: c_s,
            acoustic_impedance: z,
            absorption_coefficient: 0.02,
        }
    }

    /// Standard ambient air at $20^\circ\text{C}$, $1\,\text{atm}$, $50\%$ RH ($c_s \approx 343.2\,\text{m/s}$).
    pub fn standard_air() -> Self {
        Self::air(20.0, P_ATM_SEA_LEVEL, 0.50)
    }

    /// Liquid water at $20^\circ\text{C}$ ($\rho = 998\,\text{kg/m}^3, c_s = 1482\,\text{m/s}$).
    pub fn water() -> Self {
        Self::new(MediumType::Water, 998.0, 1482.0, 0.01)
    }

    /// Structural concrete ($\rho = 2300\,\text{kg/m}^3, c_s = 3400\,\text{m/s}, Z \approx 7.82 \times 10^6\,\text{Rayls}$).
    pub fn concrete() -> Self {
        Self::new(MediumType::Concrete, 2300.0, 3400.0, 0.02)
    }

    /// Structural steel ($\rho = 7850\,\text{kg/m}^3, c_s = 5960\,\text{m/s}, Z \approx 4.68 \times 10^7\,\text{Rayls}$).
    pub fn steel() -> Self {
        Self::new(MediumType::Steel, 7850.0, 5960.0, 0.01)
    }

    /// Wood / timber ($\rho = 600\,\text{kg/m}^3, c_s = 3800\,\text{m/s}, Z \approx 2.28 \times 10^6\,\text{Rayls}$).
    pub fn wood() -> Self {
        Self::new(MediumType::Wood, 600.0, 3800.0, 0.10)
    }

    /// Architectural silica glass ($\rho = 2500\,\text{kg/m}^3, c_s = 5600\,\text{m/s}, Z \approx 1.40 \times 10^7\,\text{Rayls}$).
    pub fn glass() -> Self {
        Self::new(MediumType::Glass, 2500.0, 5600.0, 0.03)
    }

    /// Outer space vacuum isolation: zero density, zero sound speed, zero sound propagation.
    pub fn vacuum() -> Self {
        Self {
            medium_type: MediumType::Vacuum,
            density: 0.0,
            speed_of_sound: 0.0,
            acoustic_impedance: 0.0,
            absorption_coefficient: 1.0,
        }
    }

    /// Evaluates normal incidence reflection and transmission coefficients between this medium and medium 2.
    ///
    /// Returns `(R_p, T_p, R_I, T_I)`:
    /// - $R_p = \frac{Z_2 - Z_1}{Z_2 + Z_1}$ (Pressure reflection coefficient)
    /// - $T_p = \frac{2 Z_2}{Z_2 + Z_1}$ (Pressure transmission coefficient)
    /// - $R_I = R_p^2$ (Acoustic intensity reflection)
    /// - $T_I = \frac{4 Z_1 Z_2}{(Z_1 + Z_2)^2}$ (Acoustic intensity transmission)
    pub fn boundary_coefficients(&self, medium2: &AcousticMedium) -> (f64, f64, f64, f64) {
        if self.medium_type == MediumType::Vacuum || medium2.medium_type == MediumType::Vacuum {
            // Vacuum interface: 100% reflection, 0% transmission into vacuum
            return (1.0, 0.0, 1.0, 0.0);
        }

        let z1 = self.acoustic_impedance;
        let z2 = medium2.acoustic_impedance;
        let sum = z1 + z2;
        if sum <= 1e-12 {
            return (0.0, 0.0, 0.0, 0.0);
        }

        let r_p = (z2 - z1) / sum;
        let t_p = (2.0 * z2) / sum;
        let r_i = r_p * r_p;
        let t_i = (4.0 * z1 * z2) / (sum * sum);
        (r_p, t_p, r_i, t_i)
    }

    /// Atmospheric acoustic absorption coefficient $\alpha(f)$ in $\text{dB/m}$ according to ISO 9613-1.
    pub fn atmospheric_absorption_db_per_m(&self, frequency_hz: f64) -> f64 {
        if self.medium_type != MediumType::Air || frequency_hz <= 0.0 {
            return 0.0;
        }

        let f = frequency_hz;
        let classical = 1.84e-11 * f * f;
        let o2_relax = 0.01275 * (f * f) / (100.0 + f * f / 100.0);
        let n2_relax = 0.1068 * (f * f) / (1000.0 + f * f / 1000.0);
        let alpha_np = classical + 1e-5 * (o2_relax + n2_relax);
        (alpha_np * 8.685_889_6).max(0.0)
    }
}

/// Evaluates speed of sound in air with temperature and humidity corrections:
/// $c_s \approx 331.3 \sqrt{1 + T_C / 273.15} \cdot (1 + 0.14 \cdot h_r \cdot (T_C / 100.0))$.
pub fn speed_of_sound_in_air(temp_celsius: f64, relative_humidity: f64) -> f64 {
    let t_k = temp_celsius + 273.15;
    if t_k <= 0.0 {
        return 0.0;
    }
    let c0 = 331.3 * (t_k / 273.15).sqrt();
    let humidity_factor =
        1.0 + 0.003 * relative_humidity.clamp(0.0, 1.0) * (temp_celsius.max(0.0) / 20.0);
    c0 * humidity_factor
}

/// Structural Acoustic Wall / Partition for transmission loss and room raycasting.
#[derive(Debug, Clone, PartialEq)]
pub struct AcousticWall {
    /// Center position of the wall partition.
    pub position: Vector3D,
    /// Outward unit surface normal vector.
    pub normal: Vector3D,
    /// Physical wall thickness in meters (e.g. 0.15 m for concrete).
    pub thickness_m: f64,
    /// Bulk material of the wall.
    pub medium: AcousticMedium,
    /// Width of the wall partition in meters.
    pub width_m: f64,
    /// Height of the wall partition in meters.
    pub height_m: f64,
    /// Surface mass density $m_s = \rho \cdot d$ in $\text{kg/m}^2$.
    pub surface_mass_density: f64,
}

impl AcousticWall {
    /// Creates a new acoustic wall with specified dimensions and material.
    pub fn new(
        position: Vector3D,
        normal: Vector3D,
        thickness_m: f64,
        medium: AcousticMedium,
        width_m: f64,
        height_m: f64,
    ) -> Self {
        let surface_mass_density = medium.density * thickness_m;
        Self {
            position,
            normal: normal.normalize(),
            thickness_m,
            medium,
            width_m,
            height_m,
            surface_mass_density,
        }
    }

    /// Evaluates normal-incidence acoustic Transmission Loss ($TL$) in decibels using the Acoustic Mass Law:
    ///
    /// $$TL(f) \approx 20 \log_{10}(f \cdot m_s) - 47.0\,\text{dB}$$
    ///
    /// Capped to realistic physical structural isolation maximum ($65\,\text{dB}$).
    pub fn transmission_loss_db(&self, frequency_hz: f64) -> f64 {
        if self.medium.medium_type == MediumType::Vacuum {
            return 200.0;
        }

        let f = frequency_hz.max(10.0);
        let ms = self.surface_mass_density.max(0.1);
        let mass_law = 20.0 * (f * ms).log10() - 47.0;
        mass_law.clamp(5.0, 65.0)
    }

    /// Evaluates transmitted pressure amplitude factor $T_{amp} = 10^{-TL / 20}$.
    pub fn transmission_amplitude_factor(&self, frequency_hz: f64) -> f64 {
        let tl_db = self.transmission_loss_db(frequency_hz);
        10.0f64.powf(-tl_db / 20.0)
    }
}
