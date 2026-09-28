//! Coupled electro-thermal differential-algebraic equation (DAE) solver for SNSPD.
//!
//! Integrates current diversion, normal hotspot domain growth, Joule heating,
//! Kapitza boundary cooling, and exponential inductive reset.

use phonon_models::snspd::{HotspotDynamicsModel, NanowireGeometry};

/// Configuration options for the electro-thermal transient simulation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElectroThermalConfig {
    /// Time step dt in picoseconds (typically 1.0 - 5.0 ps).
    pub time_step_ps: f64,
    /// Total simulation duration in picoseconds (typically 5000 - 15000 ps).
    pub total_time_ps: f64,
    /// Time when the photon is absorbed in picoseconds.
    pub photon_absorption_time_ps: f64,
    /// Photon wavelength in nanometers.
    pub wavelength_nm: f64,
    /// Bias ratio alpha_bias = I_b / I_c (typically 0.85 - 0.95).
    pub bias_ratio: f64,
}

impl Default for ElectroThermalConfig {
    fn default() -> Self {
        Self {
            time_step_ps: 2.0,
            total_time_ps: 8000.0,
            photon_absorption_time_ps: 200.0,
            wavelength_nm: 1550.0,
            bias_ratio: 0.90,
        }
    }
}

/// Results of the electro-thermal transient simulation.
#[derive(Debug, Clone, PartialEq)]
pub struct SnspdPulseTrace {
    /// Time points in picoseconds.
    pub time_ps: Vec<f64>,
    /// Nanowire current I(t) in microAmperes.
    pub current_ua: Vec<f64>,
    /// Output voltage V_out(t) across load R_L in milliVolts: V_out = (I_b - I) * R_L.
    pub voltage_out_mv: Vec<f64>,
    /// Hotspot / normal domain length x_n(t) in nanometers.
    pub hotspot_length_nm: Vec<f64>,
    /// Hotspot temperature T(t) in Kelvin.
    pub temperature_k: Vec<f64>,
    /// Peak output voltage in milliVolts.
    pub peak_voltage_mv: f64,
    /// Rise time (10% to 90% of peak voltage) in picoseconds.
    pub rise_time_ps: f64,
    /// Full width at half maximum (FWHM) of the voltage pulse in picoseconds.
    pub pulse_fwhm_ps: f64,
    /// Whether a full resistive breakdown occurred.
    pub triggered: bool,
}

/// Coupled electro-thermal transient solver.
pub struct ElectroThermalSolver {
    pub geom: NanowireGeometry,
    pub hotspot: HotspotDynamicsModel,
    pub config: ElectroThermalConfig,
}

impl ElectroThermalSolver {
    pub fn new(
        geom: NanowireGeometry,
        hotspot: HotspotDynamicsModel,
        config: ElectroThermalConfig,
    ) -> Self {
        Self {
            geom,
            hotspot,
            config,
        }
    }

