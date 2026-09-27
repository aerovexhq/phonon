//! Kirchhoff's Voltage Law (KVL) verification probe and branch potential consistency checker.

use phonon_core::{CircuitGraph, ComponentRecord, NodeId};

/// Detailed results of a KVL branch potential consistency evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct KvlReport {
    /// True if all branch potentials satisfy KVL consistency tolerance.
    pub is_valid: bool,
    /// Maximum voltage deviation across all branch constraints (Volts).
    pub max_voltage_error: f64,
    /// Component name with the worst KVL discrepancy.
    pub worst_component: String,
}

/// Evaluates consistency of branch voltages with respect to nodal potentials.
pub fn verify_kvl(graph: &CircuitGraph, voltages: &[f64], tol: f64) -> KvlReport {
    let mut max_err = 0.0f64;
    let mut worst_comp = String::new();
    let mut is_valid = true;

    let get_v = |node: NodeId| -> f64 {
        let idx = node.index();
        if idx == 0 || idx >= voltages.len() {
            0.0
        } else {
            voltages[idx]
        }
    };

    for comp in graph.components() {
        match comp {
            ComponentRecord::VoltageSource {
                name,
                pos,
                neg,
                dc_value,
                ..
            } => {
                let v_actual = get_v(*pos) - get_v(*neg);
                let err = (v_actual - dc_value).abs();
                if err > max_err {
                    max_err = err;
                    worst_comp = name.clone();
                }
                if err > tol {
                    is_valid = false;
                }
            }
            ComponentRecord::Vcvs {
                name,
                out_pos,
                out_neg,
                ctrl_pos,
                ctrl_neg,
                gain,
                ..
            } => {
                let v_out = get_v(*out_pos) - get_v(*out_neg);
                let v_ctrl = get_v(*ctrl_pos) - get_v(*ctrl_neg);
                let expected = gain * v_ctrl;
                let err = (v_out - expected).abs();
                if err > max_err {
                    max_err = err;
                    worst_comp = name.clone();
                }
                if err > tol {
                    is_valid = false;
                }
            }
            _ => {}
        }
    }

    KvlReport {
        is_valid,
        max_voltage_error: max_err,
        worst_component: worst_comp,
    }
}
