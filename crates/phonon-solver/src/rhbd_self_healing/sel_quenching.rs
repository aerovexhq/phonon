#![deny(unsafe_code)]

//! Sub-Microsecond Single Event Latchup (SEL) Electronic Crowbar Quenching Co-Simulator.
//!
//! Models parasitic p-n-p-n thyristor regenerative latchup dynamics and ultra-fast
//! autonomous electronic crowbar power-rail cutoff (< 50 ns) to extinguish latchup
//! before thermal junction destruction.

/// Physical parameters of the parasitic CMOS p-n-p-n thyristor structure.
#[derive(Debug, Clone)]
pub struct ParasiticThyristorParams {
    /// Nominal power rail supply voltage V_dd in Volts (e.g. 3.3 V or 1.8 V).
    pub vdd_v: f64,
    /// Holding voltage V_h in Volts below which regenerative latchup extinguishes (e.g. 1.2 V).
    pub holding_voltage_v: f64,
    /// Holding current I_h in Amperes below which latchup drops out (e.g. 0.015 A).
    pub holding_current_a: f64,
    /// Critical heavy ion deposited charge Q_crit in picocoulombs to trigger latchup.
    pub trigger_charge_pc: f64,
    /// Parasitic thyristor internal ON-state resistance in Ohms (e.g. 0.85 Ohm).
    pub parasitic_on_resistance_ohm: f64,
    /// Effective local junction thermal resistance R_th in K/W.
    pub thermal_resistance_k_per_w: f64,
    /// Effective local junction thermal capacitance C_th in J/K.
    pub thermal_capacitance_j_per_k: f64,
    /// Ambient operating temperature in deg C.
    pub ambient_temperature_c: f64,
}

impl Default for ParasiticThyristorParams {
    fn default() -> Self {
        Self {
            vdd_v: 3.3,
            holding_voltage_v: 1.2,
            holding_current_a: 0.015,
            trigger_charge_pc: 1.5,
            parasitic_on_resistance_ohm: 0.85,
            thermal_resistance_k_per_w: 120.0,
            thermal_capacitance_j_per_k: 5.0e-9,
            ambient_temperature_c: 25.0,
        }
    }
}

/// Dynamic control parameters of the fast autonomous on-chip electronic crowbar.
#[derive(Debug, Clone)]
pub struct ElectronicCrowbarParams {
    /// Overcurrent sensing threshold in Amperes (e.g. 0.200 A).
    pub overcurrent_threshold_a: f64,
    /// Current sensor comparator propagation delay tau_det in nanoseconds (e.g. 10.0 ns).
    pub detection_delay_ns: f64,
    /// Power rail series cutoff switch and shunt crowbar activation delay tau_cutoff in ns (e.g. 25.0 ns).
    pub cutoff_switch_delay_ns: f64,
    /// Shunt crowbar conducting channel resistance in Ohms (e.g. 0.05 Ohm).
    pub crowbar_clamp_resistance_ohm: f64,
    /// Autonomous recovery delay before restoring power rail in microseconds.
    pub recovery_hold_time_us: f64,
}

impl Default for ElectronicCrowbarParams {
    fn default() -> Self {
        Self {
            overcurrent_threshold_a: 0.200,
            detection_delay_ns: 10.0,
            cutoff_switch_delay_ns: 25.0,
            crowbar_clamp_resistance_ohm: 0.05,
            recovery_hold_time_us: 10.0,
        }
    }
}

impl ElectronicCrowbarParams {
    /// Total autonomous quenching response latency in nanoseconds (< 50 ns required).
    pub fn total_quenching_latency_ns(&self) -> f64 {
        self.detection_delay_ns + self.cutoff_switch_delay_ns
    }
}

