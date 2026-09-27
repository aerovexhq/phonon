//! Physical conservation law probes (KCL, KVL, thermodynamic energy balance) and verification reporting.

pub mod energy_balance;
pub mod kcl_probe;
pub mod kvl_probe;

pub use energy_balance::{verify_energy_balance, EnergyBalanceReport};
pub use kcl_probe::{
    verify_kcl, verify_kcl_dynamic, verify_transient_kcl, verify_transient_kcl_with_context,
    KclReport,
};
pub use kvl_probe::{verify_kvl, KvlReport};

use phonon_core::CircuitGraph;

/// Consolidated physical verification summary across all conservation probes.
#[derive(Debug, Clone, PartialEq)]
pub struct FullVerificationReport {
    pub kcl: KclReport,
    pub kvl: KvlReport,
    pub is_all_valid: bool,
}

/// Evaluates static conservation probes (KCL and KVL) for a solved state vector.
pub fn verify_physical_conservation(
    graph: &CircuitGraph,
    voltages: &[f64],
    branch_currents: &[f64],
    reltol: f64,
    abstol: f64,
) -> FullVerificationReport {
    let kcl = verify_kcl(graph, voltages, branch_currents, reltol, abstol);
    let kvl = verify_kvl(graph, voltages, abstol.max(1e-6));
    let is_all_valid = kcl.is_valid && kvl.is_valid;

    FullVerificationReport {
        kcl,
        kvl,
        is_all_valid,
    }
}
