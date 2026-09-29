#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for chiral phononic
//! Floquet-SBT gauge fields and dissipationless acoustic topological Hall transistors.

/// Physical parameter configuration for chiral phononic Floquet-SBT topological Hall transistors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralFloquetHallTransistorParams {
    /// Floquet periodic modulation amplitude in MHz (clamp 5.0 to 100.0, default 35.0 MHz).
    pub floquet_modulation_amplitude_mhz: f64,
    /// Floquet driving microwave frequency in GHz (clamp 1.0 to 15.0, default 4.6 GHz).
    pub floquet_drive_frequency_ghz: f64,
    /// Strain-induced Brillouin zone torsion gradient in ppm/um (clamp 10.0 to 300.0, default 85.0 ppm/um).
    pub strain_torsion_gradient_ppm_per_um: f64,
    /// Chiral valley-phonon coupling rate in MHz (clamp 1.0 to 50.0, default 18.0 MHz).
    pub chiral_valley_coupling_mhz: f64,
    /// Transistor gate electrostatic potential in Volts (clamp 0.1 to 10.0, default 2.5 V).
    pub transistor_gate_voltage_v: f64,
    /// Acoustic topological Hall transport channel length in micrometers (clamp 0.5 to 20.0, default 4.2 um).
    pub channel_length_um: f64,
    /// Dilution cryogenic operating temperature in milli-Kelvin (clamp 1.0 to 50.0, default 15.0 mK).
    pub cryogenic_temperature_mk: f64,
    /// Piezoelectric electromechanical coupling coefficient k^2 (clamp 0.01 to 0.25, default 0.08).
    pub piezoelectric_electromechanical_coupling: f64,
}

impl Default for ChiralFloquetHallTransistorParams {
    fn default() -> Self {
        Self {
            floquet_modulation_amplitude_mhz: 35.0,
            floquet_drive_frequency_ghz: 4.6,
            strain_torsion_gradient_ppm_per_um: 85.0,
            chiral_valley_coupling_mhz: 18.0,
            transistor_gate_voltage_v: 2.5,
            channel_length_um: 4.2,
            cryogenic_temperature_mk: 15.0,
            piezoelectric_electromechanical_coupling: 0.08,
        }
    }
}

impl ChiralFloquetHallTransistorParams {
    /// Creates a new parameter configuration with rigorous physical bounds clamping.
    pub fn new(
        floquet_modulation_amplitude_mhz: f64,
        floquet_drive_frequency_ghz: f64,
        strain_torsion_gradient_ppm_per_um: f64,
        chiral_valley_coupling_mhz: f64,
        transistor_gate_voltage_v: f64,
        channel_length_um: f64,
        cryogenic_temperature_mk: f64,
        piezoelectric_electromechanical_coupling: f64,
    ) -> Self {
        Self {
            floquet_modulation_amplitude_mhz: floquet_modulation_amplitude_mhz.clamp(5.0, 100.0),
            floquet_drive_frequency_ghz: floquet_drive_frequency_ghz.clamp(1.0, 15.0),
            strain_torsion_gradient_ppm_per_um: strain_torsion_gradient_ppm_per_um.clamp(10.0, 300.0),
            chiral_valley_coupling_mhz: chiral_valley_coupling_mhz.clamp(1.0, 50.0),
            transistor_gate_voltage_v: transistor_gate_voltage_v.clamp(0.1, 10.0),
            channel_length_um: channel_length_um.clamp(0.5, 20.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            piezoelectric_electromechanical_coupling: piezoelectric_electromechanical_coupling.clamp(0.01, 0.25),
        }
    }
}

/// Multi-physics evaluation metrics for chiral phononic Floquet-SBT topological Hall transistors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralFloquetHallTransistorMetrics {
    /// Valley Hall contrast ratio between forward chiral modes and reverse/opposite valley in dB (target >= 35.0 dB).
    pub valley_hall_contrast_ratio_db: f64,
    /// Topological gate switching transit and reconfiguration time in nanoseconds (target <= 15.0 ns).
    pub topological_switching_time_ns: f64,
    /// Cross-talk isolation between adjacent Hall edge channels in dB (target >= 40.0 dB).
    pub cross_talk_isolation_db: f64,
    /// Non-adiabatic insertion loss in acoustic topological channels in dB (target <= 0.60 dB).
    pub non_adiabatic_insertion_loss_db: f64,
    /// Overall topological Hall transistor state switching fidelity (target >= 0.9960).
    pub hall_transistor_state_fidelity: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
