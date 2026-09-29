//! Superconducting Kinetic Inductance Traveling-Wave Parametric Amplifier (KITWPA) Models.
//!
//! Formulates current-dependent non-linear kinetic inductance in disordered superconductors
//! (NbTiN, granular aluminum), four-wave mixing (4WM) parametric gain, periodic dispersion
//! engineering, and high 1-dB compression saturation powers.

use std::f64::consts::PI;

/// Physical parameters of the disordered superconducting thin film.
#[derive(Debug, Clone, PartialEq)]
pub struct SuperconductingFilm {
    /// Material designation (e.g. "NbTiN" or "Granular Aluminum").
    pub material_name: String,
    /// Superconducting transition critical temperature $T_c$ in Kelvin (default 14.5 K for NbTiN).
    pub critical_temperature_k: f64,
    /// Normal-state sheet resistance $R_{sq}$ in Ohms/square (default 150 Ohms/sq).
    pub normal_sheet_resistance_ohms_per_sq: f64,
    /// Kinetic inductance fraction $\xi_{nl} = L_{k0} / L_0$ (default 0.95).
    pub kinetic_inductance_fraction: f64,
    /// Characteristic non-linearity current scale $I_*$ in Amperes (default 4.0 mA).
    pub characteristic_current_scale_a: f64,
}

impl Default for SuperconductingFilm {
    fn default() -> Self {
        Self {
            material_name: "NbTiN".to_string(),
            critical_temperature_k: 14.5,
            normal_sheet_resistance_ohms_per_sq: 150.0,
            kinetic_inductance_fraction: 0.95,
            characteristic_current_scale_a: 4.0e-3,
        }
    }
}

/// Transmission line configuration for a KITWPA.
#[derive(Debug, Clone, PartialEq)]
pub struct KitwpaTransmissionLine {
    /// Thin film superconducting properties.
    pub film: SuperconductingFilm,
    /// Total meandering line length $L$ in meters (default 0.035 m = 35 mm).
    pub total_length_m: f64,
    /// Linear inductance per unit length $L_0$ in H/m (default 1.5 uH/m).
    pub linear_inductance_per_m: f64,
    /// Capacitance per unit length $C_0$ in F/m (default 300 pF/m).
    pub capacitance_per_m: f64,
    /// RF pump current amplitude $I_p$ in Amperes (default 2.4 mA).
    pub pump_current_a: f64,
    /// Pump frequency $f_p$ in Hertz (default 8.0 GHz).
    pub pump_frequency_hz: f64,
}

impl Default for KitwpaTransmissionLine {
    fn default() -> Self {
        Self {
            film: SuperconductingFilm::default(),
            total_length_m: 0.035,
            linear_inductance_per_m: 1.5e-6,
            capacitance_per_m: 300.0e-12,
            pump_current_a: 2.4e-3,
            pump_frequency_hz: 8.0e9,
        }
    }
}

impl KitwpaTransmissionLine {
    /// Phase velocity $v_p = \frac{1}{\sqrt{L_0 C_0}}$ in m/s.
    #[inline]
    pub fn phase_velocity_m_per_s(&self) -> f64 {
        1.0 / (self.linear_inductance_per_m * self.capacitance_per_m).sqrt()
    }

    /// Characteristic wave impedance $Z_0 = \sqrt{\frac{L_0}{C_0}}$ in Ohms.
    #[inline]
    pub fn characteristic_impedance_ohms(&self) -> f64 {
        (self.linear_inductance_per_m / self.capacitance_per_m).sqrt()
    }

    /// Total current-dependent non-linear inductance per unit length:
    /// $L(I) = L_0 [1 + \xi_{nl} (I / I_*)^2]$.
    #[inline]
    pub fn non_linear_inductance_per_m(&self, current_a: f64) -> f64 {
        let i_ratio = current_a / self.film.characteristic_current_scale_a;
        self.linear_inductance_per_m
            * (1.0 + self.film.kinetic_inductance_fraction * i_ratio.powi(2))
    }

    /// Parametric gain coefficient $g$ in m$^{-1}$ for 4WM:
    /// $g = \frac{1}{4} \sqrt{k_s k_i} \xi_{nl} (I_p / I_*)^2$.
    pub fn parametric_gain_coefficient_per_m(&self, signal_freq_hz: f64) -> f64 {
        let vp = self.phase_velocity_m_per_s();
        let ws = 2.0 * PI * signal_freq_hz;
        let wi = 4.0 * PI * self.pump_frequency_hz - ws;
        if wi <= 0.0 {
            return 0.0;
        }

        let ks = ws / vp;
        let ki = wi / vp;
        let i_ratio = self.pump_current_a / self.film.characteristic_current_scale_a;
        0.25 * (ks * ki).sqrt() * self.film.kinetic_inductance_fraction * i_ratio.powi(2)
    }

    /// Total phase mismatch $\Delta k$ in rad/m.
    /// In dispersion-engineered lines, periodic loading cancels Kerr self- and cross-phase modulation.
    pub fn phase_mismatch_rad_per_m(
        &self,
        signal_freq_hz: f64,
        dispersion_engineered: bool,
    ) -> f64 {
        if dispersion_engineered {
            // Dispersion engineering with sub-wavelength stubs eliminates net phase mismatch
            let df = (signal_freq_hz - self.pump_frequency_hz).abs() / self.pump_frequency_hz;
            0.5 * df.powi(2) // Residual quadratic ripple
        } else {
            // Uncompensated Kerr self-phase modulation
            let vp = self.phase_velocity_m_per_s();
            let kp = 2.0 * PI * self.pump_frequency_hz / vp;
            let i_ratio = self.pump_current_a / self.film.characteristic_current_scale_a;
            0.5 * kp * self.film.kinetic_inductance_fraction * i_ratio.powi(2)
        }
    }

    /// Analytical 4WM signal power gain $G_s$:
    /// $G_s = 1 + \left[ \frac{g}{g_{eff}} \sinh(g_{eff} L) \right]^2$.
    pub fn analytical_signal_power_gain(
        &self,
        signal_freq_hz: f64,
        dispersion_engineered: bool,
    ) -> f64 {
        let g = self.parametric_gain_coefficient_per_m(signal_freq_hz);
        let dk = self.phase_mismatch_rad_per_m(signal_freq_hz, dispersion_engineered);
        let arg = g.powi(2) - (0.5 * dk).powi(2);

        if arg > 0.0 {
            let g_eff = arg.sqrt();
            let sinh_val = (g_eff * self.total_length_m).sinh();
            1.0 + (g / g_eff * sinh_val).powi(2)
        } else {
            let g_eff = (-arg).sqrt();
            let sin_val = (g_eff * self.total_length_m).sin();
            1.0 + (g / g_eff * sin_val).powi(2)
        }
    }

    /// 1-dB compression saturation power in dBm:
    /// $P_{-1dB} \approx 10 \log_{10}\left( \frac{0.1 I_*^2 Z_0}{10^{-3}} \right)$.
    pub fn one_db_compression_power_dbm(&self) -> f64 {
        let z0 = self.characteristic_impedance_ohms();
        let i_star = self.film.characteristic_current_scale_a;
        let p_sat_watts = 0.1 * i_star.powi(2) * z0;
        10.0 * (p_sat_watts / 1e-3).log10()
    }
}
