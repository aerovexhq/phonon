//! Synchronized event-driven mixed-signal co-simulation kernel bridging continuous
//! Modified Nodal Analysis (MNA) with discrete event queues and boundary bridges.

use std::collections::HashMap;

use phonon_core::{
    CircuitGraph, ComponentRecord, CoreError, DigitalNodeId, EventQueue, LogicLevel, NodeId,
};
use phonon_models::mixed_signal::{A2dBridge, D2aBridge, DigitalNetwork};
use phonon_models::tline::lossless::BraninWaveHistory;
use phonon_models::{compute_vcrit, pn_junction_limit};

use crate::error::SolverError;
use crate::mna::non_linear_solver::{ModelContext, NewtonOptions};
use crate::mna::stamp::*;
use crate::sparse::builder::SparseMatrixBuilder;
use crate::sparse::lu::SparseLuFactorization;
use crate::sparse::markowitz::MarkowitzOptions;
use crate::transient::integrator::{CapacitorCompanion, InductorCompanion, IntegrationMethod};
use crate::transient::{TransientOptions, TransientSolution, TransientStep};

/// Unified mixed-signal circuit container combining analog continuous graph,
/// discrete digital network, and bidirectional boundary bridges.
#[derive(Debug, Clone, Default)]
pub struct MixedSignalCircuit {
    /// Continuous electrical circuit graph.
    pub analog_graph: CircuitGraph,
    /// Discrete digital logic network.
    pub digital_network: DigitalNetwork,
    /// Analog-to-digital converter boundary bridges.
    pub a2d_bridges: Vec<A2dBridge>,
    /// Digital-to-analog driver boundary bridges.
    pub d2a_bridges: Vec<D2aBridge>,
}

impl MixedSignalCircuit {
    /// Creates a new mixed-signal circuit from analog and digital sub-circuits.
    pub fn new(analog_graph: CircuitGraph, digital_network: DigitalNetwork) -> Self {
        Self {
            analog_graph,
            digital_network,
            a2d_bridges: Vec::new(),
            d2a_bridges: Vec::new(),
        }
    }

    /// Adds an A2D comparator bridge to the mixed-signal circuit.
    pub fn add_a2d(&mut self, bridge: A2dBridge) -> usize {
        let idx = self.a2d_bridges.len();
        self.a2d_bridges.push(bridge);
        idx
    }

    /// Adds a D2A driver bridge to the mixed-signal circuit.
    pub fn add_d2a(&mut self, bridge: D2aBridge) -> usize {
        let idx = self.d2a_bridges.len();
        self.d2a_bridges.push(bridge);
        idx
    }

    /// Validates circuit topology and bridge references.
    pub fn validate(&self) -> Result<(), SolverError> {
        self.analog_graph.validate_topology()?;
        let num_nodes = self.analog_graph.active_nodes();

        for (i, a2d) in self.a2d_bridges.iter().enumerate() {
            if !a2d.in_pos.is_ground() && a2d.in_pos.index() > num_nodes {
                return Err(SolverError::Core(CoreError::InvalidNodeIndex(
                    a2d.in_pos.index(),
                )));
            }
            if !a2d.in_neg.is_ground() && a2d.in_neg.index() > num_nodes {
                return Err(SolverError::Core(CoreError::InvalidNodeIndex(
                    a2d.in_neg.index(),
                )));
            }
            if a2d.vth_low >= a2d.vth_high {
                return Err(SolverError::NumericalAnomaly {
                    detail: format!(
                        "A2D bridge #{} vth_low ({:.3}) >= vth_high ({:.3})",
                        i, a2d.vth_low, a2d.vth_high
                    ),
                });
            }
        }

        for d2a in &self.d2a_bridges {
            if !d2a.out_pos.is_ground() && d2a.out_pos.index() > num_nodes {
                return Err(SolverError::Core(CoreError::InvalidNodeIndex(
                    d2a.out_pos.index(),
                )));
            }
            if !d2a.out_neg.is_ground() && d2a.out_neg.index() > num_nodes {
                return Err(SolverError::Core(CoreError::InvalidNodeIndex(
                    d2a.out_neg.index(),
                )));
            }
        }

        Ok(())
    }
}

/// Simulation controls and algorithmic options for mixed-signal co-simulation.
#[derive(Debug, Clone, PartialEq)]
pub struct MixedSignalOptions {
    /// Continuous transient solver controls.
    pub transient: TransientOptions,
    /// Maximum allowable zero-delay digital delta cycles per time point.
    pub max_delta_cycles: usize,
    /// Time tolerance for event back-tracking alignment (seconds).
    pub crossing_tolerance: f64,
}

