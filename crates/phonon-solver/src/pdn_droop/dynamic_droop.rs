#![deny(unsafe_code)]

use super::impedance_profile::PdnNetworkParams;

/// Parameters defining a high di/dt load transient step.
#[derive(Debug, Clone)]
pub struct LoadStepProfile {
    /// Baseline quiescent core current in Amperes (e.g. 50 A).
    pub i_base_a: f64,
    /// Dynamic load step increase in Amperes (e.g. 800 A).
    pub delta_i_a: f64,
    /// 10%-90% or 0%-100% current rise time in seconds (e.g. 1.0 ns = 1e-9 s).
    pub t_rise_s: f64,
    /// Timestamp when the load step initiates in seconds (e.g. 10.0 ns = 1e-8 s).
    pub t_step_start_s: f64,
    /// Total duration of transient simulation in seconds (e.g. 10.0 us = 1e-5 s).
    pub t_duration_s: f64,
}

impl Default for LoadStepProfile {
    fn default() -> Self {
        Self {
            i_base_a: 50.0,
            delta_i_a: 800.0,
            t_rise_s: 1.0e-9,      // 1.0 ns rise time -> di/dt = 800 A / 1 ns = 8e11 A/s
            t_step_start_s: 1.0e-8, // 10 ns
            t_duration_s: 1.0e-5,  // 10 us (captures 1st, 2nd, and 3rd droops)
        }
    }
}

impl LoadStepProfile {
    /// Computes load current I_load(t) at any time t.
    pub fn current_at(&self, t_s: f64) -> f64 {
        if t_s < self.t_step_start_s {
            self.i_base_a
        } else if t_s < self.t_step_start_s + self.t_rise_s {
            let frac = (t_s - self.t_step_start_s) / self.t_rise_s.max(1e-15);
            self.i_base_a + self.delta_i_a * frac
        } else {
            self.i_base_a + self.delta_i_a
        }
    }

    /// Evaluates nominal di/dt in Amperes per second.
    pub fn di_dt_a_s(&self) -> f64 {
        self.delta_i_a / self.t_rise_s.max(1e-15)
    }
}

/// Dynamic metrics for a specific droop stage.
#[derive(Debug, Clone)]
pub struct DroopStageMetrics {
    pub name: String,
    pub peak_droop_mv: f64,
    pub minimum_voltage_v: f64,
    pub timestamp_s: f64,
    pub allowed_droop_mv: f64,
    pub is_within_budget: bool,
}

/// Complete results of dynamic droop transient simulation.
#[derive(Debug, Clone)]
pub struct DynamicDroopResult {
    /// Timestamps in nanoseconds (for plotting).
    pub time_ns: Vec<f64>,
    /// On-die core voltage V_die(t) in Volts.
    pub v_die_v: Vec<f64>,
    /// Package decoupling voltage V_pkg(t) in Volts.
    pub v_pkg_v: Vec<f64>,
    /// Board bulk voltage V_bulk(t) in Volts.
    pub v_bulk_v: Vec<f64>,
    /// Load current I_load(t) in Amperes.
    pub i_load_a: Vec<f64>,
    /// Package delivery current I_pkg(t) in Amperes.
    pub i_pkg_a: Vec<f64>,

    // Droop stage metrics
    pub first_droop: DroopStageMetrics,
    pub second_droop: DroopStageMetrics,
    pub third_droop: DroopStageMetrics,

    pub peak_overall_droop_mv: f64,
    pub nominal_v_dd_v: f64,
    pub is_compliant: bool,
}

