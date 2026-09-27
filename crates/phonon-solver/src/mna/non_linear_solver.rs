//! Non-linear Newton-Raphson solver with PN junction voltage limiting,
//! Gmin stepping, and source-stepping continuation for physical semiconductor circuits.

use super::assembler::SolverOptions;
use super::diagnostics::diagnose_mna_singularity;
use super::linear_solver::DcSolution;
use super::stamp::*;
use crate::error::SolverError;
use crate::sparse::builder::SparseMatrixBuilder;
use crate::sparse::lu::SparseLuFactorization;
use crate::sparse::markowitz::MarkowitzOptions;
use phonon_core::{
    AtomisticChannelType, CircuitGraph, ComponentRecord, MemristorState, NodeId, OpticalPortId,
    OpticalSignal, T_REF,
};
use phonon_models::{
    compute_vcrit, pn_junction_limit, BjtModel, CarbonNanotube, DiodeModel,
    DisplacementDamageModel, ElectroOpticModulatorModel, ElectromigrationModel,
    FerroelectricFetModel, FilamentaryRramModel, HeavyIonStrikeModel, JosephsonRcsjModel,
    LaserDiodeModel, MosfetModel, NeuralSurrogateCompanion, NeuronState, ParasiticThyristorModel,
    PhaseChangeMemoryModel, PhotodetectorModel, SpikingNeuronModel, TcadDevice, TcadDeviceBuilder,
    TmdMonolayer, TotalIonizingDoseModel,
};
use std::collections::HashMap;

/// Atomistic device model wrapper for 2D materials, CNTs, and reliability physics.
#[derive(Debug, Clone)]
pub enum AtomisticCompanionModel {
    Tmd(TmdMonolayer),
    Cnt(CarbonNanotube),
    Electromigration(ElectromigrationModel),
}

/// Model parameters and configuration for active physical devices in the circuit.
#[derive(Debug, Clone)]
pub struct ModelContext {
    pub diode_models: HashMap<String, DiodeModel>,
    pub mosfet_models: HashMap<String, MosfetModel>,
    pub bjt_models: HashMap<String, BjtModel>,
    pub tcad_devices: HashMap<String, TcadDevice>,
    pub neural_surrogates: HashMap<String, NeuralSurrogateCompanion>,
    pub josephson_junctions: HashMap<String, JosephsonRcsjModel>,
    pub atomistic_channels: HashMap<String, AtomisticCompanionModel>,
    pub laser_diodes: HashMap<String, LaserDiodeModel>,
    pub photodetectors: HashMap<String, PhotodetectorModel>,
    pub modulators: HashMap<String, ElectroOpticModulatorModel>,
    pub optical_signals: HashMap<OpticalPortId, OpticalSignal>,
    pub rram_models: HashMap<String, FilamentaryRramModel>,
    pub pcm_models: HashMap<String, PhaseChangeMemoryModel>,
    pub fefet_models: HashMap<String, FerroelectricFetModel>,
    pub neuron_models: HashMap<String, SpikingNeuronModel>,
    pub memristor_states: HashMap<String, MemristorState>,
    pub neuron_states: HashMap<String, NeuronState>,
    pub heavy_ion_strikes: HashMap<String, HeavyIonStrikeModel>,
    pub tid_models: HashMap<String, TotalIonizingDoseModel>,
    pub ddd_models: HashMap<String, DisplacementDamageModel>,
    pub thyristor_models: HashMap<String, ParasiticThyristorModel>,
    pub current_time_s: f64,
    pub temperature_kelvin: f64,
}

impl Default for ModelContext {
    fn default() -> Self {
        Self {
            diode_models: HashMap::new(),
            mosfet_models: HashMap::new(),
            bjt_models: HashMap::new(),
            tcad_devices: HashMap::new(),
            neural_surrogates: HashMap::new(),
            josephson_junctions: HashMap::new(),
            atomistic_channels: HashMap::new(),
            laser_diodes: HashMap::new(),
            photodetectors: HashMap::new(),
            modulators: HashMap::new(),
            optical_signals: HashMap::new(),
            rram_models: HashMap::new(),
            pcm_models: HashMap::new(),
            fefet_models: HashMap::new(),
            neuron_models: HashMap::new(),
            memristor_states: HashMap::new(),
            neuron_states: HashMap::new(),
            heavy_ion_strikes: HashMap::new(),
            tid_models: HashMap::new(),
            ddd_models: HashMap::new(),
            thyristor_models: HashMap::new(),
            current_time_s: 0.0,
            temperature_kelvin: T_REF,
        }
    }
}

