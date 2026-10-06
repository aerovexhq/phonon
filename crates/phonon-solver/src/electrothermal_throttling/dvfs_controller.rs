#![deny(unsafe_code)]

use super::thermal_runaway::{
    calculate_dynamic_power, calculate_leakage_power, DynamicPowerParams, LeakageModelParams,
    T_ZERO_CELSIUS_K,
};

/// Hardware Performance State (P-State) definition.
#[derive(Debug, Clone)]
pub struct PState {
    pub name: String,
    pub frequency_ghz: f64,
    pub voltage_v: f64,
}

impl PState {
    pub fn new(name: &str, frequency_ghz: f64, voltage_v: f64) -> Self {
        Self {
            name: name.to_string(),
            frequency_ghz,
            voltage_v,
        }
    }
}

/// Dynamic Voltage and Frequency Scaling (DVFS) configuration and thermal throttling parameters.
#[derive(Debug, Clone)]
pub struct DvfsControllerConfig {
    /// Minimum allowed operating frequency (GHz).
    pub f_min_ghz: f64,
    /// Maximum boost operating frequency (GHz).
    pub f_max_ghz: f64,
    /// Minimum core supply voltage (V).
    pub v_min_v: f64,
    /// Maximum core supply voltage (V).
    pub v_max_v: f64,
    /// Target regulated junction temperature (Celsius), e.g. 85.0 C.
    pub target_temp_c: f64,
    /// Critical thermal throttling trip point (Celsius), e.g. 100.0 C.
    pub critical_temp_c: f64,
    /// Emergency thermal shutdown threshold (Celsius), e.g. 115.0 C.
    pub shutdown_temp_c: f64,
    /// Proportional gain Kp (GHz/K).
    pub kp: f64,
    /// Integral gain Ki (GHz/(K*s)).
    pub ki: f64,
    /// Integrator anti-windup clamp limit (GHz).
    pub integral_clamp_ghz: f64,
    /// Minimum clock gating duty cycle under critical throttling [0.1, 1.0].
    pub min_clock_gating_duty: f64,
    /// Discrete P-States table.
    pub p_states: Vec<PState>,
}

impl Default for DvfsControllerConfig {
    fn default() -> Self {
        Self {
            f_min_ghz: 1.2,
            f_max_ghz: 4.8,
            v_min_v: 0.75,
            v_max_v: 1.25,
            target_temp_c: 85.0,
            critical_temp_c: 98.0,
            shutdown_temp_c: 115.0,
            kp: 0.05,
            ki: 0.015,
            integral_clamp_ghz: 3.0,
            min_clock_gating_duty: 0.25,
            p_states: vec![
                PState::new("P0 (Turbo Boost)", 4.8, 1.25),
                PState::new("P1 (Nominal High)", 4.0, 1.10),
                PState::new("P2 (Balanced)", 3.2, 0.98),
                PState::new("P3 (Low Power)", 2.4, 0.88),
                PState::new("P4 (Throttled)", 1.6, 0.78),
            ],
        }
    }
}

impl DvfsControllerConfig {
    /// Computes supply voltage for a given operating frequency via linear/polynomial voltage curve.
    pub fn voltage_for_frequency(&self, frequency_ghz: f64) -> f64 {
        let f_clamped = frequency_ghz.clamp(self.f_min_ghz, self.f_max_ghz);
        let ratio = (f_clamped - self.f_min_ghz) / (self.f_max_ghz - self.f_min_ghz).max(0.1);
        self.v_min_v + ratio * (self.v_max_v - self.v_min_v)
    }

    /// Finds the closest matching discrete P-state for a given frequency.
    pub fn select_p_state(&self, frequency_ghz: f64) -> usize {
        let mut best_idx = 0;
        let mut min_diff = f64::MAX;
        for (i, p) in self.p_states.iter().enumerate() {
            let diff = (p.frequency_ghz - frequency_ghz).abs();
            if diff < min_diff {
                min_diff = diff;
                best_idx = i;
            }
        }
        best_idx
    }
}

/// Instantaneous telemetry step record during dynamic transient simulation.
#[derive(Debug, Clone)]
pub struct TransientStepRecord {
    pub time_ms: f64,
    pub junction_temp_c: f64,
    pub target_temp_c: f64,
    pub frequency_ghz: f64,
    pub voltage_v: f64,
    pub dynamic_power_w: f64,
    pub leakage_power_w: f64,
    pub total_power_w: f64,
    pub clock_gating_duty: f64,
    pub workload_activity: f64,
    pub p_state_idx: usize,
    pub is_throttling: bool,
}

/// Complete results of a closed-loop transient electro-thermal simulation.
#[derive(Debug, Clone)]
pub struct TransientSimulationResult {
    pub time_history_ms: Vec<f64>,
    pub temp_history_c: Vec<f64>,
    pub target_temp_history_c: Vec<f64>,
    pub frequency_history_ghz: Vec<f64>,
    pub voltage_history_v: Vec<f64>,
    pub power_history_w: Vec<f64>,
    pub leakage_history_w: Vec<f64>,
    pub duty_history: Vec<f64>,
    pub workload_history: Vec<f64>,
    pub peak_temperature_c: f64,
    pub steady_state_temp_c: f64,
    pub steady_state_frequency_ghz: f64,
    pub steady_state_power_w: f64,
    pub thermal_runaway_prevented: bool,
}

/// Workload profile for transient simulation.
#[derive(Debug, Clone)]
pub struct WorkloadProfile {
    /// Baseline idle activity factor [0.0, 1.0].
    pub baseline_activity: f64,
    /// Peak compute burst activity factor [0.0, 1.0].
    pub burst_activity: f64,
    /// Time when the burst starts in milliseconds.
    pub burst_start_ms: f64,
    /// Time when the burst ends in milliseconds.
    pub burst_end_ms: f64,
}

