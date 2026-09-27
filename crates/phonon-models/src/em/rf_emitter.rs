//! First-Principles RF Emitter Synthesis & Discrete Transceiver Transduction
//!
//! Provides microscopic and component-level synthesizers for physical radio emitters:
//! - LC Tank Oscillators (Barkhausen criterion, quality factor Q, damped/sustained limits)
//! - Colpitts Oscillators (capacitive voltage divider feedback, active transistor transconductance)
//! - Quartz Crystal Oscillators (Butterworth-Van Dyke BVD equivalent model, Leeson phase noise)
//! - RF Power Amplifiers (Classes A, AB, C, E, F with 1-dB compression and PAE)
//! - Discrete RF Transmitters (coupling oscillator + PA + matching network + physical antenna)

use crate::em::antenna::PhysicalAntenna;
use crate::em::vector_wave::{EmWaveSource, Vector3D};
use std::f64::consts::PI;

/// Boltzmann constant in J/K.
pub const BOLTZMANN_CONSTANT: f64 = 1.380_649e-23;

/// Reference ambient temperature $T_0 = 290\text{ K}$.
pub const REFERENCE_TEMP_K: f64 = 290.0;

/// Archetype of electronic RF oscillator.
#[derive(Debug, Clone, PartialEq)]
pub enum OscillatorType {
    /// Resonant LC tank with parallel/series inductive and capacitive elements.
    LCTank {
        /// Inductance in Henries (H).
        inductance_h: f64,
        /// Capacitance in Farads (F).
        capacitance_f: f64,
        /// Series Equivalent Resistance (ESR) in Ohms ($\Omega$).
        series_resistance_ohms: f64,
        /// Sustained peak voltage amplitude in Volts.
        peak_voltage_v: f64,
    },
    /// Colpitts oscillator with dual feedback capacitors and active transistor.
    Colpitts {
        /// Tank inductor in Henries (H).
        inductance_h: f64,
        /// Upper feedback capacitor $C_1$ in Farads (F).
        c1_f: f64,
        /// Lower feedback capacitor $C_2$ in Farads (F).
        c2_f: f64,
        /// Active device small-signal transconductance $g_m$ in Siemens (A/V).
        transconductance_s: f64,
        /// Tank parallel load resistance in Ohms ($\Omega$).
        load_resistance_ohms: f64,
        /// DC collector/drain bias current in Amperes.
        bias_current_a: f64,
    },
    /// Quartz crystal resonator modeled via Butterworth-Van Dyke (BVD) circuit.
    Crystal {
        /// Series motional inductance $L_m$ in Henries (H).
        motional_inductance_h: f64,
        /// Series motional capacitance $C_m$ in Farads (F).
        motional_capacitance_f: f64,
        /// Motional series resistance $R_m$ in Ohms ($\Omega$).
        motional_resistance_ohms: f64,
        /// Shunt parallel package capacitance $C_0$ in Farads (F).
        shunt_capacitance_f: f64,
        /// Oscillator drive level in Watts (typically $10\,\mu\text{W}$ to $1\text{ mW}$).
        drive_level_watts: f64,
    },
}

impl OscillatorType {
    /// Computes the fundamental resonant oscillation frequency in Hz.
    pub fn oscillation_frequency_hz(&self) -> f64 {
        match self {
            OscillatorType::LCTank {
                inductance_h,
                capacitance_f,
                ..
            } => 1.0 / (2.0 * PI * (inductance_h * capacitance_f).sqrt()),
            OscillatorType::Colpitts {
                inductance_h,
                c1_f,
                c2_f,
                ..
            } => {
                let c_eq = (c1_f * c2_f) / (c1_f + c2_f);
                1.0 / (2.0 * PI * (inductance_h * c_eq).sqrt())
            }
            OscillatorType::Crystal {
                motional_inductance_h,
                motional_capacitance_f,
                ..
            } => {
                // Series motional resonance frequency f_s
                1.0 / (2.0 * PI * (motional_inductance_h * motional_capacitance_f).sqrt())
            }
        }
    }

    /// Computes the parallel anti-resonance frequency $f_p$ for crystal resonators.
    pub fn anti_resonance_frequency_hz(&self) -> Option<f64> {
        match self {
            OscillatorType::Crystal {
                motional_capacitance_f,
                shunt_capacitance_f,
                ..
            } => {
                let fs = self.oscillation_frequency_hz();
                let factor = (1.0 + motional_capacitance_f / shunt_capacitance_f).sqrt();
                Some(fs * factor)
            }
            _ => None,
        }
    }

