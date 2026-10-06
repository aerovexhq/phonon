#![deny(unsafe_code)]

//! Superconducting Josephson Parametric Acoustic Waveguide Model.
//!
//! Provides multi-physics simulation of:
//! - SQUID loop flux-tunable Josephson inductance L_J(Phi) = Phi_0 / (2 * pi * I_c * |cos(pi * Phi / Phi_0)|).
//! - Total effective lumped inductance L_eff(Phi) = L_geo + L_J(Phi).
//! - Tunable plasma resonance frequency omega_0(Phi) = 1.0 / sqrt(L_eff(Phi) * C_shunt).
//! - Strong piezoelectric transduction coupling g_piezo to acoustic waveguide modes.
//! - Parametric flux pumping at omega_p ~ 2 * omega_0.

use std::f64::consts::PI;

/// Magnetic flux quantum Phi_0 = h / (2 * e) in Weber (V * s).
pub const FLUX_QUANTUM_WB: f64 = 2.067_833_848e-15;

/// Reduced Planck constant hbar in J * s.
pub const HBAR_J_S: f64 = 1.054_571_817e-34;

/// Boltzmann constant k_B in J / K.
pub const BOLTZMANN_K_J_K: f64 = 1.380_649e-23;

/// Dimensionless vacuum standard quantum limit (SQL) quadrature variance.
pub const VACUUM_SQL_VARIANCE: f64 = 0.5;

/// Physical configuration parameters for the Josephson Parametric Waveguide.
#[derive(Debug, Clone, PartialEq)]
pub struct JpaWaveguideParams {
    /// Bare acoustic/cavity resonance frequency in GHz (default ~6.0 GHz).
    pub bare_frequency_ghz: f64,
    /// SQUID junction critical current I_c in microamperes (default ~2.5 uA).
    pub critical_current_ua: f64,
    /// Shunt capacitance C_shunt in picofarads (default ~0.8 pF).
    pub shunt_capacitance_pf: f64,
    /// Geometric / stray inductance L_geo in picohenries (default ~50.0 pH).
    pub geometric_inductance_ph: f64,
    /// Electromechanical piezoelectric coupling rate g_piezo / 2pi in MHz (default ~15.0 MHz).
    pub piezo_coupling_mhz: f64,
    /// Normalized external magnetic flux bias ratio Phi / Phi_0 in [-0.45, 0.45] (default 0.25).
    pub flux_bias_ratio: f64,
    /// Microwave parametric pump frequency in GHz (default ~12.0 GHz, ~ 2 * omega_0).
    pub pump_frequency_ghz: f64,
    /// Normalized parametric pump drive power ratio in [0.0, 0.95] (default 0.70).
    pub pump_power_ratio: f64,
    /// 3-dB instantaneous bandwidth in MHz (default ~80.0 MHz).
    pub bandwidth_3db_mhz: f64,
}

impl Default for JpaWaveguideParams {
    fn default() -> Self {
        Self {
            bare_frequency_ghz: 6.0,
            critical_current_ua: 2.5,
            shunt_capacitance_pf: 0.8,
            geometric_inductance_ph: 50.0,
            piezo_coupling_mhz: 15.0,
            flux_bias_ratio: 0.25,
            pump_frequency_ghz: 12.0,
            pump_power_ratio: 0.70,
            bandwidth_3db_mhz: 80.0,
        }
    }
}

impl JpaWaveguideParams {
    /// Computes the current resonant frequency in GHz using the analytical Josephson inductance model.
    pub fn resonant_frequency_ghz(&self) -> f64 {
        JosephsonInductanceModel::resonant_frequency_ghz(self)
    }

    /// Computes the total effective inductance in picohenries (pH).
    pub fn effective_inductance_ph(&self) -> f64 {
        JosephsonInductanceModel::effective_inductance_ph(self)
    }

    /// Computes the tunable Josephson inductance in picohenries (pH).
    pub fn josephson_inductance_ph(&self) -> f64 {
        JosephsonInductanceModel::josephson_inductance_ph(self.critical_current_ua, self.flux_bias_ratio)
    }

    /// Computes plasma frequency tuning range Delta_f = f_0(0.0) - f_0(0.45) in GHz.
    pub fn tuning_range_ghz(&self) -> f64 {
        JosephsonInductanceModel::frequency_tuning_range_ghz(self)
    }
}

/// Analytical SQUID loop Josephson inductance and plasma resonance model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct JosephsonInductanceModel;

