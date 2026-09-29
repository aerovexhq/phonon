#![deny(unsafe_code)]

//! Physical parameters and multi-physics evaluation metrics for quantum
//! phonon-exciton polariton condensates and chiral optomechanical transducers.

/// Physical parameter configuration for hybrid semiconductor-piezoelectric microcavities.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhononExcitonPolaritonParams {
    /// Optical cavity resonance frequency in THz (clamp 350.0 to 450.0, default 375.0).
    pub optical_cavity_freq_thz: f64,
    /// Coherent acoustic phonon frequency in GHz (clamp 2.0 to 20.0, default 7.0).
    pub acoustic_phonon_freq_ghz: f64,
    /// Exciton binding energy in meV (clamp 5.0 to 60.0, default 28.0).
    pub exciton_binding_energy_mev: f64,
    /// Vacuum Rabi splitting energy in meV (clamp 2.0 to 30.0, default 12.0).
    pub rabi_splitting_energy_mev: f64,
    /// Piezoelectric deformation potential coupling rate in MHz (clamp 10.0 to 120.0, default 55.0).
    pub piezo_deform_coupling_mhz: f64,
    /// Optical pump power in mW (clamp 0.2 to 10.0, default 2.5).
    pub optical_pump_power_mw: f64,
    /// Operating cryostat bath temperature in Kelvin (clamp 0.01 to 4.0, default 0.30).
    pub operating_temp_k: f64,
    /// Distributed Bragg reflector (DBR) cavity quality factor (clamp 1.0e4 to 1.0e6, default 1.5e5).
    pub cavity_quality_factor: f64,
}

impl Default for PhononExcitonPolaritonParams {
    fn default() -> Self {
        Self {
            optical_cavity_freq_thz: 375.0,
            acoustic_phonon_freq_ghz: 7.0,
            exciton_binding_energy_mev: 28.0,
            rabi_splitting_energy_mev: 12.0,
            piezo_deform_coupling_mhz: 55.0,
            optical_pump_power_mw: 2.5,
            operating_temp_k: 0.30,
            cavity_quality_factor: 1.5e5,
        }
    }
}

impl PhononExcitonPolaritonParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        optical_cavity_freq_thz: f64,
        acoustic_phonon_freq_ghz: f64,
        exciton_binding_energy_mev: f64,
        rabi_splitting_energy_mev: f64,
        piezo_deform_coupling_mhz: f64,
        optical_pump_power_mw: f64,
        operating_temp_k: f64,
        cavity_quality_factor: f64,
    ) -> Self {
        Self {
            optical_cavity_freq_thz: optical_cavity_freq_thz.clamp(350.0, 450.0),
            acoustic_phonon_freq_ghz: acoustic_phonon_freq_ghz.clamp(2.0, 20.0),
            exciton_binding_energy_mev: exciton_binding_energy_mev.clamp(5.0, 60.0),
            rabi_splitting_energy_mev: rabi_splitting_energy_mev.clamp(2.0, 30.0),
            piezo_deform_coupling_mhz: piezo_deform_coupling_mhz.clamp(10.0, 120.0),
            optical_pump_power_mw: optical_pump_power_mw.clamp(0.2, 10.0),
            operating_temp_k: operating_temp_k.clamp(0.01, 4.0),
            cavity_quality_factor: cavity_quality_factor.clamp(1.0e4, 1.0e6),
        }
    }
}

/// Multi-physics performance evaluation metrics for quantum phonon-exciton polariton condensates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhononExcitonPolaritonMetrics {
    /// Quantum state transfer fidelity (target >= 0.9940).
    pub quantum_state_fidelity: f64,
    /// Polariton Bose-Einstein condensation threshold pump power in mW (target <= 1.200).
    pub condensation_threshold_pump_mw: f64,
    /// First-order temporal polariton coherence time in picoseconds (target >= 25.0).
    pub polariton_coherence_time_ps: f64,
    /// Chiral vortex quantized topological charge (target == 1).
    pub vortex_topological_charge: i32,
    /// Optomechanical-polariton coupling rate in MHz (target >= 40.0).
    pub optomechanical_coupling_rate_mhz: f64,
    /// Overall physical compliance flag across all roadmap targets.
    pub is_physically_compliant: bool,
}