impl ModelContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_diode_model(&mut self, name: &str, model: DiodeModel) {
        self.diode_models.insert(name.to_string(), model);
    }

    pub fn set_mosfet_model(&mut self, name: &str, model: MosfetModel) {
        self.mosfet_models.insert(name.to_string(), model);
    }

    pub fn set_bjt_model(&mut self, name: &str, model: BjtModel) {
        self.bjt_models.insert(name.to_string(), model);
    }

    pub fn set_tcad_device(&mut self, name: &str, device: TcadDevice) {
        self.tcad_devices.insert(name.to_string(), device);
    }

    pub fn set_neural_surrogate(&mut self, name: &str, surrogate: NeuralSurrogateCompanion) {
        self.neural_surrogates.insert(name.to_string(), surrogate);
    }

    pub fn get_diode_model(&self, name: &str) -> DiodeModel {
        self.diode_models.get(name).copied().unwrap_or_default()
    }

    pub fn get_mosfet_model(&self, name: &str) -> MosfetModel {
        self.mosfet_models.get(name).copied().unwrap_or_default()
    }

    pub fn get_bjt_model(&self, name: &str) -> BjtModel {
        self.bjt_models.get(name).copied().unwrap_or_default()
    }

    pub fn get_tcad_device(&self, name: &str) -> Option<&TcadDevice> {
        self.tcad_devices.get(name)
    }

    pub fn get_neural_surrogate(&self, name: &str) -> Option<&NeuralSurrogateCompanion> {
        self.neural_surrogates.get(name)
    }

    pub fn set_josephson_junction(&mut self, name: &str, jj: JosephsonRcsjModel) {
        self.josephson_junctions.insert(name.to_string(), jj);
    }

    pub fn get_josephson_junction(&self, name: &str) -> Option<&JosephsonRcsjModel> {
        self.josephson_junctions.get(name)
    }

    pub fn get_josephson_junction_mut(&mut self, name: &str) -> Option<&mut JosephsonRcsjModel> {
        self.josephson_junctions.get_mut(name)
    }

    pub fn set_atomistic_tmd(&mut self, name: &str, tmd: TmdMonolayer) {
        self.atomistic_channels
            .insert(name.to_string(), AtomisticCompanionModel::Tmd(tmd));
    }

    pub fn set_atomistic_cnt(&mut self, name: &str, cnt: CarbonNanotube) {
        self.atomistic_channels
            .insert(name.to_string(), AtomisticCompanionModel::Cnt(cnt));
    }

    pub fn set_atomistic_em(&mut self, name: &str, em: ElectromigrationModel) {
        self.atomistic_channels.insert(
            name.to_string(),
            AtomisticCompanionModel::Electromigration(em),
        );
    }

    pub fn get_atomistic_channel(&self, name: &str) -> Option<&AtomisticCompanionModel> {
        self.atomistic_channels.get(name)
    }

    pub fn set_laser_diode(&mut self, name: &str, laser: LaserDiodeModel) {
        self.laser_diodes.insert(name.to_string(), laser);
    }

    pub fn get_laser_diode(&self, name: &str) -> Option<&LaserDiodeModel> {
        self.laser_diodes.get(name)
    }

    pub fn set_photodetector(&mut self, name: &str, pd: PhotodetectorModel) {
        self.photodetectors.insert(name.to_string(), pd);
    }

    pub fn get_photodetector(&self, name: &str) -> Option<&PhotodetectorModel> {
        self.photodetectors.get(name)
    }

    pub fn set_modulator(&mut self, name: &str, mod_model: ElectroOpticModulatorModel) {
        self.modulators.insert(name.to_string(), mod_model);
    }

    pub fn get_modulator(&self, name: &str) -> Option<&ElectroOpticModulatorModel> {
        self.modulators.get(name)
    }

    pub fn set_optical_signal(&mut self, port: OpticalPortId, signal: OpticalSignal) {
        self.optical_signals.insert(port, signal);
    }

    pub fn get_optical_signal(&self, port: OpticalPortId) -> Option<&OpticalSignal> {
        self.optical_signals.get(&port)
    }

    pub fn set_rram_model(&mut self, name: &str, model: FilamentaryRramModel) {
        self.rram_models.insert(name.to_string(), model);
    }

    pub fn get_rram_model(&self, name: &str) -> Option<&FilamentaryRramModel> {
        self.rram_models.get(name)
    }

    pub fn set_pcm_model(&mut self, name: &str, model: PhaseChangeMemoryModel) {
        self.pcm_models.insert(name.to_string(), model);
    }

    pub fn get_pcm_model(&self, name: &str) -> Option<&PhaseChangeMemoryModel> {
        self.pcm_models.get(name)
    }

    pub fn set_fefet_model(&mut self, name: &str, model: FerroelectricFetModel) {
        self.fefet_models.insert(name.to_string(), model);
    }

    pub fn get_fefet_model(&self, name: &str) -> Option<&FerroelectricFetModel> {
        self.fefet_models.get(name)
    }

    pub fn set_neuron_model(&mut self, name: &str, model: SpikingNeuronModel) {
        self.neuron_models.insert(name.to_string(), model);
    }

    pub fn get_neuron_model(&self, name: &str) -> Option<&SpikingNeuronModel> {
        self.neuron_models.get(name)
    }

    pub fn set_memristor_state(&mut self, name: &str, state: MemristorState) {
        self.memristor_states.insert(name.to_string(), state);
    }

    pub fn get_memristor_state(&self, name: &str) -> Option<&MemristorState> {
        self.memristor_states.get(name)
    }

    pub fn set_neuron_state(&mut self, name: &str, state: NeuronState) {
        self.neuron_states.insert(name.to_string(), state);
    }

    pub fn get_neuron_state(&self, name: &str) -> Option<&NeuronState> {
        self.neuron_states.get(name)
    }

    pub fn set_heavy_ion_strike(&mut self, name: &str, model: HeavyIonStrikeModel) {
        self.heavy_ion_strikes.insert(name.to_string(), model);
    }

    pub fn get_heavy_ion_strike(&self, name: &str) -> Option<&HeavyIonStrikeModel> {
        self.heavy_ion_strikes.get(name)
    }

    pub fn set_tid_model(&mut self, name: &str, model: TotalIonizingDoseModel) {
        self.tid_models.insert(name.to_string(), model);
    }

    pub fn get_tid_model(&self, name: &str) -> Option<&TotalIonizingDoseModel> {
        self.tid_models.get(name)
    }

    pub fn set_ddd_model(&mut self, name: &str, model: DisplacementDamageModel) {
        self.ddd_models.insert(name.to_string(), model);
    }

    pub fn get_ddd_model(&self, name: &str) -> Option<&DisplacementDamageModel> {
        self.ddd_models.get(name)
    }

    pub fn set_thyristor_model(&mut self, name: &str, model: ParasiticThyristorModel) {
        self.thyristor_models.insert(name.to_string(), model);
    }

    pub fn get_thyristor_model(&self, name: &str) -> Option<&ParasiticThyristorModel> {
        self.thyristor_models.get(name)
    }

    pub fn get_thyristor_model_mut(&mut self, name: &str) -> Option<&mut ParasiticThyristorModel> {
        self.thyristor_models.get_mut(name)
    }

    pub fn set_current_time(&mut self, time_s: f64) {
        self.current_time_s = time_s;
    }
}

