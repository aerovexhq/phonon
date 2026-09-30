# Circuit Validation (ERC)

The `validate` subcommand parses a SPICE netlist, constructs the incidence matrix, checks electrical rule compliance, and flags floating or shorted nodes.

## Usage

```bash
phonon validate <netlist_path>
```

## Example Output

```text
Circuit: Bandpass Filter Stage 1
Total Nodes:      12
Active Nodes:     11
Aux Branches:     2
Components:       18
Validation:       PASSED
```
