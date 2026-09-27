//! Automated MNA matrix and RHS vector assembly from circuit graphs.

use super::stamp::*;
use crate::error::SolverError;
use crate::sparse::builder::SparseMatrixBuilder;
use crate::sparse::csc::SparseMatrixCsc;
use phonon_core::{CircuitGraph, ComponentRecord};

/// Solver operational parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolverOptions {
    /// Injects a tiny conductance to ground ($G_{shunt} = 10^{-12}\text{ S}$) at every active node
    /// to guarantee DC solvability for high-impedance nodes.
    pub auto_gmin_shunt: bool,
    /// Shunt conductance value in Siemens.
    pub gmin_value: f64,
}

impl Default for SolverOptions {
    fn default() -> Self {
        Self {
            auto_gmin_shunt: false,
            gmin_value: 1e-12,
        }
    }
}

/// Assembled Modified Nodal Analysis system matrices:
/// $\mathbf{G} \mathbf{x} + \mathbf{C} \mathbf{\dot{x}} = \mathbf{b}$.
#[derive(Debug, Clone)]
pub struct MnaSystem {
    pub g_matrix: SparseMatrixCsc,
    pub c_matrix: SparseMatrixCsc,
    pub rhs: Vec<f64>,
    pub active_nodes: usize,
    pub total_branches: usize,
    pub total_dim: usize,
}

/// Assembles the linear DC MNA matrices from a given circuit graph.
pub fn assemble_mna_dc(
    graph: &CircuitGraph,
    options: &SolverOptions,
) -> Result<MnaSystem, SolverError> {
    let active_nodes = graph.active_nodes();
    let total_branches = graph.total_branches();
    let total_dim = active_nodes + total_branches;

    if total_dim == 0 {
        return Ok(MnaSystem {
            g_matrix: SparseMatrixCsc::zeros(0, 0),
            c_matrix: SparseMatrixCsc::zeros(0, 0),
            rhs: Vec::new(),
            active_nodes: 0,
            total_branches: 0,
            total_dim: 0,
        });
    }

    let mut g_builder = SparseMatrixBuilder::with_capacity(total_dim, total_dim, total_dim * 4);
    let mut c_builder = SparseMatrixBuilder::with_capacity(total_dim, total_dim, total_dim * 2);
    let mut rhs = vec![0.0; total_dim];

    // Optional Gmin shunt conductance to ground for DC robustness
    if options.auto_gmin_shunt {
        for node_idx in 0..active_nodes {
            g_builder.add(node_idx, node_idx, options.gmin_value);
        }
    }

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
            ComponentRecord::Capacitor {
                pos,
                neg,
                capacitance,
                ..
            } => {
                // In DC steady state, capacitors are open circuits.
                // Their dynamic stamp is preserved in C_matrix for transient analysis.
                stamp_capacitance(&mut c_builder, *pos, *neg, *capacitance);
            }
            ComponentRecord::Inductor {
                pos,
                neg,
                branch,
                inductance,
                ..
            } => {
                // In DC steady state, ideal inductors act as zero-volt sources (short circuits).
                let branch_mna_idx = active_nodes + branch.index();
                stamp_inductor(
                    &mut g_builder,
                    &mut c_builder,
                    *pos,
                    *neg,
                    branch_mna_idx,
                    *inductance,
                );
            }
            ComponentRecord::VoltageSource {
                pos,
                neg,
                dc_value,
                branch,
                ..
            } => {
                let branch_mna_idx = active_nodes + branch.index();
                stamp_voltage_source(
                    &mut g_builder,
                    &mut rhs,
                    *pos,
                    *neg,
                    branch_mna_idx,
                    *dc_value,
                );
            }
            ComponentRecord::CurrentSource {
                pos, neg, dc_value, ..
            } => {
                stamp_current_source(&mut rhs, *pos, *neg, *dc_value);
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
                let branch_mna_idx = active_nodes + branch.index();
                stamp_vcvs(
                    &mut g_builder,
                    *out_pos,
                    *out_neg,
                    *ctrl_pos,
                    *ctrl_neg,
                    branch_mna_idx,
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
            ComponentRecord::TransmissionLine {
                in_pos,
                in_neg,
                out_pos,
                out_neg,
                ..
            } => {
                // In DC steady state, a lossless transmission line connects in to out
                let g_dc = 1e6;
                stamp_conductance(&mut g_builder, *in_pos, *out_pos, g_dc);
                stamp_conductance(&mut g_builder, *in_neg, *out_neg, g_dc);
            }
            ComponentRecord::OpticalWaveguide { .. }
            | ComponentRecord::MicroRingResonator { .. }
            | ComponentRecord::RadiationStrike { .. } => {
                // Passive optical devices and transient radiation strikes do not directly stamp DC MNA equations
            }
            ComponentRecord::ElectroOpticModulator {
                elec_pos, elec_neg, ..
            } => {
                // Modulator electrodes have minute leakage conductance in DC
                stamp_conductance(&mut g_builder, *elec_pos, *elec_neg, 1e-12);
            }
            ComponentRecord::Diode { name, .. }
            | ComponentRecord::Mosfet { name, .. }
            | ComponentRecord::Bjt { name, .. }
            | ComponentRecord::TcadDiode { name, .. }
            | ComponentRecord::TcadMosfet { name, .. }
            | ComponentRecord::NeuralSurrogate { name, .. }
            | ComponentRecord::JosephsonJunction { name, .. }
            | ComponentRecord::AtomisticChannel { name, .. }
            | ComponentRecord::LaserDiode { name, .. }
            | ComponentRecord::Photodetector { name, .. }
            | ComponentRecord::Memristor { name, .. }
            | ComponentRecord::SpikingNeuron { name, .. } => {
                return Err(SolverError::NumericalAnomaly {
                    detail: format!(
                        "Linear MNA assembler encountered non-linear component '{name}'. Use solve_non_linear_dc instead."
                    ),
                });
            }
        }
    }

    let g_matrix = g_builder.build_csc();
    let c_matrix = c_builder.build_csc();

    Ok(MnaSystem {
        g_matrix,
        c_matrix,
        rhs,
        active_nodes,
        total_branches,
        total_dim,
    })
}
