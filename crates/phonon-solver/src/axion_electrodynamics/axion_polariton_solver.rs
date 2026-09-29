//! Solvers for quantum axion electrodynamics, axion-polariton dispersion,
//! Witten effect anomalous Hall conductivity, and dark matter haloscope transducers.

use phonon_models::axion_electrodynamics::axion_constants::*;
use phonon_models::axion_electrodynamics::{AxionConversionMetrics, AxionElectrodynamicsParams};

/// Solver for quantum axion electrodynamics and axion-polariton interactions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxionPolaritonSolver {
    pub params: AxionElectrodynamicsParams,
}

impl AxionPolaritonSolver {
    /// Creates a new axion-polariton solver instance.
    pub fn new(params: AxionElectrodynamicsParams) -> Self {
        Self { params }
    }

    /// Evaluates the axion resonance frequency $\nu_a = m_a c^2 / h$ in $\text{GHz}$.
    pub fn compute_resonance_frequency_ghz(&self) -> f64 {
        // m_a in ueV -> Joules: (m_a * 1e-6) * e
        let mass_joules = self.params.axion_mass_uev * 1.0e-6 * ELEMENTARY_CHARGE;
        let freq_hz = mass_joules / H_PLANCK;
        freq_hz / 1.0e9
    }

    /// Evaluates the galactic halo axion line virial bandwidth $\Delta \nu_a \approx 10^{-6} \nu_a$ in $\text{kHz}$.
    pub fn compute_virial_bandwidth_khz(&self) -> f64 {
        let freq_ghz = self.compute_resonance_frequency_ghz();
        // Delta nu_a / nu_a ~ (v_virial / c)^2 ~ 1e-6
        // freq_ghz * 1e6 kHz * 1e-6 = freq_ghz in kHz
        freq_ghz
    }

    /// Evaluates the Witten effect anomalous Hall conductance $\sigma_{xy} = \frac{e^2}{h} \frac{\theta}{2\pi}$ in Siemens.
    pub fn compute_witten_hall_conductance_siemens(&self) -> f64 {
        let theta = self.params.theta_angle_rad;
        QUANTUM_CONDUCTANCE * (theta / (2.0 * std::f64::consts::PI))
    }

    /// Evaluates the Witten induced fractional electric charge on a magnetic monopole $q / e = \theta / (2\pi)$.
    pub fn compute_witten_monopole_charge_fraction(&self) -> f64 {
        self.params.theta_angle_rad / (2.0 * std::f64::consts::PI)
    }

    /// Evaluates the axion-to-photon converted signal power $P_{\mathrm{conv}}$ in Watts.
    pub fn compute_conversion_power_watts(&self) -> f64 {
        let p = &self.params;
        // Standard haloscope cavity power scaling:
        // P_0 = (g_agg / 1e-13)^2 * (rho / 0.45) * (20 / m_a) * (B / 8.0)^2 * (V / 0.01) * (C / 0.58) * (Q / 80,000) * [4 beta / (1 + beta)^2]
        let g_norm = p.axion_photon_coupling_gev_inv / 1.0e-13;
        let rho_norm = p.local_dark_matter_density_gev_cm3 / 0.45;
        let m_norm = 20.0 / p.axion_mass_uev;
        let b_norm = p.magnetic_field_tesla / 8.0;
        let v_norm = p.cavity_volume_m3 / 0.010;
        let c_norm = p.form_factor_c010 / 0.58;
        let q_norm = p.cavity_q_factor / 80_000.0;
        let beta = p.cavity_coupling_beta;
        let beta_factor = (4.0 * beta) / ((1.0 + beta) * (1.0 + beta));
        let topo_norm = p.topological_polariton_enhancement / 5_000.0;

        let base_power_watts = 2.5e-22; // Natural baseline haloscope power ~ 0.25 zW
        let enhanced_power = base_power_watts
            * (g_norm * g_norm)
            * rho_norm
            * m_norm
            * (b_norm * b_norm)
            * v_norm
            * c_norm
            * q_norm
            * beta_factor
            * topo_norm
            * 5_000.0; // Metamaterial polariton enhancement factor

        enhanced_power.max(1.0e-26)
    }

    /// Evaluates the haloscope signal-to-noise ratio (SNR) in decibels using Dicke's radiometer equation.
    pub fn compute_snr_db(&self) -> f64 {
        let p_conv = self.compute_conversion_power_watts();
        let bandwidth_hz = self.compute_virial_bandwidth_khz() * 1.0e3;
        let tau = self.params.integration_time_s.max(1.0e-3);
        let t_sys = self.params.system_noise_temp_k.max(0.01);

        // Dicke radiometer noise fluctuation: sigma_P = k_B * T_sys * sqrt(Delta nu_a / tau)
        let noise_power_watts = K_BOLTZMANN * t_sys * (bandwidth_hz / tau).sqrt();
        let snr_linear = p_conv / noise_power_watts.max(1.0e-30);
        10.0 * snr_linear.max(1.0).log10()
    }

    /// Evaluates the axion-polariton anti-crossing gap in $\text{GHz}$.
    pub fn compute_polariton_gap_ghz(&self) -> f64 {
        let p = &self.params;
        let b0 = p.magnetic_field_tesla;
        let g_norm = p.axion_photon_coupling_gev_inv / 1.0e-13;
        let topo_factor = (p.topological_polariton_enhancement / 5_000.0).sqrt();

        // Effective magnetoelectric coupling gap:
        // Delta omega_pol / 2pi ~ 0.18 * B0 * g_norm * topo_factor + 0.85 GHz
        (0.18 * b0 * g_norm * topo_factor + 0.85).max(0.20)
    }

    /// Solves the full quantum axion electrodynamics metrics.
    pub fn solve(&self) -> AxionConversionMetrics {
        let p_watts = self.compute_conversion_power_watts();
        let p_dbm = 10.0 * (p_watts * 1.0e3).max(1.0e-30).log10();
        let snr_db = self.compute_snr_db();
        let hall_cond = self.compute_witten_hall_conductance_siemens();
        let charge_frac = self.compute_witten_monopole_charge_fraction();
        let gap_ghz = self.compute_polariton_gap_ghz();
        let nu_ghz = self.compute_resonance_frequency_ghz();
        let bwidth_khz = self.compute_virial_bandwidth_khz();

        AxionConversionMetrics {
            conversion_power_watts: p_watts,
            conversion_power_dbm: p_dbm,
            snr_db,
            witten_anomalous_hall_conductance_siemens: hall_cond,
            witten_charge_fraction: charge_frac,
            polariton_gap_ghz: gap_ghz,
            resonance_frequency_ghz: nu_ghz,
            axion_line_bandwidth_khz: bwidth_khz,
        }
    }

    /// Evaluates upper and lower axion-polariton dispersion frequencies $\omega_\pm(k)$ in $\text{GHz}$.
    pub fn evaluate_polariton_branches(&self, wavevector_k_inv_m: f64) -> (f64, f64) {
        let nu_0 = self.compute_resonance_frequency_ghz();
        let delta_gap = self.compute_polariton_gap_ghz();
        let photon_freq =
            (SPEED_OF_LIGHT * wavevector_k_inv_m / (2.0 * std::f64::consts::PI)) / 1.0e9;

        let avg = 0.5 * (photon_freq + nu_0);
        let diff = photon_freq - nu_0;
        let splitting = 0.5 * (diff * diff + delta_gap * delta_gap).sqrt();

        (avg + splitting, (avg - splitting).max(0.01))
    }
}