/// Instantaneous simulation state sample at time step t.
#[derive(Debug, Clone, PartialEq)]
pub struct SelTransientPoint {
    /// Time relative to ion strike in nanoseconds.
    pub time_ns: f64,
    /// Power rail voltage in Volts.
    pub v_rail_v: f64,
    /// Total supply current drawn by the thyristor in Amperes.
    pub i_rail_a: f64,
    /// Local silicon junction temperature in deg C.
    pub t_junction_c: f64,
    /// Whether the parasitic thyristor is currently in regenerative latchup state.
    pub is_latched: bool,
    /// Whether the electronic crowbar is actively clamping/cutoff.
    pub crowbar_active: bool,
}

/// Comprehensive results from co-simulating fast SEL quenching vs unquenched baseline.
#[derive(Debug, Clone)]
pub struct SelSimulationResult {
    /// Time-series trajectory with autonomous electronic crowbar protection enabled.
    pub quenched_points: Vec<SelTransientPoint>,
    /// Time-series trajectory without crowbar protection (unquenched baseline).
    pub unquenched_points: Vec<SelTransientPoint>,
    /// Peak junction temperature achieved with crowbar in deg C.
    pub quenched_peak_temp_c: f64,
    /// Peak junction temperature achieved without crowbar in deg C.
    pub unquenched_peak_temp_c: f64,
    /// Total elapsed time in nanoseconds to extinguish latchup.
    pub quenching_latency_ns: f64,
    /// Total electrical energy dissipated in the micro-junction with crowbar in microjoules.
    pub energy_dissipated_uj: f64,
    /// Whether catastrophic thermal burnout (> 350 deg C) was successfully averted.
    pub burnout_prevented: bool,
}

/// Simulator engine for parasitic thyristor latchup and electronic crowbar quenching.
#[derive(Debug, Clone)]
pub struct SelQuenchingSimulator {
    /// Parasitic thyristor model parameters.
    pub thyristor: ParasiticThyristorParams,
    /// Electronic crowbar detection and cutoff parameters.
    pub crowbar: ElectronicCrowbarParams,
}

impl Default for SelQuenchingSimulator {
    fn default() -> Self {
        Self {
            thyristor: ParasiticThyristorParams::default(),
            crowbar: ElectronicCrowbarParams::default(),
        }
    }
}

impl SelQuenchingSimulator {
    /// Creates a new simulator with specified thyristor and crowbar parameters.
    pub fn new(thyristor: ParasiticThyristorParams, crowbar: ElectronicCrowbarParams) -> Self {
        Self { thyristor, crowbar }
    }

