//! Piezoelectric surface acoustic wave (SAW) dynamic quantum dot potential wells,
//! single-electron confinement, and quantized acoustoelectric charge pumping.
//!
//! # Physical Formalism
//! - Traveling SAW Piezoelectric Potential:
//!   $$\Phi(x, t) = \Phi_0 \cos(k_{\mathrm{saw}} x - \omega_{\mathrm{saw}} t)$$
//!   where $k_{\mathrm{saw}} = \frac{2\pi f_{\mathrm{saw}}}{v_{\mathrm{saw}}}$.
//! - Combined DQD Confinement Potential:
//!   $$V(x, t) = -e \Phi(x, t) + V_{\mathrm{gate}}(x)$$
//! - Dynamic Dot Confinement Frequency:
//!   $$\omega_{\mathrm{conf}} = \sqrt{\frac{e \Phi_0 k_{\mathrm{saw}}^2}{m^*}}$$
//! - Charging Energy:
//!   $$E_C = \frac{e^2}{2 C_{\mathrm{dot}}}$$
//! - Quantized Acoustoelectric Current:
//!   $$I_{\mathrm{pump}} = N \cdot e \cdot f_{\mathrm{saw}} [1 - P_{\mathrm{esc}}]$$
//!   with precision requirement $|I / (e f_{\mathrm{saw}}) - 1| < 10^{-4}$.

use std::f64::consts::PI;

pub const ELEMENTARY_CHARGE_C: f64 = 1.602_176_634e-19; // Coulomb
pub const ELECTRON_MASS_KG: f64 = 9.109_383_701_5e-31; // kg
pub const HBAR_J_S: f64 = 1.054_571_817e-34; // J*s
pub const BOLTZMANN_J_K: f64 = 1.380_649e-23; // J/K

/// Material and geometric parameters for piezoelectric SAW device.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PiezoelectricSawParams {
    /// SAW RF drive frequency $f_{saw}$ in Hz (e.g. 1 - 5 GHz).
    pub frequency_hz: f64,
    /// Surface acoustic wave phase velocity $v_{saw}$ in m/s (e.g. 2865 m/s for GaAs).
    pub sound_velocity_m_s: f64,
    /// Piezoelectric potential amplitude $\Phi_0$ in Volts (typically 20 - 150 mV).
    pub potential_amplitude_v: f64,
    /// Effective conduction band electron mass in kg (e.g. $0.067 m_0$ in GaAs).
    pub effective_mass_kg: f64,
    /// Piezoelectric electromechanical coupling coefficient $K^2$ (e.g. 0.0007 for GaAs, 0.05 for LiNbO3).
    pub coupling_coefficient_k2: f64,
}

impl PiezoelectricSawParams {
    /// Standard GaAs/AlGaAs 2DEG piezoelectric heterostructure at 3.0 GHz.
    pub fn gaas_standard_3ghz() -> Self {
        Self {
            frequency_hz: 3.0e9,
            sound_velocity_m_s: 2865.0,
            potential_amplitude_v: 0.050, // 50 mV
            effective_mass_kg: 0.067 * ELECTRON_MASS_KG,
            coupling_coefficient_k2: 7.2e-4,
        }
    }

    /// High-coupling Lithium Niobate ($\text{LiNbO}_3$) / 2DEG hybrid at 1.5 GHz.
    pub fn linbo3_hybrid_1_5ghz() -> Self {
        Self {
            frequency_hz: 1.5e9,
            sound_velocity_m_s: 3480.0,
            potential_amplitude_v: 0.120, // 120 mV
            effective_mass_kg: 0.067 * ELECTRON_MASS_KG,
            coupling_coefficient_k2: 0.048,
        }
    }

    /// Acoustic wavelength $\lambda_{saw} = v_{saw} / f_{saw}$ in meters.
    #[inline]
    pub fn wavelength_m(&self) -> f64 {
        self.sound_velocity_m_s / self.frequency_hz
    }

    /// Acoustic wavenumber $k_{saw} = 2\pi / \lambda_{saw}$ in $\text{rad/m}$.
    #[inline]
    pub fn wavenumber_rad_per_m(&self) -> f64 {
        2.0 * PI * self.frequency_hz / self.sound_velocity_m_s
    }

    /// Angular frequency $\omega_{saw} = 2\pi f_{saw}$ in rad/s.
    #[inline]
    pub fn omega_rad_s(&self) -> f64 {
        2.0 * PI * self.frequency_hz
    }
}

/// 1D electrostatic channel defined by split depletion gates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SplitGateChannel {
    /// Gate bias voltage $V_{gate}$ in Volts (e.g. -0.8 V to -0.3 V).
    pub gate_voltage_v: f64,
    /// Pinch-off threshold voltage $V_p$ in Volts.
    pub pinch_off_voltage_v: f64,
    /// Channel length $L_{ch}$ in meters (typically 500 nm to 2000 nm).
    pub channel_length_m: f64,
    /// Channel barrier width $W_b$ in meters (typically 20 nm to 50 nm).
    pub barrier_width_m: f64,
}

impl SplitGateChannel {
    pub fn new(gate_voltage_v: f64, pinch_off_voltage_v: f64, channel_length_m: f64) -> Self {
        Self {
            gate_voltage_v,
            pinch_off_voltage_v,
            channel_length_m,
            barrier_width_m: 30.0e-9,
        }
    }

