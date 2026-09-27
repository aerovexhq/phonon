//! High-level linear DC operating point solver producing strongly-typed circuit solutions.

use super::assembler::{assemble_mna_dc, SolverOptions};
use super::diagnostics::diagnose_mna_singularity;
use crate::error::SolverError;
use crate::sparse::lu::SparseLuFactorization;
use crate::sparse::markowitz::MarkowitzOptions;
use phonon_core::{BranchId, CircuitGraph, NodeId};

/// Computed DC operating point solution vector containing all node voltages and branch currents.
#[derive(Debug, Clone, PartialEq)]
pub struct DcSolution {
    /// Node voltages indexed by NodeId.
    /// Index 0 is permanently 0.0 V (Ground).
    pub node_voltages: Vec<f64>,
    /// Auxiliary branch currents indexed by BranchId.
    pub branch_currents: Vec<f64>,
    /// Condition ratio of the factorized MNA conductance matrix.
    pub condition_ratio: f64,
}

impl DcSolution {
    /// Returns the solved voltage at the specified node.
    /// Returns 0.0 V for Ground (`NodeId::GROUND`).
    #[inline(always)]
    pub fn node_voltage(&self, node: NodeId) -> f64 {
        let idx = node.index();
        if idx < self.node_voltages.len() {
            self.node_voltages[idx]
        } else {
            0.0
        }
    }

    /// Returns the solved voltage at the node identified by name.
    pub fn node_voltage_by_name(&self, graph: &CircuitGraph, name: &str) -> Option<f64> {
        graph.get_node(name).map(|id| self.node_voltage(id))
    }

    /// Returns the solved current through the specified auxiliary branch.
    #[inline(always)]
    pub fn branch_current(&self, branch: BranchId) -> f64 {
        let idx = branch.index();
        if idx < self.branch_currents.len() {
            self.branch_currents[idx]
        } else {
            0.0
        }
    }
}

/// Solves the linear DC operating point $\mathbf{G}_{MNA} \mathbf{x} = \mathbf{b}_{MNA}$ for a given circuit graph.
pub fn solve_dc_linear(
    graph: &CircuitGraph,
    options: &SolverOptions,
) -> Result<DcSolution, SolverError> {
    // Step 1: Validate static graph topology
    graph.validate_topology()?;

    // Step 2: Assemble MNA system
    let mna = assemble_mna_dc(graph, options)?;

    if mna.total_dim == 0 {
        return Ok(DcSolution {
            node_voltages: vec![0.0; graph.total_nodes()],
            branch_currents: Vec::new(),
            condition_ratio: 1.0,
        });
    }

    // Step 3: Factorize G matrix with Markowitz threshold pivoting
    let markowitz_opts = MarkowitzOptions::default();
    let lu = match SparseLuFactorization::factor(&mna.g_matrix, &markowitz_opts) {
        Ok(factorization) => factorization,
        Err(SolverError::SingularMatrix {
            step,
            row,
            col,
            pivot_value,
            ..
        }) => {
            let diagnostic = diagnose_mna_singularity(graph, row);
            return Err(SolverError::SingularMatrix {
                step,
                row,
                col,
                pivot_value,
                entity_diagnostic: diagnostic,
            });
        }
        Err(e) => return Err(e),
    };

    // Step 4: Solve for state vector x
    let mut x = vec![0.0; mna.total_dim];
    lu.solve(&mna.rhs, &mut x)?;

    // Step 5: Validate IEEE 754 hygiene
    for (i, &val) in x.iter().enumerate() {
        if val.is_nan() {
            let diag = diagnose_mna_singularity(graph, i);
            return Err(SolverError::NumericalAnomaly {
                detail: format!("NaN produced at MNA index {}. {}", i, diag),
            });
        }
        if val.is_infinite() {
            let diag = diagnose_mna_singularity(graph, i);
            return Err(SolverError::NumericalAnomaly {
                detail: format!("Infinite value produced at MNA index {}. {}", i, diag),
            });
        }
    }

    // Step 6: Map back to node voltages and branch currents
    let mut node_voltages = vec![0.0; graph.total_nodes()];
    if mna.active_nodes > 0 {
        node_voltages[1..=mna.active_nodes].copy_from_slice(&x[..mna.active_nodes]);
    }

    let mut branch_currents = vec![0.0; mna.total_branches];
    if mna.total_branches > 0 {
        branch_currents
            .copy_from_slice(&x[mna.active_nodes..(mna.active_nodes + mna.total_branches)]);
    }

    Ok(DcSolution {
        node_voltages,
        branch_currents,
        condition_ratio: lu.pivot_condition_ratio(),
    })
}
