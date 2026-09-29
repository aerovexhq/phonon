//! Dark Matter Axion Haloscope Quantum-Limited Readout Solver.
//!
//! Solves axion-to-photon Sikivie resonant conversion, Dicke radiometer equation,
//! KITWPA vs HEMT system noise temperatures, signal-to-noise ratio (SNR),
//! and scan rate acceleration factor > 100x.

use phonon_models::kitwpa::{AxionModel, HaloscopeCavity, KitwpaTransmissionLine};

/// Comprehensive haloscope readout analysis result.
#[derive(Debug, Clone, PartialEq)]
pub struct HaloscopeReadoutAnalysis {
    /// Converted microwave signal frequency in Hz.
    pub signal_frequency_hz: f64,
    /// Converted axion power in Watts.
    pub converted_power_watts: f64,
    /// Converted axion power in dBm.
    pub converted_power_dbm: f64,
    /// KITWPA power gain at signal frequency.
    pub kitwpa_power_gain: f64,
    /// KITWPA power gain in dB.
    pub kitwpa_power_gain_db: f64,
    /// Caves added quantum noise quanta N_add.
    pub added_noise_quanta: f64,
    /// KITWPA added noise temperature T_add in Kelvin.
    pub added_noise_temp_k: f64,
    /// Total KITWPA haloscope system noise temperature T_sys,KITWPA in Kelvin.
    pub kitwpa_system_temp_k: f64,
    /// Conventional cryogenic HEMT system noise temperature T_sys,HEMT in Kelvin.
    pub hemt_system_temp_k: f64,
    /// Haloscope frequency scan rate speedup factor (T_sys,HEMT / T_sys,KITWPA)^2.
    pub scan_rate_speedup: f64,
    /// Axion virial resonance linewidth Delta nu_a in Hz.
    pub axion_linewidth_hz: f64,
    /// Dicke radiometer integration time in seconds required to reach target SNR.
    pub required_integration_time_sec: f64,
    /// Achieved SNR for a reference 1.0 second integration time.
    pub snr_reference_1s: f64,
}

/// Haloscope quantum readout solver.
#[derive(Debug, Default, Clone)]
pub struct HaloscopeReadoutSolver;

impl HaloscopeReadoutSolver {
    pub fn new() -> Self {
        Self
    }

    /// Evaluates quantum-limited haloscope readout performance.
    pub fn analyze_readout(
        &self,
        cavity: &HaloscopeCavity,
        axion: &AxionModel,
        kitwpa: &KitwpaTransmissionLine,
        hemt_noise_temp_k: f64,
        target_snr: f64,
    ) -> HaloscopeReadoutAnalysis {
        let f_a = axion.axion_frequency_hz();
        let p_a = axion.converted_signal_power_watts(cavity);
        let p_dbm = 10.0 * (p_a / 1e-3).max(1e-30).log10();

        // KITWPA gain at axion frequency
        let gain = kitwpa.analytical_signal_power_gain(f_a, true);
        let gain_db = 10.0 * gain.max(1.0).log10();

        // Caves added noise quanta and noise temperature
        let n_add = axion.caves_added_noise_quanta(gain);
        let t_add = axion.quantum_noise_temperature_k(f_a, gain);

        // System noise temperatures:
        // T_sys = T_cavity + T_amp
        let kitwpa_sys_temp = cavity.physical_temperature_k + t_add;
        let hemt_sys_temp = cavity.physical_temperature_k + hemt_noise_temp_k;

        let speedup = axion.scan_rate_speedup_factor(hemt_sys_temp, kitwpa_sys_temp);

        // Axion dark matter linewidth: Q_a ~ 10^6 due to Galactic virial dispersion v ~ 10^-3 c:
        // Delta nu_a = f_a * (v/c)^2 ~ 10^-6 * f_a
        let delta_nu_a = f_a * 1.0e-6;

        // Dicke radiometer formula: SNR = (P_a / (k_B * T_sys)) * sqrt(tau / Delta nu_a)
        let kb = 1.380_649e-23;
        let snr_coeff = p_a / (kb * kitwpa_sys_temp * delta_nu_a.sqrt());
        let snr_1s = snr_coeff * (1.0f64).sqrt();

        // tau = ((target_snr * kb * T_sys) / P_a)^2 * delta_nu_a
        let target = target_snr.max(1e-6);
        let req_tau = ((target * kb * kitwpa_sys_temp) / p_a).powi(2) * delta_nu_a;

        HaloscopeReadoutAnalysis {
            signal_frequency_hz: f_a,
            converted_power_watts: p_a,
            converted_power_dbm: p_dbm,
            kitwpa_power_gain: gain,
            kitwpa_power_gain_db: gain_db,
            added_noise_quanta: n_add,
            added_noise_temp_k: t_add,
            kitwpa_system_temp_k: kitwpa_sys_temp,
            hemt_system_temp_k: hemt_sys_temp,
            scan_rate_speedup: speedup,
            axion_linewidth_hz: delta_nu_a,
            required_integration_time_sec: req_tau,
            snr_reference_1s: snr_1s,
        }
    }
}
