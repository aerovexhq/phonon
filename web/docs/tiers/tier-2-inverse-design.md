# Tier 2: Inverse Multi-Objective Synthesis

Tier 2 incorporates genetic optimization and adjoint gradients to synthesize optimal semiconductor geometries.

## Features
- **NSGA-II Non-Dominated Sorting**: Multi-objective genetic algorithm optimizing power, delay, and thermal dissipation simultaneously.
- **Transistor Genome Encoding**: Gate-All-Around (GAA) Nanosheet, Complementary FET (CFET), and FinFET geometric representations.
- **Fast Adjoint Gradient Refinement**: Hybrid search combining broad evolutionary exploration with rapid local gradient ascent.
- **Pareto Optimal Frontier Exploration**: Interactive trade-off curves exportable directly to compact SPICE models.