    /// Solves the transient response to an absorbed photon.
    pub fn solve(&self) -> SnspdPulseTrace {
        let dt_ps = self.config.time_step_ps;
        let dt_s = dt_ps * 1e-12;
        let num_steps = (self.config.total_time_ps / dt_ps).ceil() as usize;

        let i_b_ua = self.geom.bias_current_ua(self.config.bias_ratio);
        let l_k_h = self.geom.kinetic_inductance_henries();
        let r_l = self.geom.load_impedance_ohms;
        let t_sub = self.geom.substrate_temperature_k;
        let i_retrap_ua = self.hotspot.retrapping_current_ua(&self.geom);

        let mut time_ps = Vec::with_capacity(num_steps);
        let mut current_ua = Vec::with_capacity(num_steps);
        let mut voltage_out_mv = Vec::with_capacity(num_steps);
        let mut hotspot_length_nm = Vec::with_capacity(num_steps);
        let mut temperature_k = Vec::with_capacity(num_steps);

        let mut curr_i_ua = i_b_ua;
        let mut curr_xn_nm = 0.0_f64;
        let mut curr_temp_k = t_sub;
        let mut hotspot_active = false;
        let mut ever_triggered = false;

        let mut peak_v_mv = 0.0_f64;

        for step in 0..num_steps {
            let t_ps = step as f64 * dt_ps;
            time_ps.push(t_ps);

            // Check photon arrival
            if !hotspot_active
                && (t_ps - self.config.photon_absorption_time_ps).abs() < dt_ps * 0.51
            {
                let triggered = self.hotspot.triggers_resistive_barrier(
                    &self.geom,
                    curr_i_ua,
                    self.config.wavelength_nm,
                );
                if triggered {
                    hotspot_active = true;
                    ever_triggered = true;
                    let r_init_nm = self.hotspot.initial_hotspot_radius_nm();
                    curr_xn_nm = (2.0 * r_init_nm).max(850.0); // Non-equilibrium quasiparticle slab spans ~850 nm
                    curr_temp_k = self.geom.critical_temperature_k * 1.5;
                }
            }

            // Normal domain resistance
            let r_hs = if hotspot_active {
                self.hotspot
                    .normal_domain_resistance_ohms(&self.geom, curr_xn_nm)
            } else {
                0.0
            };

            // Output voltage: V_out = (I_b - I) * R_L in Volts
            let diverted_current_ua = (i_b_ua - curr_i_ua).max(0.0);
            let v_out_v = (diverted_current_ua * 1e-6) * r_l;
            let v_out_mv = v_out_v * 1000.0;
            if v_out_mv > peak_v_mv {
                peak_v_mv = v_out_mv;
            }

            current_ua.push(curr_i_ua);
            voltage_out_mv.push(v_out_mv);
            hotspot_length_nm.push(curr_xn_nm);
            temperature_k.push(curr_temp_k);

            // Advance DAE system
            if hotspot_active {
                // Current dynamics: dI/dt = [R_L * (I_b - I) - I * R_hs] / L_k
                let i_amps = curr_i_ua * 1e-6;
                let i_b_amps = i_b_ua * 1e-6;
                let didt = (r_l * (i_b_amps - i_amps) - i_amps * r_hs) / l_k_h;
                let next_i_amps = (i_amps + didt * dt_s).max(0.0);
                curr_i_ua = next_i_amps * 1e6;

                // Domain growth velocity: dx_n / dt = v_norm * (I / I_r - 1)
                let v_domain = self
                    .hotspot
                    .domain_growth_velocity_m_s(curr_i_ua, i_retrap_ua);
                let dx_nm = (v_domain * dt_s) * 1e9;
                curr_xn_nm = (curr_xn_nm + dx_nm).max(0.0);

                // Thermal balance: C_th * dT/dt = P_J - P_cool
                let p_j = self.hotspot.joule_heating_power_watts(curr_i_ua, r_hs);
                let p_cool =
                    self.hotspot
                        .substrate_cooling_power_watts(&self.geom, curr_xn_nm, curr_temp_k);
                let vol_m3 = (curr_xn_nm * 1e-9)
                    * (self.geom.width_nm * 1e-9)
                    * (self.geom.thickness_nm * 1e-9);
                let heat_cap = (self.hotspot.volumetric_heat_capacity_j_m3_k * vol_m3).max(1e-22);
                let d_temp = ((p_j - p_cool) / heat_cap) * dt_s;
                curr_temp_k = (curr_temp_k + d_temp).clamp(t_sub, 30.0);

                // Domain collapse condition: if current diverts below I_retrap and domain shrinks to 0
                if curr_xn_nm <= 1.0
                    || (curr_i_ua < i_retrap_ua && curr_temp_k < self.geom.critical_temperature_k)
                {
                    hotspot_active = false;
                    curr_xn_nm = 0.0;
                    curr_temp_k = t_sub;
                }
            } else {
                // Hotspot inactive: inductive recovery towards I_b
                // dI/dt = R_L * (I_b - I) / L_k
                let i_amps = curr_i_ua * 1e-6;
                let i_b_amps = i_b_ua * 1e-6;
                let didt = (r_l * (i_b_amps - i_amps)) / l_k_h;
                let next_i_amps = (i_amps + didt * dt_s).min(i_b_amps);
                curr_i_ua = next_i_amps * 1e6;
                curr_temp_k = t_sub;
                curr_xn_nm = 0.0;
            }
        }

        // Calculate rise time and FWHM
        let (rise_time_ps, pulse_fwhm_ps) =
            self.calculate_pulse_metrics(&time_ps, &voltage_out_mv, peak_v_mv);

        SnspdPulseTrace {
            time_ps,
            current_ua,
            voltage_out_mv,
            hotspot_length_nm,
            temperature_k,
            peak_voltage_mv: peak_v_mv,
            rise_time_ps,
            pulse_fwhm_ps,
            triggered: ever_triggered,
        }
    }

    fn calculate_pulse_metrics(&self, time_ps: &[f64], v_mv: &[f64], peak_mv: f64) -> (f64, f64) {
        if peak_mv <= 1e-3 || v_mv.is_empty() {
            return (0.0, 0.0);
        }

        let v10 = 0.10 * peak_mv;
        let v50 = 0.50 * peak_mv;
        let v90 = 0.90 * peak_mv;

        let mut t10 = None;
        let mut t50_lead = None;
        let mut t90 = None;
        let mut t50_trail = None;

        for (i, &v) in v_mv.iter().enumerate() {
            let t = time_ps[i];
            if t10.is_none() && v >= v10 {
                t10 = Some(t);
            }
            if t50_lead.is_none() && v >= v50 {
                t50_lead = Some(t);
            }
            if t90.is_none() && v >= v90 {
                t90 = Some(t);
            }
            if t90.is_some() && v < v50 && t50_trail.is_none() {
                t50_trail = Some(t);
            }
        }

        let rise_time = match (t10, t90) {
            (Some(t_start), Some(t_end)) => (t_end - t_start).max(0.0),
            _ => 0.0,
        };

        let fwhm = match (t50_lead, t50_trail) {
            (Some(t_lead), Some(t_trail)) => (t_trail - t_lead).max(0.0),
            _ => 0.0,
        };

        (rise_time, fwhm)
    }
}
