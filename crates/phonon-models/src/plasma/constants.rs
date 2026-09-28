//! Fundamental plasma physics, electromagnetism, and thermonuclear fusion constants.

/// Elementary charge $e$ in Coulombs (C).
pub const ELEMENTARY_CHARGE: f64 = 1.602_176_634e-19;

/// Vacuum magnetic permeability $\mu_0 = 4\pi \times 10^{-7}\text{ H/m}$.
pub const VACUUM_PERMEABILITY: f64 = 1.256_637_062_12e-6;

/// Vacuum electric permittivity $\epsilon_0$ in Farads per meter (F/m).
pub const VACUUM_PERMITTIVITY: f64 = 8.854_187_812_8e-12;

/// Boltzmann constant $k_B$ in Joules per Kelvin (J/K).
pub const BOLTZMANN_CONSTANT: f64 = 1.380_649e-23;

/// Speed of light in vacuum $c$ in meters per second (m/s).
pub const SPEED_OF_LIGHT: f64 = 299_792_458.0;

/// Proton mass $m_p$ in kilograms (kg).
pub const PROTON_MASS: f64 = 1.672_621_923_69e-27;

/// Deuteron mass $m_d \approx 2.014\text{ u}$ in kilograms (kg).
pub const DEUTERON_MASS: f64 = 3.343_583_772_4e-27;

/// Triton mass $m_t \approx 3.016\text{ u}$ in kilograms (kg).
pub const TRITON_MASS: f64 = 5.007_356_744_6e-27;

/// Alpha particle ($^4\text{He}^{2+}$) mass $m_\alpha \approx 4.0015\text{ u}$ in kilograms (kg).
pub const ALPHA_MASS: f64 = 6.644_657_335_7e-27;

/// Electron mass $m_e$ in kilograms (kg).
pub const ELECTRON_MASS: f64 = 9.109_383_701_5e-31;

/// Conversion factor from electron-volts to Joules (J/eV).
pub const EV_TO_JOULES: f64 = 1.602_176_634e-19;

/// Conversion factor from kilo-electron-volts to Joules (J/keV).
pub const KEV_TO_JOULES: f64 = 1.602_176_634e-16;

/// Energy of alpha particle produced in deuterium-tritium fusion: $E_\alpha = 3.52\text{ MeV}$ (J).
pub const DT_ALPHA_ENERGY_JOULES: f64 = 3.52e6 * EV_TO_JOULES;

/// Total energy released per deuterium-tritium fusion reaction: $Q_{DT} = 17.59\text{ MeV}$ (J).
pub const DT_TOTAL_ENERGY_JOULES: f64 = 17.59e6 * EV_TO_JOULES;

/// Ideal gas adiabatic index $\gamma = 5/3$ for monoatomic plasma.
pub const PLASMA_ADIABATIC_INDEX: f64 = 5.0 / 3.0;
