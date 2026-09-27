//! Structure-of-Arrays (SoA) layout for high-throughput cache-conscious component evaluation.
//!
//! Organizes homogeneous electrical devices into contiguous columnar memory,
//! maximizing L1 data cache prefetching and enabling SIMD batch evaluation.

use crate::mna::stamp::{stamp_conductance, MosfetCompanion};
use crate::mna::ModelContext;
use crate::sparse::SparseMatrixBuilder;
use phonon_core::{CircuitGraph, ComponentRecord, NodeId};
use rayon::prelude::*;

/// Structure-of-Arrays collection for linear resistors.
#[derive(Debug, Clone, Default)]
pub struct ResistorSoA {
    pub pos: Vec<NodeId>,
    pub neg: Vec<NodeId>,
    pub conductance: Vec<f64>,
}

impl ResistorSoA {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, pos: NodeId, neg: NodeId, resistance: f64) {
        self.pos.push(pos);
        self.neg.push(neg);
        self.conductance.push(1.0 / resistance);
    }

    pub fn len(&self) -> usize {
        self.pos.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pos.is_empty()
    }

    /// Stamps all resistors into the sparse matrix builder using a contiguous cache-local loop.
    #[inline]
    pub fn stamp(&self, builder: &mut SparseMatrixBuilder) {
        for i in 0..self.pos.len() {
            stamp_conductance(builder, self.pos[i], self.neg[i], self.conductance[i]);
        }
    }
}

/// Structure-of-Arrays collection for semiconductor diodes.
#[derive(Debug, Clone, Default)]
pub struct DiodeSoA {
    pub name: Vec<String>,
    pub pos: Vec<NodeId>,
    pub neg: Vec<NodeId>,
}

impl DiodeSoA {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, name: &str, pos: NodeId, neg: NodeId) {
        self.name.push(name.to_string());
        self.pos.push(pos);
        self.neg.push(neg);
    }

    pub fn len(&self) -> usize {
        self.name.len()
    }

    pub fn is_empty(&self) -> bool {
        self.name.is_empty()
    }

    /// Evaluates all diodes in parallel across Rayon worker threads.
    pub fn evaluate_parallel(
        &self,
        x: &[f64],
        context: &ModelContext,
        temp_k: f64,
    ) -> Vec<(NodeId, NodeId, f64, f64)> {
        let get_v = |node: NodeId| -> f64 {
            if node.is_ground() {
                0.0
            } else {
                x[node.index() - 1]
            }
        };

        (0..self.name.len())
            .into_par_iter()
            .map(|i| {
                let name = &self.name[i];
                let pos = self.pos[i];
                let neg = self.neg[i];
                let vd = get_v(pos) - get_v(neg);
                let model = context.get_diode_model(name);
                let eval = model.evaluate(vd, temp_k);
                let i_eq = eval.g_d * vd - eval.i_d;
                (pos, neg, eval.g_d, i_eq)
            })
            .collect()
    }
}

/// Structure-of-Arrays collection for MOSFET transistors.
#[derive(Debug, Clone, Default)]
pub struct MosfetSoA {
    pub name: Vec<String>,
    pub drain: Vec<NodeId>,
    pub gate: Vec<NodeId>,
    pub source: Vec<NodeId>,
    pub bulk: Vec<NodeId>,
}

impl MosfetSoA {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, name: &str, drain: NodeId, gate: NodeId, source: NodeId, bulk: NodeId) {
        self.name.push(name.to_string());
        self.drain.push(drain);
        self.gate.push(gate);
        self.source.push(source);
        self.bulk.push(bulk);
    }

    pub fn len(&self) -> usize {
        self.name.len()
    }

    pub fn is_empty(&self) -> bool {
        self.name.is_empty()
    }

    /// Evaluates all MOSFET operating points and Jacobians in parallel with Rayon.
    pub fn evaluate_parallel(
        &self,
        x: &[f64],
        context: &ModelContext,
        temp_k: f64,
    ) -> Vec<(NodeId, NodeId, NodeId, NodeId, MosfetCompanion)> {
        let get_v = |node: NodeId| -> f64 {
            if node.is_ground() {
                0.0
            } else {
                x[node.index() - 1]
            }
        };

        (0..self.name.len())
            .into_par_iter()
            .map(|i| {
                let name = &self.name[i];
                let d = self.drain[i];
                let g = self.gate[i];
                let s = self.source[i];
                let b = self.bulk[i];

                let v_d = get_v(d);
                let v_g = get_v(g);
                let v_s = get_v(s);
                let v_b = get_v(b);

                let model = context.get_mosfet_model(name);

                let is_reverse = match model.mos_type {
                    phonon_models::MosfetType::Nmos => v_d < v_s,
                    phonon_models::MosfetType::Pmos => v_d > v_s,
                };

                let (eff_d, eff_s, v_deff, v_seff) = if is_reverse {
                    (s, d, v_s, v_d)
                } else {
                    (d, s, v_d, v_s)
                };

                let eval = model.evaluate(v_deff, v_g, v_seff, v_b, temp_k);
                let v_ds_eff = v_deff - v_seff;
                let v_gs_eff = v_g - v_seff;
                let v_bs_eff = v_b - v_seff;

                let i_eq =
                    eval.g_m * v_gs_eff + eval.g_ds * v_ds_eff + eval.g_mbs * v_bs_eff - eval.i_ds;
                let comp = MosfetCompanion {
                    g_m: eval.g_m,
                    g_ds: eval.g_ds,
                    g_mbs: eval.g_mbs,
                    i_eq,
                };

                (eff_d, g, eff_s, b, comp)
            })
            .collect()
    }
}

/// Unified Structure-of-Arrays extracted representation of an entire circuit.
#[derive(Debug, Clone, Default)]
pub struct CircuitSoA {
    pub resistors: ResistorSoA,
    pub diodes: DiodeSoA,
    pub mosfets: MosfetSoA,
}

impl CircuitSoA {
    /// Extracts a Data-Oriented SoA circuit representation from a graph.
    pub fn from_circuit_graph(graph: &CircuitGraph) -> Self {
        let mut soa = Self::default();

        for comp in graph.components() {
            match comp {
                ComponentRecord::Resistor {
                    pos,
                    neg,
                    resistance,
                    ..
                } => {
                    soa.resistors.push(*pos, *neg, *resistance);
                }
                ComponentRecord::Diode { name, pos, neg } => {
                    soa.diodes.push(name, *pos, *neg);
                }
                ComponentRecord::Mosfet {
                    name,
                    drain,
                    gate,
                    source,
                    bulk,
                } => {
                    soa.mosfets.push(name, *drain, *gate, *source, *bulk);
                }
                _ => {}
            }
        }

        soa
    }
}