    /// Resonator Quality Factor $Q$.
    pub fn quality_factor(&self) -> f64 {
        let f0 = self.oscillation_frequency_hz();
        let omega0 = 2.0 * PI * f0;
        match self {
            OscillatorType::LCTank {
                inductance_h,
                series_resistance_ohms,
                ..
            } => (omega0 * inductance_h) / series_resistance_ohms.max(1e-9),
            OscillatorType::Colpitts {
                inductance_h,
                load_resistance_ohms,
                c1_f,
                c2_f,
                ..
            } => {
                let c_eq = (c1_f * c2_f) / (c1_f + c2_f);
                let z0 = (inductance_h / c_eq).sqrt();
                load_resistance_ohms / z0
            }
            OscillatorType::Crystal {
                motional_inductance_h,
                motional_resistance_ohms,
                ..
            } => (omega0 * motional_inductance_h) / motional_resistance_ohms.max(1e-9),
        }
    }

    /// Verifies Barkhausen oscillation stability criterion (loop gain $\ge 1.0$).
    pub fn verifies_barkhausen_criterion(&self) -> bool {
        match self {
            OscillatorType::LCTank { .. } => true, // Assumed actively compensated
            OscillatorType::Colpitts {
                transconductance_s,
                load_resistance_ohms,
                c1_f,
                c2_f,
                ..
            } => {
                // Loop gain A_loop = g_m * R_L * (C1 / C2)
                let loop_gain = transconductance_s * load_resistance_ohms * (c1_f / c2_f);
                loop_gain >= 1.0
            }
            OscillatorType::Crystal { .. } => true,
        }
    }

    /// Available RF output power from the oscillator stage in Watts.
    pub fn output_power_watts(&self) -> f64 {
        match self {
            OscillatorType::LCTank {
                peak_voltage_v,
                series_resistance_ohms,
                ..
            } => {
                let v_rms = peak_voltage_v / std::f64::consts::SQRT_2;
                (v_rms * v_rms) / (50.0 + series_resistance_ohms)
            }
            OscillatorType::Colpitts {
                bias_current_a,
                load_resistance_ohms,
                ..
            } => {
                // Large-signal fundamental peak voltage ~ (4/pi) * I_bias * R_load
                let v_peak = (4.0 / PI) * bias_current_a * load_resistance_ohms * 0.5;
                let v_rms = v_peak / std::f64::consts::SQRT_2;
                (v_rms * v_rms) / load_resistance_ohms
            }
            OscillatorType::Crystal {
                drive_level_watts, ..
            } => *drive_level_watts,
        }
    }

    /// Leeson single-sideband phase noise $\mathcal{L}(\Delta f)$ in dBc/Hz at frequency offset $\Delta f$ (Hz).
    pub fn phase_noise_dbc_per_hz(&self, offset_hz: f64) -> f64 {
        let f0 = self.oscillation_frequency_hz();
        let q = self.quality_factor();
        let p_sig = self.output_power_watts().max(1e-9);
        let df = offset_hz.abs().max(1.0);

        // Noise factor F ~ 2.0 (3 dB), flicker corner f_c ~ 1 kHz
        let noise_factor = 2.0;
        let flicker_corner_hz = 1000.0;

        let thermal_part = (2.0 * noise_factor * BOLTZMANN_CONSTANT * REFERENCE_TEMP_K) / p_sig;
        let resonator_part = 1.0 + (f0 / (2.0 * q * df)).powi(2);
        let flicker_part = 1.0 + flicker_corner_hz / df;

        let linear_noise = thermal_part * resonator_part * flicker_part;
        10.0 * linear_noise.max(1e-25).log10()
    }
}

/// Operating class of the RF Power Amplifier (PA).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AmplifierClass {
    /// Class A: High linearity, conduction angle $360^\circ$, max theoretical efficiency 50%.
    ClassA,
    /// Class AB: Compromise linearity and efficiency, conduction angle between $180^\circ$ and $360^\circ$.
    ClassAB,
    /// Class C: High efficiency, non-linear, conduction angle $< 180^\circ$, max theoretical efficiency ~78.5%.
    ClassC,
    /// Class E: Zero-Voltage-Switching (ZVS) resonant switched-mode PA, theoretical efficiency up to 100%.
    ClassE,
    /// Class F: Harmonic-tuned (odd voltage, even current harmonics), high efficiency ~85-90%.
    ClassF,
}

