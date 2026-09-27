//! Thermodynamic First Law energy conservation tracker and verification probe.

use crate::transient::TransientSolution;
use phonon_core::{CircuitGraph, ComponentRecord, NodeId};

/// Detailed results of dynamic energy conservation tracking across a transient simulation.
#[derive(Debug, Clone, PartialEq)]
pub struct EnergyBalanceReport {
    /// True if energy balance error remained within tolerance over the entire simulation.
    pub is_valid: bool,
    /// Maximum relative energy conservation error ($0.001 = 0.1\%$).
    pub max_relative_error: f64,
    /// Total cumulative energy delivered by excitation sources (Joules).
    pub total_energy_supplied: f64,
    /// Total cumulative energy dissipated as Joule heat (Joules).
    pub total_energy_dissipated: f64,
    /// Net change in electromagnetic stored energy $\Delta E_{EM} = E(t_{end}) - E(0)$ (Joules).
    pub stored_energy_change: f64,
    /// Time trajectory of relative energy error: `Vec<(time, error_ratio)>`.
    pub trajectory: Vec<(f64, f64)>,
}

/// Evaluates the First Law of Thermodynamics across a transient solution:
/// $$\int_0^t P_{\text{supply}}(\tau) d\tau = \int_0^t P_{\text{joule}}(\tau) d\tau + \Delta E_{\text{EM}}(t)$$
pub fn verify_energy_balance(
    graph: &CircuitGraph,
    solution: &TransientSolution,
    tolerance: f64,
) -> EnergyBalanceReport {
    if solution.len() < 2 {
        return EnergyBalanceReport {
            is_valid: true,
            max_relative_error: 0.0,
            total_energy_supplied: 0.0,
            total_energy_dissipated: 0.0,
            stored_energy_change: 0.0,
            trajectory: Vec::new(),
        };
    }

    let get_v = |node: NodeId, voltages: &[f64]| -> f64 {
        let idx = node.index();
        if idx == 0 || idx >= voltages.len() {
            0.0
        } else {
            voltages[idx]
        }
    };

    // Calculate instantaneous power and stored energy at a given step
    let calc_powers = |voltages: &[f64], branch_currents: &[f64]| -> (f64, f64, f64) {
        let mut p_supply = 0.0;
        let mut p_joule = 0.0;
        let mut e_stored = 0.0;

        for comp in graph.components() {
            match comp {
                ComponentRecord::Resistor {
                    pos,
                    neg,
                    resistance,
                    ..
                } => {
                    let v = get_v(*pos, voltages) - get_v(*neg, voltages);
                    p_joule += (v * v) / resistance.max(1e-18);
                }
                ComponentRecord::VoltageSource {
                    pos, neg, branch, ..
                } => {
                    let v = get_v(*pos, voltages) - get_v(*neg, voltages);
                    let i_br = branch_currents[branch.index()];
                    // Power supplied to the circuit = - v * i_branch (where i_branch leaves pos)
                    p_supply += -v * i_br;
                }
                ComponentRecord::CurrentSource {
                    pos, neg, dc_value, ..
                } => {
                    let v = get_v(*pos, voltages) - get_v(*neg, voltages);
                    // Power supplied = (v_pos - v_neg) * I_dc
                    p_supply += v * dc_value;
                }
                ComponentRecord::Capacitor {
                    pos,
                    neg,
                    capacitance,
                    ..
                } => {
                    let v = get_v(*pos, voltages) - get_v(*neg, voltages);
                    e_stored += 0.5 * capacitance * v * v;
                }
                ComponentRecord::Inductor {
                    branch, inductance, ..
                } => {
                    let i = branch_currents[branch.index()];
                    e_stored += 0.5 * inductance * i * i;
                }
                _ => {}
            }
        }

        (p_supply, p_joule, e_stored)
    };

    let (_, _, e_stored_0) = calc_powers(
        &solution.steps[0].voltages,
        &solution.steps[0].branch_currents,
    );

    let mut cumulative_supply = 0.0;
    let mut cumulative_joule = 0.0;
    let mut max_rel_err = 0.0f64;
    let mut is_valid = true;
    let mut trajectory = Vec::new();

    let mut prev_time = solution.steps[0].time;
    let (mut prev_p_sup, mut prev_p_joule, _) = calc_powers(
        &solution.steps[0].voltages,
        &solution.steps[0].branch_currents,
    );

    for step in &solution.steps[1..] {
        let dt = step.time - prev_time;
        let (p_sup, p_joule, e_stored) = calc_powers(&step.voltages, &step.branch_currents);

        // Trapezoidal integration of power
        cumulative_supply += 0.5 * dt * (prev_p_sup + p_sup);
        cumulative_joule += 0.5 * dt * (prev_p_joule + p_joule);

        let delta_stored = e_stored - e_stored_0;
        let discrepancy = (cumulative_supply - (cumulative_joule + delta_stored)).abs();

        let norm_base = cumulative_supply
            .abs()
            .max(cumulative_joule.abs())
            .max(delta_stored.abs())
            .max(1e-9);

        let rel_err = discrepancy / norm_base;
        if rel_err > max_rel_err {
            max_rel_err = rel_err;
        }
        if rel_err > tolerance {
            is_valid = false;
        }

        trajectory.push((step.time, rel_err));

        prev_time = step.time;
        prev_p_sup = p_sup;
        prev_p_joule = p_joule;
    }

    let (_, _, final_e_stored) = calc_powers(
        &solution.steps.last().unwrap().voltages,
        &solution.steps.last().unwrap().branch_currents,
    );

    EnergyBalanceReport {
        is_valid,
        max_relative_error: max_rel_err,
        total_energy_supplied: cumulative_supply,
        total_energy_dissipated: cumulative_joule,
        stored_energy_change: final_e_stored - e_stored_0,
        trajectory,
    }
}
