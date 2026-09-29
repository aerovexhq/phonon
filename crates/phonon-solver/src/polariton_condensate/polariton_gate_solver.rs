//! Autonomous solver for polaritonic optical transistor switches
//! and bistable logic gates.

use phonon_models::polariton_condensate::{
    PolaritonCondensateParams, PolaritonGateMetrics, PolaritonGateParams,
};

/// Solver for polariton logic gates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolaritonGateSolver {
    pub condensate_params: PolaritonCondensateParams,
    pub gate_params: PolaritonGateParams,
}

impl PolaritonGateSolver {
    /// Creates a new polariton gate solver.
    pub fn new(
        condensate_params: PolaritonCondensateParams,
        gate_params: PolaritonGateParams,
    ) -> Self {
        Self {
            condensate_params,
            gate_params,
        }
    }

    /// Solves logic switching metrics.
    pub fn solve_gate_metrics(&self) -> PolaritonGateMetrics {
        self.gate_params.evaluate_gate(&self.condensate_params)
    }
}
