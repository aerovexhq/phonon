#![deny(unsafe_code)]

//! Monolithic Transient Electro-Thermal Solver coupling electrical circuit dynamics
//! with 2D dynamic floorplan heat diffusion mesh.
//!
//! Provides lockstep time-stepping, continuous Joule heat dissipation evaluation across
//! active semiconductor devices, local temperature-dependent companion updates, and
//! thermal runaway detection.

use crate::floorplan::DynamicFloorplanMesh;
use phonon_core::{CircuitGraph, NodeId};
use phonon_solver::mna::{solve_dc_non_linear, ModelContext, NewtonOptions};
use phonon_solver::SolverError;
use std::collections::HashMap;

/// Snapshot record of an electro-thermal co-simulation step.
#[derive(Debug, Clone, PartialEq)]
pub struct TransientStepRecord {
    /// Timestamp in seconds.
    pub time_s: f64,
    /// Electrical node voltages at this time step.
    pub node_voltages: Vec<f64>,
    /// Instantaneous power dissipation per component in Watts.
    pub component_powers: HashMap<String, f64>,
    /// Junction temperatures per placed component in Kelvin.
    pub component_temperatures: HashMap<String, f64>,
    /// Flattened 2D grid temperature field snapshot in Kelvin.
    pub grid_temperatures: Vec<f64>,
    /// Peak maximum temperature across the die in Kelvin.
    pub peak_temp_k: f64,
    /// Physical location (x, y) of the peak hotspot in meters.
    pub peak_pos: (f64, f64),
    /// Flag indicating whether thermal runaway threshold was breached.
    pub is_runaway: bool,
}

/// Recorded transient trajectory over time.
#[derive(Debug, Clone, PartialEq)]
pub struct ElectroThermalTransientTrajectory {
    /// Timestamps in seconds.
    pub timestamps: Vec<f64>,
    /// Step records.
    pub records: Vec<TransientStepRecord>,
    /// Flag indicating whether thermal runaway was triggered during the simulation.
    pub runaway_detected: bool,
    /// Identifier of the component that triggered thermal runaway, if any.
    pub runaway_component: Option<String>,
    /// Peak temperature observed anywhere on the die across all time steps.
    pub peak_die_temp_k: f64,
}

impl Default for ElectroThermalTransientTrajectory {
    fn default() -> Self {
        Self::new()
    }
}

impl ElectroThermalTransientTrajectory {
    /// Creates an empty transient trajectory container.
    pub fn new() -> Self {
        Self {
            timestamps: Vec::new(),
            records: Vec::new(),
            runaway_detected: false,
            runaway_component: None,
            peak_die_temp_k: 300.0,
        }
    }

    /// Number of recorded simulation steps.
    #[inline]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Returns true if no steps have been recorded yet.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Returns the most recently recorded step record.
    #[inline]
    pub fn last_record(&self) -> Option<&TransientStepRecord> {
        self.records.last()
    }

    /// Extracts temperature time series (time_s, temp_k) for the specified component.
    pub fn component_temperature_series(&self, name: &str) -> Vec<(f64, f64)> {
        self.records
            .iter()
            .filter_map(|r| r.component_temperatures.get(name).map(|&t| (r.time_s, t)))
            .collect()
    }

    /// Extracts power dissipation time series (time_s, power_w) for the specified component.
    pub fn component_power_series(&self, name: &str) -> Vec<(f64, f64)> {
        self.records
            .iter()
            .filter_map(|r| r.component_powers.get(name).map(|&p| (r.time_s, p)))
            .collect()
    }

    /// Extracts peak die temperature series (time_s, peak_temp_k).
    pub fn peak_temperature_series(&self) -> Vec<(f64, f64)> {
        self.records
            .iter()
            .map(|r| (r.time_s, r.peak_temp_k))
            .collect()
    }
}

/// Monolithic Transient Electro-Thermal Co-Simulator.
pub struct ElectroThermalCoSimulator {
    /// Electrical schematic circuit graph.
    pub graph: CircuitGraph,
    /// Dynamic floorplan thermal finite difference mesh.
    pub mesh: DynamicFloorplanMesh,
    /// Critical thermal runaway threshold temperature in Kelvin (default: 500 K).
    pub runaway_threshold_k: f64,
    /// Non-linear Newton solver convergence settings.
    pub newton_options: NewtonOptions,
    /// Current simulation time in seconds.
    pub current_time_s: f64,
    /// Complete recorded transient trajectory.
    pub trajectory: ElectroThermalTransientTrajectory,
}

impl ElectroThermalCoSimulator {
    /// Creates a new electro-thermal co-simulator coupling electrical circuit with floorplan mesh.
    pub fn new(graph: CircuitGraph, mesh: DynamicFloorplanMesh) -> Self {
        Self {
            graph,
            mesh,
            runaway_threshold_k: 500.0,
            newton_options: NewtonOptions::default(),
            current_time_s: 0.0,
            trajectory: ElectroThermalTransientTrajectory::new(),
        }
    }