/// Physical RF Power Amplifier (PA) specification.
#[derive(Debug, Clone, PartialEq)]
pub struct RfPowerAmplifier {
    /// Amplifier architecture class.
    pub class: AmplifierClass,
    /// Small-signal power gain in dB.
    pub gain_db: f64,
    /// 1-dB gain compression output power $P_{1\text{dB}}$ in dBm.
    pub p1db_dbm: f64,
    /// Saturated output power $P_{sat}$ in dBm.
    pub psat_dbm: f64,
    /// DC supply voltage in Volts.
    pub supply_voltage_v: f64,
    /// Maximum Power-Added Efficiency (PAE) $\in [0.1, 0.95]$.
    pub max_pae: f64,
}

impl RfPowerAmplifier {
    /// Creates a typical mobile Wi-Fi / cellular Class-AB PA.
    pub fn wifi_class_ab() -> Self {
        Self {
            class: AmplifierClass::ClassAB,
            gain_db: 24.0,
            p1db_dbm: 23.0,
            psat_dbm: 25.0,
            supply_voltage_v: 3.3,
            max_pae: 0.45,
        }
    }

    /// Creates a high-efficiency Class-E switching PA for discrete transmitters.
    pub fn switching_class_e() -> Self {
        Self {
            class: AmplifierClass::ClassE,
            gain_db: 20.0,
            p1db_dbm: 29.0,
            psat_dbm: 30.0, // 1 Watt
            supply_voltage_v: 5.0,
            max_pae: 0.82,
        }
    }

    /// Computes actual output RF power in dBm and Watts for a given input power $P_{in}$ (dBm).
    pub fn amplify(&self, pin_dbm: f64) -> (f64, f64) {
        let linear_pout_dbm = pin_dbm + self.gain_db;
        // Non-linear hyperbolic tangent saturation compression curve:
        // P_out = P_sat / (1 + (P_linear / P_sat)^2)^(1/2)
        let psat_mw = 10.0_f64.powf(self.psat_dbm / 10.0);
        let plin_mw = 10.0_f64.powf(linear_pout_dbm / 10.0);

        let pout_mw = plin_mw / (1.0 + (plin_mw / psat_mw).powi(2)).sqrt();
        let pout_dbm = 10.0 * pout_mw.max(1e-12).log10();
        let pout_watts = pout_mw * 1e-3;

        (pout_dbm, pout_watts)
    }

    /// Power Added Efficiency (PAE) for an operating output power.
    pub fn evaluate_pae(&self, pout_watts: f64, pin_watts: f64) -> f64 {
        let psat_watts = 10.0_f64.powf((self.psat_dbm - 30.0) / 10.0);
        let drive_fraction = (pout_watts / psat_watts).clamp(0.0, 1.0);
        let pae = self.max_pae * drive_fraction;
        if pout_watts > pin_watts {
            pae
        } else {
            0.0
        }
    }
}

/// Fully integrated first-principles discrete radio transmitter.
#[derive(Debug, Clone, PartialEq)]
pub struct DiscreteTransmitter {
    /// Human-readable identifier.
    pub name: String,
    /// Physical 3D location $(X, Y, Z)$ in meters.
    pub position: Vector3D,
    /// Resonant frequency generator stage.
    pub oscillator: OscillatorType,
    /// Power amplification stage.
    pub power_amplifier: RfPowerAmplifier,
    /// Transmitting antenna transducer.
    pub antenna: PhysicalAntenna,
}

impl DiscreteTransmitter {
    /// Assembles a new discrete RF transmitter.
    pub fn new(
        name: impl Into<String>,
        position: Vector3D,
        oscillator: OscillatorType,
        power_amplifier: RfPowerAmplifier,
        antenna: PhysicalAntenna,
    ) -> Self {
        Self {
            name: name.into(),
            position,
            oscillator,
            power_amplifier,
            antenna,
        }
    }

