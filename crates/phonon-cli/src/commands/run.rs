//! Simulation execution command (`phonon run`).

use crate::error::CliError;
use crate::telemetry::{
    CsvTelemetryWriter, JsonLinesTelemetryWriter, OutputFormat, TelemetryWriter,
};
use phonon_core::{BranchId, ComponentRecord, NodeId};
use phonon_netlist::{elaborate_netlist, parse_netlist, ElaboratedCircuit, SimulationPlan};
use phonon_solver::mna::{solve_dc_linear, solve_dc_non_linear, NewtonOptions, SolverOptions};
use phonon_solver::{solve_transient, IntegrationMethod, StepControlOptions, TransientOptions};
use std::fs::File;
use std::io::{stdout, BufWriter, Write};
use std::path::Path;

/// Executes the simulation defined by the netlist file and streams results to telemetry.
pub fn execute_run(
    netlist_path: &Path,
    format: OutputFormat,
    output_path: Option<&Path>,
) -> Result<(), CliError> {
    let content = std::fs::read_to_string(netlist_path)?;
    let parsed = parse_netlist(&content)?;
    let elaborated = elaborate_netlist(&parsed)?;

    if let Some(out_p) = output_path {
        let file = File::create(out_p)?;
        let writer = BufWriter::new(file);
        run_with_writer(elaborated, format, writer)?;
    } else {
        let writer = BufWriter::new(stdout());
        run_with_writer(elaborated, format, writer)?;
    }

    Ok(())
}

fn run_with_writer<W: Write>(
    mut circuit: ElaboratedCircuit,
    format: OutputFormat,
    writer: W,
) -> Result<(), CliError> {
    match format {
        OutputFormat::Csv => {
            let mut telemetry = CsvTelemetryWriter::new(writer);
            dispatch_simulation(&mut circuit, &mut telemetry)
        }
        OutputFormat::Jsonl => {
            let mut telemetry = JsonLinesTelemetryWriter::new(writer);
            dispatch_simulation(&mut circuit, &mut telemetry)
        }
    }
}