    /// Configures the thermal runaway threshold temperature in Kelvin.
    pub fn with_runaway_threshold(mut self, threshold_k: f64) -> Self {
        self.runaway_threshold_k = threshold_k;
        self
    }

    /// Configures the Newton solver options.
    pub fn with_newton_options(mut self, opts: NewtonOptions) -> Self {
        self.newton_options = opts;
        self
    }

    /// Calculates instantaneous Joule power dissipation across all circuit components.
    ///
    /// Diodes: P = I_d * V_d
    /// Resistors: P = V^2 / R
    /// MOSFETs: P = V_ds * I_ds
    /// BJTs: P = V_ce * I_c + V_be * I_b
    pub fn compute_powers(
        &self,
        sol: &phonon_solver::mna::DcSolution,
        ctx: &ModelContext,
    ) -> HashMap<String, f64> {
        let mut powers = HashMap::new();
        let get_v = |node: NodeId| -> f64 { sol.node_voltage(node) };

        for comp in self.graph.components() {
            let name = comp.name();
            let comp_temp = self
                .mesh
                .get_component_temperature(name)
                .unwrap_or(ctx.temperature_kelvin);

            let p = match comp {
                phonon_core::ComponentRecord::Diode { pos, neg, .. } => {
                    let vd = get_v(*pos) - get_v(*neg);
                    let model = ctx.get_diode_model(name);
                    let id = model.evaluate(vd, comp_temp).i_d;
                    (vd * id).max(0.0)
                }
                phonon_core::ComponentRecord::Resistor {
                    pos,
                    neg,
                    resistance,
                    ..
                } => {
                    let v = get_v(*pos) - get_v(*neg);
                    (v * v / resistance).max(0.0)
                }
                phonon_core::ComponentRecord::Mosfet {
                    drain,
                    gate,
                    source,
                    bulk,
                    ..
                } => {
                    let vd = get_v(*drain);
                    let vg = get_v(*gate);
                    let vs = get_v(*source);
                    let vb = get_v(*bulk);
                    let model = ctx.get_mosfet_model(name);
                    let ids = model.evaluate(vd, vg, vs, vb, comp_temp).i_ds;
                    let vds = vd - vs;
                    (vds * ids).max(0.0)
                }
                phonon_core::ComponentRecord::Bjt {
                    collector,
                    base,
                    emitter,
                    ..
                } => {
                    let vc = get_v(*collector);
                    let vb = get_v(*base);
                    let ve = get_v(*emitter);
                    let model = ctx.get_bjt_model(name);
                    let eval = model.evaluate(vc, vb, ve, comp_temp);
                    ((vc - ve) * eval.i_c + (vb - ve) * eval.i_b).max(0.0)
                }
                _ => 0.0,
            };

            powers.insert(name.to_string(), p);
        }

        powers
    }

