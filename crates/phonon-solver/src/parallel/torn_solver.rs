use super::diakoptics::DiakopticsSolver;
use super::node_tearing::PartitionedCircuit;
use crate::error::SolverError;
use crate::mna::assembler::SolverOptions;
use crate::mna::linear_solver::DcSolution;

/// Solves a partitioned circuit operating point using multi-threaded Diakoptics.
pub fn solve_torn_dc(
    circuit: &PartitionedCircuit,
    options: &SolverOptions,
) -> Result<DcSolution, SolverError> {
    DiakopticsSolver::solve_linear(circuit, options)
}