/// Dispatches simulation execution based on the elaborated circuit simulation plan.
pub fn dispatch_simulation<T: TelemetryWriter>(
    circuit: &mut ElaboratedCircuit,
    telemetry: &mut T,
) -> Result<(), CliError> {
    let is_non_linear = circuit.graph.components().iter().any(|c| {
        matches!(
            c,
            ComponentRecord::Diode { .. }
                | ComponentRecord::Mosfet { .. }
                | ComponentRecord::Bjt { .. }
                | ComponentRecord::TcadDiode { .. }
                | ComponentRecord::TcadMosfet { .. }
                | ComponentRecord::NeuralSurrogate { .. }
        )
    });

    match &circuit.plan {
        SimulationPlan::Op => {
            // Build header columns
            let mut headers = Vec::new();
            for i in 1..circuit.graph.total_nodes() {
                let name = circuit.graph.node_name(NodeId::new(i as u32));
                headers.push(format!("V({})", name));
            }
            for b in 0..circuit.graph.total_branches() {
                let label = circuit.graph.branch_label(BranchId::new(b as u32));
                headers.push(label.to_string());
            }

            let header_strs: Vec<&str> = headers.iter().map(|s| s.as_str()).collect();
            telemetry.write_header(&header_strs)?;

            // Solve DC operating point
            let sol = if is_non_linear {
                solve_dc_non_linear(
                    &circuit.graph,
                    &circuit.model_ctx,
                    &NewtonOptions::default(),
                )?
            } else {
                solve_dc_linear(&circuit.graph, &SolverOptions::default())?
            };

            let mut row = Vec::new();
            for i in 1..circuit.graph.total_nodes() {
                row.push(sol.node_voltage(NodeId::new(i as u32)));
            }
            for b in 0..circuit.graph.total_branches() {
                row.push(sol.branch_current(BranchId::new(b as u32)));
            }

            telemetry.write_row(&row)?;
            telemetry.flush()?;
        }

        SimulationPlan::Dc {
            source_name,
            start,
            stop,
            step,
        } => {
            let mut headers = vec![source_name.clone()];
            for i in 1..circuit.graph.total_nodes() {
                let name = circuit.graph.node_name(NodeId::new(i as u32));
                headers.push(format!("V({})", name));
            }
            for b in 0..circuit.graph.total_branches() {
                let label = circuit.graph.branch_label(BranchId::new(b as u32));
                headers.push(label.to_string());
            }

            let header_strs: Vec<&str> = headers.iter().map(|s| s.as_str()).collect();
            telemetry.write_header(&header_strs)?;

            let start_val = *start;
            let stop_val = *stop;
            let step_size = step.abs().max(1e-12);
            let sign = if stop_val >= start_val { 1.0 } else { -1.0 };
            let total_span = (stop_val - start_val).abs();
            let num_steps = (total_span / step_size).round() as usize;

            for step_idx in 0..=num_steps {
                let current_val = if num_steps == 0 {
                    start_val
                } else {
                    let unclipped = start_val + sign * (step_idx as f64) * step_size;
                    if sign > 0.0 {
                        unclipped.min(stop_val)
                    } else {
                        unclipped.max(stop_val)
                    }
                };

                circuit
                    .graph
                    .update_component_value(source_name, current_val)?;

                let sol = if is_non_linear {
                    solve_dc_non_linear(
                        &circuit.graph,
                        &circuit.model_ctx,
                        &NewtonOptions::default(),
                    )?
                } else {
                    solve_dc_linear(&circuit.graph, &SolverOptions::default())?
                };

                let mut row = vec![current_val];
                for i in 1..circuit.graph.total_nodes() {
                    row.push(sol.node_voltage(NodeId::new(i as u32)));
                }
                for b in 0..circuit.graph.total_branches() {
                    row.push(sol.branch_current(BranchId::new(b as u32)));
                }

                telemetry.write_row(&row)?;
            }

            telemetry.flush()?;
        }

        SimulationPlan::Tran { tstep, tstop } => {
            let mut headers = vec!["time".to_string()];
            for i in 1..circuit.graph.total_nodes() {
                let name = circuit.graph.node_name(NodeId::new(i as u32));
                headers.push(format!("V({})", name));
            }
            for b in 0..circuit.graph.total_branches() {
                let label = circuit.graph.branch_label(BranchId::new(b as u32));
                headers.push(label.to_string());
            }

            let header_strs: Vec<&str> = headers.iter().map(|s| s.as_str()).collect();
            telemetry.write_header(&header_strs)?;

            let options = TransientOptions {
                tstop: *tstop,
                tstep: *tstep,
                tstart: 0.0,
                tmax: Some(*tstep),
                uic: false,
                method: IntegrationMethod::TrBdf2,
                step_control: StepControlOptions::default(),
                newton: NewtonOptions::default(),
                waveforms: std::collections::HashMap::new(),
            };

            let solution = solve_transient(&circuit.graph, &circuit.model_ctx, &options)?;

            for step in &solution.steps {
                let mut row = vec![step.time];
                for i in 1..circuit.graph.total_nodes() {
                    let node_id = NodeId::new(i as u32);
                    let v = if node_id.index() < step.voltages.len() {
                        step.voltages[node_id.index()]
                    } else {
                        0.0
                    };
                    row.push(v);
                }
                for b in 0..circuit.graph.total_branches() {
                    let br_id = BranchId::new(b as u32);
                    let i_br = if br_id.index() < step.branch_currents.len() {
                        step.branch_currents[br_id.index()]
                    } else {
                        0.0
                    };
                    row.push(i_br);
                }
                telemetry.write_row(&row)?;
            }

            telemetry.flush()?;
        }
    }

    Ok(())
}