/// Algorithmic controls for non-linear Newton-Raphson iteration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NewtonOptions {
    /// Relative convergence tolerance (typically 1e-3).
    pub reltol: f64,
    /// Absolute node voltage tolerance in Volts (typically 1e-6 V).
    pub vntol: f64,
    /// Absolute branch current tolerance in Amperes (typically 1e-12 A).
    pub abstol: f64,
    /// Maximum allowed Newton iterations before continuation fallback.
    pub max_iters: usize,
    /// Enable Gmin stepping continuation when standard Newton iteration fails.
    pub enable_gmin_stepping: bool,
    /// Enable source stepping continuation when Gmin stepping fails.
    pub enable_source_stepping: bool,
}

impl Default for NewtonOptions {
    fn default() -> Self {
        Self {
            reltol: 1e-3,
            vntol: 1e-6,
            abstol: 1e-12,
            max_iters: 100,
            enable_gmin_stepping: true,
            enable_source_stepping: true,
        }
    }
}

/// Solves the non-linear DC operating point for a circuit containing diodes, MOSFETs, and BJTs.
pub fn solve_dc_non_linear(
    graph: &CircuitGraph,
    context: &ModelContext,
    options: &NewtonOptions,
) -> Result<DcSolution, SolverError> {
    graph.validate_topology()?;

    let active_nodes = graph.active_nodes();
    let total_branches = graph.total_branches();
    let total_dim = active_nodes + total_branches;

    if total_dim == 0 {
        return Ok(DcSolution {
            node_voltages: vec![0.0; graph.total_nodes()],
            branch_currents: Vec::new(),
            condition_ratio: 1.0,
        });
    }

    // Step 1: Standard damped Newton-Raphson attempt
    let mut initial_guess = vec![0.0; total_dim];
    let solver_opts = SolverOptions::default();

    match run_newton_loop(
        graph,
        context,
        options,
        &solver_opts,
        &mut initial_guess,
        1.0,
        0.0,
    ) {
        Ok(sol) => return Ok(sol),
        Err(e) => {
            if !options.enable_gmin_stepping && !options.enable_source_stepping {
                return Err(e);
            }
        }
    }

    // Step 2: Gmin Stepping Continuation
    if options.enable_gmin_stepping {
        let mut gmin = 1e-2;
        let mut state = vec![0.0; total_dim];
        let mut gmin_success = true;

        for _ in 0..10 {
            let s_opts = SolverOptions {
                auto_gmin_shunt: true,
                gmin_value: gmin,
            };
            if run_newton_loop(graph, context, options, &s_opts, &mut state, 1.0, gmin).is_err() {
                gmin_success = false;
                break;
            }
            gmin *= 0.1;
        }

        if gmin_success {
            // Final solve without artificial gmin
            let final_opts = SolverOptions::default();
            if let Ok(sol) =
                run_newton_loop(graph, context, options, &final_opts, &mut state, 1.0, 0.0)
            {
                return Ok(sol);
            }
        }
    }

    // Step 3: Source Stepping Continuation (lambda from 0.0 to 1.0)
    if options.enable_source_stepping {
        let mut state = vec![0.0; total_dim];
        let num_steps = 20;

        for step in 1..=num_steps {
            let lambda = (step as f64) / (num_steps as f64);
            let s_opts = SolverOptions {
                auto_gmin_shunt: true,
                gmin_value: 1e-11,
            };
            run_newton_loop(graph, context, options, &s_opts, &mut state, lambda, 0.0)?;
        }

        let final_opts = SolverOptions::default();
        return run_newton_loop(graph, context, options, &final_opts, &mut state, 1.0, 0.0);
    }

    Err(SolverError::NumericalAnomaly {
        detail: "Newton-Raphson iteration and all continuation methods failed to converge"
            .to_string(),
    })
}

