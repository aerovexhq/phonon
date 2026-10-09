#![deny(unsafe_code)]

//! Phonon CLI entry point.

use clap::Parser;
use phonon_cli::commands::{execute_monte_carlo, execute_run, execute_sweep, execute_validate};
use phonon_cli::{Cli, Commands};
use std::process::ExitCode;

fn print_banner() {
    println!(
r#"  ____  _   _  ___  _   _  ___  _   _ 
 |  _ \| | | |/ _ \| \ | |/ _ \| \ | |
 | |_) | |_| | | | |  \| | | | |  \| |
 |  __/|  _  | |_| | |\  | |_| | |\  |
 |_|   |_| |_|\___/|_| \_|\___/|_| \_|
 Phonon Universal Multi-Scale Visual CAD Studio & Semiconductor Solver (v{})
 Documentation: https://phonon.aerovex.net

Usage:
  phonon [COMMAND]
  phonon gui          Launch native GPU-accelerated desktop CAD studio
  phonon --help      Display full CLI commands and flag options

Commands:
  gui                Launch desktop CAD visual studio
  validate <netlist> Validate circuit topology and electrical rules (ERC)
  run <netlist>      Execute simulation (.OP, .DC, .TRAN) and stream telemetry
  sweep <netlist>    Execute parallel parametric component sweep
  mc <netlist>       Execute Monte Carlo statistical tolerance analysis

Run 'phonon --help' or 'phonon <command> --help' for detailed syntax."#,
        env!("CARGO_PKG_VERSION")
    );
}

fn main() -> ExitCode {
    let args = Cli::parse();

    let command = match args.command {
        Some(cmd) => cmd,
        None => {
            print_banner();
            return ExitCode::SUCCESS;
        }
    };

    match command {
        Commands::Gui => {
            #[cfg(not(target_arch = "wasm32"))]
            match phonon_gui::run_gui() {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("Phonon GUI runtime error: {e}");
                    ExitCode::FAILURE
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                eprintln!("Phonon GUI cannot be run directly via CLI on WebAssembly");
                ExitCode::FAILURE
            }
        }
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
