//! Multirate waveform relaxation solver bridging fast electrical dynamics with slow thermal diffusion.

use crate::cauer::CauerNetwork;
use phonon_core::{CircuitGraph, NodeId};
use phonon_solver::mna::{solve_dc_non_linear, ModelContext, NewtonOptions};
use phonon_solver::SolverError;

/// Recorded trajectory point in a multirate electro-thermal simulation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ElectroThermalTracePoint {
    pub time_s: f64,
    pub junction_temp_k: f64,
    pub power_watts: f64,
}

/// Multirate waveform relaxation simulator for transient electro-thermal self-heating.
pub struct MultirateElectroThermalSimulator<'a> {
    pub graph: &'a CircuitGraph,
    pub component_name: String,
    pub cauer: CauerNetwork,
    pub ambient_k: f64,
}

impl<'a> MultirateElectroThermalSimulator<'a> {
    pub fn new(
        graph: &'a CircuitGraph,
        component_name: &str,
        cauer: CauerNetwork,
        ambient_k: f64,
    ) -> Self {
        Self {
            graph,
            component_name: component_name.to_string(),
            cauer,
            ambient_k,
        }
    }

    /// Simulates dynamic transient self-heating from $t=0$ to $t_{stop}$ with coarse thermal step $\Delta t_{thermal}$.
    ///
    /// At each thermal step:
    /// 1. Solves the electrical operating point at current junction temperature $T_j$.
    /// 2. Calculates Joule power $P_{diss} = V \cdot I$.
    /// 3. Advances the thermal Cauer ladder by $\Delta t$ using backward Euler integration.
    /// 4. Updates junction temperature $T_j = T_{node 0}$.
    pub fn simulate(
        &self,
        t_stop: f64,
        dt_thermal: f64,
        initial_context: &ModelContext,
        newton_opts: &NewtonOptions,
    ) -> Result<Vec<ElectroThermalTracePoint>, SolverError> {
        let mut trace = Vec::new();
        let mut t = 0.0;
        let mut t_nodes = vec![self.ambient_k; self.cauer.num_nodes()];

        let mut ctx = initial_context.clone();

        while t <= t_stop + 1e-12 {
            let t_j = if !t_nodes.is_empty() {
                t_nodes[0]
            } else {
                self.ambient_k
            };
            ctx.temperature_kelvin = t_j;

            // 1. Solve electrical state at current junction temperature
            let sol = solve_dc_non_linear(self.graph, &ctx, newton_opts)?;

            // 2. Compute power dissipation
            let p_diss = self.evaluate_power(&sol, &ctx);

            trace.push(ElectroThermalTracePoint {
                time_s: t,
                junction_temp_k: t_j,
                power_watts: p_diss,
            });

            // 3. Advance thermal state by dt_thermal
            t_nodes = self.cauer.step_transient_backward_euler(
                &t_nodes,
                &[p_diss],
                self.ambient_k,
                dt_thermal,
            );

            t += dt_thermal;
        }

        Ok(trace)
    }

    fn evaluate_power(&self, sol: &phonon_solver::mna::DcSolution, ctx: &ModelContext) -> f64 {
        let get_v = |node: NodeId| -> f64 { sol.node_voltage(node) };

        for comp in self.graph.components() {
            if comp.name() == self.component_name {
                match comp {
                    phonon_core::ComponentRecord::Diode { pos, neg, .. } => {
                        let vd = get_v(*pos) - get_v(*neg);
                        let model = ctx.get_diode_model(&self.component_name);
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
                        let model = ctx.get_mosfet_model(&self.component_name);
                        let ids = model.evaluate(vd, vg, vs, vb, ctx.temperature_kelvin).i_ds;
                        return ((vd - vs) * ids).max(0.0);
                    }
                    _ => {}
                }
            }
        }
        0.0
    }
}