    /// Electrostatic potential profile $V_{gate}(x)$ in Volts across the channel.
    pub fn electrostatic_potential(&self, x: f64) -> f64 {
        let half_l = 0.5 * self.channel_length_m;
        if x.abs() > half_l {
            0.0
        } else {
            // Parabolic saddle-point barrier profile
            let rel = x / half_l;
            self.gate_voltage_v * (1.0 - rel.powi(2))
        }
    }
}

/// Moving Dynamic Quantum Dot (DQD) carrier formed by SAW piezoelectric wave.
#[derive(Debug, Clone, PartialEq)]
pub struct DynamicQuantumDot {
    pub saw_params: PiezoelectricSawParams,
    pub channel: SplitGateChannel,
    /// Capacitance of the dynamic dot $C_{dot}$ in Farads (~ $5 \times 10^{-17}\text{ F}$).
    pub dot_capacitance_f: f64,
    /// Operating ambient temperature in Kelvin.
    pub temperature_k: f64,
}

impl DynamicQuantumDot {
    pub fn new(saw_params: PiezoelectricSawParams, channel: SplitGateChannel) -> Self {
        Self {
            saw_params,
            channel,
            dot_capacitance_f: 4.5e-17,
            temperature_k: 0.100, // 100 mK
        }
    }

    /// Evaluates total potential energy $U(x, t)$ in Joules:
    /// $$U(x, t) = -e \Phi_0 \cos(k_{\mathrm{saw}} x - \omega_{\mathrm{saw}} t) - e V_{\mathrm{gate}}(x)$$
    pub fn potential_energy_j(&self, x: f64, t: f64) -> f64 {
        let k = self.saw_params.wavenumber_rad_per_m();
        let w = self.saw_params.omega_rad_s();
        let saw_pot = self.saw_params.potential_amplitude_v * (k * x - w * t).cos();
        let gate_pot = self.channel.electrostatic_potential(x);

        -ELEMENTARY_CHARGE_C * (saw_pot + gate_pot)
    }

    /// Harmonic confinement frequency $\omega_{conf}$ in the minimum of the moving well:
    /// $$\omega_{\mathrm{conf}} = \sqrt{\frac{e \Phi_0 k_{\mathrm{saw}}^2}{m^*}}$$
    pub fn confinement_frequency_rad_s(&self) -> f64 {
        let e = ELEMENTARY_CHARGE_C;
        let phi = self.saw_params.potential_amplitude_v;
        let k = self.saw_params.wavenumber_rad_per_m();
        let m = self.saw_params.effective_mass_kg;

        ((e * phi * k.powi(2)) / m).sqrt()
    }

    /// Single-particle orbital ground state energy in the dynamic dot:
    /// $$E_0 = \frac{1}{2} \hbar \omega_{\mathrm{conf}}$$
    pub fn ground_state_energy_j(&self) -> f64 {
        0.5 * HBAR_J_S * self.confinement_frequency_rad_s()
    }

    /// Electrostatic charging energy $E_C = \frac{e^2}{2 C_{dot}}$ in Joules.
    pub fn charging_energy_j(&self) -> f64 {
        let e = ELEMENTARY_CHARGE_C;
        (e.powi(2)) / (2.0 * self.dot_capacitance_f)
    }

    /// Evaluates non-adiabatic back-tunneling escape probability $P_{esc}$:
    /// Flensberg-Talyanskii capture model:
    /// $$P_{\mathrm{esc}} \approx \exp\left( - \frac{4 \sqrt{2 m^*} E_b^{3/2}}{3 e \hbar \mathcal{E}} \right)$$
    pub fn escape_probability(&self) -> f64 {
        let delta_v = (self.channel.gate_voltage_v - self.channel.pinch_off_voltage_v).abs();
        let barrier = (ELEMENTARY_CHARGE_C * delta_v + self.charging_energy_j()).max(1e-21);

        // Effective electric field during pinch-off
        let e_field = (self.saw_params.potential_amplitude_v
            / (self.saw_params.wavelength_m() * 0.25))
            .max(1e3);

        let m = self.saw_params.effective_mass_kg;
        let exponent = (4.0 * (2.0 * m).sqrt() * barrier.powf(1.5))
            / (3.0 * ELEMENTARY_CHARGE_C * HBAR_J_S * e_field);

        // Also incorporate thermal activation: exp(-E_b / k_B T)
        let thermal_term = (-barrier / (BOLTZMANN_J_K * self.temperature_k.max(1e-3))).exp();

        let tunnel_term = (-exponent.min(100.0)).exp();

        // Non-adiabatic excitation factor during dynamic dot capture: ~ 1e-6
        let non_adiabatic_factor = 1.0e-6;

        (tunnel_term + thermal_term + non_adiabatic_factor).clamp(1e-12, 0.5)
    }

    /// Evaluates expected acoustoelectric current $I$ in Amperes for $N = 1$ electron:
    /// $$I = e \cdot f_{\mathrm{saw}} \cdot (1 - P_{\mathrm{esc}})$$
    pub fn evaluate_pumped_current(&self) -> f64 {
        let p_esc = self.escape_probability();
        ELEMENTARY_CHARGE_C * self.saw_params.frequency_hz * (1.0 - p_esc)
    }

    /// Returns relative current quantization error:
    /// $$\epsilon_I = \left| \frac{I}{e f_{\mathrm{saw}}} - 1 \right|$$
    pub fn quantization_error(&self) -> f64 {
        let i = self.evaluate_pumped_current();
        let ideal = ELEMENTARY_CHARGE_C * self.saw_params.frequency_hz;
        ((i - ideal) / ideal).abs()
    }
}
