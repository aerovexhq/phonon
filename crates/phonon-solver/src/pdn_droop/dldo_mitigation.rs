#![deny(unsafe_code)]

use super::dynamic_droop::{simulate_dynamic_droop, DynamicDroopResult, LoadStepProfile};
use super::impedance_profile::PdnNetworkParams;

/// Configuration for Digital Low-Dropout (DLDO) fast transient current injection array.
#[derive(Debug, Clone)]
pub struct DldoControllerParams {
    pub enabled: bool,
    /// Voltage droop threshold to trigger DLDO current injection in mV (e.g. 15.0 mV).
    pub trigger_threshold_mv: f64,
    /// Auxiliary supply rail voltage in Volts (e.g. 1.20 V).
    pub v_aux_v: f64,
    /// Peak injected current by the parallel PMOS transistor bank in Amperes (e.g. 350 A).
    pub max_injected_current_a: f64,
    /// Turn-on reaction delay in seconds (e.g. 300 ps = 3e-10 s).
    pub response_time_s: f64,
    /// Injection pulse duration in seconds (e.g. 8.0 ns = 8e-9 s).
    pub pulse_duration_s: f64,
}

impl Default for DldoControllerParams {
    fn default() -> Self {
        Self {
            enabled: true,
            trigger_threshold_mv: 18.0,
            v_aux_v: 1.20,
            max_injected_current_a: 380.0,
            response_time_s: 3.5e-10, // 350 ps fast comparator + driver delay
            pulse_duration_s: 7.5e-9,  // 7.5 ns active assistance window
        }
    }
}

/// Configuration for hardware Active Clock-Stretching droop mitigation.
#[derive(Debug, Clone)]
pub struct ClockStretchParams {
    pub enabled: bool,
    /// Droop threshold to trigger clock stretching in mV (e.g. 22.0 mV).
    pub trigger_threshold_mv: f64,
    /// Clock period elongation ratio (e.g. 0.40 -> +40% clock period, reducing frequency by ~28%).
    pub stretch_ratio: f64,
    /// Duration of clock stretching in seconds (e.g. 5.0 ns = 5e-9 s).
    pub duration_s: f64,
}

impl Default for ClockStretchParams {
    fn default() -> Self {
        Self {
            enabled: true,
            trigger_threshold_mv: 24.0,
            stretch_ratio: 0.45,
            duration_s: 6.0e-9, // 6.0 ns
        }
    }
}

/// Mitigation comparison report detailing droop reduction.
#[derive(Debug, Clone)]
pub struct MitigationReport {
    pub unmitigated_peak_droop_mv: f64,
    pub mitigated_peak_droop_mv: f64,
    pub droop_reduction_mv: f64,
    pub droop_reduction_pct: f64,

    pub unmitigated_1st_droop_mv: f64,
    pub mitigated_1st_droop_mv: f64,
    pub first_droop_reduction_pct: f64,

    pub dldo_peak_current_a: f64,
    pub dldo_energy_consumed_nj: f64,
    pub clock_stretching_triggered: bool,
    pub is_budget_restored: bool,
}

/// Simulated combined transient waveforms with active mitigation.
#[derive(Debug, Clone)]
pub struct MitigatedTransientResult {
    pub unmitigated: DynamicDroopResult,
    pub time_ns: Vec<f64>,
    pub v_mitigated_v: Vec<f64>,
    pub i_dldo_injected_a: Vec<f64>,
    pub clock_frequency_scale: Vec<f64>,
    pub report: MitigationReport,
}