    /// Advances the coupled electro-thermal state by a single time step dt in lockstep.
    ///
    /// 1. Updates device temperature context from local floorplan grid cells.
    /// 2. Solves the electrical operating point.
    /// 3. Computes instantaneous Joule power dissipations.
    /// 4. Advances the thermal diffusion mesh by dt using ADI integration.
    /// 5. Validates thermal stability and detects runaway.
    pub fn step(
        &mut self,
        dt: f64,
        ctx: &mut ModelContext,
    ) -> Result<TransientStepRecord, SolverError> {
        // Synchronize context temperature with highest component temperature
        let mut max_comp_temp = self.mesh.die.ambient_temperature;
        for comp in &self.mesh.components {
            if let Some(t) = self.mesh.get_component_temperature(&comp.name) {
                if t > max_comp_temp {
                    max_comp_temp = t;
                }
            }
        }
        ctx.temperature_kelvin = max_comp_temp;

        // 1. Solve electrical circuit operating point
        let sol = solve_dc_non_linear(&self.graph, ctx, &self.newton_options)?;

        // 2. Compute power dissipation
        let powers = self.compute_powers(&sol, ctx);

        // 3. Step thermal floorplan mesh
        self.mesh
            .step_transient(dt, &powers)
            .map_err(|e| SolverError::NumericalAnomaly { detail: e })?;

        self.current_time_s += dt;

        // 4. Sample updated component temperatures
        let mut comp_temps = HashMap::new();
        for comp in &self.mesh.components {
            if let Some(t) = self.mesh.get_component_temperature(&comp.name) {
                comp_temps.insert(comp.name.clone(), t);
            }
        }

        // 5. Check peak temperature & runaway
        let (peak_temp, peak_x, peak_y) = self.mesh.peak_temperature();
        let is_runaway = peak_temp >= self.runaway_threshold_k || peak_temp.is_nan();

        let record = TransientStepRecord {
            time_s: self.current_time_s,
            node_voltages: sol.node_voltages,
            component_powers: powers,
            component_temperatures: comp_temps,
            grid_temperatures: self.mesh.temperatures.clone(),
            peak_temp_k: peak_temp,
            peak_pos: (peak_x, peak_y),
            is_runaway,
        };

        if is_runaway {
            let runaway_comp = self
                .mesh
                .components
                .iter()
                .max_by(|a, b| {
                    let ta = self.mesh.get_component_temperature(&a.name).unwrap_or(0.0);
                    let tb = self.mesh.get_component_temperature(&b.name).unwrap_or(0.0);
                    ta.partial_cmp(&tb).unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|c| c.name.clone())
                .unwrap_or_else(|| "Die".to_string());

            self.trajectory.runaway_detected = true;
            self.trajectory.runaway_component = Some(runaway_comp.clone());
            self.trajectory.records.push(record.clone());
            self.trajectory.timestamps.push(record.time_s);
            if peak_temp > self.trajectory.peak_die_temp_k {
                self.trajectory.peak_die_temp_k = peak_temp;
            }

            return Err(SolverError::NumericalAnomaly {
                detail: format!(
                    "Thermal runaway detected on component '{runaway_comp}': junction temperature exceeded {:.1} K ({:.1} K) at ({:.4}, {:.4})",
                    self.runaway_threshold_k, peak_temp, peak_x, peak_y
                ),
            });
        }

        self.trajectory.records.push(record.clone());
        self.trajectory.timestamps.push(record.time_s);
        if peak_temp > self.trajectory.peak_die_temp_k {
            self.trajectory.peak_die_temp_k = peak_temp;
        }

        Ok(record)
    }

    /// Runs full transient electro-thermal co-simulation from t = 0 to t = t_stop with step dt.
    ///
    /// Halts and returns SolverError::NumericalAnomaly if thermal runaway is detected.
    pub fn simulate(
        &mut self,
        t_stop: f64,
        dt: f64,
        initial_context: &ModelContext,
    ) -> Result<ElectroThermalTransientTrajectory, SolverError> {
        if dt <= 0.0 || !dt.is_finite() {
            return Err(SolverError::NumericalAnomaly {
                detail: "Time step dt must be positive and finite".to_string(),
            });
        }
        if t_stop <= 0.0 || !t_stop.is_finite() {
            return Err(SolverError::NumericalAnomaly {
                detail: "Stop time t_stop must be positive and finite".to_string(),
            });
        }

        let mut ctx = initial_context.clone();
        self.current_time_s = 0.0;
        self.trajectory = ElectroThermalTransientTrajectory::new();

        // 1. Initial operating point at t = 0
        let mut max_active_temp = self.mesh.die.ambient_temperature;
        for comp in &self.mesh.components {
            if let Some(t) = self.mesh.get_component_temperature(&comp.name) {
                if t > max_active_temp {
                    max_active_temp = t;
                }
            }
        }
        ctx.temperature_kelvin = max_active_temp;

        let init_sol = solve_dc_non_linear(&self.graph, &ctx, &self.newton_options)?;
        let init_powers = self.compute_powers(&init_sol, &ctx);

        let mut init_temps = HashMap::new();
        for comp in &self.mesh.components {
            if let Some(t) = self.mesh.get_component_temperature(&comp.name) {
                init_temps.insert(comp.name.clone(), t);
            }
        }
        let (init_peak, init_px, init_py) = self.mesh.peak_temperature();
        let init_runaway = init_peak >= self.runaway_threshold_k;

        let init_record = TransientStepRecord {
            time_s: 0.0,
            node_voltages: init_sol.node_voltages,
            component_powers: init_powers,
            component_temperatures: init_temps,
            grid_temperatures: self.mesh.temperatures.clone(),
            peak_temp_k: init_peak,
            peak_pos: (init_px, init_py),
            is_runaway: init_runaway,
        };

        self.trajectory.records.push(init_record);
        self.trajectory.timestamps.push(0.0);
        self.trajectory.peak_die_temp_k = init_peak;

        if init_runaway {
            self.trajectory.runaway_detected = true;
            return Err(SolverError::NumericalAnomaly {
                detail: format!(
                    "Thermal runaway detected at t=0: initial junction temperature {:.1} K exceeds threshold {:.1} K",
                    init_peak, self.runaway_threshold_k
                ),
            });
        }

        // 2. Lockstep transient integration loop
        while self.current_time_s < t_stop - 1e-12 {
            let step_dt = dt.min(t_stop - self.current_time_s);
            self.step(step_dt, &mut ctx)?;
        }

        Ok(self.trajectory.clone())
    }
}
