//! Parallel Monte Carlo statistical tolerance analysis command (`phonon mc`).

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

/// Simple fast PRNG (XorShift64*) for deterministic, reproducible Monte Carlo sampling.
struct FastRng(u64);

impl FastRng {
    fn new(seed: u64) -> Self {
        Self(if seed == 0 { 0x853c49e6748fea9b } else { seed })
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    /// Uniform floating point in [-1.0, 1.0].
    fn next_symmetric_f64(&mut self) -> f64 {
        let u = (self.next_u64() >> 11) as f64 / ((1u64 << 53) as f64);
        2.0 * u - 1.0
    }
}

/// Statistical summary of Monte Carlo analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct MonteCarloStats {
    pub mean: Vec<f64>,
    pub std_dev: Vec<f64>,
    pub min: Vec<f64>,
    pub max: Vec<f64>,
}

/// Executes parallel Monte Carlo tolerance analysis across $N$ sample circuits.
pub fn execute_monte_carlo(
    netlist_path: &Path,
    samples: usize,
    tolerance: f64,
    seed: u64,
    format: OutputFormat,
    output_path: Option<&Path>,
) -> Result<MonteCarloStats, CliError> {
    let content = std::fs::read_to_string(netlist_path)?;
    let parsed = parse_netlist(&content)?;
    let elaborated = elaborate_netlist(&parsed)?;

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

    let n_samples = samples.max(1);

    // Parallel Monte Carlo evaluation using Rayon
    let sample_indices: Vec<usize> = (0..n_samples).collect();

    let sample_results: Result<Vec<Vec<f64>>, CliError> = sample_indices
        .par_iter()
        .map(|&idx| {
            let sample_seed = seed.wrapping_add((idx as u64).wrapping_mul(0x9E3779B97F4A7C15));
            let mut rng = FastRng::new(sample_seed);

            let mut graph = elaborated.graph.clone();

            // Perturb passive components within +/- tolerance
            for comp in graph.components_mut() {
                match comp {
                    ComponentRecord::Resistor { resistance, .. } => {
                        let delta = rng.next_symmetric_f64() * tolerance;
                        *resistance *= (1.0 + delta).max(1e-6);
                    }
                    ComponentRecord::Capacitor { capacitance, .. } => {
                        let delta = rng.next_symmetric_f64() * tolerance;
                        *capacitance *= (1.0 + delta).max(1e-15);
                    }
                    ComponentRecord::Inductor { inductance, .. } => {
                        let delta = rng.next_symmetric_f64() * tolerance;
                        *inductance *= (1.0 + delta).max(1e-12);
                    }
                    _ => {}
                }
            }

            let sol = if is_non_linear {
                solve_dc_non_linear(&graph, &elaborated.model_ctx, &NewtonOptions::default())?
            } else {
                solve_dc_linear(&graph, &SolverOptions::default())?
            };

            let mut row = vec![idx as f64];
            for i in 1..graph.total_nodes() {
                row.push(sol.node_voltage(NodeId::new(i as u32)));
            }
            for b in 0..graph.total_branches() {
                row.push(sol.branch_current(BranchId::new(b as u32)));
            }

            Ok(row)
        })
        .collect();

    let rows = sample_results?;

    // Build headers
    let mut headers = vec!["sample_id".to_string()];
    for i in 1..elaborated.graph.total_nodes() {
        let name = elaborated.graph.node_name(NodeId::new(i as u32));
        headers.push(format!("V({})", name));
    }
    for b in 0..elaborated.graph.total_branches() {
        let label = elaborated.graph.branch_label(BranchId::new(b as u32));
        headers.push(label.to_string());
    }

    // Compute statistics across the output variables (excluding sample_id)
    let num_vars = headers.len() - 1;
    let mut sums = vec![0.0; num_vars];
    let mut min_vals = vec![f64::INFINITY; num_vars];
    let mut max_vals = vec![f64::NEG_INFINITY; num_vars];

    for row in &rows {
        for (v_idx, &val) in row.iter().skip(1).enumerate() {
            sums[v_idx] += val;
            if val < min_vals[v_idx] {
                min_vals[v_idx] = val;
            }
            if val > max_vals[v_idx] {
                max_vals[v_idx] = val;
            }
        }
    }

    let mut means = vec![0.0; num_vars];
    for (i, &s) in sums.iter().enumerate() {
        means[i] = s / (n_samples as f64);
    }

    let mut var_sums = vec![0.0; num_vars];
    for row in &rows {
        for (v_idx, &val) in row.iter().skip(1).enumerate() {
            let diff = val - means[v_idx];
            var_sums[v_idx] += diff * diff;
        }
    }

    let mut std_devs = vec![0.0; num_vars];
    for (i, &vs) in var_sums.iter().enumerate() {
        std_devs[i] = (vs / (n_samples as f64)).sqrt();
    }

    if let Some(out_p) = output_path {
        let file = File::create(out_p)?;
        let writer = BufWriter::new(file);
        write_mc_results(&headers, &rows, format, writer)?;
    } else {
        let writer = BufWriter::new(stdout());
        write_mc_results(&headers, &rows, format, writer)?;
    }

    Ok(MonteCarloStats {
        mean: means,
        std_dev: std_devs,
        min: min_vals,
        max: max_vals,
    })
}

fn write_mc_results<W: Write>(
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
