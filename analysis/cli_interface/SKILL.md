---
name: cli_interface
description: Architecture for headless high-throughput CLI simulation, SPICE/netlist parsing, parametric Monte Carlo sweeps, telemetry streaming (Arrow, CSV, VCD), and CI/CD integration.
---

# Headless CLI Engine, Netlist Ingestion & Telemetry

## 1. CLI Architecture & Command Structure

The command-line interface (`phonon-cli`) provides a scriptable, headless execution environment for automated verification, batch simulation runs, parametric optimization, and CI/CD pipelines.

Built with `clap` (using the derive macro pattern):

```
phonon [OPTIONS] <COMMAND>

COMMANDS:
  run         Execute a circuit simulation netlist directly
  sweep       Execute parametric or Monte Carlo sweeps across component tolerances
  bench       Run standard SPICE / NIST benchmark netlists and output timing statistics
  convert     Convert between netlist formats (SPICE 3f5 <-> HSPICE <-> Phonon RON)
  validate    Perform static electrical rule checks (ERC) and KCL sanity validation

OPTIONS:
  -f, --format <FORMAT>      Output format: csv, arrow, jsonl, vcd [default: arrow]
  -o, --output <PATH>        Output file path for waveform telemetry
  -t, --threads <N>          Number of worker threads for parallel sweeps [default: auto]
  -v, --verbose              Enable detailed convergence diagnostics and LTE step traces
  --thermal-mode <MODE>      Electro-thermal coupling: disabled, partitioned, monolithic [default: monolithic]
```

---

## 2. Netlist Ingestion Pipeline

Phonon incorporates a high-speed, zero-copy lexer and parser for industry-standard SPICE netlists and modern structured formats:

```
[ Raw File / Netlist ]
          |
          v (Logos Zero-Copy Lexer)
[ Token Stream: Identifiers, Floats, Units (10u, 1n, 5meg) ]
          |
          v (Recursive Descent AST Parser)
[ Raw AST: Components, Models, Subcircuits, Directives (.TRAN, .OP) ]
          |
          v (Subcircuit Flattening & Parameter Substitution)
[ Flattened Circuit Graph: Nodes, Pins, Devices ]
          |
          v (Static Electrical Rule Check: Floating nodes, loops of V-sources)
[ Elaborated MNA Matrix Topology ]
```

### 2.1 SPICE Unit Suffix Parsing
Phonon strictly adheres to standard SPICE unit multipliers (case-insensitive):
- `F` = $10^{-15}$ (Femto)
- `P` = $10^{-12}$ (Pico)
- `N` = $10^{-9}$ (Nano)
- `U` = $10^{-6}$ (Micro)
- `M` / `MIL` = $10^{-3}$ (Milli) / $2.54 \times 10^{-5}\text{ m}$
- `K` = $10^{3}$ (Kilo)
- `MEG` = $10^{6}$ (Mega)
- `G` = $10^{9}$ (Giga)
- `T` = $10^{12}$ (Tera)

*Critical Pitfall*: In SPICE, `1M` represents $10^{-3}$ (milli), whereas `1MEG` represents $10^6$ (mega). Phonon correctly parses this distinction to avoid catastrophic capacitor/resistor sizing bugs.

---

## 3. High-Throughput Telemetry Streaming

Simulating stiff circuits over extended intervals can generate tens of millions of time-steps. Storing all states in memory causes out-of-memory (OOM) crashes. Phonon streams data incrementally:

```rust
pub trait TelemetryWriter {
    fn write_header(&mut self, signal_names: &[&str]) -> Result<(), IoError>;
    fn write_sample(&mut self, time: f64, values: &[f64]) -> Result<(), IoError>;
    fn flush(&mut self) -> Result<(), IoError>;
}
```

### 3.1 Apache Arrow & Parquet Streaming
- For massive data sets ($>10^7$ samples), Phonon writes directly into **Apache Arrow RecordBatches**.
- Utilizes columnar storage with Snappy/ZSTD compression.
- Allows instant zero-copy loading in Python (`polars`, `pandas`), Julia, or Rust for post-simulation data analysis.

