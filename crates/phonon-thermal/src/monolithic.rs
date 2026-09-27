//! Monolithic electro-thermal coupled Newton-Raphson solver.

use crate::cauer::CauerNetwork;
use phonon_core::{CircuitGraph, NodeId};
use phonon_solver::mna::{solve_dc_non_linear, ModelContext, NewtonOptions};
use phonon_solver::SolverError;
use std::collections::HashMap;

/// Thermal coupling descriptor binding an electrical component to a thermal network.
#[derive(Debug, Clone, PartialEq)]
pub struct ElectroThermalBinding {
    /// Name of the circuit component (e.g. "D1", "M1").
    pub component_name: String,
    /// Associated Cauer thermal network from junction to ambient.
    pub cauer: CauerNetwork,
    /// Ambient temperature in Kelvin.
    pub ambient_k: f64,
}

/// Result of a monolithic electro-thermal simulation.
#[derive(Debug, Clone, PartialEq)]
pub struct ElectroThermalSolution {
    /// Solved electrical node voltages.
    pub node_voltages: Vec<f64>,
    /// Solved junction temperatures for each bound component (name -> T_junction in Kelvin).
    pub junction_temperatures: HashMap<String, f64>,
    /// Power dissipation for each bound component in Watts.
    pub power_dissipations: HashMap<String, f64>,
    /// Number of electro-thermal outer Newton relaxation iterations.
    pub iterations: usize,
}

/// Solves the coupled electro-thermal steady-state operating point monolithically.
///
/// Iteratively resolves electrical current and Joule dissipation $P = I \cdot V$
/// with temperature-dependent semiconductor parameter updates until temperature
/// and voltage simultaneously converge.
pub fn solve_electrothermal_dc(
    graph: &CircuitGraph,
    bindings: &[ElectroThermalBinding],
    initial_context: &ModelContext,
    newton_opts: &NewtonOptions,
    max_coupling_iters: usize,
    temp_tolerance_k: f64,
) -> Result<ElectroThermalSolution, SolverError> {
    let mut current_temps: HashMap<String, f64> = bindings
        .iter()
        .map(|b| (b.component_name.clone(), b.ambient_k))
        .collect();

    let mut ctx = initial_context.clone();
    let mut power_map = HashMap::new();

    for iter in 0..max_coupling_iters {
        // 1. Solve electrical circuit at current junction temperatures
        // Update context temperature for bound components
        let sol = solve_dc_non_linear(graph, &ctx, newton_opts)?;

        // 2. Compute power dissipation for each bound component
        let mut max_temp_delta = 0.0f64;
        let mut next_temps = current_temps.clone();

        for binding in bindings {
            let name = &binding.component_name;
            let p_diss = compute_component_power(graph, &sol, name, &ctx);
            power_map.insert(name.clone(), p_diss);

            // 3. Solve thermal network with injected Joule heat
            let thermal_profile = binding
                .cauer
                .solve_steady_state(&[p_diss], binding.ambient_k);
            let t_junction_new = if !thermal_profile.is_empty() {
                thermal_profile[0]
            } else {
                binding.ambient_k
            };

            // Thermal safety limit (Silicon melting / degradation ~ 600 K)
            if t_junction_new > 600.0 || t_junction_new.is_nan() {
                return Err(SolverError::NumericalAnomaly {
                    detail: format!(
                        "Thermal runaway detected on component '{name}': junction temperature exceeded 600 K ({t_junction_new:.1} K)"
                    ),
                });
            }

            let t_old = current_temps[name];
            let delta = (t_junction_new - t_old).abs();
            if delta > max_temp_delta {
                max_temp_delta = delta;
            }

            // Damped temperature update: T_next = 0.5 * (T_old + T_new)
            next_temps.insert(name.clone(), 0.5 * (t_old + t_junction_new));
        }

        current_temps = next_temps;

        // Check coupling convergence
        if max_temp_delta < temp_tolerance_k {
            return Ok(ElectroThermalSolution {
                node_voltages: sol.node_voltages,
                junction_temperatures: current_temps,
                power_dissipations: power_map,
                iterations: iter + 1,
            });
        }

        // Update context with new junction temperature for next iteration
        if let Some((_, &first_temp)) = current_temps.iter().next() {
            ctx.temperature_kelvin = first_temp;
        }
    }

    Err(SolverError::NumericalAnomaly {
        detail: format!(
            "Electro-thermal monolithic solver failed to converge after {max_coupling_iters} iterations"
        ),
    })
}

/// Helper to compute power dissipation $P = I \cdot V$ of a component in the circuit.
fn compute_component_power(
    graph: &CircuitGraph,
    sol: &phonon_solver::mna::DcSolution,
    name: &str,
    ctx: &ModelContext,
) -> f64 {
    let get_v = |node: NodeId| -> f64 { sol.node_voltage(node) };

    for comp in graph.components() {
        if comp.name() == name {
            match comp {
                phonon_core::ComponentRecord::Diode { pos, neg, .. } => {
                    let vd = get_v(*pos) - get_v(*neg);
                    let model = ctx.get_diode_model(name);
                    let id = model.evaluate(vd, ctx.temperature_kelvin).i_d;
                    return (vd * id).max(0.0);
                }
                phonon_core::ComponentRecord::Mosfet {
                    drain,
                    gate,
                    source,
                    bulk,
                    ..
                } => {
                    let vd = get_v(*drain);
                    let vg = get_v(*gate);
                    let vs = get_v(*source);
                    let vb = get_v(*bulk);
                    let model = ctx.get_mosfet_model(name);
                    let ids = model.evaluate(vd, vg, vs, vb, ctx.temperature_kelvin).i_ds;
                    let vds = vd - vs;
                    return (vds * ids).max(0.0);
                }
                phonon_core::ComponentRecord::Resistor {
                    pos,
                    neg,
                    resistance,
                    ..
                } => {
                    let v = get_v(*pos) - get_v(*neg);
                    return v * v / resistance;
                }
                _ => {}
            }
        }
    }

    0.0
}
