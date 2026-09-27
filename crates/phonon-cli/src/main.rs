//! Phonon CLI entry point.

use clap::{Parser, Subcommand};
use phonon_cli::commands::{execute_monte_carlo, execute_run, execute_sweep, execute_validate};
use phonon_cli::telemetry::OutputFormat;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(
    name = "phonon",
    author = "Aerovex Engineers <dev@aerovex.com>",
    version,
    about = "Industry-grade physically rigorous electro-thermal circuit simulator and transistor-level solver",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Validates circuit topology and electrical rules (ERC)
    Validate {
        /// Path to the SPICE netlist file
        netlist: PathBuf,
    },
    /// Runs a simulation (.OP, .DC, or .TRAN) and streams telemetry
    Run {
        /// Path to the SPICE netlist file
        netlist: PathBuf,
        /// Telemetry output format
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Csv)]
        format: OutputFormat,
        /// Output file destination (defaults to stdout if omitted)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Performs a parallel parametric sweep across component value ranges
    Sweep {
        /// Path to the SPICE netlist file
        netlist: PathBuf,
        /// Component parameter name to sweep (e.g. R1, V1)
        #[arg(short, long)]
        param: String,
        /// Starting parameter value
        #[arg(long)]
        start: f64,
        /// Stopping parameter value
        #[arg(long)]
        stop: f64,
        /// Number of sweep intervals
        #[arg(long, default_value_t = 10)]
        steps: usize,
        /// Telemetry output format
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Csv)]
        format: OutputFormat,
        /// Output file destination (defaults to stdout if omitted)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Performs parallel Monte Carlo statistical tolerance analysis
    Mc {
        /// Path to the SPICE netlist file
        netlist: PathBuf,
        /// Number of Monte Carlo sample circuits
        #[arg(short, long, default_value_t = 100)]
        samples: usize,
        /// Component tolerance fraction (e.g. 0.05 for 5%)
        #[arg(short, long, default_value_t = 0.05)]
        tol: f64,
        /// Random number generator seed
        #[arg(long, default_value_t = 42)]
        seed: u64,
        /// Telemetry output format
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Csv)]
        format: OutputFormat,
        /// Output file destination (defaults to stdout if omitted)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    let args = Cli::parse();

    match args.command {
        Commands::Validate { netlist } => match execute_validate(&netlist) {
            Ok(report) => {
                println!(
                    "Circuit: {}",
                    if report.title.is_empty() {
                        "<unnamed>"
                    } else {
                        &report.title
                    }
                );
                println!("Total Nodes:      {}", report.total_nodes);
                println!("Active Nodes:     {}", report.active_nodes);
                println!("Aux Branches:     {}", report.total_branches);
                println!("Components:       {}", report.total_components);
                println!("Validation:       PASSED");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("Validation FAILED: {e}");
                ExitCode::FAILURE
            }
        },
        Commands::Run {
            netlist,
            format,
            output,
        } => {
            if let Err(e) = execute_run(&netlist, format, output.as_deref()) {
                eprintln!("Simulation error: {e}");
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Commands::Sweep {
            netlist,
            param,
            start,
            stop,
            steps,
            format,
            output,
        } => {
            if let Err(e) = execute_sweep(
                &netlist,
                &param,
                start,
                stop,
                steps,
                format,
                output.as_deref(),
            ) {
                eprintln!("Sweep error: {e}");
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Commands::Mc {
            netlist,
            samples,
            tol,
            seed,
            format,
            output,
        } => match execute_monte_carlo(&netlist, samples, tol, seed, format, output.as_deref()) {
            Ok(stats) => {
                eprintln!(
                    "Monte Carlo completed: {} samples evaluated across active variables.",
                    samples
                );
                for (i, (&mean, &std)) in stats.mean.iter().zip(stats.std_dev.iter()).enumerate() {
                    eprintln!("  Var {}: mean = {:.6e}, std = {:.6e}", i + 1, mean, std);
                }
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("Monte Carlo error: {e}");
                ExitCode::FAILURE
            }
        },
    }
}
