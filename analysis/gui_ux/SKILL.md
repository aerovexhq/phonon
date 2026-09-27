---
name: gui_ux
description: Cross-platform hardware CAD GUI architecture, interactive schematic capture, virtual oscilloscope, dynamic thermal heatmaps, egui/wgpu vs Tauri trade-offs, and WebAssembly deployment.
---

# Cross-Platform Hardware CAD GUI & Visualization

## 1. GUI Architectural Strategy: `egui` + `wgpu` vs. `Tauri`

To deliver an industry-grade hardware simulator that is both ultra-responsive and visually modern across Windows, macOS, Linux, and the Web, we analyze two premier Rust GUI architectures:

| Evaluation Metric | Strategy A: `egui` + `wgpu` (Native Immediate-Mode) | Strategy B: `Tauri v2` + Web Frontend |
| :--- | :--- | :--- |
| **Rendering Performance** | **Exceptional (60–144+ FPS)**: Direct GPU draw calls via `wgpu`. Zero IPC overhead. | **Moderate**: UI in WebView; waveform data must cross IPC bridge (JSON/Binary IPC serialization). |
| **Memory Footprint** | Extremely low (~25–40 MB RAM). | Moderate (~80–180 MB RAM due to system WebView engine). |
| **Oscilloscope Waveforms** | Can render $10^6+$ dynamic points smoothly using custom vertex shaders. | Requires specialized WebGL/WebGPU canvas inside the webview to avoid DOM lag. |
| **Schematic CAD Pan/Zoom** | Sub-millisecond latency on infinite vector canvas. | Fast with Canvas2D/Pixi.js, but susceptible to WebView gesture jitter. |
| **Web Deployment (WASM)** | **Native WASM target via `eframe`**: identical code compiles to WebGL2/WebGPU in browser. | Web version is already written in web tech, but solver must be compiled separately to WASM. |
| **Visual Customization** | Highly functional CAD aesthetic; fully themeable with modern dark/light styling. | Full CSS/Tailwind freedom; looks like a modern SaaS app out of the box. |

### Architectural Decision:
- **Primary Engine**: `phonon-gui` is built on **`egui` + `wgpu`** via `eframe`. This guarantees zero-latency interactive circuit probing, real-time 144 FPS oscilloscope scrubbing, and 100% native compilation to desktop binaries and browser WebAssembly.
- **Web Portability**: Compiles directly to `wasm32-unknown-unknown` running in modern browsers with WebGPU/WebGL2 and virtual filesystem persistence via `IndexedDB`.
- **Tauri Alternative**: The core solver exposes a C/FFI and WASM JS bridge, allowing a secondary Tauri v2 / Svelte web frontend if an HTML/CSS UI is desired for enterprise cloud deployments.

---

## 2. Interactive Schematic Capture Engine

### 2.1 Infinite Canvas & Coordinate Transformations
The schematic editor uses a dual-coordinate projection system:
1. **Screen Space** $(x_s, y_s) \in \text{Pixels}$: Mouse interactions, HUD overlays, toolbars.
2. **World / Schematic Space** $(x_w, y_w) \in \text{Mils / Grid Units}$: Component pins, wires, net junctions.

$$\begin{bmatrix} x_s \\ y_s \end{bmatrix} = \text{Scale} \cdot \begin{bmatrix} x_w \\ y_w \end{bmatrix} + \begin{bmatrix} \text{Pan}_x \\ \text{Pan}_y \end{bmatrix}$$

### 2.2 Orthogonal Wire Routing & Net Graph
- **Pin Snapping**: Wires automatically snap to component pin bounding radii (default $8\text{px}$).
- **Manhattan Routing**: Wires route orthogonally (strictly horizontal and vertical segments).
- **Automated Junction Creation**: When a wire endpoint touches an existing wire segment, a physical Net Junction node is inserted automatically.
- **Topological Netlist Generation**: The visual graph is converted into an electrical incidence graph during circuit compilation with zero manual netlist coding required.

---

## 3. Real-Time Virtual Oscilloscope & Spectrum Analyzer

```
+-----------------------------------------------------------------+
|  [Probe: Node V(OUT)]   [Scale: 500mV/div]   [Time: 20us/div]   |
+-----------------------------------------------------------------+
|   /\          /\          /\          /\          /\            |
|  /  \        /  \        /  \        /  \        /  \           |
| /    \  /\  /    \  /\  /    \  /\  /    \  /\  /    \  /\      |
|/      \/  \/      \/  \/      \/  \/      \/  \/      \/  \     |
+-----------------------------------------------------------------+
|  V_pp: 3.28 V  |  Freq: 45.2 kHz  |  THD: 0.04%  |  Temp: 48.2 C|
+-----------------------------------------------------------------+
```

### 3.1 Hardware-Accelerated Trace Rendering
- **Ring Buffer Stream**: The simulation engine pushes time-voltage tuples `(t, v)` across a lock-free crossbeam channel directly into a circular ring buffer (`TraceBuffer`).
- **Decimation & Level of Detail (LoD)**: For waveforms with $> 10^5$ samples, the visualizer applies Min-Max decimation (preserving critical voltage peaks and troughs) to eliminate unnecessary vertex GPU overhead.
- **Trigger Modes**: Edge-triggered (Rising, Falling, Hysteresis window) sweeps keep high-frequency periodic signals stationary on screen.