impl Default for MixedSignalOptions {
    fn default() -> Self {
        Self {
            transient: TransientOptions::default(),
            max_delta_cycles: 1000,
            crossing_tolerance: 1e-12,
        }
    }
}

/// A logged discrete state change on a digital net during simulation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DigitalTraceStep {
    /// Time when the state change occurred.
    pub time: f64,
    /// Digital net ID.
    pub node: DigitalNodeId,
    /// New logic level.
    pub level: LogicLevel,
}

/// Comprehensive mixed-signal solution containing synchronized analog and digital waveforms.
#[derive(Debug, Clone, PartialEq)]
pub struct MixedSignalSolution {
    /// Continuous analog nodal voltages and branch currents.
    pub analog: TransientSolution,
    /// Discrete digital signal transition history.
    pub digital_trace: Vec<DigitalTraceStep>,
    /// Total number of discrete digital events processed.
    pub total_digital_events: usize,
    /// Total number of zero-delay delta cycles executed.
    pub total_delta_cycles: usize,
    /// Total number of continuous step back-tracks triggered by A2D crossings.
    pub total_backtrack_steps: usize,
}

impl MixedSignalSolution {
    /// Extracts the discrete logic state history `(time, LogicLevel)` for a digital net.
    pub fn digital_waveform(&self, node: DigitalNodeId) -> Vec<(f64, LogicLevel)> {
        self.digital_trace
            .iter()
            .filter(|step| step.node == node)
            .map(|step| (step.time, step.level))
            .collect()
    }

    /// Evaluates the logic state of a digital net at an arbitrary time point $t$.
    pub fn digital_state_at(&self, node: DigitalNodeId, time: f64) -> LogicLevel {
        let mut last_level = LogicLevel::U;
        for step in &self.digital_trace {
            if step.node == node {
                if step.time <= time {
                    last_level = step.level;
                } else {
                    break;
                }
            }
        }
        last_level
    }

    /// Extracts continuous analog voltage waveform `(time, V(t))` for a circuit node.
    pub fn analog_node_waveform(&self, node: NodeId) -> Vec<(f64, f64)> {
        self.analog.node_waveform(node)
    }
}

/// Dynamic reactive component history for mixed-signal transient stepping.
#[derive(Debug, Clone, Default)]
struct DynamicHistory {
    capacitors: HashMap<String, (f64, f64)>,
    inductors: HashMap<String, (f64, f64)>,
    tlines: HashMap<String, BraninWaveHistory>,
}