impl JosephsonInductanceModel {
    /// Computes Josephson inductance L_J(Phi) in Henries for a given critical current (uA) and flux bias ratio.
    ///
    /// L_J(Phi) = Phi_0 / (2 * pi * I_c * |cos(pi * Phi / Phi_0)|)
    pub fn josephson_inductance_h(critical_current_ua: f64, flux_bias_ratio: f64) -> f64 {
        let i_c_amps = (critical_current_ua.max(1e-3)) * 1e-6;
        let l_0 = FLUX_QUANTUM_WB / (2.0 * PI * i_c_amps);
        let cos_phi = (PI * flux_bias_ratio).cos().abs().max(0.01);
        l_0 / cos_phi
    }

    /// Computes Josephson inductance L_J(Phi) in picohenries (pH).
    pub fn josephson_inductance_ph(critical_current_ua: f64, flux_bias_ratio: f64) -> f64 {
        Self::josephson_inductance_h(critical_current_ua, flux_bias_ratio) * 1e12
    }

    /// Computes total effective inductance L_eff(Phi) = L_geo + L_J(Phi) in Henries.
    pub fn effective_inductance_h(params: &JpaWaveguideParams) -> f64 {
        let l_geo_h = params.geometric_inductance_ph * 1e-12;
        let l_j_h = Self::josephson_inductance_h(params.critical_current_ua, params.flux_bias_ratio);
        l_geo_h + l_j_h
    }

    /// Computes total effective inductance in picohenries (pH).
    pub fn effective_inductance_ph(params: &JpaWaveguideParams) -> f64 {
        Self::effective_inductance_h(params) * 1e12
    }

    /// Computes total effective inductance at a specific flux bias ratio in picohenries (pH).
    pub fn effective_inductance_ph_at_flux(params: &JpaWaveguideParams, flux_ratio: f64) -> f64 {
        let l_geo_h = params.geometric_inductance_ph * 1e-12;
        let l_j_h = Self::josephson_inductance_h(params.critical_current_ua, flux_ratio);
        (l_geo_h + l_j_h) * 1e12
    }

    /// Computes tunable plasma resonance angular frequency omega_0(Phi) = 1.0 / sqrt(L_eff * C_shunt) in rad/s.
    pub fn plasma_frequency_rad_s(params: &JpaWaveguideParams) -> f64 {
        let l_eff = Self::effective_inductance_h(params);
        let c_shunt = (params.shunt_capacitance_pf.max(1e-6)) * 1e-12;
        1.0 / (l_eff * c_shunt).sqrt()
    }

    /// Computes tunable plasma resonance frequency f_0(Phi) = omega_0 / (2 * pi) in GHz.
    pub fn resonant_frequency_ghz(params: &JpaWaveguideParams) -> f64 {
        let omega = Self::plasma_frequency_rad_s(params);
        (omega / (2.0 * PI)) * 1e-9
    }

    /// Computes resonant frequency at a specific flux bias ratio in GHz.
    pub fn frequency_at_flux(params: &JpaWaveguideParams, flux_ratio: f64) -> f64 {
        let mut p = params.clone();
        p.flux_bias_ratio = flux_ratio;
        Self::resonant_frequency_ghz(&p)
    }

    /// Evaluates frequency tuning curve across flux bias Phi / Phi_0 in [-0.45, 0.45].
    pub fn compute_tuning_curve(params: &JpaWaveguideParams, num_points: usize) -> Vec<[f64; 2]> {
        let n = num_points.max(2);
        let mut curve = Vec::with_capacity(n);
        let min_flux = -0.45;
        let max_flux = 0.45;
        for i in 0..n {
            let frac = i as f64 / (n - 1) as f64;
            let flux = min_flux + (max_flux - min_flux) * frac;
            let f = Self::frequency_at_flux(params, flux);
            curve.push([flux, f]);
        }
        curve
    }

    /// Evaluates inductance tuning curve across flux bias Phi / Phi_0 in [-0.45, 0.45] in pH.
    pub fn compute_inductance_curve(params: &JpaWaveguideParams, num_points: usize) -> Vec<[f64; 2]> {
        let n = num_points.max(2);
        let mut curve = Vec::with_capacity(n);
        let min_flux = -0.45;
        let max_flux = 0.45;
        for i in 0..n {
            let frac = i as f64 / (n - 1) as f64;
            let flux = min_flux + (max_flux - min_flux) * frac;
            let l_ph = Self::effective_inductance_ph_at_flux(params, flux);
            curve.push([flux, l_ph]);
        }
        curve
    }

    /// Computes plasma frequency tuning range Delta_f = f_0(0.0) - f_0(0.45) in GHz.
    pub fn frequency_tuning_range_ghz(params: &JpaWaveguideParams) -> f64 {
        let f_max = Self::frequency_at_flux(params, 0.0);
        let f_min = Self::frequency_at_flux(params, 0.45);
        (f_max - f_min).abs()
    }
}
