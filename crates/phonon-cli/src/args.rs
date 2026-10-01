#![deny(unsafe_code)]

//! CLI argument definitions and subcommand schema for Phonon.

use crate::telemetry::OutputFormat;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "phonon",
    author = "Aerovex Engineers <dev@aerovex.com>",
    version,
    about = "Industry-grade physically rigorous electro-thermal circuit simulator and transistor-level solver",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum Commands {
    /// Launches the native desktop CAD interface and interactive visual studio
    Gui,
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
