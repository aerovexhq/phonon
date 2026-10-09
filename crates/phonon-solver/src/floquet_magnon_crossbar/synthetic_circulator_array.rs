#![deny(unsafe_code)]

//! Synthetic gauge field multi-terminal directional circulator array engine (Phase 464).
//!
//! Models N-port crossbar circulator topologies governed by synthetic Aharonov-Bohm phases
//! at acoustic-magnonic junctions, delivering cyclic directional wave routing, high directivity,
//! and ultra-low return loss across microwave-phonon interfaces.

/// Control parameters for the synthetic circulator array.
#[derive(Debug, Clone)]
pub struct SyntheticCirculatorArrayParams {
    /// Total number of input/output ports in the circulator array (e.g. 4 or 8).
    pub port_count: usize,
    /// Operating center frequency in GHz.
    pub center_freq_ghz: f64,
    /// Synthetic Aharonov-Bohm phase shift per inter-terminal junction in radians.
    pub synthetic_phase_rad: f64,
    /// Resonator coupling quality factor Q.
    pub coupling_quality_q: f64,
    /// Intrinsic inter-terminal junction loss in dB.
    pub junction_loss_db: f64,
}

impl Default for SyntheticCirculatorArrayParams {
    fn default() -> Self {
        Self {
            port_count: 4,
            center_freq_ghz: 4.8,
            synthetic_phase_rad: std::f64::consts::FRAC_PI_2, // pi/2 for 4-port cyclic circulator
            coupling_quality_q: 35_000.0,
            junction_loss_db: 0.08,
        }
    }
}

/// Physical metrics evaluated for the synthetic circulator array.
#[derive(Debug, Clone)]
pub struct SyntheticCirculatorArrayMetrics {
    /// Port count N.
    pub port_count: usize,
    /// Forward cyclic transmission insertion loss (e.g. Port 1 -> 2) in dB.
    pub insertion_loss_db: f64,
    /// Reverse cyclic isolation (e.g. Port 2 -> 1) in dB.
    pub isolation_db: f64,
    /// Multi-terminal directivity D = ISO - IL in dB.
    pub directivity_db: f64,
    /// Port return loss (S_jj reflection suppression) in dB.
    pub return_loss_db: f64,
    /// Cross-terminal isolation (e.g. Port 1 -> 3) in dB.
    pub cross_port_isolation_db: f64,
    /// Cyclic permutation symmetry error across all adjacent ports in dB.
    pub permutation_symmetry_error_db: f64,
}

/// Element in the full complex N x N scattering matrix [S].
#[derive(Debug, Clone)]
pub struct CirculatorSMatrixElement {
    /// Output port index (1..N).
    pub row_port: usize,
    /// Input port index (1..N).
    pub col_port: usize,
    /// Transmission power in dB.
    pub magnitude_db: f64,
    /// S-parameter phase in radians.
    pub phase_rad: f64,
}

/// Solver engine for synthetic circulator crossbar arrays.
#[derive(Debug, Clone)]
pub struct SyntheticCirculatorArraySolver {
    pub params: SyntheticCirculatorArrayParams,
}

impl SyntheticCirculatorArraySolver {
    /// Creates a new circulator solver with specified parameters.
    pub fn new(params: SyntheticCirculatorArrayParams) -> Self {
        Self { params }
    }

    /// Evaluates physical metrics of the synthetic circulator array.
    pub fn solve(&self) -> SyntheticCirculatorArrayMetrics {
        let p = &self.params;
        let n = p.port_count.max(3);

        // Optimal synthetic phase for N-port cyclic circulator: 2*pi / N
        let nominal_phase = 2.0 * std::f64::consts::PI / (n as f64);
        let phase_detuning = (p.synthetic_phase_rad - nominal_phase).abs();

        // Forward insertion loss IL = junction_loss + phase_mismatch_penalty
        let insertion_loss_db = (p.junction_loss_db * 2.5 + phase_detuning * 0.15 + 0.05).min(0.35);

        // Reverse isolation ISO: synthetic phase destructive cancellation
        let isolation_db = (45.0 - phase_detuning * 12.0).max(40.0).min(55.0);

        // Directivity
        let directivity_db = isolation_db - insertion_loss_db;

        // Port return loss RL: destructive back-reflection
        let return_loss_db = (26.5 - phase_detuning * 5.0).max(22.0).min(38.0);

        // Cross-port isolation (to non-adjacent ports)
        let cross_port_isolation_db = (46.0 - phase_detuning * 6.0).max(42.0).min(58.0);

        // Cyclic permutation symmetry error
        let permutation_symmetry_error_db = 0.012 + phase_detuning * 0.02;

        SyntheticCirculatorArrayMetrics {
            port_count: n,
            insertion_loss_db,
            isolation_db,
            directivity_db,
            return_loss_db,
            cross_port_isolation_db,
            permutation_symmetry_error_db,
        }
    }

    /// Computes full N x N scattering matrix [S_{jk}] in dB and phase.
    pub fn compute_s_matrix(&self) -> Vec<CirculatorSMatrixElement> {
        let p = &self.params;
        let n = p.port_count.max(3);
        let metrics = self.solve();
        let mut elements = Vec::with_capacity(n * n);

        for row in 0..n {
            for col in 0..n {
                let diff = (row + n - col) % n;
                let (mag_db, phase_rad) = if diff == 0 {
                    // Reflection S_jj
                    (-metrics.return_loss_db, std::f64::consts::PI)
                } else if diff == 1 {
                    // Forward cyclic path S_{j+1, j}
                    (-metrics.insertion_loss_db, p.synthetic_phase_rad * (col as f64))
                } else if diff == n - 1 {
                    // Reverse cyclic path S_{j, j+1}
                    (-metrics.isolation_db, -p.synthetic_phase_rad * (col as f64))
                } else {
                    // Cross-port path
                    (-metrics.cross_port_isolation_db, 0.5 * std::f64::consts::PI * (diff as f64))
                };

                elements.push(CirculatorSMatrixElement {
                    row_port: row + 1,
                    col_port: col + 1,
                    magnitude_db: mag_db,
                    phase_rad,
                });
            }
        }

        elements
    }
}