### 3.2 FFT Spectrum Analyzer
Performs real-time Fast Fourier Transforms using `rustfft`:
- Windowing functions: Hanning, Blackman-Harris, Flat-Top.
- Total Harmonic Distortion (THD) calculation and phase margin estimation.

---

## 4. 2D/3D Spatial Thermal Heatmap Overlay

In electro-thermal simulation mode, heat spreads dynamically across the circuit layout. Phonon overlays a spatial thermal heatmap directly on top of the schematic or die view:

```
+-------------------------------------------------------------+
|  Schematic Canvas with Dynamic Thermal Overlay             |
|                                                             |
|       [R1: 32C]               [Power MOSFET M1: 94C]        |
|          :::                         (HOTSPOT)              |
|          :::                    ..::*******::..             |
|                                .:::*********:::.            |
|                                .:::*********:::.            |
|                                 ..::*******::..             |
|       [C1: 29C]                       :::                   |
|                                  [Diode D1: 65C]            |
+-------------------------------------------------------------+
```
- **Shader Implementation**: A fragment shader renders a smooth interpolated scalar field using Bicubic or Gaussian interpolation between discrete thermal nodes.
- **Colormaps**: Turbo, Magma, and Inferno colormaps mapping temperature ranges ($T_{amb} \to T_{max}$) into intuitive visual gradients.
- **Real-Time Hotspot Detection**: Automatically places warning flags on components exceeding safe thermal design limits ($T_j > 125^\circ\text{C}$).

---

## 5. WebAssembly & Browser Storage (IndexedDB)

When compiled to WebAssembly (`wasm32-unknown-unknown`):
1. **Graphics**: `wgpu` binds automatically to WebGPU (or falls back to WebGL2).
2. **Filesystem Virtualization**: Phonon interfaces with browser storage through an abstraction layer:
   - On native desktop: reads/writes standard OS files via `std::fs`.
   - In browser WASM: reads/writes via a virtual in-memory filesystem synchronized asynchronously with browser `IndexedDB`.
3. **Web Worker Simulation**: The numerical solver runs in a dedicated Web Worker thread (using SharedArrayBuffer and Atomics where supported), ensuring UI frame rates never stutter during heavy computations.

---

## 6. Cryogenic, Quantum & Multi-Physical CAD Overlays

Phonon's CAD canvas renders multi-physical phenomenon beyond classical voltage and current:

### 6.1 Rotated Surface-Code Lattice Visualizer
```
+-------------------------------------------------------------+
| Surface Code (d=3) Lattice View                             |
|                                                             |
|       (D0)-------[X0]-------(D1)-------[X1]-------(D2)      |
|        |                     |                     |        |
|       [Z0]                  [Z1]                  [Z2]      |
|        |                     |                     |        |
|       (D3)-------[X2]-------(D4)-------[X3]-------(D5)      |
|        |                     |                     |        |
|       [Z3]                  [Z4]                  [Z5]      |
|        |                     |                     |        |
|       (D6)-------[X4]-------(D7)-------[X5]-------(D8)      |
+-------------------------------------------------------------+
```
- **Real-Time Syndrome Highlighting**: Active error syndromes ($S_k = -1$) flash visually (red for $Z$-stabilizers indicating bit-flips, blue for $X$-stabilizers indicating phase-flips).
- **Minimum-Weight Perfect Matching (MWPM) & Lookup Correction Vectors**: Graphical correction lines show the exact Pauli corrections ($X_i, Z_i, Y_i$) applied by the cryogenic coprocessor.

### 6.2 Spintronic Bloch Spheres & Magnetization Glyphs
- Interactive 3D vector arrows on magnetic tunnel junctions showing instantaneous direction $\vec{m}(t) = (m_x, m_y, m_z)$.
- Damped precessional trajectories traced on unit Bloch spheres during spin-transfer torque switching.

### 6.3 Integrated Photonics & Waveguide Optical Flow
- Color intensity gradient mapping optical power: $P_{opt}(z, t) = P_0 e^{-\alpha z}$.
- Real-time spectral resonance curves of micro-ring resonators with interactive Q-factor and wavelength slider controls.

---

## 7. High-DPI Zero-Lag Immediate-Mode Rendering Pipeline

Rendering millions of analog waveform points in an immediate-mode GUI without frame drops requires specialized GPU techniques:

1. **Vertex Instancing & Min-Max Decimation**:
   - For traces with $N > 10^5$ samples, points within the same pixel column are decimated to their minimum and maximum values:
     $$\Delta y_{col} = \big[ \min_{t \in col} V(t), \, \max_{t \in col} V(t) \big]$$
   - Generates only 2 vertices per horizontal pixel column, capping GPU geometry to $2 \times \text{ScreenWidth}$ (e.g., ~5,120 vertices for a 4K display) regardless of simulation dataset size.
2. **Lock-Free Triple Buffering**:
   - The solver worker writes to an alternating back buffer.
   - An atomic pointer swap exposes the latest complete telemetry frame to the render loop, guaranteeing zero torn frames and zero mutex lock contention.