    /// Simulates the electro-thermal transient response following a heavy ion strike.
    ///
    /// Evaluates both protected (crowbar enabled) and unprotected (unquenched) trajectories
    /// over duration `duration_ns` with time step `dt_ns`.
    pub fn simulate_transient(
        &self,
        ion_strike_charge_pc: f64,
        duration_ns: f64,
        dt_ns: f64,
    ) -> SelSimulationResult {
        let dt = dt_ns.max(0.1);
        let duration = duration_ns.max(50.0);
        let steps = (duration / dt).round() as usize;

        let strike_triggers = ion_strike_charge_pc >= self.thyristor.trigger_charge_pc;
        let quench_latency = self.crowbar.total_quenching_latency_ns();

        // 1. Protected Trajectory (with fast electronic crowbar)
        let mut quenched_points = Vec::with_capacity(steps + 1);
        let mut t_j_quenched = self.thyristor.ambient_temperature_c;
        let mut latched_quenched = strike_triggers;
        let mut crowbar_active = false;
        let mut total_energy_j: f64 = 0.0;
        let mut peak_temp_quenched = t_j_quenched;

        for step in 0..=steps {
            let t_ns = step as f64 * dt;

            // Crowbar triggers after detection delay + cutoff delay
            if latched_quenched && t_ns >= quench_latency {
                crowbar_active = true;
            }

            let (v_rail, i_rail) = if !latched_quenched {
                // Quiescent operating state
                (self.thyristor.vdd_v, 0.005)
            } else if crowbar_active {
                // Shunt crowbar drops rail below holding voltage
                let v = self.thyristor.holding_voltage_v * 0.25;
                let i = v / (self.thyristor.parasitic_on_resistance_ohm + self.crowbar.crowbar_clamp_resistance_ohm);
                // Thyristor extinguishes when rail drops below holding voltage
                if v < self.thyristor.holding_voltage_v {
                    latched_quenched = false;
                }
                (v, i)
            } else {
                // Active latchup state before crowbar fires
                let v = self.thyristor.vdd_v;
                let i = (v - self.thyristor.holding_voltage_v) / self.thyristor.parasitic_on_resistance_ohm;
                (v, i)
            };

            // Electro-thermal dissipation: P = V * I
            let p_diss = v_rail * i_rail;
            let dt_s = dt * 1.0e-9;
            total_energy_j += p_diss * dt_s;

            // Thermal rate: dT/dt = (P - (T - T_amb)/R_th) / C_th
            let q_out = (t_j_quenched - self.thyristor.ambient_temperature_c) / self.thyristor.thermal_resistance_k_per_w;
            let dt_j = ((p_diss - q_out) / self.thyristor.thermal_capacitance_j_per_k) * dt_s;
            t_j_quenched += dt_j;

            if t_j_quenched > peak_temp_quenched {
                peak_temp_quenched = t_j_quenched;
            }

            quenched_points.push(SelTransientPoint {
                time_ns: t_ns,
                v_rail_v: v_rail,
                i_rail_a: i_rail,
                t_junction_c: t_j_quenched,
                is_latched: latched_quenched,
                crowbar_active,
            });
        }

        // 2. Unprotected Trajectory (unquenched baseline)
        let mut unquenched_points = Vec::with_capacity(steps + 1);
        let mut t_j_unquenched = self.thyristor.ambient_temperature_c;
        let mut latched_unquenched = strike_triggers;
        let mut peak_temp_unquenched = t_j_unquenched;

        for step in 0..=steps {
            let t_ns = step as f64 * dt;

            let (v_rail, i_rail) = if latched_unquenched {
                let v = self.thyristor.vdd_v;
                let i = (v - self.thyristor.holding_voltage_v) / self.thyristor.parasitic_on_resistance_ohm;
                (v, i)
            } else {
                (self.thyristor.vdd_v, 0.005)
            };

            let p_diss = v_rail * i_rail;
            let dt_s = dt * 1.0e-9;

            let q_out = (t_j_unquenched - self.thyristor.ambient_temperature_c) / self.thyristor.thermal_resistance_k_per_w;
            let dt_j = ((p_diss - q_out) / self.thyristor.thermal_capacitance_j_per_k) * dt_s;
            t_j_unquenched += dt_j;

            // Junction destruction at 400 deg C clamped
            if t_j_unquenched > 450.0 {
                t_j_unquenched = 450.0;
                latched_unquenched = false; // burnout open/melt
            }

            if t_j_unquenched > peak_temp_unquenched {
                peak_temp_unquenched = t_j_unquenched;
            }

            unquenched_points.push(SelTransientPoint {
                time_ns: t_ns,
                v_rail_v: v_rail,
                i_rail_a: i_rail,
                t_junction_c: t_j_unquenched,
                is_latched: latched_unquenched,
                crowbar_active: false,
            });
        }

        let burnout_prevented = peak_temp_quenched < 150.0 && quench_latency < 50.0 && peak_temp_unquenched > peak_temp_quenched;

        SelSimulationResult {
            quenched_points,
            unquenched_points,
            quenched_peak_temp_c: peak_temp_quenched,
            unquenched_peak_temp_c: peak_temp_unquenched,
            quenching_latency_ns: quench_latency,
            energy_dissipated_uj: total_energy_j * 1.0e6,
            burnout_prevented,
        }
    }
}
