//! Time-domain transient circuit simulation using L-stable TR-BDF2, Trapezoidal, and Backward Euler integration.

pub mod integrator;
pub mod sources;
pub mod step_control;

pub use integrator::{CapacitorCompanion, InductorCompanion, IntegrationMethod, TR_BDF2_GAMMA};
pub use sources::TimeWaveform;
pub use step_control::{evaluate_tr_bdf2_lte, StepControlOptions};

use crate::error::SolverError;
use crate::mna::non_linear_solver::{solve_dc_non_linear, ModelContext, NewtonOptions};
use crate::mna::stamp::*;
use crate::sparse::builder::SparseMatrixBuilder;
use crate::sparse::lu::SparseLuFactorization;
use crate::sparse::markowitz::MarkowitzOptions;
use phonon_core::{BranchId, CircuitGraph, ComponentRecord, NodeId};
use phonon_models::tline::lossless::BraninWaveHistory;
use phonon_models::{compute_vcrit, pn_junction_limit};
use std::collections::HashMap;

/// Configuration options for transient analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct TransientOptions {
    /// Final simulation stop time in seconds.
    pub tstop: f64,
    /// Suggested or nominal time step in seconds.
    pub tstep: f64,
    /// Start time for saving output data in seconds (default 0.0).
    pub tstart: f64,
    /// Maximum time step cap (default `Some(tstep)`).
    pub tmax: Option<f64>,
    /// Use Initial Conditions (skip initial DC operating point solve if true).
    pub uic: bool,
    /// Numerical integration method.
    pub method: IntegrationMethod,
    /// Adaptive step controller options.
    pub step_control: StepControlOptions,
    /// Non-linear Newton-Raphson iteration controls.
    pub newton: NewtonOptions,
    /// Time-varying excitation sources by component name.
    pub waveforms: HashMap<String, TimeWaveform>,
}

impl Default for TransientOptions {
    fn default() -> Self {
        Self {
            tstop: 1e-3,
            tstep: 1e-6,
            tstart: 0.0,
            tmax: None,
            uic: false,
            method: IntegrationMethod::TrBdf2,
            step_control: StepControlOptions::default(),
            newton: NewtonOptions::default(),
            waveforms: HashMap::new(),
        }
    }
}

/// A single saved simulation time step.
#[derive(Debug, Clone, PartialEq)]
pub struct TransientStep {
    pub time: f64,
    pub voltages: Vec<f64>,
    pub branch_currents: Vec<f64>,
    pub capacitor_currents: HashMap<String, f64>,
    pub iterations: usize,
}

/// The complete solution trajectory produced by transient simulation.
#[derive(Debug, Clone, PartialEq)]
pub struct TransientSolution {
    pub steps: Vec<TransientStep>,
    pub accepted_steps: usize,
    pub rejected_steps: usize,
    pub total_newton_iters: usize,
}

impl TransientSolution {
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Extracts the time-series voltage waveform $(t, V(t))$ for a specific circuit node.
    pub fn node_waveform(&self, node: NodeId) -> Vec<(f64, f64)> {
        let idx = node.index();
        self.steps
            .iter()
            .map(|s| {
                let v = if idx == 0 {
                    0.0
                } else if idx < s.voltages.len() {
                    s.voltages[idx]
                } else {
                    0.0
                };
                (s.time, v)
            })
            .collect()
    }

    /// Extracts the time-series branch current waveform $(t, I(t))$ for a specific auxiliary branch.
    pub fn branch_waveform(&self, branch: BranchId) -> Vec<(f64, f64)> {
        let idx = branch.index();
        self.steps
            .iter()
            .map(|s| {
                let i = if idx < s.branch_currents.len() {
                    s.branch_currents[idx]
                } else {
                    0.0
                };
                (s.time, i)
            })
            .collect()
    }
}

