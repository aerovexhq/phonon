//! Cryogenic CMOS control circuitry at 4 K: cryo-PLLs, multi-bit DAC gate drivers,
//! transimpedance readout amplifiers, and thermal phonon self-heating back-action.

/// Cryogenic Phase-Locked Loop (Cryo-PLL) operating at 4 K.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CryoPll {
    /// Clock frequency in Hertz (typically $1 - 10\text{ GHz}$).
    pub frequency_hz: f64,
    /// Operating temperature in Kelvin (typically $4.0\text{ K}$).
    pub temperature_k: f64,
    /// Standby electrical power consumption in Watts (typically $1 - 5\text{ mW}$).
    pub power_consumption_watts: f64,
    /// Root-mean-square phase jitter in femtoseconds ($10^{-15}\text{ s}$).
    pub phase_jitter_fs: f64,
}

impl CryoPll {
    /// Standard 4 K Cryo-PLL configuration at 2.5 GHz.
    pub fn standard_4k() -> Self {
        Self {
            frequency_hz: 2.5e9,
            temperature_k: 4.0,
            power_consumption_watts: 2.5e-3, // 2.5 mW
            phase_jitter_fs: 45.0,           // 45 fs at 4K
        }
    }

    /// Evaluates thermal phase noise floor in $\text{dBc/Hz}$:
    /// $$\mathcal{L}_{floor} \approx 10 \log_{10}\left( \frac{2 k_B T}{P_{osc}} \right)$$
    pub fn phase_noise_floor_dbc_per_hz(&self) -> f64 {
        let kb = 1.380_649e-23;
        let p_osc = self.power_consumption_watts * 0.2; // 20% into RF tank
        let noise_ratio = (2.0 * kb * self.temperature_k) / p_osc.max(1e-9);
        10.0 * noise_ratio.log10()
    }
}

/// Cryogenic Digital-to-Analog Converter (Cryo-DAC) steering gate voltages for MZM braiding.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CryoDac {
    /// DAC resolution in bits (typically 10-14 bits).
    pub resolution_bits: usize,
    /// Full-scale voltage range $V_{FS}$ in Volts (typically $\pm 0.5\text{ V}$).
    pub voltage_range_volts: f64,
    /// Settling time $t_{settling}$ in nanoseconds (typically $1 - 5\text{ ns}$).
    pub settling_time_ns: f64,
    /// Static power dissipation in Watts (typically $0.5 - 2\text{ mW}$).
    pub power_consumption_watts: f64,
}

impl CryoDac {
    /// Standard 12-bit Cryo-DAC operating at 4 K.
    pub fn standard_12bit() -> Self {
        Self {
            resolution_bits: 12,
            voltage_range_volts: 1.0,
            settling_time_ns: 2.0,
            power_consumption_watts: 1.2e-3, // 1.2 mW
        }
    }

    /// Least Significant Bit (LSB) voltage step $\Delta V = V_{FS} / 2^N$ in Volts.
    pub fn lsb_step_volts(&self) -> f64 {
        self.voltage_range_volts / (1 << self.resolution_bits) as f64
    }

    /// Converts integer digital code $D \in [0, 2^N - 1]$ to output gate voltage.
    pub fn code_to_voltage(&self, code: usize) -> f64 {
        let max_code = (1 << self.resolution_bits) - 1;
        let clamped = code.min(max_code);
        let frac = (clamped as f64) / (max_code as f64);
        -0.5 * self.voltage_range_volts + frac * self.voltage_range_volts
    }
}

/// Cryogenic Transimpedance Readout Amplifier (TIA) for reflectometry and conductance sensing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CryoReadoutTia {
    /// Transimpedance gain $R_{TIA}$ in Ohms (typically $10^4 - 10^5\text{ }\Omega$).
    pub transimpedance_gain_ohms: f64,
    /// Noise temperature $T_N$ in Kelvin (typically $3 - 8\text{ K}$).
    pub noise_temperature_k: f64,
    /// Bandwidth in Hertz (typically $50 - 500\text{ MHz}$).
    pub bandwidth_hz: f64,
    /// Power dissipation in Watts (typically $1 - 3\text{ mW}$).
    pub power_consumption_watts: f64,
}

impl CryoReadoutTia {
    /// Standard Cryo-TIA for 4 K qubit readout.
    pub fn standard_4k() -> Self {
        Self {
            transimpedance_gain_ohms: 50_000.0,
            noise_temperature_k: 4.5,
            bandwidth_hz: 100.0e6,
            power_consumption_watts: 1.8e-3,
        }
    }

    /// Evaluates input-referred current noise spectral density $S_I = \sqrt{4 k_B T_N / R_{in}}$ in $A/\sqrt{\text{Hz}}$.
    pub fn input_current_noise(&self, r_in_ohms: f64) -> f64 {
        let kb = 1.380_649e-23;
        ((4.0 * kb * self.noise_temperature_k) / r_in_ohms.max(1.0)).sqrt()
    }

    /// Output voltage for an input Zero-Bias Conductance current $I = G_{ZBCP} \cdot V_{bias}$.
    pub fn amplify_conductance_signal(&self, g_siemens: f64, v_bias_volts: f64) -> f64 {
        let i_in = g_siemens * v_bias_volts;
        i_in * self.transimpedance_gain_ohms
    }
}

/// Thermal phonon self-heating back-action coupling Cryo-CMOS dissipation to sub-Kelvin dilution stage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CryoThermalBackaction {
    /// Nominal stage base temperature in Kelvin (typically $4.0\text{ K}$ or $0.020\text{ K}$).
    pub base_temperature_k: f64,
    /// Thermal conductance $K_{th}$ to cold finger in $W/K$.
    pub thermal_conductance_w_per_k: f64,
    /// Total electrical power dissipation in Cryo-CMOS ASIC in Watts.
    pub total_dissipation_watts: f64,
}

impl CryoThermalBackaction {
    /// Standard 4 K stage thermal model with copper cold plate link.
    pub fn stage_4k(total_dissipation_watts: f64) -> Self {
        Self {
            base_temperature_k: 4.0,
            thermal_conductance_w_per_k: 0.5, // 0.5 W/K at 4 K
            total_dissipation_watts,
        }
    }

    /// Sub-Kelvin mixing chamber stage model ($T_{base} = 20\text{ mK}$).
    pub fn mixing_chamber_stage(total_dissipation_watts: f64) -> Self {
        Self {
            base_temperature_k: 0.020,
            thermal_conductance_w_per_k: 1.0e-3, // 1 mW/K at 20 mK
            total_dissipation_watts,
        }
    }

    /// Local elevated steady-state temperature:
    /// $$T_{local} = T_{base} + \frac{P_{diss}}{K_{th}}$$
    pub fn local_temperature_k(&self) -> f64 {
        self.base_temperature_k
            + (self.total_dissipation_watts / self.thermal_conductance_w_per_k.max(1e-6))
    }

    /// Temperature-induced dephasing rate enhancement $\Gamma_{th} = \Gamma_0 (T / T_{base})$:
    pub fn dephasing_enhancement_factor(&self) -> f64 {
        self.local_temperature_k() / self.base_temperature_k.max(1e-4)
    }
}
