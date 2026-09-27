//! Physical and semiconductor constants according to CODATA 2018 / 2022 standards.

/// Elementary electric charge $q$ in Coulombs ($C$).
pub const ELEMENTARY_CHARGE: f64 = 1.602_176_634e-19;

/// Boltzmann constant $k_B$ in Joules per Kelvin ($J/K$).
pub const BOLTZMANN_CONSTANT: f64 = 1.380_649e-23;

/// Permittivity of free space (vacuum) $\epsilon_0$ in Farads per meter ($F/m$).
pub const EPSILON_0: f64 = 8.854_187_812_8e-12;

/// Permeability of free space (vacuum) $\mu_0$ in Henries per meter ($H/m$).
pub const MU_0: f64 = 1.256_637_062_12e-6;

/// Speed of light in vacuum $c$ in meters per second ($m/s$).
pub const SPEED_OF_LIGHT: f64 = 299_792_458.0;

/// Planck constant $h$ in Joule-seconds ($J \cdot s$).
pub const PLANCK_CONSTANT: f64 = 6.626_070_15e-34;

/// Reduced Planck constant $\hbar = h / (2\pi)$ in Joule-seconds ($J \cdot s$).
pub const H_BAR: f64 = 1.054_571_817e-34;

/// Magnetic flux quantum $\Phi_0 = \frac{h}{2e}$ in Webers ($Wb$) or Volt-seconds ($V \cdot s$).
pub const FLUX_QUANTUM: f64 = 2.067_833_848e-15;

/// Conductance quantum $G_0 = \frac{2e^2}{h}$ in Siemens ($S$).
pub const CONDUCTANCE_QUANTUM: f64 = 7.748_091_729e-5;

/// Standard reference temperature $T_{ref} = 300.15 \text{ K}$ ($27^\circ\text{C}$).
pub const T_REF: f64 = 300.15;

/// Standard room temperature reference in Kelvin ($300.15\text{ K}$).
pub const ROOM_TEMPERATURE_KELVIN: f64 = T_REF;

/// Absolute zero in Celsius ($^\circ\text{C}$).
pub const ABSOLUTE_ZERO_CELSIUS: f64 = -273.15;

/// Mass density of crystalline Silicon $\rho_{Si}$ in $\text{kg/m}^3$ ($2.328\text{ g/cm}^3$).
pub const SILICON_DENSITY: f64 = 2328.0;

/// Relative permittivity of Silicon $\epsilon_{r, Si}$.
pub const EPSILON_R_SI: f64 = 11.7;

/// Relative permittivity of Silicon Dioxide $\epsilon_{r, SiO2}$.
pub const EPSILON_R_OX: f64 = 3.9;

/// Permittivity of Silicon Dioxide $\epsilon_{SiO2} = \epsilon_{r, SiO2} \cdot \epsilon_0$ in $\text{F/m}$.
pub const EPSILON_SIO2: f64 = EPSILON_R_OX * EPSILON_0;

/// Silicon energy bandgap at $0\text{ K}$ in electron-volts ($eV$).
pub const EG_SI_0K: f64 = 1.166;

/// Varshni temperature parameter $\alpha$ for Silicon in $eV/K$.
pub const VARSHNI_ALPHA_SI: f64 = 4.73e-4;

/// Varshni temperature parameter $\beta$ for Silicon in $K$.
pub const VARSHNI_BETA_SI: f64 = 636.0;

/// Computes the thermal voltage $V_t = \frac{k_B T}{q}$ at temperature $T$ (in Kelvin).
#[inline(always)]
pub fn thermal_voltage(temp_kelvin: f64) -> f64 {
    (BOLTZMANN_CONSTANT * temp_kelvin) / ELEMENTARY_CHARGE
}

/// Computes the temperature-dependent bandgap of Silicon $E_g(T)$ via the Varshni relation.
#[inline(always)]
pub fn silicon_bandgap(temp_kelvin: f64) -> f64 {
    let t = temp_kelvin.max(0.1);
    EG_SI_0K - (VARSHNI_ALPHA_SI * t * t) / (t + VARSHNI_BETA_SI)
}

/// Converts temperature from Celsius to Kelvin.
#[inline(always)]
pub fn celsius_to_kelvin(celsius: f64) -> f64 {
    celsius - ABSOLUTE_ZERO_CELSIUS
}

/// Converts temperature from Kelvin to Celsius.
#[inline(always)]
pub fn kelvin_to_celsius(kelvin: f64) -> f64 {
    kelvin + ABSOLUTE_ZERO_CELSIUS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thermal_voltage_at_300k() {
        let vt = thermal_voltage(300.15);
        // Vt at 300.15 K should be approx 25.864 mV
        assert!((vt - 0.02586419).abs() < 1e-6);
    }

    #[test]
    fn test_silicon_bandgap_at_room_temp() {
        let eg = silicon_bandgap(300.15);
        // Eg of Silicon at 300K is ~1.12 eV
        assert!((eg - 1.124).abs() < 0.01);
    }

    #[test]
    fn test_temperature_conversions() {
        assert_eq!(celsius_to_kelvin(27.0), 300.15);
        assert_eq!(kelvin_to_celsius(300.15), 27.0);
    }
}