impl Default for WorkloadProfile {
    fn default() -> Self {
        Self {
            baseline_activity: 0.25,
            burst_activity: 0.95,
            burst_start_ms: 5.0,
            burst_end_ms: 45.0,
        }
    }
}

impl WorkloadProfile {
    pub fn activity_at_time(&self, time_ms: f64) -> f64 {
        if time_ms >= self.burst_start_ms && time_ms <= self.burst_end_ms {
            self.burst_activity
        } else {
            self.baseline_activity
        }
    }
}

/// Executes dynamic closed-loop electro-thermal transient simulation with PI thermal throttling.
pub fn run_closed_loop_transient(
    config: &DvfsControllerConfig,
    leak_params: &LeakageModelParams,
    dyn_params: &DynamicPowerParams,
    workload: &WorkloadProfile,
    ambient_temp_c: f64,
    total_r_th_kw: f64,
    die_thermal_mass_j_k: f64,
    duration_ms: f64,
    time_step_ms: f64,
) -> TransientSimulationResult {
    let t_amb_k = ambient_temp_c + T_ZERO_CELSIUS_K;
    let total_steps = ((duration_ms / time_step_ms).ceil() as usize).max(10);
    let dt_s = time_step_ms * 1e-3;

    let mut time_vec = Vec::with_capacity(total_steps);
    let mut temp_vec = Vec::with_capacity(total_steps);
    let mut target_vec = Vec::with_capacity(total_steps);
    let mut freq_vec = Vec::with_capacity(total_steps);
    let mut volt_vec = Vec::with_capacity(total_steps);
    let mut power_vec = Vec::with_capacity(total_steps);
    let mut leak_vec = Vec::with_capacity(total_steps);
    let mut duty_vec = Vec::with_capacity(total_steps);
    let mut work_vec = Vec::with_capacity(total_steps);

    let mut current_temp_k = t_amb_k + 5.0; // Start slightly warm
    let mut current_freq_ghz = config.f_max_ghz;
    let mut integral_error = 0.0;
    let mut peak_temp_c = ambient_temp_c;

    for step in 0..total_steps {
        let t_ms = (step as f64) * time_step_ms;
        let t_c = current_temp_k - T_ZERO_CELSIUS_K;
        if t_c > peak_temp_c {
            peak_temp_c = t_c;
        }

        let activity = workload.activity_at_time(t_ms);

        // Dynamic PI Closed-Loop Throttling
        let temp_error = t_c - config.target_temp_c;
        if temp_error > 0.0 {
            integral_error = (integral_error + temp_error * dt_s).clamp(-config.integral_clamp_ghz, config.integral_clamp_ghz);
        } else {
            // Cool-down unwinding
            integral_error = (integral_error + 0.5 * temp_error * dt_s).max(0.0);
        }

        let throttle_reduction = if temp_error > 0.0 {
            config.kp * temp_error + config.ki * integral_error
        } else {
            0.0
        };

        current_freq_ghz = (config.f_max_ghz - throttle_reduction).clamp(config.f_min_ghz, config.f_max_ghz);
        let current_voltage = config.voltage_for_frequency(current_freq_ghz);

        // Hardware Fast Clock Gating when exceeding critical temperature
        let clock_duty = if t_c >= config.critical_temp_c {
            let over = (t_c - config.critical_temp_c) / (config.shutdown_temp_c - config.critical_temp_c).max(1.0);
            (1.0 - over * (1.0 - config.min_clock_gating_duty)).clamp(config.min_clock_gating_duty, 1.0)
        } else {
            1.0
        };

        // Instantaneous power calculation
        let mut active_dyn_params = dyn_params.clone();
        active_dyn_params.activity_factor = activity;

        let p_dyn = calculate_dynamic_power(
            &active_dyn_params,
            current_voltage,
            current_freq_ghz * 1e9,
            clock_duty,
        );
        let p_leak = calculate_leakage_power(leak_params, current_temp_k, current_voltage);
        let p_tot = p_dyn + p_leak;

        // Thermal ODE Euler integration: C_th * dT/dt = P_tot - (T - T_amb) / R_th
        let heat_removal = (current_temp_k - t_amb_k) / total_r_th_kw.max(0.001);
        let net_heat_rate = p_tot - heat_removal;
        let d_temp = (net_heat_rate / die_thermal_mass_j_k.max(0.001)) * dt_s;
        current_temp_k += d_temp;

        // Record history
        time_vec.push(t_ms);
        temp_vec.push(t_c);
        target_vec.push(config.target_temp_c);
        freq_vec.push(current_freq_ghz);
        volt_vec.push(current_voltage);
        power_vec.push(p_tot);
        leak_vec.push(p_leak);
        duty_vec.push(clock_duty);
        work_vec.push(activity);
    }

    let final_temp_c = current_temp_k - T_ZERO_CELSIUS_K;
    let final_freq = current_freq_ghz;
    let final_power = power_vec.last().copied().unwrap_or(0.0);
    let runaway_prevented = peak_temp_c < config.shutdown_temp_c;

    TransientSimulationResult {
        time_history_ms: time_vec,
        temp_history_c: temp_vec,
        target_temp_history_c: target_vec,
        frequency_history_ghz: freq_vec,
        voltage_history_v: volt_vec,
        power_history_w: power_vec,
        leakage_history_w: leak_vec,
        duty_history: duty_vec,
        workload_history: work_vec,
        peak_temperature_c: peak_temp_c,
        steady_state_temp_c: final_temp_c,
        steady_state_frequency_ghz: final_freq,
        steady_state_power_w: final_power,
        thermal_runaway_prevented: runaway_prevented,
    }
}