/// Simulates and evaluates active hardware droop mitigation (DLDO + Clock Stretching).
pub fn simulate_mitigated_droop(
    params: &PdnNetworkParams,
    load: &LoadStepProfile,
    dldo: &DldoControllerParams,
    stretch: &ClockStretchParams,
    num_output_points: usize,
) -> MitigatedTransientResult {
    // 1. Run unmitigated baseline simulation
    let unmitigated = simulate_dynamic_droop(params, load, num_output_points);
    let v_nom = params.vrm.v_dd_v;
    let allowed_droop_v = v_nom * params.max_voltage_ripple_ratio;
    let allowed_droop_mv = allowed_droop_v * 1000.0;

    // Component parameters
    let c_die = params.total_on_die_capacitance_f().max(1e-9);
    let l_pkg = params.package_inductance_henry.max(1e-14);
    let r_pkg = params.package_resistance_ohm.max(1e-6);

    let c_pkg = params.total_package_capacitance_f().max(1e-8);
    let l_pcb = params.pcb_plane_inductance_henry.max(1e-14);
    let r_pcb = params.pcb_plane_resistance_ohm.max(1e-6);

    let c_bulk = params.total_bulk_capacitance_f().max(1e-6);
    let l_vrm = params.vrm.l_out_henry.max(1e-11);
    let r_vrm = params.vrm.r_dc_ohm.max(1e-6);
    let k_vrm_gain = params.vrm.loop_gain_a0 * 0.05;

    let esr_die = if !params.on_die_capacitors.is_empty() {
        let total_adm: f64 = params.on_die_capacitors.iter().map(|c| (c.count as f64) / c.esr_ohm.max(1e-6)).sum();
        1.0 / total_adm.max(1e-6)
    } else {
        1e-4
    };

    let esr_pkg = if !params.package_capacitors.is_empty() {
        let total_adm: f64 = params.package_capacitors.iter().map(|c| (c.count as f64) / c.esr_ohm.max(1e-6)).sum();
        1.0 / total_adm.max(1e-6)
    } else {
        1e-4
    };

    let esr_bulk = if !params.bulk_capacitors.is_empty() {
        let total_adm: f64 = params.bulk_capacitors.iter().map(|c| (c.count as f64) / c.esr_ohm.max(1e-6)).sum();
        1.0 / total_adm.max(1e-6)
    } else {
        1e-4
    };

    // State vector: capacitor charge voltages and branch currents
    let mut v_die_cap = v_nom - load.i_base_a * (params.on_die_grid_resistance_ohm + r_pkg + r_pcb + r_vrm);
    let mut i_pkg = load.i_base_a;
    let mut v_pkg_cap = v_nom - load.i_base_a * (r_pcb + r_vrm);
    let mut i_pcb = load.i_base_a;
    let mut v_bulk_cap = v_nom - load.i_base_a * r_vrm;
    let mut i_vrm = load.i_base_a;

    let dt_fine = 5.0e-12;
    let dt_coarse = 2.0e-10;
    let t_fine_cutoff = load.t_step_start_s + 5.0e-7;

    let mut current_t = 0.0f64;
    let t_end = load.t_duration_s;
    let sample_interval = t_end / (num_output_points.max(50) as f64);
    let mut next_sample_t = 0.0f64;

    let mut time_ns_vec = Vec::with_capacity(num_output_points);
    let mut v_mit_vec = Vec::with_capacity(num_output_points);
    let mut i_dldo_vec = Vec::with_capacity(num_output_points);
    let mut freq_scale_vec = Vec::with_capacity(num_output_points);

    // Active controller state
    let mut dldo_active = false;
    let mut dldo_trigger_t = 0.0f64;
    let mut dldo_energy_j = 0.0f64;
    let mut dldo_peak_i = 0.0f64;

    let mut stretch_active = false;
    let mut stretch_triggered = false;
    let mut stretch_trigger_t = 0.0f64;

    let mut min_mit_v1 = v_nom;
    let mut global_min_mit_v = v_nom;

    while current_t <= t_end {
        let dt = if current_t < t_fine_cutoff { dt_fine } else { dt_coarse };

        let raw_load = load.current_at(current_t);
        let curr_droop_mv = ((v_nom - v_die_cap) * 1000.0).max(0.0);

        // Clock stretch controller with hysteresis
        if stretch.enabled && current_t >= load.t_step_start_s {
            if !stretch_active && curr_droop_mv >= stretch.trigger_threshold_mv {
                stretch_active = true;
                stretch_triggered = true;
                stretch_trigger_t = current_t;
            } else if stretch_active {
                let stretch_elapsed = current_t - stretch_trigger_t;
                if stretch_elapsed > stretch.duration_s && curr_droop_mv < stretch.trigger_threshold_mv * 0.7 {
                    stretch_active = false;
                }
            }
        }

        let effective_load = if stretch_active {
            raw_load / (1.0 + stretch.stretch_ratio)
        } else {
            raw_load
        };

        let f_scale = if stretch_active {
            1.0 / (1.0 + stretch.stretch_ratio)
        } else {
            1.0
        };

        // Closed-loop fast DLDO current injection controller
        if dldo.enabled && current_t >= load.t_step_start_s {
            if !dldo_active && curr_droop_mv >= dldo.trigger_threshold_mv {
                dldo_active = true;
                dldo_trigger_t = current_t + dldo.response_time_s;
            } else if dldo_active {
                let dldo_elapsed = current_t - dldo_trigger_t;
                if dldo_elapsed > dldo.pulse_duration_s && curr_droop_mv < dldo.trigger_threshold_mv * 0.6 {
                    dldo_active = false;
                }
            }
        }

        let i_dldo = if dldo_active && current_t >= dldo_trigger_t {
            let demand_factor = ((curr_droop_mv - dldo.trigger_threshold_mv) / 10.0 + 0.5).clamp(0.2, 1.0);
            dldo.max_injected_current_a * demand_factor
        } else {
            0.0
        };

        if i_dldo > dldo_peak_i {
            dldo_peak_i = i_dldo;
        }
        dldo_energy_j += dldo.v_aux_v * i_dldo * dt;

        // Terminal voltages with ESR
        let i_c_die = i_pkg + i_dldo - effective_load;
        let v_die = v_die_cap + i_c_die * esr_die;

        let i_c_pkg = i_pcb - i_pkg;
        let v_pkg = v_pkg_cap + i_c_pkg * esr_pkg;

        let i_c_bulk = i_vrm - i_pcb;
        let v_bulk = v_bulk_cap + i_c_bulk * esr_bulk;

        // ODE integration with DLDO injected current directly into die node
        let dv_die = i_c_die / c_die;
        let di_pkg = (v_pkg - v_die - r_pkg * i_pkg) / l_pkg;
        let dv_pkg = i_c_pkg / c_pkg;
        let di_pcb = (v_bulk - v_pkg - r_pcb * i_pcb) / l_pcb;
        let dv_bulk = i_c_bulk / c_bulk;

        let v_err = (v_nom - v_bulk).clamp(-0.10, 0.10);
        let di_vrm = (v_err * (1.0 + k_vrm_gain) - r_vrm * i_vrm) / l_vrm;

        v_die_cap += dv_die * dt;
        i_pkg += di_pkg * dt;
        v_pkg_cap += dv_pkg * dt;
        i_pcb += di_pcb * dt;
        v_bulk_cap += dv_bulk * dt;
        i_vrm = (i_vrm + di_vrm * dt).max(0.0);

        current_t += dt;

        if current_t >= load.t_step_start_s {
            if v_die < global_min_mit_v {
                global_min_mit_v = v_die;
            }
            let rel_t = current_t - load.t_step_start_s;
            if rel_t <= 2.5e-8 && v_die < min_mit_v1 {
                min_mit_v1 = v_die;
            }
        }

        if current_t >= next_sample_t {
            time_ns_vec.push(current_t * 1e9);
            v_mit_vec.push(v_die);
            i_dldo_vec.push(i_dldo);
            freq_scale_vec.push(f_scale);
            next_sample_t += sample_interval;
        }
    }

    let unmit_peak_droop_mv = unmitigated.peak_overall_droop_mv;
    let mit_peak_droop_mv = ((v_nom - global_min_mit_v) * 1000.0).max(0.0);
    let droop_red_mv = (unmit_peak_droop_mv - mit_peak_droop_mv).max(0.0);
    let droop_red_pct = if unmit_peak_droop_mv > 0.01 {
        (droop_red_mv / unmit_peak_droop_mv) * 100.0
    } else {
        0.0
    };

    let unmit_1st_droop_mv = unmitigated.first_droop.peak_droop_mv;
    let mit_1st_droop_mv = ((v_nom - min_mit_v1) * 1000.0).max(0.0);
    let first_droop_red_pct = if unmit_1st_droop_mv > 0.01 {
        ((unmit_1st_droop_mv - mit_1st_droop_mv).max(0.0) / unmit_1st_droop_mv) * 100.0
    } else {
        0.0
    };

    let report = MitigationReport {
        unmitigated_peak_droop_mv: unmit_peak_droop_mv,
        mitigated_peak_droop_mv: mit_peak_droop_mv,
        droop_reduction_mv: droop_red_mv,
        droop_reduction_pct: droop_red_pct,
        unmitigated_1st_droop_mv: unmit_1st_droop_mv,
        mitigated_1st_droop_mv: mit_1st_droop_mv,
        first_droop_reduction_pct: first_droop_red_pct,
        dldo_peak_current_a: dldo_peak_i,
        dldo_energy_consumed_nj: dldo_energy_j * 1e9,
        clock_stretching_triggered: stretch_triggered,
        is_budget_restored: mit_peak_droop_mv <= allowed_droop_mv,
    };

    MitigatedTransientResult {
        unmitigated,
        time_ns: time_ns_vec,
        v_mitigated_v: v_mit_vec,
        i_dldo_injected_a: i_dldo_vec,
        clock_frequency_scale: freq_scale_vec,
        report,
    }
}
