# CLI Command Reference

Phonon features a clean, unified command-line interface.

## Command Overview

| Command | Syntax | Description |
| :--- | :--- | :--- |
| **`phonon`** | `phonon` | Display system version, interactive banner, and command index |
| **`phonon gui`** | `phonon gui` | Launch native desktop GPU-accelerated CAD studio |
| **`phonon validate`** | `phonon validate <netlist>` | Validate circuit topology and electrical rules (ERC) |
| **`phonon run`** | `phonon run <netlist> [options]` | Execute simulation (.OP, .DC, .TRAN) and stream telemetry |
| **`phonon sweep`** | `phonon sweep <netlist> [options]` | Execute parallel parametric sweep over component values |
| **`phonon mc`** | `phonon mc <netlist> [options]` | Execute Monte Carlo statistical tolerance analysis |
| **`phonon help`** | `phonon --help` | Display general or subcommand help |

## Global Flags

- `-h, --help`: Print comprehensive help information.
- `-V, --version`: Print Phonon version (`0.1.0`).