/// Executes an inner Newton-Raphson iteration loop for a fixed source factor lambda and gmin.
fn run_newton_loop(
    graph: &CircuitGraph,
    context: &ModelContext,
    options: &NewtonOptions,
    solver_opts: &SolverOptions,
    x: &mut [f64],
    lambda: f64,
    gmin: f64,
) -> Result<DcSolution, SolverError> {
    let active_nodes = graph.active_nodes();
    let total_branches = graph.total_branches();
    let total_dim = active_nodes + total_branches;

    let markowitz_opts = MarkowitzOptions::default();
    let temp_k = context.temperature_kelvin;

    let mut last_cond_ratio;

    for _iter in 0..options.max_iters {
        let mut g_builder = SparseMatrixBuilder::with_capacity(total_dim, total_dim, total_dim * 6);
        let mut rhs = vec![0.0; total_dim];

        if solver_opts.auto_gmin_shunt || gmin > 0.0 {
            let g_val = solver_opts.gmin_value.max(gmin);
            for node_idx in 0..active_nodes {
                g_builder.add(node_idx, node_idx, g_val);
            }
        }

        // Helper closures to read voltage at a node from current state x
        let get_v = |node: NodeId, state: &[f64]| -> f64 {
            if node.is_ground() {
                0.0
            } else {
                state[node.index() - 1]
            }
        };

        // 1. Stamp Linear Components
        for comp in graph.components() {
            match comp {
                ComponentRecord::Resistor {
                    pos,
                    neg,
                    resistance,
                    ..
                } => {
                    let g = 1.0 / resistance;
                    stamp_conductance(&mut g_builder, *pos, *neg, g);
                }
                ComponentRecord::Capacitor { .. } => {
                    // Open circuit in DC
                }
                ComponentRecord::Inductor {
                    pos, neg, branch, ..
                } => {
                    let br_idx = active_nodes + branch.index();
                    stamp_voltage_source(&mut g_builder, &mut rhs, *pos, *neg, br_idx, 0.0);
                }
                ComponentRecord::VoltageSource {
                    pos,
                    neg,
                    dc_value,
                    branch,
                    ..
                } => {
                    let br_idx = active_nodes + branch.index();
                    stamp_voltage_source(
                        &mut g_builder,
                        &mut rhs,
                        *pos,
                        *neg,
                        br_idx,
                        lambda * dc_value,
                    );
                }
                ComponentRecord::CurrentSource {
                    pos, neg, dc_value, ..
                } => {
                    stamp_current_source(&mut rhs, *pos, *neg, lambda * dc_value);
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
                // 2. Stamp Non-Linear Semiconductor Companion Models
                ComponentRecord::Diode { name, pos, neg } => {
                    let model = context.get_diode_model(name);
                    let v_pos = get_v(*pos, x);
                    let v_neg = get_v(*neg, x);
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
                    let v_d = get_v(*drain, x);
                    let v_g = get_v(*gate, x);
                    let v_s = get_v(*source, x);
                    let v_b = get_v(*bulk, x);

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
                    let v_c = get_v(*collector, x);
                    let v_b = get_v(*base, x);
                    let v_e = get_v(*emitter, x);

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
                ComponentRecord::TransmissionLine {
                    in_pos,
                    in_neg,
                    out_pos,
                    out_neg,
                    ..
                } => {
                    let g_dc = 1e6;
                    stamp_conductance(&mut g_builder, *in_pos, *out_pos, g_dc);
                    stamp_conductance(&mut g_builder, *in_neg, *out_neg, g_dc);
                }
                ComponentRecord::TcadDiode { name, pos, neg } => {
                    let fallback = TcadDeviceBuilder::new_pn_junction(name).build();
                    let tcad = context.get_tcad_device(name).unwrap_or(&fallback);
                    let v_pos = get_v(*pos, x);
                    let v_neg = get_v(*neg, x);
                    let v_d = v_pos - v_neg;

                    let (i_d, g_d) = tcad.evaluate_diode(v_d, temp_k);
                    let i_eq = g_d * v_d - i_d;

                    stamp_diode_companion(&mut g_builder, &mut rhs, *pos, *neg, g_d, i_eq);
                }
                ComponentRecord::TcadMosfet {
                    name,
                    drain,
                    gate,
                    source,
                    bulk,
                } => {
                    let fallback = TcadDeviceBuilder::new_mosfet(name).build();
                    let tcad = context.get_tcad_device(name).unwrap_or(&fallback);
                    let v_d = get_v(*drain, x);
                    let v_g = get_v(*gate, x);
                    let v_s = get_v(*source, x);
                    let v_b = get_v(*bulk, x);

                    let v_ds = v_d - v_s;
                    let v_gs = v_g - v_s;
                    let v_bs = v_b - v_s;

                    let (i_ds, g_m, g_ds, g_mbs) = tcad.evaluate_mosfet(v_ds, v_gs, v_bs, temp_k);
                    let i_eq = g_m * v_gs + g_ds * v_ds + g_mbs * v_bs - i_ds;
                    let comp_stamp = MosfetCompanion {
                        g_m,
                        g_ds,
                        g_mbs,
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
                ComponentRecord::NeuralSurrogate {
                    name,
                    nodes,
                    surrogate_id,
                } => {
                    let surrogate = context
                        .get_neural_surrogate(surrogate_id)
                        .or_else(|| context.get_neural_surrogate(name));
                    if let Some(surr) = surrogate {
                        if nodes.len() == 2 {
                            let pos = nodes[0];
                            let neg = nodes[1];
                            let v_pos = get_v(pos, x);
                            let v_neg = get_v(neg, x);
                            let v_d = v_pos - v_neg;

                            let (i_d, g_d) = surr.evaluate_diode(v_d);
                            let i_eq = g_d * v_d - i_d;

                            stamp_diode_companion(&mut g_builder, &mut rhs, pos, neg, g_d, i_eq);
                        } else if nodes.len() >= 3 {
                            let drain = nodes[0];
                            let gate = nodes[1];
                            let source = nodes[2];
                            let bulk = if nodes.len() > 3 { nodes[3] } else { nodes[2] };

                            let v_d = get_v(drain, x);
                            let v_g = get_v(gate, x);
                            let v_s = get_v(source, x);
                            let v_b = get_v(bulk, x);

                            let v_ds = v_d - v_s;
                            let v_gs = v_g - v_s;
                            let v_bs = v_b - v_s;

                            let (i_ds, g_m, g_ds, g_mbs) =
                                surr.evaluate_transistor(v_ds, v_gs, v_bs);
                            let i_eq = g_m * v_gs + g_ds * v_ds + g_mbs * v_bs - i_ds;
                            let comp_stamp = MosfetCompanion {
                                g_m,
                                g_ds,
                                g_mbs,
                                i_eq,
                            };

                            stamp_mosfet_companion(
                                &mut g_builder,
                                &mut rhs,
                                drain,
                                gate,
                                source,
                                bulk,
                                &comp_stamp,
                            );
                        }
                    }
                }
                ComponentRecord::JosephsonJunction {
                    name,
                    pos,
                    neg,
                    ic,
                    rn,
                    phase_init,
                    ..
                } => {
                    let v_pos = get_v(*pos, x);
                    let v_neg = get_v(*neg, x);
                    let v_jj = v_pos - v_neg;

                    let (g_eq, i_eq) = if let Some(jj) = context.get_josephson_junction(name) {
                        jj.dc_stamp(v_jj)
                    } else {
                        let default_jj = JosephsonRcsjModel::new(*ic, *rn, 1e-13, *phase_init);
                        default_jj.dc_stamp(v_jj)
                    };

                    stamp_diode_companion(&mut g_builder, &mut rhs, *pos, *neg, g_eq, i_eq);
                }
                ComponentRecord::AtomisticChannel {
                    name,
                    drain,
                    gate,
                    source,
                    channel_type,
                } => {
                    let v_d = get_v(*drain, x);
                    let v_g = get_v(*gate, x);
                    let v_s = get_v(*source, x);
                    let v_ds = v_d - v_s;
                    let v_gs = v_g - v_s;

                    let eval_ids = |vgs: f64, vds: f64| -> f64 {
                        if let Some(comp) = context.get_atomistic_channel(name) {
                            match comp {
                                AtomisticCompanionModel::Tmd(tmd) => tmd.evaluate_fet_current(
                                    vgs, vds, 0.35, 1e-6, 30e-9, 1.2e-9, temp_k,
                                ),
                                AtomisticCompanionModel::Cnt(cnt) => {
                                    cnt.evaluate_fet_current(vgs, vds, 0.4, 50e-9, temp_k)
                                }
                                AtomisticCompanionModel::Electromigration(em) => {
                                    vds / em.initial_resistance_ohms.max(1e-12)
                                }
                            }
                        } else {
                            match channel_type {
                                AtomisticChannelType::TmdMonolayer {
                                    species,
                                    length_m,
                                    width_m,
                                } => {
                                    let tmd = if species.eq_ignore_ascii_case("ws2") {
                                        TmdMonolayer::ws2()
                                    } else {
                                        TmdMonolayer::mos2()
                                    };
                                    tmd.evaluate_fet_current(
                                        vgs, vds, 0.35, *width_m, *length_m, 1.2e-9, temp_k,
                                    )
                                }
                                AtomisticChannelType::CarbonNanotube { n, m, length_m } => {
                                    let cnt = CarbonNanotube::new(*n as u32, *m as u32);
                                    cnt.evaluate_fet_current(vgs, vds, 0.4, *length_m, temp_k)
                                }
                                AtomisticChannelType::InterconnectNanowire {
                                    length_m,
                                    width_m,
                                    height_m,
                                    ..
                                } => {
                                    let r0: f64 =
                                        (1.68e-8 * length_m) / (width_m * height_m).max(1e-20);
                                    vds / r0.max(1e-6)
                                }
                            }
                        }
                    };

                    let delta = 1e-5;
                    let ids_base = eval_ids(v_gs, v_ds);
                    let ids_vgs = eval_ids(v_gs + delta, v_ds);
                    let ids_vds = eval_ids(v_gs, v_ds + delta);

                    let gm = (ids_vgs - ids_base) / delta;
                    let gds = (ids_vds - ids_base) / delta;
                    let i_eq = gm * v_gs + gds * v_ds - ids_base;

                    let comp_stamp = MosfetCompanion {
                        g_m: gm,
                        g_ds: gds,
                        g_mbs: 0.0,
                        i_eq,
                    };

                    stamp_mosfet_companion(
                        &mut g_builder,
                        &mut rhs,
                        *drain,
                        *gate,
                        *source,
                        *source,
                        &comp_stamp,
                    );
                }
                ComponentRecord::OpticalWaveguide { .. }
                | ComponentRecord::MicroRingResonator { .. } => {
                    // Optical passive components do not stamp electrical MNA equations
                }
                ComponentRecord::ElectroOpticModulator {
                    elec_pos, elec_neg, ..
                } => {
                    // Modulator electrodes have minute leakage conductance in DC
                    stamp_conductance(&mut g_builder, *elec_pos, *elec_neg, 1e-12);
                }
                ComponentRecord::LaserDiode {
                    name,
                    anode,
                    cathode,
                    ..
                } => {
                    let v_anode = get_v(*anode, x);
                    let v_cathode = get_v(*cathode, x);
                    let v_d = v_anode - v_cathode;

                    let (i_d, g_d) = if let Some(laser) = context.get_laser_diode(name) {
                        laser.electrical_companion(v_d, temp_k)
                    } else {
                        let default_laser = LaserDiodeModel::dfb_1550nm();
                        default_laser.electrical_companion(v_d, temp_k)
                    };

                    let i_eq = g_d * v_d - i_d;
                    stamp_diode_companion(&mut g_builder, &mut rhs, *anode, *cathode, g_d, i_eq);
                }
                ComponentRecord::Photodetector {
                    name,
                    opt_in,
                    anode,
                    cathode,
                    ..
                } => {
                    let v_anode = get_v(*anode, x);
                    let v_cathode = get_v(*cathode, x);
                    let v_rev = (v_cathode - v_anode).max(0.0);

                    let (detector, opt_sig) = {
                        let pd = context
                            .get_photodetector(name)
                            .cloned()
                            .unwrap_or_else(PhotodetectorModel::ge_on_si_pin);
                        let sig = context
                            .get_optical_signal(*opt_in)
                            .copied()
                            .unwrap_or(OpticalSignal::new(1.55e-6, 0.0, 0.0));
                        (pd, sig)
                    };

                    let i_ph = detector.generate_photocurrent(&opt_sig, v_rev);
                    let i_dark = detector.dark_current(temp_k);
                    let i_det = i_ph + i_dark;

                    // Photocurrent flows from cathode to anode under reverse bias
                    stamp_current_source(&mut rhs, *cathode, *anode, i_det);
                    stamp_conductance(&mut g_builder, *anode, *cathode, 1e-12);
                }
                ComponentRecord::Memristor {
                    name,
                    pos,
                    neg,
                    initial_conductance,
                    ..
                } => {
                    let v_pos = get_v(*pos, x);
                    let v_neg = get_v(*neg, x);
                    let v = v_pos - v_neg;

                    if let Some(rram) = context.get_rram_model(name) {
                        let w = context
                            .get_memristor_state(name)
                            .map(|s| s.internal_state_w)
                            .unwrap_or(0.5);
                        let (i_curr, g_eq) = rram.evaluate_current_and_conductance(w, v);
                        let g_stamp = g_eq.max(1e-12);
                        let i_eq = g_stamp * v - i_curr;
                        stamp_diode_companion(&mut g_builder, &mut rhs, *pos, *neg, g_stamp, i_eq);
                    } else if let Some(pcm) = context.get_pcm_model(name) {
                        let uc = context
                            .get_memristor_state(name)
                            .map(|s| s.internal_state_w)
                            .unwrap_or(0.5);
                        let (i_curr, g_eq) = pcm.evaluate_current_and_conductance(uc, v);
                        let g_stamp = g_eq.max(1e-12);
                        let i_eq = g_stamp * v - i_curr;
                        stamp_diode_companion(&mut g_builder, &mut rhs, *pos, *neg, g_stamp, i_eq);
                    } else {
                        let g: f64 = if let Some(st) = context.get_memristor_state(name) {
                            st.conductance_s
                        } else if *initial_conductance > 0.0 {
                            *initial_conductance
                        } else {
                            1e-3
                        };
                        stamp_conductance(&mut g_builder, *pos, *neg, g.max(1e-12));
                    }
                }
                ComponentRecord::SpikingNeuron {
                    name,
                    input_node,
                    output_node,
                    v_thresh,
                    v_reset: _,
                } => {
                    let v_in = get_v(*input_node, x);
                    let (r_m, v_th, amp) = if let Some(neuron) = context.get_neuron_model(name) {
                        (
                            neuron.r_mem_ohms,
                            neuron.v_thresh_volts,
                            neuron.spike_amplitude_volts,
                        )
                    } else {
                        (1e6, *v_thresh, 1.0)
                    };
                    let g_leak: f64 = (1.0 / r_m.max(1.0)).max(1e-12);
                    stamp_conductance(&mut g_builder, *input_node, NodeId::GROUND, g_leak);

                    let target_v = if v_in >= v_th { amp } else { 0.0 };
                    let g_out = 1.0;
                    stamp_conductance(&mut g_builder, *output_node, NodeId::GROUND, g_out);
                    stamp_current_source(&mut rhs, NodeId::GROUND, *output_node, g_out * target_v);
                }
                ComponentRecord::RadiationStrike {
                    name,
                    target_node,
                    strike_time_s,
                    let_mev,
                } => {
                    let i_strike = if let Some(strike) = context.get_heavy_ion_strike(name) {
                        strike.current_at_time(context.current_time_s)
                    } else {
                        let default_strike =
                            HeavyIonStrikeModel::typical_30nm_heavy_ion(*strike_time_s, *let_mev);
                        default_strike.current_at_time(context.current_time_s)
                    };

                    if i_strike.abs() > 1e-18 {
                        // Current injected into target_node from ground reference:
                        // Current source leaves GROUND and enters target_node
                        stamp_current_source(&mut rhs, NodeId::GROUND, *target_node, i_strike);
                    }
                    // Minute shunt conductance to ground to maintain matrix regularity
                    stamp_conductance(&mut g_builder, *target_node, NodeId::GROUND, 1e-12);
                }
            }
        }

        let g_mat = g_builder.build_csc();
        let lu = match SparseLuFactorization::factor(&g_mat, &markowitz_opts) {
            Ok(fact) => fact,
            Err(SolverError::SingularMatrix {
                step,
                row,
                col,
                pivot_value,
                ..
            }) => {
                let diag = diagnose_mna_singularity(graph, row);
                return Err(SolverError::SingularMatrix {
                    step,
                    row,
                    col,
                    pivot_value,
                    entity_diagnostic: diag,
                });
            }
            Err(e) => return Err(e),
        };

        last_cond_ratio = lu.pivot_condition_ratio();

        let mut x_next = vec![0.0; total_dim];
        lu.solve(&rhs, &mut x_next)?;

        // Apply PN junction voltage limiting across all diodes and BJTs
        for comp in graph.components() {
            match comp {
                ComponentRecord::Diode { name, pos, neg } => {
                    let model = context.get_diode_model(name);
                    let vt = model.n * phonon_core::thermal_voltage(temp_k);
                    let vcrit = compute_vcrit(model.is, vt);

                    let vd_old = get_v(*pos, x) - get_v(*neg, x);
                    let vd_new = get_v(*pos, &x_next) - get_v(*neg, &x_next);
                    let vd_lim = pn_junction_limit(vd_new, vd_old, vt, vcrit);

                    if (vd_lim - vd_new).abs() > 1e-6 {
                        if let Some(p) = node_to_mna_idx(*pos) {
                            x_next[p] = get_v(*neg, &x_next) + vd_lim;
                        }
                    }
                }
                ComponentRecord::Bjt {
                    name,
                    collector,
                    base,
                    emitter,
                } => {
                    let model = context.get_bjt_model(name);
                    let vt_f = model.nf * phonon_core::thermal_voltage(temp_k);
                    let vt_r = model.nr * phonon_core::thermal_voltage(temp_k);
                    let vcrit_f = compute_vcrit(model.is, vt_f);
                    let vcrit_r = compute_vcrit(model.is, vt_r);

                    // Base-Emitter junction limiting
                    let vbe_old = get_v(*base, x) - get_v(*emitter, x);
                    let vbe_new = get_v(*base, &x_next) - get_v(*emitter, &x_next);
                    let vbe_lim = pn_junction_limit(vbe_new, vbe_old, vt_f, vcrit_f);
                    if (vbe_lim - vbe_new).abs() > 1e-6 {
                        if let Some(b) = node_to_mna_idx(*base) {
                            x_next[b] = get_v(*emitter, &x_next) + vbe_lim;
                        }
                    }

                    // Base-Collector junction limiting
                    let vbc_old = get_v(*base, x) - get_v(*collector, x);
                    let vbc_new = get_v(*base, &x_next) - get_v(*collector, &x_next);
                    let vbc_lim = pn_junction_limit(vbc_new, vbc_old, vt_r, vcrit_r);
                    if (vbc_lim - vbc_new).abs() > 1e-6 {
                        if let Some(c) = node_to_mna_idx(*collector) {
                            x_next[c] = get_v(*base, &x_next) - vbc_lim;
                        }
                    }
                }
                ComponentRecord::TcadDiode { pos, neg, .. } => {
                    let vt = phonon_core::thermal_voltage(temp_k);
                    let vcrit = 0.6;
                    let vd_old = get_v(*pos, x) - get_v(*neg, x);
                    let vd_new = get_v(*pos, &x_next) - get_v(*neg, &x_next);
                    let vd_lim = pn_junction_limit(vd_new, vd_old, vt, vcrit);

                    if (vd_lim - vd_new).abs() > 1e-6 {
                        if let Some(p) = node_to_mna_idx(*pos) {
                            x_next[p] = get_v(*neg, &x_next) + vd_lim;
                        }
                    }
                }
                ComponentRecord::Mosfet { drain, source, .. }
                | ComponentRecord::TcadMosfet { drain, source, .. } => {
                    let v_d_old = get_v(*drain, x);
                    let v_s_old = get_v(*source, x);
                    let v_ds_old = v_d_old - v_s_old;

                    let v_d_new = get_v(*drain, &x_next);
                    let v_s_new = get_v(*source, &x_next);
                    let v_ds_new = v_d_new - v_s_new;

                    // Limit large Vds excursions and prevent crossing zero with overshoot
                    let mut v_ds_lim = v_ds_new;
                    if (v_ds_old > 0.0 && v_ds_new < 0.0) || (v_ds_old < 0.0 && v_ds_new > 0.0) {
                        v_ds_lim = 0.1 * v_ds_old;
                    } else if (v_ds_new - v_ds_old).abs() > 2.0 {
                        v_ds_lim = v_ds_old + 2.0 * (v_ds_new - v_ds_old).signum();
                    }

                    if let Some(d_idx) = node_to_mna_idx(*drain) {
                        x_next[d_idx] = v_s_new + v_ds_lim;
                    }
                }
                ComponentRecord::NeuralSurrogate { nodes, .. } => {
                    if nodes.len() == 2 {
                        let pos = nodes[0];
                        let neg = nodes[1];
                        let vt = phonon_core::thermal_voltage(temp_k);
                        let vcrit = 0.6;
                        let vd_old = get_v(pos, x) - get_v(neg, x);
                        let vd_new = get_v(pos, &x_next) - get_v(neg, &x_next);
                        let vd_lim = pn_junction_limit(vd_new, vd_old, vt, vcrit);

                        if (vd_lim - vd_new).abs() > 1e-6 {
                            if let Some(p) = node_to_mna_idx(pos) {
                                x_next[p] = get_v(neg, &x_next) + vd_lim;
                            }
                        }
                    } else if nodes.len() >= 3 {
                        let drain = nodes[0];
                        let source = nodes[2];
                        let v_d_old = get_v(drain, x);
                        let v_s_old = get_v(source, x);
                        let v_ds_old = v_d_old - v_s_old;

                        let v_d_new = get_v(drain, &x_next);
                        let v_s_new = get_v(source, &x_next);
                        let v_ds_new = v_d_new - v_s_new;

                        let mut v_ds_lim = v_ds_new;
                        if (v_ds_old > 0.0 && v_ds_new < 0.0) || (v_ds_old < 0.0 && v_ds_new > 0.0)
                        {
                            v_ds_lim = 0.1 * v_ds_old;
                        } else if (v_ds_new - v_ds_old).abs() > 2.0 {
                            v_ds_lim = v_ds_old + 2.0 * (v_ds_new - v_ds_old).signum();
                        }

                        if let Some(d_idx) = node_to_mna_idx(drain) {
                            x_next[d_idx] = v_s_new + v_ds_lim;
                        }
                    }
                }
                ComponentRecord::LaserDiode { anode, cathode, .. } => {
                    let vt = 1.5 * phonon_core::thermal_voltage(temp_k);
                    let vcrit = 0.8;
                    let vd_old = get_v(*anode, x) - get_v(*cathode, x);
                    let vd_new = get_v(*anode, &x_next) - get_v(*cathode, &x_next);
                    let vd_lim = pn_junction_limit(vd_new, vd_old, vt, vcrit);

                    if (vd_lim - vd_new).abs() > 1e-6 {
                        if let Some(p) = node_to_mna_idx(*anode) {
                            x_next[p] = get_v(*cathode, &x_next) + vd_lim;
                        }
                    }
                }
                _ => {}
            }
        }

        // Check convergence
        let mut converged = true;
        for i in 0..active_nodes {
            let dx = (x_next[i] - x[i]).abs();
            let tol = options.reltol * x_next[i].abs().max(x[i].abs()) + options.vntol;
            if dx > tol {
                converged = false;
                break;
            }
        }

        if converged {
            for i in active_nodes..total_dim {
                let dx = (x_next[i] - x[i]).abs();
                let tol = options.reltol * x_next[i].abs().max(x[i].abs()) + options.abstol;
                if dx > tol {
                    converged = false;
                    break;
                }
            }
        }

        x.copy_from_slice(&x_next);

        if converged {
            let mut node_voltages = vec![0.0; graph.total_nodes()];
            if active_nodes > 0 {
                node_voltages[1..=active_nodes].copy_from_slice(&x[..active_nodes]);
            }
            let mut branch_currents = vec![0.0; total_branches];
            if total_branches > 0 {
                branch_currents.copy_from_slice(&x[active_nodes..total_dim]);
            }

            return Ok(DcSolution {
                node_voltages,
                branch_currents,
                condition_ratio: last_cond_ratio,
            });
        }
    }

    Err(SolverError::NumericalAnomaly {
        detail: format!(
            "Newton-Raphson failed to converge after {} iterations",
            options.max_iters
        ),
    })
}