### 3.2 Value Change Dump (VCD)
- For mixed-signal simulations involving digital states, Phonon outputs standard IEEE 1364 VCD files.
- Inspectable using industry viewers like GTKWave or Phonon's built-in GUI visualizer.

---

## 4. Multi-Threaded Parametric Sweeps & Monte Carlo

Using `rayon`, Phonon distributes parameter sweeps across all available CPU cores:

### 4.1 Monte Carlo Tolerance Analysis
Given component tolerance distributions (Gaussian or Uniform):
$$R_i \sim \mathcal{N}\left( R_{nominal}, (\text{tol} \cdot R_{nominal} / 3)^2 \right)$$
$$C_j \sim \mathcal{U}\left( C_{nominal}(1 - \text{tol}), C_{nominal}(1 + \text{tol}) \right)$$

```rust
pub fn run_monte_carlo(
    netlist: &Netlist,
    num_runs: usize,
    tolerances: &[ComponentTolerance],
) -> Vec<SimulationResult> {
    use rayon::prelude::*;
    (0..num_runs)
        .into_par_iter()
        .map(|seed| {
            let mut randomized_circuit = netlist.sample_with_seed(seed, tolerances);
            let mut solver = Solver::new(&randomized_circuit);
            solver.run_transient()
        })
        .collect()
}
```
Linear scaling: A 1000-run Monte Carlo sweep runs in parallel across 16–64 cores with zero inter-thread lock contention.

---

## 5. Telemetry Serialization Schemas & Formats

### 5.1 Apache Arrow RecordBatch Columnar Schema
For large-scale analog, spintronic, and quantum simulation runs, Phonon serializes state data into Arrow RecordBatches:

```
+--------------------------------------------------------------------------------+
| Column Name   | Arrow DataType     | Physical Interpretation                   |
|:--------------|:-------------------|:------------------------------------------|
| `time`        | `Float64`          | Simulation time $t$ in seconds            |
| `v(node_*)`   | `Float64`          | Node voltages in Volts                    |
| `i(branch_*)` | `Float64`          | Branch currents in Amperes                |
| `temp(die_*)` | `Float64`          | Local thermal node temperatures in Kelvin  |
| `m_x,m_y,m_z` | `Float64`          | Normalized magnetization components       |
| `syndrome`    | `UInt64` / `Binary`| Surface code QEC syndrome bitmask vectors  |
| `phase_slip`  | `UInt8` (Boolean)  | Single flux quantum ($2\pi$) slip event    |
+--------------------------------------------------------------------------------+
```
- **Chunked Compression**: Arrow batches are written with Snappy or ZSTD compression in 64KB blocks, reducing file size by up to $85\%$ compared to raw ASCII CSV.
- **Direct Parquet Writing**: Batches are converted directly into Parquet files with embedded metadata dictionary encoding.

### 5.2 IEEE 1364 Value Change Dump (VCD)
For digital logic, RSFQ circuits, and mixed-signal interfaces:
- Generates 4-state ($0, 1, X, Z$) logic transitions based on threshold crossing.
- Compact time-stamped delta encoding: only signals that undergo transition are recorded.

---

## 6. Batch Automation, Scripting & CI/CD Integration

### 6.1 Machine-Readable Headless Diagnostics
When executed with `--json`, `phonon-cli` suppresses terminal ANSI styling and streams structured NDJSON telemetry:
```json
{"event":"step_completed","time":1.24e-9,"dt":5.0e-12,"nr_iterations":3,"residual_norm":1.42e-7}
{"event":"qec_syndrome_decoded","round":42,"syndrome_hex":"0x1A","correction":"X(2)","latency_ps":85.4}
```

### 6.2 Standardized Exit Codes
- `0`: Simulation completed successfully with all tolerances satisfied.
- `1`: Syntax or netlist elaboration error (unresolved node, circular subcircuit).
- `2`: Electrical rule check (ERC) failure (floating node, shorted voltage source).
- `3`: Numerical convergence failure (singular Jacobian, time step underflow).
- `4`: Physical invariant violation (magnetization norm drift, flux leak).

