# Simulation Execution (run)

The `run` subcommand executes the SPICE directives found in the specified netlist (such as `.TRAN`, `.DC`, or `.OP`) and outputs timestamped telemetry.

## Usage

```bash
phonon run <netlist_path> [OPTIONS]
```

## Options

- `-f, --format <FORMAT>`: Output format (`csv`, `json`, `binary`). Default: `csv`.
- `-o, --output <PATH>`: Destination file path (defaults to standard output).

## Examples

```bash
# Stream CSV to stdout
phonon run oscillator.cir

# Write JSON telemetry to file
phonon run amplifier.cir -f json -o output.json
```