/// Solves a synchronized mixed-signal circuit over the specified time interval.
pub fn solve_mixed_signal(
    circuit: &mut MixedSignalCircuit,
    context: &ModelContext,
    options: &MixedSignalOptions,
) -> Result<MixedSignalSolution, SolverError> {
    circuit.validate()?;

    let graph = &circuit.analog_graph;
    let active_nodes = graph.active_nodes();
    let total_branches = graph.total_branches();
    let total_dim = active_nodes + total_branches;

    let mut event_queue = EventQueue::new();
    let mut digital_trace = Vec::new();
    let mut total_digital_events = 0;
    let mut total_delta_cycles = 0;
    let mut total_backtrack_steps = 0;

    let mut analog_solution = TransientSolution {
        steps: Vec::new(),
        accepted_steps: 0,
        rejected_steps: 0,
        total_newton_iters: 0,
    };

    if total_dim == 0 {
        return Ok(MixedSignalSolution {
            analog: analog_solution,
            digital_trace,
            total_digital_events,
            total_delta_cycles,
            total_backtrack_steps,
        });
    }

    // Helper to extract node voltage from continuous state vector
    let get_v = |node: NodeId, state: &[f64]| -> f64 {
        if node.is_ground() {
            0.0
        } else {
            state[node.index() - 1]
        }
    };

    // 1. Initialize digital levels and D2A bridges at t = 0.0
    for d2a in &mut circuit.d2a_bridges {
        let lvl = circuit.digital_network.get_level(d2a.in_dig);
        let eff_lvl = if lvl == LogicLevel::U {
            LogicLevel::Zero
        } else {
            lvl
        };
        d2a.set_level(eff_lvl, 0.0);
    }

    // 2. Solve initial DC operating point or enforce UIC at t = 0.0
    let (mut current_state, dc_iters) = if !options.transient.uic {
        solve_mixed_stage(
            graph,
            &circuit.a2d_bridges,
            &circuit.d2a_bridges,
            context,
            &options.transient.newton,
            &options.transient.waveforms,
            0.0,
            1.0,
            &vec![0.0; total_dim],
            &DynamicHistory::default(),
            StageType::DcOperatingPoint,
        )?
    } else {
        let mut state = vec![0.0; total_dim];
        for comp in graph.components() {
            match comp {
                ComponentRecord::Capacitor {
                    pos,
                    neg,
                    initial_voltage: Some(v_init),
                    ..
                } => {
                    if !pos.is_ground() && neg.is_ground() {
                        state[pos.index() - 1] = *v_init;
                    } else if pos.is_ground() && !neg.is_ground() {
                        state[neg.index() - 1] = -*v_init;
                    } else if !pos.is_ground() && !neg.is_ground() {
                        state[pos.index() - 1] = state[neg.index() - 1] + *v_init;
                    }
                }
                ComponentRecord::Inductor {
                    branch,
                    initial_current: Some(i_init),
                    ..
                } => {
                    state[active_nodes + branch.index()] = *i_init;
                }
                _ => {}
            }
        }
        (state, 0)
    };
    analog_solution.total_newton_iters += dc_iters;

    // 3. Initialize reactive component histories
    let mut dyn_state = DynamicHistory::default();
    for comp in graph.components() {
        match comp {
            ComponentRecord::Capacitor {
                name,
                pos,
                neg,
                initial_voltage,
                ..
            } => {
                let v = if options.transient.uic {
                    initial_voltage.unwrap_or(0.0)
                } else {
                    get_v(*pos, &current_state) - get_v(*neg, &current_state)
                };
                dyn_state.capacitors.insert(name.clone(), (v, 0.0));
            }
            ComponentRecord::Inductor {
                name,
                pos,
                neg,
                branch,
                initial_current,
                ..
            } => {
                let i = if options.transient.uic {
                    initial_current.unwrap_or(0.0)
                } else {
                    current_state[active_nodes + branch.index()]
                };
                let v = get_v(*pos, &current_state) - get_v(*neg, &current_state);
                dyn_state.inductors.insert(name.clone(), (i, v));
            }
            ComponentRecord::TransmissionLine {
                name,
                in_pos,
                in_neg,
                out_pos,
                out_neg,
                td,
                ..
            } => {
                let mut history = BraninWaveHistory::new();
                let v1 = get_v(*in_pos, &current_state) - get_v(*in_neg, &current_state);
                let v2 = get_v(*out_pos, &current_state) - get_v(*out_neg, &current_state);
                history.record_step(0.0, v1, v2, *td);
                dyn_state.tlines.insert(name.clone(), history);
            }
            _ => {}
        }
    }

    // 4. Update A2D bridges with initial voltages and populate digital network
    for a2d in &mut circuit.a2d_bridges {
        let v_in = get_v(a2d.in_pos, &current_state) - get_v(a2d.in_neg, &current_state);
        let level = a2d.evaluate_dc(v_in);
        circuit.digital_network.set_level(a2d.out_dig, level);
        digital_trace.push(DigitalTraceStep {
            time: 0.0,
            node: a2d.out_dig,
            level,
        });
    }

    // 5. Initialize autonomous digital clocks into the event queue
    circuit.digital_network.init_clocks(&mut event_queue, 0.0);

    // Process initial delta cycles at t = 0.0
    let delta = circuit
        .digital_network
        .process_delta_cycles(&mut event_queue, 0.0, options.max_delta_cycles)
        .map_err(|detail| SolverError::NumericalAnomaly { detail })?;
    total_delta_cycles += delta;

    // Sync D2A bridges with initial digital states
    for d2a in &mut circuit.d2a_bridges {
        let lvl = circuit.digital_network.get_level(d2a.in_dig);
        if lvl != d2a.current_level() {
            d2a.set_level(lvl, 0.0);
        }
    }

    // Record initial step
    record_step(
        &mut analog_solution,
        0.0,
        &current_state,
        &dyn_state,
        active_nodes,
        total_branches,
        dc_iters,
    );

    // Log initial digital states
    for (name, &node_id) in &circuit.digital_network.node_names {
        let _ = name;
        let lvl = circuit.digital_network.get_level(node_id);
        if !digital_trace.iter().any(|s| s.node == node_id) {
            digital_trace.push(DigitalTraceStep {
                time: 0.0,
                node: node_id,
                level: lvl,
            });
        }
    }

    // 6. Master Co-Simulation Synchronization Loop
    let mut current_time = 0.0;
    let tstop = options.transient.tstop;
    let nominal_step = options.transient.tstep;
    let max_step = options.transient.tmax.unwrap_or(nominal_step);
    let min_step = options.transient.step_control.min_step;

    while current_time < tstop {
        // A. Drain and dispatch any digital events scheduled at or before current_time
        let mut events_processed = 0;
        while let Some(next_t) = event_queue.next_event_time() {
            if next_t <= current_time + 1e-15 {
                let ev = event_queue.pop().unwrap();
                total_digital_events += 1;
                events_processed += 1;

                circuit.digital_network.dispatch_event(ev, &mut event_queue);
                digital_trace.push(DigitalTraceStep {
                    time: current_time,
                    node: ev.node,
                    level: ev.level,
                });

                // Update connected D2A bridges
                for d2a in &mut circuit.d2a_bridges {
                    if d2a.in_dig == ev.node {
                        d2a.set_level(ev.level, current_time);
                    }
                }
            } else {
                break;
            }
        }

        // Run zero-delay delta cycles if events occurred
        if events_processed > 0 {
            let delta = circuit
                .digital_network
                .process_delta_cycles(&mut event_queue, current_time, options.max_delta_cycles)
                .map_err(|detail| SolverError::NumericalAnomaly { detail })?;
            total_delta_cycles += delta;

            // Sync any newly switched D2A inputs from delta cycles
            for d2a in &mut circuit.d2a_bridges {
                let lvl = circuit.digital_network.get_level(d2a.in_dig);
                if lvl != d2a.current_level() {
                    d2a.set_level(lvl, current_time);
                }
            }
        }

        // B. Determine next step size bounded by upcoming digital events and tstop
        let mut h = nominal_step.clamp(min_step, max_step);
        if let Some(t_event) = event_queue.next_event_time() {
            if t_event > current_time {
                let dt_event = t_event - current_time;
                if dt_event < h {
                    h = dt_event;
                }
            }
        }

        if current_time + h > tstop {
            h = tstop - current_time;
        }

        if h < 1e-18 {
            break;
        }

        let target_time = current_time + h;

        // C. Solve trial continuous step to target_time
        let stage_mode = match options.transient.method {
            IntegrationMethod::BackwardEuler => StageType::BackwardEuler,
            IntegrationMethod::Trapezoidal => StageType::Trapezoidal,
            IntegrationMethod::TrBdf2 => StageType::BackwardEuler, // Variable-step robust Euler
        };

        let (candidate_state, iters) = solve_mixed_stage(
            graph,
            &circuit.a2d_bridges,
            &circuit.d2a_bridges,
            context,
            &options.transient.newton,
            &options.transient.waveforms,
            target_time,
            h,
            &current_state,
            &dyn_state,
            stage_mode,
        )?;
        analog_solution.total_newton_iters += iters;

        // D. Inspect A2D bridges for threshold crossings
        let mut earliest_crossing: Option<(f64, usize, LogicLevel)> = None;
        for (idx, a2d) in circuit.a2d_bridges.iter().enumerate() {
            let v_old = get_v(a2d.in_pos, &current_state) - get_v(a2d.in_neg, &current_state);
            let v_new = get_v(a2d.in_pos, &candidate_state) - get_v(a2d.in_neg, &candidate_state);

            if let Some((t_cross, new_lvl)) =
                a2d.detect_crossing(v_old, v_new, current_time, target_time)
            {
                if let Some((best_t, _, _)) = earliest_crossing {
                    if t_cross < best_t {
                        earliest_crossing = Some((t_cross, idx, new_lvl));
                    }
                } else {
                    earliest_crossing = Some((t_cross, idx, new_lvl));
                }
            }
        }

        // E. Step Acceptance or Back-tracking
        let (accepted_time, accepted_h, accepted_state) =
            if let Some((t_cross, _, _)) = earliest_crossing {
                if t_cross < target_time - options.crossing_tolerance {
                    // Back-track to exact crossing time!
                    let h_backtrack = (t_cross - current_time).max(1e-15);
                    let (bt_state, bt_iters) = solve_mixed_stage(
                        graph,
                        &circuit.a2d_bridges,
                        &circuit.d2a_bridges,
                        context,
                        &options.transient.newton,
                        &options.transient.waveforms,
                        t_cross,
                        h_backtrack,
                        &current_state,
                        &dyn_state,
                        StageType::BackwardEuler,
                    )?;
                    analog_solution.total_newton_iters += bt_iters;
                    total_backtrack_steps += 1;
                    (t_cross, h_backtrack, bt_state)
                } else {
                    (target_time, h, candidate_state)
                }
            } else {
                (target_time, h, candidate_state)
            };

        // F. Schedule any crossed A2D events at accepted_time
        if let Some((_, cross_idx, new_lvl)) = earliest_crossing {
            let a2d = &mut circuit.a2d_bridges[cross_idx];
            a2d.current_level = new_lvl;
            event_queue.schedule(accepted_time, a2d.out_dig, new_lvl);
        }

        for (idx, a2d) in circuit.a2d_bridges.iter_mut().enumerate() {
            if let Some((_, cross_idx, _)) = earliest_crossing {
                if idx == cross_idx {
                    continue;
                }
            }
            let v_old = get_v(a2d.in_pos, &current_state) - get_v(a2d.in_neg, &current_state);
            let v_new = get_v(a2d.in_pos, &accepted_state) - get_v(a2d.in_neg, &accepted_state);
            if let Some((_, new_lvl)) =
                a2d.detect_crossing(v_old, v_new, current_time, accepted_time)
            {
                a2d.current_level = new_lvl;
                event_queue.schedule(accepted_time, a2d.out_dig, new_lvl);
            }
        }

        // G. Update dynamic history
        update_dynamic_history(
            &mut dyn_state,
            graph,
            &current_state,
            &accepted_state,
            accepted_time,
            accepted_h,
            stage_mode,
        );

        current_time = accepted_time;
        current_state = accepted_state;
        analog_solution.accepted_steps += 1;

        record_step(
            &mut analog_solution,
            current_time,
            &current_state,
            &dyn_state,
            active_nodes,
            total_branches,
            iters,
        );
    }

    Ok(MixedSignalSolution {
        analog: analog_solution,
        digital_trace,
        total_digital_events,
        total_delta_cycles,
        total_backtrack_steps,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StageType {
    DcOperatingPoint,
    BackwardEuler,
    Trapezoidal,
}

/// Solves a single continuous MNA stage at time $t$ with mixed-signal bridge companion stamps.
#[allow(clippy::too_many_arguments)]
fn solve_mixed_stage(
    graph: &CircuitGraph,
    a2d_bridges: &[A2dBridge],
    d2a_bridges: &[D2aBridge],
    context: &ModelContext,
    newton: &NewtonOptions,
    waveforms: &HashMap<String, crate::transient::TimeWaveform>,
    t: f64,
    h: f64,
    initial_guess: &[f64],
    dyn_state: &DynamicHistory,
    stage: StageType,
) -> Result<(Vec<f64>, usize), SolverError> {
    let active_nodes = graph.active_nodes();
    let total_branches = graph.total_branches();
    let total_dim = active_nodes + total_branches;

    let mut x = initial_guess.to_vec();
    let markowitz_opts = MarkowitzOptions::default();
    let temp_k = context.temperature_kelvin;

    let get_v = |node: NodeId, state: &[f64]| -> f64 {
        if node.is_ground() {
            0.0
        } else {
            state[node.index() - 1]
        }
    };

    for iter in 0..newton.max_iters {
        let mut g_builder = SparseMatrixBuilder::with_capacity(total_dim, total_dim, total_dim * 6);
        let mut rhs = vec![0.0; total_dim];

        // 1. Stamp Resistors and Linear Components
        for comp in graph.components() {
            match comp {
                ComponentRecord::Resistor {
                    pos,
                    neg,
                    resistance,
                    ..
                } => {
                    stamp_conductance(&mut g_builder, *pos, *neg, 1.0 / resistance);
                }
                ComponentRecord::VoltageSource {
                    pos,
                    neg,
                    branch,
                    dc_value,
                    name,
                    ..
                } => {
                    let br_idx = active_nodes + branch.index();
                    let v_val = if stage == StageType::DcOperatingPoint {
                        *dc_value
                    } else if let Some(wf) = waveforms.get(name) {
                        wf.evaluate(t)
                    } else {
                        *dc_value
                    };
                    stamp_voltage_source(&mut g_builder, &mut rhs, *pos, *neg, br_idx, v_val);
                }
                ComponentRecord::CurrentSource {
                    pos,
                    neg,
                    dc_value,
                    name,
                    ..
                } => {
                    let i_val = if stage == StageType::DcOperatingPoint {
                        *dc_value
                    } else if let Some(wf) = waveforms.get(name) {
                        wf.evaluate(t)
                    } else {
                        *dc_value
                    };
                    stamp_current_source(&mut rhs, *pos, *neg, i_val);
                }
                ComponentRecord::Vcvs {
                    out_pos,
                    out_neg,
                    ctrl_pos,
                    ctrl_neg,
                    gain,
                    branch,
                    ..
                } => {
                    let br_idx = active_nodes + branch.index();
                    stamp_vcvs(
                        &mut g_builder,
                        *out_pos,
                        *out_neg,
                        *ctrl_pos,
                        *ctrl_neg,
                        br_idx,
                        *gain,
                    );
                }
                ComponentRecord::Vccs {
                    out_pos,
                    out_neg,
                    ctrl_pos,
                    ctrl_neg,
                    transconductance,
                    ..
                } => {
                    stamp_vccs(
                        &mut g_builder,
                        *out_pos,
                        *out_neg,
                        *ctrl_pos,
                        *ctrl_neg,
                        *transconductance,
                    );
                }
                ComponentRecord::Capacitor {
                    name,
                    pos,
                    neg,
                    capacitance,
                    ..
                } => match stage {
                    StageType::DcOperatingPoint => {
                        // Open circuit in DC
                    }
                    StageType::BackwardEuler => {
                        let (v_n, _) = dyn_state
                            .capacitors
                            .get(name)
                            .copied()
                            .unwrap_or((0.0, 0.0));
                        let comp = CapacitorCompanion::backward_euler(*capacitance, h, v_n);
                        stamp_conductance(&mut g_builder, *pos, *neg, comp.g_eq);
                        if let Some(p) = node_to_mna_idx(*pos) {
                            rhs[p] += comp.i_eq;
                        }
                        if let Some(n) = node_to_mna_idx(*neg) {
                            rhs[n] -= comp.i_eq;
                        }
                    }
                    StageType::Trapezoidal => {
                        let (v_n, i_n) = dyn_state
                            .capacitors
                            .get(name)
                            .copied()
                            .unwrap_or((0.0, 0.0));
                        let comp = CapacitorCompanion::trapezoidal(*capacitance, h, v_n, i_n);
                        stamp_conductance(&mut g_builder, *pos, *neg, comp.g_eq);
                        if let Some(p) = node_to_mna_idx(*pos) {
                            rhs[p] += comp.i_eq;
                        }
                        if let Some(n) = node_to_mna_idx(*neg) {
                            rhs[n] -= comp.i_eq;
                        }
                    }
                },
                ComponentRecord::Inductor {
                    name,
                    pos,
                    neg,
                    branch,
                    inductance,
                    ..
                } => {
                    let br_idx = active_nodes + branch.index();
                    match stage {
                        StageType::DcOperatingPoint => {
                            // Short circuit in DC: V_pos - V_neg = 0
                            stamp_voltage_source(&mut g_builder, &mut rhs, *pos, *neg, br_idx, 0.0);
                        }
                        StageType::BackwardEuler => {
                            let (i_n, _) =
                                dyn_state.inductors.get(name).copied().unwrap_or((0.0, 0.0));
                            let comp = InductorCompanion::backward_euler(*inductance, h, i_n);
                            if let Some(p) = node_to_mna_idx(*pos) {
                                g_builder.add(br_idx, p, 1.0);
                                g_builder.add(p, br_idx, 1.0);
                            }
                            if let Some(n) = node_to_mna_idx(*neg) {
                                g_builder.add(br_idx, n, -1.0);
                                g_builder.add(n, br_idx, -1.0);
                            }
                            g_builder.add(br_idx, br_idx, -comp.r_eq);
                            rhs[br_idx] -= comp.v_eq;
                        }
                        StageType::Trapezoidal => {
                            let (i_n, v_n) =
                                dyn_state.inductors.get(name).copied().unwrap_or((0.0, 0.0));
                            let comp = InductorCompanion::trapezoidal(*inductance, h, i_n, v_n);
                            if let Some(p) = node_to_mna_idx(*pos) {
                                g_builder.add(br_idx, p, 1.0);
                                g_builder.add(p, br_idx, 1.0);
                            }
                            if let Some(n) = node_to_mna_idx(*neg) {
                                g_builder.add(br_idx, n, -1.0);
                                g_builder.add(n, br_idx, -1.0);
                            }
                            g_builder.add(br_idx, br_idx, -comp.r_eq);
                            rhs[br_idx] -= comp.v_eq;
                        }
                    }
                }
                ComponentRecord::TransmissionLine {
                    name,
                    in_pos,
                    in_neg,
                    out_pos,
                    out_neg,
                    z0,
                    td,
                } => {
                    let g0 = 1.0 / z0;
                    stamp_conductance(&mut g_builder, *in_pos, *in_neg, g0);
                    stamp_conductance(&mut g_builder, *out_pos, *out_neg, g0);

                    if let Some(history) = dyn_state.tlines.get(name) {
                        let (i1_eq, i2_eq) = history.companion_sources(t, *z0, *td);
                        if let Some(p) = node_to_mna_idx(*in_pos) {
                            rhs[p] += i1_eq;
                        }
                        if let Some(n) = node_to_mna_idx(*in_neg) {
                            rhs[n] -= i1_eq;
                        }
                        if let Some(p) = node_to_mna_idx(*out_pos) {
                            rhs[p] += i2_eq;
                        }
                        if let Some(n) = node_to_mna_idx(*out_neg) {
                            rhs[n] -= i2_eq;
                        }
                    }
                }
                _ => {}
            }
        }

        // 2. Stamp Boundary Bridges
        // A2D Input Impedance Loading
        for a2d in a2d_bridges {
            stamp_conductance(&mut g_builder, a2d.in_pos, a2d.in_neg, a2d.conductance());
        }

        // D2A Norton Drivers
        for d2a in d2a_bridges {
            let comp = d2a.evaluate(t);
            stamp_conductance(&mut g_builder, d2a.out_pos, d2a.out_neg, comp.g_eq);
            if let Some(p) = node_to_mna_idx(d2a.out_pos) {
                rhs[p] += comp.i_eq;
            }
            if let Some(n) = node_to_mna_idx(d2a.out_neg) {
                rhs[n] -= comp.i_eq;
            }
        }

        // 3. Stamp Active Non-linear Devices (Diodes, MOSFETs, BJTs)
        for comp in graph.components() {
            match comp {
                ComponentRecord::Diode { name, pos, neg } => {
                    let model = context.get_diode_model(name);
                    let v_pos = get_v(*pos, &x);
                    let v_neg = get_v(*neg, &x);
                    let v_d = v_pos - v_neg;

                    let eval = model.evaluate(v_d, temp_k);
                    let i_eq = eval.g_d * v_d - eval.i_d;
                    stamp_diode_companion(&mut g_builder, &mut rhs, *pos, *neg, eval.g_d, i_eq);
                }
                ComponentRecord::Mosfet {
                    name,
                    drain,
                    gate,
                    source,
                    bulk,
                } => {
                    let model = context.get_mosfet_model(name);
                    let v_d = get_v(*drain, &x);
                    let v_g = get_v(*gate, &x);
                    let v_s = get_v(*source, &x);
                    let v_b = get_v(*bulk, &x);

                    // Symmetrically swap terminals if operating in reverse mode
                    let is_reverse = match model.mos_type {
                        phonon_models::MosfetType::Nmos => v_d < v_s,
                        phonon_models::MosfetType::Pmos => v_d > v_s,
                    };

                    let (eff_d, eff_s, v_deff, v_seff) = if is_reverse {
                        (*source, *drain, v_s, v_d)
                    } else {
                        (*drain, *source, v_d, v_s)
                    };

                    let eval = model.evaluate(v_deff, v_g, v_seff, v_b, temp_k);
                    let v_ds_eff = v_deff - v_seff;
                    let v_gs_eff = v_g - v_seff;
                    let v_bs_eff = v_b - v_seff;

                    let i_eq = eval.g_m * v_gs_eff + eval.g_ds * v_ds_eff + eval.g_mbs * v_bs_eff
                        - eval.i_ds;
                    let comp_stamp = MosfetCompanion {
                        g_m: eval.g_m,
                        g_ds: eval.g_ds,
                        g_mbs: eval.g_mbs,
                        i_eq,
                    };

                    stamp_mosfet_companion(
                        &mut g_builder,
                        &mut rhs,
                        eff_d,
                        *gate,
                        eff_s,
                        *bulk,
                        &comp_stamp,
                    );
                }
                ComponentRecord::Bjt {
                    name,
                    collector,
                    base,
                    emitter,
                } => {
                    let model = context.get_bjt_model(name);
                    let v_c = get_v(*collector, &x);
                    let v_b = get_v(*base, &x);
                    let v_e = get_v(*emitter, &x);

                    let eval = model.evaluate(v_c, v_b, v_e, temp_k);
                    let v_be = v_b - v_e;
                    let v_bc = v_b - v_c;

                    let i_c_eq = eval.g_m * v_be - (eval.g_o + eval.g_mu) * v_bc - eval.i_c;
                    let i_b_eq = eval.g_pi * v_be + eval.g_mu * v_bc - eval.i_b;
                    let comp_stamp = BjtCompanion {
                        g_m: eval.g_m,
                        g_pi: eval.g_pi,
                        g_o: eval.g_o,
                        g_mu: eval.g_mu,
                        i_c_eq,
                        i_b_eq,
                    };

                    stamp_bjt_companion(
                        &mut g_builder,
                        &mut rhs,
                        *collector,
                        *base,
                        *emitter,
                        &comp_stamp,
                    );
                }
                _ => {}
            }
        }

        let csc = g_builder.build_csc();
        let lu = SparseLuFactorization::factor(&csc, &markowitz_opts)?;
        let mut x_new = vec![0.0; total_dim];
        lu.solve(&rhs, &mut x_new)?;

        // Check convergence
        let mut converged = true;
        for i in 0..total_dim {
            let tol = if i < active_nodes {
                newton.reltol * x[i].abs().max(x_new[i].abs()) + newton.vntol
            } else {
                newton.reltol * x[i].abs().max(x_new[i].abs()) + newton.abstol
            };
            if (x_new[i] - x[i]).abs() > tol {
                converged = false;
                break;
            }
        }

        // Apply PN junction and MOSFET voltage limiting
        let mut x_damped = x_new.clone();
        for comp in graph.components() {
            if let ComponentRecord::Mosfet { drain, source, .. } = comp {
                let v_d_old = get_v(*drain, &x);
                let v_s_old = get_v(*source, &x);
                let v_ds_old = v_d_old - v_s_old;

                let v_d_new = get_v(*drain, &x_new);
                let v_s_new = get_v(*source, &x_new);
                let v_ds_new = v_d_new - v_s_new;

                // Limit large Vds excursions and prevent crossing zero with overshoot
                let mut v_ds_lim = v_ds_new;
                if (v_ds_old > 0.0 && v_ds_new < 0.0) || (v_ds_old < 0.0 && v_ds_new > 0.0) {
                    v_ds_lim = 0.1 * v_ds_old;
                } else if (v_ds_new - v_ds_old).abs() > 2.0 {
                    v_ds_lim = v_ds_old + 2.0 * (v_ds_new - v_ds_old).signum();
                }

                if let Some(d_idx) = node_to_mna_idx(*drain) {
                    x_damped[d_idx] = v_s_new + v_ds_lim;
                }
            }
        }
        for comp in graph.components() {
            if let ComponentRecord::Diode { pos, neg, .. } = comp {
                let v_old = get_v(*pos, &x) - get_v(*neg, &x);
                let v_next = get_v(*pos, &x_new) - get_v(*neg, &x_new);
                let vt = phonon_core::constants::thermal_voltage(temp_k);
                let vcrit = compute_vcrit(1e-14, vt);
                let v_lim = pn_junction_limit(v_next, v_old, vt, vcrit);

                if let Some(p) = node_to_mna_idx(*pos) {
                    x_damped[p] = get_v(*neg, &x_damped) + v_lim;
                }
            }
        }

        x = x_damped;

        if converged {
            return Ok((x, iter + 1));
        }
    }

    Ok((x, newton.max_iters))
}

fn update_dynamic_history(
    dyn_state: &mut DynamicHistory,
    graph: &CircuitGraph,
    state_old: &[f64],
    state_new: &[f64],
    target_time: f64,
    h: f64,
    stage: StageType,
) {
    let active_nodes = graph.active_nodes();
    let get_v = |node: NodeId, state: &[f64]| -> f64 {
        if node.is_ground() {
            0.0
        } else {
            state[node.index() - 1]
        }
    };

    for comp in graph.components() {
        match comp {
            ComponentRecord::Capacitor {
                name,
                pos,
                neg,
                capacitance,
                ..
            } => {
                let v_old = get_v(*pos, state_old) - get_v(*neg, state_old);
                let v_new = get_v(*pos, state_new) - get_v(*neg, state_new);
                let (_, i_old) = dyn_state
                    .capacitors
                    .get(name)
                    .copied()
                    .unwrap_or((v_old, 0.0));

                let i_new = match stage {
                    StageType::DcOperatingPoint | StageType::BackwardEuler => {
                        *capacitance * (v_new - v_old) / h
                    }
                    StageType::Trapezoidal => (2.0 * capacitance / h) * (v_new - v_old) - i_old,
                };
                dyn_state.capacitors.insert(name.clone(), (v_new, i_new));
            }
            ComponentRecord::Inductor {
                name,
                pos,
                neg,
                branch,
                ..
            } => {
                let br_idx = active_nodes + branch.index();
                let v_new = get_v(*pos, state_new) - get_v(*neg, state_new);
                let i_new = state_new[br_idx];
                dyn_state.inductors.insert(name.clone(), (i_new, v_new));
            }
            ComponentRecord::TransmissionLine {
                name,
                in_pos,
                in_neg,
                out_pos,
                out_neg,
                td,
                ..
            } => {
                let v1 = get_v(*in_pos, state_new) - get_v(*in_neg, state_new);
                let v2 = get_v(*out_pos, state_new) - get_v(*out_neg, state_new);
                if let Some(history) = dyn_state.tlines.get_mut(name) {
                    history.record_step(target_time, v1, v2, *td);
                }
            }
            _ => {}
        }
    }
}

fn record_step(
    solution: &mut TransientSolution,
    time: f64,
    state: &[f64],
    dyn_state: &DynamicHistory,
    active_nodes: usize,
    total_branches: usize,
    iterations: usize,
) {
    let mut voltages = vec![0.0; active_nodes + 1];
    voltages[1..=active_nodes].copy_from_slice(&state[..active_nodes]);

    let mut branch_currents = vec![0.0; total_branches];
    if total_branches > 0 {
        branch_currents.copy_from_slice(&state[active_nodes..active_nodes + total_branches]);
    }

    let mut capacitor_currents = HashMap::new();
    for (name, &(_, i_c)) in &dyn_state.capacitors {
        capacitor_currents.insert(name.clone(), i_c);
    }

    solution.steps.push(TransientStep {
        time,
        voltages,
        branch_currents,
        capacitor_currents,
        iterations,
    });
}
