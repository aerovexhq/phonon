# Tier 5: Coupled Electro-Thermal Multi-Physics

Tier 5 models the bidirectional feedback coupling between electrical power dissipation and thermal temperature rise.

## Governing Equations
- **Joule Self-Heating**: Local electrical dissipation converted to heat flux.
- **Dynamic Cauer RC Thermal Ladders**: Multi-stage thermal resistance and capacitance networks modeling transient heat conduction through die, thermal interface materials, and heatsinks.
- **Coupled Newton-Raphson Matrix**: Simultaneous electrical node voltages and thermal junction temperatures solved in a unified Jacobian matrix.
- **Thermal Runaway Safeguards**: Automatic detection of positive electro-thermal feedback loops and stability limits.