/// Solves the multi-stage lumped RLC dynamic droop differential system.
pub fn simulate_dynamic_droop(
    params: &PdnNetworkParams,
    load: &LoadStepProfile,
    num_output_points: usize,
) -> DynamicDroopResult {
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
    let k_vrm_gain = params.vrm.loop_gain_a0 * 0.05; // VRM proportional feedback gain

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

    // Adaptive time stepping: fine dt around fast load step, coarser later
    let dt_fine = 5.0e-12; // 5 picoseconds for sub-nanosecond di/dt
    let dt_coarse = 2.0e-10; // 200 picoseconds for microsecond tail
    let t_fine_cutoff = load.t_step_start_s + 5.0e-7; // First 500 ns with ultra-fine stepping

    let mut current_t = 0.0f64;
    let t_end = load.t_duration_s;

    // Sampling intervals
    let sample_interval = t_end / (num_output_points.max(50) as f64);
    let mut next_sample_t = 0.0f64;

    let mut time_ns_vec = Vec::with_capacity(num_output_points);
    let mut v_die_vec = Vec::with_capacity(num_output_points);
    let mut v_pkg_vec = Vec::with_capacity(num_output_points);
    let mut v_bulk_vec = Vec::with_capacity(num_output_points);
    let mut i_load_vec = Vec::with_capacity(num_output_points);
    let mut i_pkg_vec = Vec::with_capacity(num_output_points);

    // Track minimum voltages in each window
    let mut min_v_droop1 = v_nom;
    let mut t_droop1 = load.t_step_start_s;

    let mut min_v_droop2 = v_nom;
    let mut t_droop2 = load.t_step_start_s + 5e-8;

    let mut min_v_droop3 = v_nom;
    let mut t_droop3 = load.t_step_start_s + 1e-6;

    let mut global_min_v = v_nom;

    while current_t <= t_end {
        let dt = if current_t < t_fine_cutoff {
            dt_fine
        } else {
            dt_coarse
        };

        let i_load = load.current_at(current_t);

        // Terminal voltages with ESR drops
        let i_c_die = i_pkg - i_load;
        let v_die = v_die_cap + i_c_die * esr_die;

        let i_c_pkg = i_pcb - i_pkg;
        let v_pkg = v_pkg_cap + i_c_pkg * esr_pkg;

        let i_c_bulk = i_vrm - i_pcb;
        let v_bulk = v_bulk_cap + i_c_bulk * esr_bulk;

        // System of 6 coupled ODEs with physical ESR damping
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

        // Droop window tracking
        if current_t >= load.t_step_start_s {
            let rel_t = current_t - load.t_step_start_s;
            if v_die < global_min_v {
                global_min_v = v_die;
            }

            // Window 1: 0 to 25 ns (1st Droop: Die/Package anti-resonance)
            if rel_t <= 2.5e-8 && v_die < min_v_droop1 {
                min_v_droop1 = v_die;
                t_droop1 = current_t;
            }
            // Window 2: 25 ns to 600 ns (2nd Droop: Package/Board anti-resonance)
            else if rel_t > 2.5e-8 && rel_t <= 6.0e-7 && v_die < min_v_droop2 {
                min_v_droop2 = v_die;
                t_droop2 = current_t;
            }
            // Window 3: 600 ns to 10 us (3rd Droop: VRM loop response)
            else if rel_t > 6.0e-7 && v_die < min_v_droop3 {
                min_v_droop3 = v_die;
                t_droop3 = current_t;
            }
        }

        // Output sampling
        if current_t >= next_sample_t {
            time_ns_vec.push(current_t * 1e9);
            v_die_vec.push(v_die);
            v_pkg_vec.push(v_pkg);
            v_bulk_vec.push(v_bulk);
            i_load_vec.push(i_load);
            i_pkg_vec.push(i_pkg);
            next_sample_t += sample_interval;
        }
    }

    let droop1_mv = ((v_nom - min_v_droop1) * 1000.0).max(0.0);
    let droop2_mv = ((v_nom - min_v_droop2) * 1000.0).max(0.0);
    let droop3_mv = ((v_nom - min_v_droop3) * 1000.0).max(0.0);
    let peak_droop_mv = ((v_nom - global_min_v) * 1000.0).max(0.0);

    let first_droop = DroopStageMetrics {
        name: "1st Droop (Package/Die LC Anti-Resonance)".to_string(),
        peak_droop_mv: droop1_mv,
        minimum_voltage_v: min_v_droop1,
        timestamp_s: t_droop1,
        allowed_droop_mv,
        is_within_budget: droop1_mv <= allowed_droop_mv,
    };

    let second_droop = DroopStageMetrics {
        name: "2nd Droop (Board/Package LC Anti-Resonance)".to_string(),
        peak_droop_mv: droop2_mv,
        minimum_voltage_v: min_v_droop2,
        timestamp_s: t_droop2,
        allowed_droop_mv,
        is_within_budget: droop2_mv <= allowed_droop_mv,
    };

    let third_droop = DroopStageMetrics {
        name: "3rd Droop (VRM Control Loop Response)".to_string(),
        peak_droop_mv: droop3_mv,
        minimum_voltage_v: min_v_droop3,
        timestamp_s: t_droop3,
        allowed_droop_mv,
        is_within_budget: droop3_mv <= allowed_droop_mv,
    };

    let is_compliant = peak_droop_mv <= allowed_droop_mv;

    DynamicDroopResult {
        time_ns: time_ns_vec,
        v_die_v: v_die_vec,
        v_pkg_v: v_pkg_vec,
        v_bulk_v: v_bulk_vec,
        i_load_a: i_load_vec,
        i_pkg_a: i_pkg_vec,
        first_droop,
        second_droop,
        third_droop,
        peak_overall_droop_mv: peak_droop_mv,
        nominal_v_dd_v: v_nom,
        is_compliant,
    }
}
