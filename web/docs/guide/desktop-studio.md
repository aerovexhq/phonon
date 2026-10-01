# Desktop Visual CAD Studio

The Phonon Desktop Visual CAD Studio is a high-performance native application built with Rust, `eframe`, and `egui`. It delivers immediate GPU-accelerated schematic capture, virtual multi-channel oscilloscope visualization, and coupled electro-thermal contour rendering.

## Launching the Desktop Studio

Launch the interface from terminal or your desktop application launcher:

```bash
phonon gui
```

## Studio Capabilities

### 1. Interactive Schematic Capture
- Fast component placement: resistors, capacitors, inductors, diodes, BJTs, MOSFETs, and topological acoustic cavities.
- Orthogonal net routing with automatic node junction resolution.
- Dynamic parameter inspector allowing immediate edits to resistance, capacitance, gate lengths, and cryogenic operating temperatures.

### 2. Virtual Multi-Channel Oscilloscope
- Hardware-accelerated plotting of multi-node node voltages and branch currents.
- Transient (.TRAN), DC operating point (.OP), and DC voltage/current transfer curves (.DC).
- Interactive cursors, zoom, pan, and real-time FFT frequency domain spectrum analysis.

### 3. Coupled Electro-Thermal Contour Viewer
- Monolithic steady-state and dynamic thermal maps.
- Real-time visualization of silicon junction hot-spots, thermal dissipation paths, and packaging boundary condition heat sinks.
- Early warning indicator for thermal runaway conditions.

### 4. Direct SPICE Netlist Import & Export
- Load existing industry-standard SPICE netlists (.cir, .sp, .net).
- Live ERC (Electrical Rules Checking) syntax validation.
- Export modified netlists and high-resolution schematics in vector SVG or PNG.
