# Monte Carlo Statistical Analysis (mc)

The `mc` subcommand conducts parallel Monte Carlo statistical tolerance analysis with Gaussian component value variations.

## Usage

```bash
phonon mc <netlist_path> [OPTIONS]
```

## Options

- `-s, --samples <N>`: Number of sample circuits to evaluate (default: `100`).
- `-t, --tol <FRACTION>`: Component standard deviation tolerance (e.g. `0.05` for 5%, default: `0.05`).
- `--seed <N>`: Pseudo-random generator seed for reproducible evaluations (default: `42`).
- `-f, --format <FORMAT>`: Output format (`csv`, `json`, `binary`). Default: `csv`.
- `-o, --output <PATH>`: Output file destination.
