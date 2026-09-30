# Parametric Sweep (sweep)

The `sweep` subcommand executes parallel parameter sweeps over component values across worker threads.

## Usage

```bash
phonon sweep <netlist_path> --param <NAME> --start <VAL> --stop <VAL> [OPTIONS]
```

## Options

- `-p, --param <NAME>`: Component parameter to vary (e.g. `R1`, `Cload`, `Vbias`).
- `--start <VAL>`: Starting numeric value.
- `--stop <VAL>`: Ending numeric value.
- `--steps <N>`: Number of evaluation intervals (default: `10`).
- `-f, --format <FORMAT>`: Output format (`csv`, `json`, `binary`). Default: `csv`.
- `-o, --output <PATH>`: Output destination path.
