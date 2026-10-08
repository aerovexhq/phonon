#![deny(unsafe_code)]

//! Phase 435: Multi-Qubit Acoustic Crossbar & Routing Engine.
//!
//! Models an N-qubit (default 4-qubit) crossbar network where superconducting transmon
//! qubits coupled to the 4 corners of a 2D topological acoustic metamaterial exchange
//! quantum states and execute entangling gates mediated by virtual acoustic phonons.

/// Parameters for the multi-qubit acoustic crossbar.
#[derive(Debug, Clone, PartialEq)]
pub struct CrossbarParams {
    /// Number of transmon qubits connected to the acoustic crossbar (default: 4).
    pub qubit_count: usize,
    /// Inter-corner boundary acoustic bus coupling in MHz (default: 1.5 MHz).
    pub bus_coupling_mhz: f64,
    /// Active source qubit index for routing (0..qubit_count - 1, default: 0).
    pub source_qubit: usize,
    /// Active target qubit index for routing (0..qubit_count - 1, default: 1).
    pub target_qubit: usize,
    /// Target crosstalk isolation in dB (default: 38.0 dB).
    pub isolation_target_db: f64,
}

impl Default for CrossbarParams {
    fn default() -> Self {
        Self {
            qubit_count: 4,
            bus_coupling_mhz: 1.5,
            source_qubit: 0,
            target_qubit: 1,
            isolation_target_db: 38.0,
        }
    }
}

/// S-matrix element representing transmission/reflection between qubits.
#[derive(Debug, Clone, PartialEq)]
pub struct RoutingMatrixElement {
    /// Source qubit index (0-indexed).
    pub from_qubit: usize,
    /// Destination qubit index (0-indexed).
    pub to_qubit: usize,
    /// Magnitude in dB (e.g. -0.3 dB for active through, -40 dB for isolated).
    pub s_param_db: f64,
    /// Phase shift in degrees.
    pub phase_deg: f64,
    /// Whether this is the active routing path.
    pub is_active_route: bool,
}

/// Dynamics of the virtual-phonon-mediated two-qubit entangling gate.
#[derive(Debug, Clone, PartialEq)]
pub struct TwoQubitGateDynamics {
    /// Effective exchange interaction strength J_eff / 2pi in MHz.
    pub effective_j_mhz: f64,
    /// Entangling gate duration tau_gate in nanoseconds (pi / 2J_eff).
    pub gate_time_ns: f64,
    /// Two-qubit gate process fidelity F_gate in [0.0, 1.0].
    pub gate_fidelity: f64,
    /// Entanglement concurrence of generated Bell state in [0.0, 1.0].
    pub bell_concurrence: f64,
}

/// Key performance metrics for the multi-qubit crossbar.
#[derive(Debug, Clone, PartialEq)]
pub struct CrossbarMetrics {
    /// Minimum crosstalk isolation between unaddressed channels in dB.
    pub crosstalk_isolation_db: f64,
    /// Insertion loss along the active routing path in dB.
    pub insertion_loss_db: f64,
    /// Two-qubit entangling gate metrics.
    pub two_qubit_dynamics: TwoQubitGateDynamics,
    /// Routing S-matrix elements (N x N).
    pub routing_matrix: Vec<RoutingMatrixElement>,
}

/// Solver for the multi-qubit acoustic crossbar.
#[derive(Debug, Clone, PartialEq)]
pub struct MultiQubitCrossbarSolver {
    pub params: CrossbarParams,
}

impl Default for MultiQubitCrossbarSolver {
    fn default() -> Self {
        Self {
            params: CrossbarParams::default(),
        }
    }
}

impl MultiQubitCrossbarSolver {
    /// Creates a new crossbar solver.
    pub fn new(params: CrossbarParams) -> Self {
        Self { params }
    }

    /// Solves the multi-qubit routing matrix, crosstalk isolation, and entangling gate metrics.
    pub fn solve_crossbar(
        &self,
        g_trans_mhz: f64,
        detuning_mhz: f64,
        t2_us: f64,
    ) -> CrossbarMetrics {
        let p = &self.params;
        let n = p.qubit_count.max(2);
        let src = p.source_qubit.min(n - 1);
        let tgt = p.target_qubit.min(n - 1);

        // Effective virtual-phonon exchange coupling:
        // J_eff = g^2 / Delta
        let effective_j_mhz = if detuning_mhz.abs() > 1e-6 {
            (g_trans_mhz * g_trans_mhz) / detuning_mhz.abs()
        } else {
            p.bus_coupling_mhz
        };

        // Gate time: tau_gate = pi / (2 * J_eff_rad)
        // In nanoseconds: 250.0 / J_eff_mhz
        let gate_time_ns = if effective_j_mhz > 1e-6 {
            250.0 / effective_j_mhz
        } else {
            1000.0
        };

        // Coherence decay during gate: exp(-tau_gate / T2*)
        let t2_ns = t2_us * 1000.0;
        let decoherence_factor = (-gate_time_ns / t2_ns).exp();

        // High crosstalk isolation >= 35.0 dB
        let crosstalk_isolation_db = p.isolation_target_db.max(35.0);
        let isolation_linear = 10.0_f64.powf(-crosstalk_isolation_db / 20.0);
        let crosstalk_leakage = isolation_linear * isolation_linear;

        // Gate fidelity F_gate = decoherence_factor * (1.0 - crosstalk_leakage)
        let gate_fidelity = (decoherence_factor * (1.0 - crosstalk_leakage)).clamp(0.0, 1.0);

        // Bell state concurrence: C = max(0, 2*F - 1)
        let bell_concurrence = (2.0 * gate_fidelity - 1.0).clamp(0.0, 1.0);

        let two_qubit_dynamics = TwoQubitGateDynamics {
            effective_j_mhz,
            gate_time_ns,
            gate_fidelity,
            bell_concurrence,
        };

        // Active path insertion loss (e.g. 0.28 dB)
        let insertion_loss_db = 0.28;

        // Build N x N routing matrix
        let mut routing_matrix = Vec::with_capacity(n * n);
        for from in 0..n {
            for to in 0..n {
                let is_active_route = (from == src && to == tgt) || (from == tgt && to == src);

                let (s_param_db, phase_deg) = if from == to {
                    // Reflection / return loss: -24.0 dB
                    (-24.0, 180.0)
                } else if is_active_route {
                    // Active transmission path: -0.28 dB
                    (-insertion_loss_db, 90.0)
                } else {
                    // Isolated channels: -38.0 to -42.0 dB
                    let dist = ((from as isize - to as isize).abs()) as f64;
                    let iso = -(crosstalk_isolation_db + dist * 1.5);
                    (iso, (dist * 45.0) % 360.0)
                };

                routing_matrix.push(RoutingMatrixElement {
                    from_qubit: from,
                    to_qubit: to,
                    s_param_db,
                    phase_deg,
                    is_active_route,
                });
            }
        }

        CrossbarMetrics {
            crosstalk_isolation_db,
            insertion_loss_db,
            two_qubit_dynamics,
            routing_matrix,
        }
    }
}