    /// Creates a 2.45 GHz Wi-Fi transmitter with Colpitts oscillator, Class-AB PA, and microstrip patch antenna.
    pub fn wifi_2_4ghz_patch(name: impl Into<String>, position: Vector3D) -> Self {
        let freq_hz = 2.45e9;
        // Colpitts oscillator: L = 2.1 nH, C1 = 4.0 pF, C2 = 4.0 pF => C_eq = 2.0 pF => f0 ~ 2.456 GHz
        let oscillator = OscillatorType::Colpitts {
            inductance_h: 2.108e-9,
            c1_f: 4.0e-12,
            c2_f: 4.0e-12,
            transconductance_s: 0.05, // 50 mS
            load_resistance_ohms: 50.0,
            bias_current_a: 0.02, // 20 mA
        };

        let pa = RfPowerAmplifier::wifi_class_ab();
        let antenna = PhysicalAntenna::microstrip_patch("Patch_2.45GHz", freq_hz, 4.4, 0.0016);

        Self {
            name: name.into(),
            position,
            oscillator,
            power_amplifier: pa,
            antenna,
        }
    }

    /// Creates a 433.92 MHz ISM band crystal-stabilized transmitter with quarter-wave monopole antenna.
    pub fn ism_433mhz_crystal(name: impl Into<String>, position: Vector3D) -> Self {
        let freq_hz = 433.92e6;
        let omega = 2.0 * PI * freq_hz;
        let lm = 15.0e-3; // 15 mH motional inductance
        let cm = 1.0 / (omega * omega * lm);
        let oscillator = OscillatorType::Crystal {
            motional_inductance_h: lm,
            motional_capacitance_f: cm,
            motional_resistance_ohms: 25.0,
            shunt_capacitance_f: 4.5e-12,
            drive_level_watts: 0.0005, // 0.5 mW
        };

        let pa = RfPowerAmplifier::switching_class_e();
        let antenna = PhysicalAntenna::quarter_wave_monopole("Monopole_433MHz", freq_hz);

        Self {
            name: name.into(),
            position,
            oscillator,
            power_amplifier: pa,
            antenna,
        }
    }

    /// Operating carrier frequency in Hz.
    #[inline]
    pub fn carrier_frequency_hz(&self) -> f64 {
        self.oscillator.oscillation_frequency_hz()
    }

    /// Computes terminal RF power delivered into the antenna in Watts.
    pub fn power_into_antenna_watts(&self) -> f64 {
        let p_osc_watts = self.oscillator.output_power_watts();
        let p_osc_dbm = 10.0 * (p_osc_watts * 1000.0).max(1e-12).log10();
        let (_, pout_watts) = self.power_amplifier.amplify(p_osc_dbm);
        pout_watts
    }

    /// Peak terminal RF current amplitude $I_{ant}$ in Amperes into the antenna.
    pub fn antenna_terminal_current_a(&self) -> f64 {
        let p_in = self.power_into_antenna_watts();
        let r_in = self.antenna.radiation_resistance_ohms() + self.antenna.loss_resistance_ohms();
        // P = 1/2 * I_peak^2 * R_in => I_peak = sqrt(2 * P / R_in)
        (2.0 * p_in / r_in.max(1e-6)).sqrt()
    }

    /// Net radiated electromagnetic power $P_{rad} = \eta_{rad} \cdot P_{in}$ in Watts.
    #[inline]
    pub fn radiated_power_watts(&self) -> f64 {
        self.power_into_antenna_watts() * self.antenna.radiation_efficiency()
    }

    /// Equivalent Isotropically Radiated Power (EIRP) in Watts along the peak boresight axis.
    #[inline]
    pub fn eirp_boresight_watts(&self) -> f64 {
        self.power_into_antenna_watts() * self.antenna.max_gain_linear()
    }

    /// Converts this discrete synthesized radio transmitter into an `EmWaveSource`
    /// compatible with the Phase 29 EM wave propagation and space link solvers.
    pub fn to_em_wave_source(&self, boresight: Vector3D) -> EmWaveSource {
        let p_tx = self.power_into_antenna_watts();
        let gain_linear = self.antenna.max_gain_linear();
        EmWaveSource::new(
            self.carrier_frequency_hz(),
            p_tx,
            gain_linear,
            self.antenna.polarization,
            self.position,
            boresight,
        )
    }

    /// Instantaneous terminal current waveform $I_{ant}(t) = I_0 \cos(2\pi f_c t + \phi_0)$.
    #[inline]
    pub fn instantaneous_current(&self, time_s: f64, phase_rad: f64) -> f64 {
        let i0 = self.antenna_terminal_current_a();
        let omega = 2.0 * PI * self.carrier_frequency_hz();
        i0 * (omega * time_s + phase_rad).cos()
    }
}