/// Dynamic history state tracked for reactive components across integration time steps.
#[derive(Debug, Clone, Default)]
struct DynamicState {
    /// Capacitor name -> (voltage, current)
    capacitors: HashMap<String, (f64, f64)>,
    /// Inductor name -> (current, voltage)
    inductors: HashMap<String, (f64, f64)>,
    /// Transmission line name -> BraninWaveHistory
    tlines: HashMap<String, BraninWaveHistory>,
}

/// Solves the transient time-domain behavior of a physical circuit graph.
pub fn solve_transient(
    graph: &CircuitGraph,
    context: &ModelContext,
    options: &TransientOptions,
) -> Result<TransientSolution, SolverError> {
    graph.validate_topology()?;

    let active_nodes = graph.active_nodes();
    let total_branches = graph.total_branches();
    let total_dim = active_nodes + total_branches;

    let mut solution = TransientSolution {
        steps: Vec::new(),
        accepted_steps: 0,
        rejected_steps: 0,
        total_newton_iters: 0,
    };

    if total_dim == 0 {
        return Ok(solution);
    }

    // 1. Initial Operating Point at t = 0.0
    let mut current_state = vec![0.0; total_dim];
    let mut dyn_state = DynamicState::default();

    if !options.uic {
        let dc_sol = solve_dc_non_linear(graph, context, &options.newton)?;
        current_state[..active_nodes].copy_from_slice(&dc_sol.node_voltages[1..=active_nodes]);
        current_state[active_nodes..active_nodes + total_branches]
            .copy_from_slice(&dc_sol.branch_currents[..total_branches]);
    } else {
        // Enforce user-specified initial conditions (UIC) into the initial state vector
        for comp in graph.components() {
            match comp {
                ComponentRecord::Capacitor {
                    pos,
                    neg,
                    initial_voltage: Some(v_init),
                    ..
                } => {
                    if !pos.is_ground() && neg.is_ground() {
                        current_state[pos.index() - 1] = *v_init;
                    } else if pos.is_ground() && !neg.is_ground() {
                        current_state[neg.index() - 1] = -*v_init;
                    } else if !pos.is_ground() && !neg.is_ground() {
                        current_state[pos.index() - 1] = current_state[neg.index() - 1] + *v_init;
                    }
                }
                ComponentRecord::Inductor {
                    branch,
                    initial_current: Some(i_init),
                    ..
                } => {
                    current_state[active_nodes + branch.index()] = *i_init;
                }
                _ => {}
            }
        }
    }

    // Helper to read node voltage from state vector
    let get_v = |node: NodeId, state: &[f64]| -> f64 {
        if node.is_ground() {
            0.0
        } else {
            state[node.index() - 1]
        }
    };

    // Initialize reactive component history
    for comp in graph.components() {
        match comp {
            ComponentRecord::Capacitor {
                name,
                pos,
                neg,
                initial_voltage,
                ..
            } => {
                let v = if options.uic {
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
                let i = if options.uic {
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

    // Record t = 0 step
    record_step(
        &mut solution,
        0.0,
        &current_state,
        &dyn_state,
        active_nodes,
        total_branches,
        0,
    );

    // 2. Transient Time Marching Loop
    let mut current_time = 0.0;
    let mut h = options.tstep.clamp(
        options.step_control.min_step,
        options.tmax.unwrap_or(options.tstep),
    );

    while current_time < options.tstop {
        if current_time + h > options.tstop {
            h = options.tstop - current_time;
        }

        match options.method {
            IntegrationMethod::BackwardEuler => {
                let target_time = current_time + h;
                let (next_state, iters) = solve_transient_stage(
                    graph,
                    context,
                    options,
                    target_time,
                    h,
                    &current_state,
                    &dyn_state,
                    StageMode::BackwardEuler,
                )?;

                solution.total_newton_iters += iters;
                solution.accepted_steps += 1;
                current_time = target_time;

                update_dynamic_history(
                    &mut dyn_state,
                    graph,
                    &current_state,
                    &next_state,
                    target_time,
                    h,
                    StageMode::BackwardEuler,
                );
                current_state = next_state;

                record_step(
                    &mut solution,
                    current_time,
                    &current_state,
                    &dyn_state,
                    active_nodes,
                    total_branches,
                    iters,
                );
            }
            IntegrationMethod::Trapezoidal => {
                let target_time = current_time + h;
                let (next_state, iters) = solve_transient_stage(
                    graph,
                    context,
                    options,
                    target_time,
                    h,
                    &current_state,
                    &dyn_state,
                    StageMode::Trapezoidal,
                )?;

                solution.total_newton_iters += iters;
                solution.accepted_steps += 1;
                current_time = target_time;

                update_dynamic_history(
                    &mut dyn_state,
                    graph,
                    &current_state,
                    &next_state,
                    target_time,
                    h,
                    StageMode::Trapezoidal,
                );
                current_state = next_state;

                record_step(
                    &mut solution,
                    current_time,
                    &current_state,
                    &dyn_state,
                    active_nodes,
                    total_branches,
                    iters,
                );
            }
            IntegrationMethod::TrBdf2 => {
                let gamma = TR_BDF2_GAMMA;
                let t_gamma = current_time + gamma * h;
                let t_next = current_time + h;

                // Stage 1: Trapezoidal step to t_gamma
                let (state_gamma, iters1) = solve_transient_stage(
                    graph,
                    context,
                    options,
                    t_gamma,
                    h,
                    &current_state,
                    &dyn_state,
                    StageMode::TrBdf2Stage1,
                )?;

                // Stage 2: BDF2 step to t_next
                let mut dyn_gamma = dyn_state.clone();
                update_dynamic_history(
                    &mut dyn_gamma,
                    graph,
                    &current_state,
                    &state_gamma,
                    t_gamma,
                    gamma * h,
                    StageMode::Trapezoidal,
                );

                let (state_next, iters2) = solve_transient_stage(
                    graph,
                    context,
                    options,
                    t_next,
                    h,
                    &state_gamma,
                    &dyn_gamma,
                    StageMode::TrBdf2Stage2 {
                        state_n: &current_state,
                        state_gamma: &state_gamma,
                    },
                )?;

                let total_iters = iters1 + iters2;
                solution.total_newton_iters += total_iters;

                // Local Truncation Error evaluation
                let (_err_norm, accepted, next_h) = evaluate_tr_bdf2_lte(
                    &current_state,
                    &state_gamma,
                    &state_next,
                    h,
                    &options.step_control,
                );

                if accepted {
                    solution.accepted_steps += 1;
                    current_time = t_next;

                    update_dynamic_history(
                        &mut dyn_state,
                        graph,
                        &current_state,
                        &state_next,
                        t_next,
                        h,
                        StageMode::TrBdf2Full {
                            state_gamma: &state_gamma,
                        },
                    );
                    current_state = state_next;

                    record_step(
                        &mut solution,
                        current_time,
                        &current_state,
                        &dyn_state,
                        active_nodes,
                        total_branches,
                        total_iters,
                    );
                    h = next_h.clamp(
                        options.step_control.min_step,
                        options.tmax.unwrap_or(options.step_control.max_step),
                    );
                } else {
                    solution.rejected_steps += 1;
                    h = next_h.max(options.step_control.min_step);
                }
            }
        }
    }

    Ok(solution)
}

#[derive(Debug, Clone, Copy)]
enum StageMode<'a> {
    BackwardEuler,
    Trapezoidal,
    TrBdf2Stage1,
    TrBdf2Stage2 {
        state_n: &'a [f64],
        state_gamma: &'a [f64],
    },
    TrBdf2Full {
        state_gamma: &'a [f64],
    },
}

/// Solves a single non-linear algebraic MNA stage at time $t$.
#[allow(clippy::too_many_arguments)]
fn solve_transient_stage(
    graph: &CircuitGraph,
    context: &ModelContext,
    options: &TransientOptions,
    t: f64,
    h: f64,
    initial_guess: &[f64],
    dyn_state: &DynamicState,
    stage: StageMode<'_>,
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

    for iter in 0..options.newton.max_iters {
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
                    let v_val = if let Some(wf) = options.waveforms.get(name) {
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
                    let i_val = if let Some(wf) = options.waveforms.get(name) {
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
                } => {
                    let (v_n, i_n) = dyn_state
                        .capacitors
                        .get(name)
                        .copied()
                        .unwrap_or((0.0, 0.0));
                    let companion = match stage {
                        StageMode::BackwardEuler => {
                            CapacitorCompanion::backward_euler(*capacitance, h, v_n)
                        }
                        StageMode::Trapezoidal => {
                            CapacitorCompanion::trapezoidal(*capacitance, h, v_n, i_n)
                        }
                        StageMode::TrBdf2Stage1 => {
                            CapacitorCompanion::tr_bdf2_stage1(*capacitance, h, v_n, i_n)
                        }
                        StageMode::TrBdf2Stage2 {
                            state_n,
                            state_gamma,
                        } => {
                            let v_n_state = get_v(*pos, state_n) - get_v(*neg, state_n);
                            let v_gamma_state = get_v(*pos, state_gamma) - get_v(*neg, state_gamma);
                            CapacitorCompanion::tr_bdf2_stage2(
                                *capacitance,
                                h,
                                v_n_state,
                                v_gamma_state,
                            )
                        }
                        StageMode::TrBdf2Full { .. } => unreachable!(),
                    };

                    stamp_conductance(&mut g_builder, *pos, *neg, companion.g_eq);
                    if let Some(p) = node_to_mna_idx(*pos) {
                        rhs[p] += companion.i_eq;
                    }
                    if let Some(n) = node_to_mna_idx(*neg) {
                        rhs[n] -= companion.i_eq;
                    }
                }
                ComponentRecord::Inductor {
                    name,
                    pos,
                    neg,
                    branch,
                    inductance,
                    ..
                } => {
                    let br_idx = active_nodes + branch.index();
                    let (i_n, v_n) = dyn_state.inductors.get(name).copied().unwrap_or((0.0, 0.0));
                    let companion = match stage {
                        StageMode::BackwardEuler => {
                            InductorCompanion::backward_euler(*inductance, h, i_n)
                        }
                        StageMode::Trapezoidal => {
                            InductorCompanion::trapezoidal(*inductance, h, i_n, v_n)
                        }
                        StageMode::TrBdf2Stage1 => {
                            InductorCompanion::tr_bdf2_stage1(*inductance, h, i_n, v_n)
                        }
                        StageMode::TrBdf2Stage2 {
                            state_n,
                            state_gamma,
                        } => {
                            let i_n_state = state_n[br_idx];
                            let i_gamma_state = state_gamma[br_idx];
                            InductorCompanion::tr_bdf2_stage2(
                                *inductance,
                                h,
                                i_n_state,
                                i_gamma_state,
                            )
                        }
                        StageMode::TrBdf2Full { .. } => unreachable!(),
                    };

                    // Auxiliary branch row: (v_pos - v_neg) - R_eq * i_L = - V_eq
                    if let Some(p) = node_to_mna_idx(*pos) {
                        g_builder.add(br_idx, p, 1.0);
                        g_builder.add(p, br_idx, 1.0);
                    }
                    if let Some(n) = node_to_mna_idx(*neg) {
                        g_builder.add(br_idx, n, -1.0);
                        g_builder.add(n, br_idx, -1.0);
                    }
                    g_builder.add(br_idx, br_idx, -companion.r_eq);
                    rhs[br_idx] -= companion.v_eq;
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

        // 2. Stamp Non-Linear Semiconductor Devices (Diodes, MOSFETs, BJTs)
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

                    let eval = model.evaluate(v_d, v_g, v_s, v_b, temp_k);
                    let v_ds = v_d - v_s;
                    let v_gs = v_g - v_s;
                    let v_bs = v_b - v_s;

                    let i_eq = eval.g_m * v_gs + eval.g_ds * v_ds + eval.g_mbs * v_bs - eval.i_ds;
                    let comp_stamp = MosfetCompanion {
                        g_m: eval.g_m,
                        g_ds: eval.g_ds,
                        g_mbs: eval.g_mbs,
                        i_eq,
                    };

                    stamp_mosfet_companion(
                        &mut g_builder,
                        &mut rhs,
                        *drain,
                        *gate,
                        *source,
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
                options.newton.reltol * x[i].abs().max(x_new[i].abs()) + options.newton.vntol
            } else {
                options.newton.reltol * x[i].abs().max(x_new[i].abs()) + options.newton.abstol
            };
            if (x_new[i] - x[i]).abs() > tol {
                converged = false;
                break;
            }
        }

        // Apply damping / voltage limiting on PN junctions
        let mut x_damped = x_new.clone();
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

    Ok((x, options.newton.max_iters))
}

fn update_dynamic_history(
    dyn_state: &mut DynamicState,
    graph: &CircuitGraph,
    state_old: &[f64],
    state_new: &[f64],
    target_time: f64,
    h: f64,
    stage: StageMode<'_>,
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
                    StageMode::BackwardEuler => *capacitance * (v_new - v_old) / h,
                    StageMode::Trapezoidal => (2.0 * capacitance / h) * (v_new - v_old) - i_old,
                    StageMode::TrBdf2Stage1 => {
                        let h1 = TR_BDF2_GAMMA * h;
                        (2.0 * capacitance / h1) * (v_new - v_old) - i_old
                    }
                    StageMode::TrBdf2Stage2 {
                        state_n,
                        state_gamma,
                    } => {
                        let gamma = TR_BDF2_GAMMA;
                        let v_n = get_v(*pos, state_n) - get_v(*neg, state_n);
                        let v_gamma = get_v(*pos, state_gamma) - get_v(*neg, state_gamma);
                        let dv_dt = (2.0 - gamma) / ((1.0 - gamma) * h) * v_new
                            - 1.0 / (gamma * (1.0 - gamma) * h) * v_gamma
                            + (1.0 - gamma) / (gamma * h) * v_n;
                        *capacitance * dv_dt
                    }
                    StageMode::TrBdf2Full { state_gamma } => {
                        let gamma = TR_BDF2_GAMMA;
                        let v_n = get_v(*pos, state_old) - get_v(*neg, state_old);
                        let v_gamma = get_v(*pos, state_gamma) - get_v(*neg, state_gamma);
                        let dv_dt = (2.0 - gamma) / ((1.0 - gamma) * h) * v_new
                            - 1.0 / (gamma * (1.0 - gamma) * h) * v_gamma
                            + (1.0 - gamma) / (gamma * h) * v_n;
                        *capacitance * dv_dt
                    }
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
                    match stage {
                        StageMode::TrBdf2Full { state_gamma } => {
                            let gamma = TR_BDF2_GAMMA;
                            let t_gamma = target_time - (1.0 - gamma) * h;
                            let v1_gamma =
                                get_v(*in_pos, state_gamma) - get_v(*in_neg, state_gamma);
                            let v2_gamma =
                                get_v(*out_pos, state_gamma) - get_v(*out_neg, state_gamma);
                            history.record_step(t_gamma, v1_gamma, v2_gamma, *td);
                            history.record_step(target_time, v1, v2, *td);
                        }
                        _ => {
                            history.record_step(target_time, v1, v2, *td);
                        }
                    }
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
    dyn_state: &DynamicState,
    active_nodes: usize,
    total_branches: usize,
    iterations: usize,
) {
    let mut voltages = vec![0.0; active_nodes + 1];
    voltages[0] = 0.0;
    voltages[1..=active_nodes].copy_from_slice(&state[..active_nodes]);

    let mut branch_currents = vec![0.0; total_branches];
    branch_currents.copy_from_slice(&state[active_nodes..active_nodes + total_branches]);

    let capacitor_currents = dyn_state
        .capacitors
        .iter()
        .map(|(k, v)| (k.clone(), v.1))
        .collect();

    solution.steps.push(TransientStep {
        time,
        voltages,
        branch_currents,
        capacitor_currents,
        iterations,
    });
}
