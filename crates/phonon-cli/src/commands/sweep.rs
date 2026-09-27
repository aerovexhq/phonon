//! Parallel parametric sweep command (`phonon sweep`).

use crate::error::CliError;
use crate::telemetry::{
    CsvTelemetryWriter, JsonLinesTelemetryWriter, OutputFormat, TelemetryWriter,
};
use phonon_core::{BranchId, ComponentRecord, NodeId};
use phonon_netlist::{elaborate_netlist, parse_netlist};
use phonon_solver::mna::{solve_dc_linear, solve_dc_non_linear, NewtonOptions, SolverOptions};
use rayon::prelude::*;
use std::fs::File;
use std::io::{stdout, BufWriter, Write};
use std::path::Path;

/// Executes a parallel parametric sweep across component value ranges using Rayon.
pub fn execute_sweep(
    netlist_path: &Path,
    param_name: &str,
    start: f64,
    stop: f64,
    steps: usize,
    format: OutputFormat,
    output_path: Option<&Path>,
) -> Result<(), CliError> {
    let content = std::fs::read_to_string(netlist_path)?;
    let parsed = parse_netlist(&content)?;
    let elaborated = elaborate_netlist(&parsed)?;

    // Verify parameter component exists
    if elaborated.graph.get_component(param_name).is_none() {
        return Err(CliError::Execution(format!(
            "Parameter component '{}' was not found in the circuit",
            param_name
        )));
    }

    let is_non_linear = elaborated.graph.components().iter().any(|c| {
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

    let n_steps = steps.max(1);
    let sweep_values: Vec<f64> = (0..=n_steps)
        .map(|i| start + (stop - start) * (i as f64) / (n_steps as f64))
        .collect();

    // Parallel evaluation using Rayon
    let results: Result<Vec<Vec<f64>>, CliError> = sweep_values
        .par_iter()
        .map(|&val| {
            let mut graph = elaborated.graph.clone();
            graph.update_component_value(param_name, val)?;

            let sol = if is_non_linear {
                solve_dc_non_linear(&graph, &elaborated.model_ctx, &NewtonOptions::default())?
            } else {
                solve_dc_linear(&graph, &SolverOptions::default())?
            };

            let mut row = vec![val];
            for i in 1..graph.total_nodes() {
                row.push(sol.node_voltage(NodeId::new(i as u32)));
            }
            for b in 0..graph.total_branches() {
                row.push(sol.branch_current(BranchId::new(b as u32)));
            }

            Ok(row)
        })
        .collect();

    let rows = results?;

    // Build headers
    let mut headers = vec![param_name.to_string()];
    for i in 1..elaborated.graph.total_nodes() {
        let name = elaborated.graph.node_name(NodeId::new(i as u32));
        headers.push(format!("V({})", name));
    }
    for b in 0..elaborated.graph.total_branches() {
        let label = elaborated.graph.branch_label(BranchId::new(b as u32));
        headers.push(label.to_string());
    }

    if let Some(out_p) = output_path {
        let file = File::create(out_p)?;
        let writer = BufWriter::new(file);
        write_sweep_results(&headers, &rows, format, writer)?;
    } else {
        let writer = BufWriter::new(stdout());
        write_sweep_results(&headers, &rows, format, writer)?;
    }

    Ok(())
}

fn write_sweep_results<W: Write>(
    headers: &[String],
    rows: &[Vec<f64>],
    format: OutputFormat,
    writer: W,
) -> Result<(), CliError> {
    let header_strs: Vec<&str> = headers.iter().map(|s| s.as_str()).collect();

    match format {
        OutputFormat::Csv => {
            let mut telemetry = CsvTelemetryWriter::new(writer);
            telemetry.write_header(&header_strs)?;
            for row in rows {
                telemetry.write_row(row)?;
            }
            telemetry.flush()?;
        }
        OutputFormat::Jsonl => {
            let mut telemetry = JsonLinesTelemetryWriter::new(writer);
            telemetry.write_header(&header_strs)?;
            for row in rows {
                telemetry.write_row(row)?;
            }
            telemetry.flush()?;
        }
    }

    Ok(())
}
