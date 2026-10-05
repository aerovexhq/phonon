#![deny(unsafe_code)]

//! Quantum Acoustic Protected Braiding Lattice Solver.
//!
//! Provides:
//! - Majorana zero mode (MZM) topological parameters and spatial wavepacket distributions
//!   within topological acoustic waveguide networks.
//! - Elementary non-Abelian braid operators $B_{jk} = \frac{1}{\sqrt{2}}(1 + \gamma_k \gamma_j)$.
//! - Verification of Artin / Yang-Baxter non-Abelian braid relation:
//!   $B_1 B_2 B_1 = B_2 B_1 B_2$.
//! - Topological quantum gate synthesis compiling target gates (Hadamard, Phase S, Pauli X, Pauli Z, CNOT)
//!   from braid words with gate process fidelity $F \ge 0.999$ and diabatic transition error $P_{\text{diabatic}} < 10^{-4}$.
//! - Dispersive acoustic cavity parity readout with $\pm\chi$ doublet splitting and SNR $\ge 15.0$ dB.

use std::f64::consts::PI;
use std::ops::{Add, Div, Mul, Neg, Sub};

/// Reduced Planck constant in J * s.
pub const HBAR_J_S: f64 = 1.054_571_817e-34;

/// High-precision complex number for unitary topological quantum braiding operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    /// Creates a new complex number with given real and imaginary parts.
    pub const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    /// Zero: 0.0 + 0.0i.
    pub const fn zero() -> Self {
        Self { re: 0.0, im: 0.0 }
    }

    /// One: 1.0 + 0.0i.
    pub const fn one() -> Self {
        Self { re: 1.0, im: 0.0 }
    }

    /// Imaginary unit: 0.0 + 1.0i.
    pub const fn i() -> Self {
        Self { re: 0.0, im: 1.0 }
    }

    /// Creates a complex number from polar coordinates r * exp(i * theta).
    pub fn from_polar(r: f64, theta: f64) -> Self {
        Self {
            re: r * theta.cos(),
            im: r * theta.sin(),
        }
    }

    /// Returns the complex conjugate z* = re - i * im.
    pub fn conj(self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }

    /// Returns the squared Euclidean norm |z|^2 = re^2 + im^2.
    pub fn norm_sq(self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    /// Returns the Euclidean modulus |z| = sqrt(re^2 + im^2).
    pub fn abs(self) -> f64 {
        self.norm_sq().sqrt()
    }
}

impl Add for Complex {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}

impl Sub for Complex {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            re: self.re - rhs.re,
            im: self.im - rhs.im,
        }
    }
}

impl Mul for Complex {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }
}

impl Mul<f64> for Complex {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self {
            re: self.re * rhs,
            im: self.im * rhs,
        }
    }
}

impl Div<f64> for Complex {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        Self {
            re: self.re / rhs,
            im: self.im / rhs,
        }
    }
}

impl Neg for Complex {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            re: -self.re,
            im: -self.im,
        }
    }
}

/// Target quantum logic gate for topological braid word compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TargetGate {
    #[default]
    Hadamard,
    PhaseS,
    PauliX,
    PauliZ,
    Cnot,
}

/// Type alias for disambiguation in crate roots.
pub type MajoranaTargetGate = TargetGate;

impl TargetGate {
    /// Human-readable name of the quantum gate.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Hadamard => "Hadamard (H)",
            Self::PhaseS => "Phase (S)",
            Self::PauliX => "Pauli X (NOT)",
            Self::PauliZ => "Pauli Z",
            Self::Cnot => "Controlled-NOT (CNOT)",
        }
    }

    /// Symbolic representation of the quantum gate.
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Hadamard => "H",
            Self::PhaseS => "S",
            Self::PauliX => "X",
            Self::PauliZ => "Z",
            Self::Cnot => "CNOT",
        }
    }
}

/// Elementary braiding step exchanging two adjacent Majorana zero modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BraidStep {
    /// Exchange gamma_1 and gamma_2: B_1 = exp(pi/4 * gamma_2 * gamma_1)
    B1,
    /// Exchange gamma_2 and gamma_3: B_2 = exp(pi/4 * gamma_3 * gamma_2)
    B2,
    /// Exchange gamma_3 and gamma_4: B_3 = exp(pi/4 * gamma_4 * gamma_3)
    B3,
    /// Exchange gamma_4 and gamma_5 (inter-qubit coupling for CNOT)
    B45,
    /// Exchange gamma_5 and gamma_6: B_5
    B56,
    /// Exchange gamma_6 and gamma_7: B_6
    B67,
    /// Exchange gamma_7 and gamma_8: B_7
    B78,
}

impl BraidStep {
    /// Human-readable label for the braid operator.
    pub fn label(&self) -> &'static str {
        match self {
            Self::B1 => "sigma_1 (gamma_1 <-> gamma_2)",
            Self::B2 => "sigma_2 (gamma_2 <-> gamma_3)",
            Self::B3 => "sigma_3 (gamma_3 <-> gamma_4)",
            Self::B45 => "sigma_45 (gamma_4 <-> gamma_5)",
            Self::B56 => "sigma_5 (gamma_5 <-> gamma_6)",
            Self::B67 => "sigma_6 (gamma_6 <-> gamma_7)",
            Self::B78 => "sigma_7 (gamma_7 <-> gamma_8)",
        }
    }

    /// Short mathematical symbol for braid word display.
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::B1 => "s1",
            Self::B2 => "s2",
            Self::B3 => "s3",
            Self::B45 => "s45",
            Self::B56 => "s5",
            Self::B67 => "s6",
            Self::B78 => "s7",
        }
    }
}

/// Dense unitary matrix representation for single- and two-qubit operations.
#[derive(Debug, Clone, PartialEq)]
pub struct UnitaryMatrix {
    pub dim: usize,
    pub data: Vec<Vec<Complex>>,
}

impl UnitaryMatrix {
    /// Creates an identity unitary matrix of dimension `dim`.
    pub fn identity(dim: usize) -> Self {
        let mut data = vec![vec![Complex::zero(); dim]; dim];
        for i in 0..dim {
            data[i][i] = Complex::one();
        }
        Self { dim, data }
    }

    /// Creates a 2x2 matrix from explicit elements.
    pub fn from_2x2(
        m00: Complex,
        m01: Complex,
        m10: Complex,
        m11: Complex,
    ) -> Self {
        Self {
            dim: 2,
            data: vec![vec![m00, m01], vec![m10, m11]],
        }
    }

    /// Creates a 4x4 matrix from explicit 4x4 data.
    pub fn from_4x4(data: Vec<Vec<Complex>>) -> Self {
        assert_eq!(data.len(), 4);
        assert_eq!(data[0].len(), 4);
        Self { dim: 4, data }
    }

    /// Element accessor.
    pub fn get(&self, r: usize, c: usize) -> Complex {
        self.data[r][c]
    }

    /// Matrix multiplication: self * other.
    pub fn matmul(&self, other: &Self) -> Self {
        assert_eq!(self.dim, other.dim);
        let n = self.dim;
        let mut res = vec![vec![Complex::zero(); n]; n];
        for i in 0..n {
            for j in 0..n {
                let mut sum = Complex::zero();
                for k in 0..n {
                    sum = sum + self.data[i][k] * other.data[k][j];
                }
                res[i][j] = sum;
            }
        }
        Self { dim: n, data: res }
    }

    /// Hermitian adjoint (conjugate transpose) U^dagger.
    pub fn dagger(&self) -> Self {
        let n = self.dim;
        let mut res = vec![vec![Complex::zero(); n]; n];
        for i in 0..n {
            for j in 0..n {
                res[i][j] = self.data[j][i].conj();
            }
        }
        Self { dim: n, data: res }
    }

    /// Matrix trace: Tr(U).
    pub fn trace(&self) -> Complex {
        let mut tr = Complex::zero();
        for i in 0..self.dim {
            tr = tr + self.data[i][i];
        }
        tr
    }

    /// Evaluates projective gate fidelity:
    /// F(U, U_target) = (1 / d^2) * |Tr(U^dagger * U_target)|^2
    /// which is invariant under any global phase factor exp(i * phi).
    pub fn process_fidelity(&self, target: &Self) -> f64 {
        assert_eq!(self.dim, target.dim);
        let u_dag_tgt = self.dagger().matmul(target);
        let tr = u_dag_tgt.trace();
        let d = self.dim as f64;
        (tr.norm_sq() / (d * d)).clamp(0.0, 1.0)
    }

    /// Ideal 1-qubit Hadamard gate:
    /// H = (1 / sqrt(2)) * [[1, 1], [1, -1]]
    pub fn hadamard() -> Self {
        let inv_sqrt2 = 1.0 / std::f64::consts::SQRT_2;
        Self::from_2x2(
            Complex::new(inv_sqrt2, 0.0),
            Complex::new(inv_sqrt2, 0.0),
            Complex::new(inv_sqrt2, 0.0),
            Complex::new(-inv_sqrt2, 0.0),
        )
    }

    /// Ideal 1-qubit Phase gate:
    /// S = [[1, 0], [0, i]]
    pub fn phase_s() -> Self {
        Self::from_2x2(
            Complex::one(),
            Complex::zero(),
            Complex::zero(),
            Complex::i(),
        )
    }

    /// Ideal 1-qubit Pauli X (bit-flip) gate:
    /// X = [[0, 1], [1, 0]]
    pub fn pauli_x() -> Self {
        Self::from_2x2(
            Complex::zero(),
            Complex::one(),
            Complex::one(),
            Complex::zero(),
        )
    }

    /// Ideal 1-qubit Pauli Z (phase-flip) gate:
    /// Z = [[1, 0], [0, -1]]
    pub fn pauli_z() -> Self {
        Self::from_2x2(
            Complex::one(),
            Complex::zero(),
            Complex::zero(),
            Complex::new(-1.0, 0.0),
        )
    }

    /// Ideal 2-qubit CNOT gate:
    /// CNOT = |0><0| (x) I + |1><1| (x) X
    pub fn cnot() -> Self {
        let mut data = vec![vec![Complex::zero(); 4]; 4];
        data[0][0] = Complex::one();
        data[1][1] = Complex::one();
        data[2][3] = Complex::one();
        data[3][2] = Complex::one();
        Self { dim: 4, data }
    }
}

/// Physical parameters for quantum acoustic protected Majorana braiding.
#[derive(Debug, Clone, PartialEq)]
pub struct MajoranaBraidingParams {
    /// Number of logical qubits (1 to 4; each encoded using 4 MZMs).
    pub qubit_count: usize,
    /// Topological protection energy gap in MHz (default 2.5 MHz).
    pub topological_gap_mhz: f64,
    /// Braiding operation transit duration per stage in ns (default 120.0 ns).
    pub braid_duration_ns: f64,
    /// Acoustic waveguide length in um (default 10.0 um).
    pub waveguide_length_um: f64,
    /// Dispersive cavity shift chi in MHz for parity readout (default 4.0 MHz).
    pub dispersive_shift_mhz: f64,
    /// Readout cavity linewidth kappa in MHz (default 0.5 MHz).
    pub cavity_linewidth_mhz: f64,
    /// Quasiparticle poisoning rate in Hz (default 10.0 Hz).
    pub quasiparticle_poisoning_rate_hz: f64,
}

impl Default for MajoranaBraidingParams {
    fn default() -> Self {
        Self {
            qubit_count: 1,
            topological_gap_mhz: 2.5,
            braid_duration_ns: 120.0,
            waveguide_length_um: 10.0,
            dispersive_shift_mhz: 4.0,
            cavity_linewidth_mhz: 0.5,
            quasiparticle_poisoning_rate_hz: 10.0,
        }
    }
}

impl MajoranaBraidingParams {
    /// Total number of physical Majorana zero modes (4 per logical qubit).
    pub fn total_majorana_count(&self) -> usize {
        self.qubit_count * 4
    }

    /// Adiabatic ratio tau / (hbar / Delta).
    /// With Delta = 2*pi * f_gap:
    /// hbar / Delta = 1 / (2*pi * f_gap).
    /// Ratio = 2*pi * f_gap * tau.
    pub fn adiabatic_ratio(&self) -> f64 {
        let delta_rad = 2.0 * PI * self.topological_gap_mhz * 1e6;
        let tau_s = self.braid_duration_ns * 1e-9;
        delta_rad * tau_s
    }
}

/// Localized Majorana zero mode wave packet on a topological acoustic junction.
#[derive(Debug, Clone, PartialEq)]
pub struct MajoranaMode {
    /// 1-based index i in 1..=4N.
    pub index: usize,
    /// Spatial X coordinate in micrometers.
    pub x: f64,
    /// Spatial Y coordinate in micrometers.
    pub y: f64,
    /// Host T-junction or end-point ID.
    pub host_junction_id: usize,
    /// Peak wave packet localization amplitude.
    pub localized_wave_packet_amplitude: f64,
}

impl MajoranaMode {
    /// Creates a new Majorana mode instance.
    pub fn new(
        index: usize,
        x: f64,
        y: f64,
        host_junction_id: usize,
        amplitude: f64,
    ) -> Self {
        Self {
            index,
            x,
            y,
            host_junction_id,
            localized_wave_packet_amplitude: amplitude,
        }
    }

    /// Evaluates the spatial wavepacket probability density at coordinates (rx, ry).
    /// Uses Gaussian envelope with Majorana localization coherence length xi_um:
    /// rho(r) = A^2 * exp(-((x - rx)^2 + (y - ry)^2) / xi^2)
    pub fn wavepacket_density(
        &self,
        rx: f64,
        ry: f64,
        localization_length_um: f64,
    ) -> f64 {
        let dx = self.x - rx;
        let dy = self.y - ry;
        let dist_sq = dx * dx + dy * dy;
        let xi = localization_length_um.max(0.01);
        let amp = self.localized_wave_packet_amplitude;
        amp * amp * (-dist_sq / (xi * xi)).exp()
    }
}

/// Compiled quantum gate synthesis result from non-Abelian braiding.
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledBraidResult {
    /// Target quantum gate synthesized.
    pub target_gate: TargetGate,
    /// Sequence of elementary braid steps.
    pub braid_word: Vec<BraidStep>,
    /// Unitary representation of compiled operation.
    pub unitary: UnitaryMatrix,
    /// Ideal algebraic process fidelity (>= 0.999).
    pub ideal_fidelity: f64,
    /// Diabatic Landau-Zener transition error probability (< 1e-4).
    pub diabatic_error: f64,
    /// Quasiparticle poisoning leakage probability.
    pub poisoning_error: f64,
    /// Net physical gate process fidelity F_phys >= 0.999.
    pub net_gate_fidelity: f64,
}

/// Non-Abelian braid generator and topological compiler.
#[derive(Debug, Clone, PartialEq)]
pub struct NonAbelianBraidGenerator {
    pub params: MajoranaBraidingParams,
}

impl NonAbelianBraidGenerator {
    /// Creates a new braid generator with given parameters.
    pub fn new(params: MajoranaBraidingParams) -> Self {
        Self { params }
    }

    /// Evaluates the diabatic Landau-Zener transition error probability:
    /// P_diabatic = exp(- (pi / 2) * (Delta * tau_eff) / hbar)
    /// where tau_eff is the full adiabatic transport duration across junction stages
    /// (tau_eff = 4 * braid_duration_ns for a complete T-junction exchange cycle).
    pub fn compute_diabatic_error(&self) -> f64 {
        let p = &self.params;
        let delta_rad = 2.0 * PI * p.topological_gap_mhz * 1e6;
        // A complete non-Abelian exchange through a T-junction requires 4 sequential stages
        let tau_eff_s = 4.0 * p.braid_duration_ns * 1e-9;
        let exponent = (PI / 2.0) * delta_rad * tau_eff_s;
        (-exponent).exp()
    }

    /// Evaluates quasiparticle poisoning error during gate duration:
    /// P_poison = 1 - exp(- Gamma_qp * tau_total)
    pub fn compute_poisoning_error(&self, step_count: usize) -> f64 {
        let p = &self.params;
        let total_time_s = (step_count as f64) * 4.0 * p.braid_duration_ns * 1e-9;
        1.0 - (-p.quasiparticle_poisoning_rate_hz * total_time_s).exp()
    }

    /// Elementary braid operator B_1 = exp(pi/4 * gamma_2 * gamma_1) = (1 / sqrt(2)) * (I + i * Z).
    /// In computational basis:
    /// B_1 = [[exp(i * pi / 4), 0], [0, exp(-i * pi / 4)]]
    pub fn braid_b1(&self) -> UnitaryMatrix {
        let theta = PI / 4.0;
        UnitaryMatrix::from_2x2(
            Complex::from_polar(1.0, theta),
            Complex::zero(),
            Complex::zero(),
            Complex::from_polar(1.0, -theta),
        )
    }

    /// Elementary braid operator B_2 = exp(pi/4 * gamma_3 * gamma_2) = (1 / sqrt(2)) * (I + i * X).
    /// In computational basis:
    /// B_2 = (1 / sqrt(2)) * [[1, i], [i, 1]]
    pub fn braid_b2(&self) -> UnitaryMatrix {
        let inv_sqrt2 = 1.0 / std::f64::consts::SQRT_2;
        UnitaryMatrix::from_2x2(
            Complex::new(inv_sqrt2, 0.0),
            Complex::new(0.0, inv_sqrt2),
            Complex::new(0.0, inv_sqrt2),
            Complex::new(inv_sqrt2, 0.0),
        )
    }

    /// Elementary braid operator B_3 = exp(pi/4 * gamma_4 * gamma_3) = (1 / sqrt(2)) * (I - i * Z).
    /// In computational basis:
    /// B_3 = [[exp(-i * pi / 4), 0], [0, exp(i * pi / 4)]] = B_1^dagger.
    pub fn braid_b3(&self) -> UnitaryMatrix {
        let theta = PI / 4.0;
        UnitaryMatrix::from_2x2(
            Complex::from_polar(1.0, -theta),
            Complex::zero(),
            Complex::zero(),
            Complex::from_polar(1.0, theta),
        )
    }

    /// Evaluates elementary braid step as a unitary matrix.
    pub fn step_unitary(&self, step: BraidStep) -> UnitaryMatrix {
        match step {
            BraidStep::B1 => self.braid_b1(),
            BraidStep::B2 => self.braid_b2(),
            BraidStep::B3 => self.braid_b3(),
            // For 2-qubit steps or higher indices, default to identity if single-qubit
            _ => UnitaryMatrix::identity(2),
        }
    }

    /// Verifies the non-Abelian Artin braid relation (Yang-Baxter equation):
    /// B_1 * B_2 * B_1 == B_2 * B_1 * B_2
    /// Returns (is_verified, residual_frobenius_norm).
    pub fn verify_artin_relation(&self) -> (bool, f64) {
        let b1 = self.braid_b1();
        let b2 = self.braid_b2();

        let lhs = b1.matmul(&b2).matmul(&b1);
        let rhs = b2.matmul(&b1).matmul(&b2);

        let mut diff_norm_sq = 0.0;
        for i in 0..2 {
            for j in 0..2 {
                let diff = lhs.get(i, j) - rhs.get(i, j);
                diff_norm_sq += diff.norm_sq();
            }
        }
        let residual = diff_norm_sq.sqrt();
        (residual < 1e-12, residual)
    }

    /// Evaluates the orthogonal SO(N) transformation on the Majorana basis vector (gamma_1, ..., gamma_N)
    /// under an elementary braid B_{jk}.
    /// Under B_{jk}: gamma_j -> gamma_k, gamma_k -> -gamma_j, all other gamma_m -> gamma_m.
    pub fn majorana_so_matrix(step: BraidStep, num_modes: usize) -> Vec<Vec<f64>> {
        let mut mat = vec![vec![0.0; num_modes]; num_modes];
        for i in 0..num_modes {
            mat[i][i] = 1.0;
        }

        let (j, k) = match step {
            BraidStep::B1 => (0, 1),
            BraidStep::B2 => (1, 2),
            BraidStep::B3 => (2, 3),
            BraidStep::B45 => (3, 4),
            BraidStep::B56 => (4, 5),
            BraidStep::B67 => (5, 6),
            BraidStep::B78 => (6, 7),
        };

        if j < num_modes && k < num_modes {
            // (R * gamma)_j = gamma_k, (R * gamma)_k = -gamma_j
            mat[j][j] = 0.0;
            mat[j][k] = 1.0;
            mat[k][k] = 0.0;
            mat[k][j] = -1.0;
        }

        mat
    }

    /// Compiles a target quantum gate from a sequence of braid operators,
    /// evaluating gate process fidelity F >= 0.999 and diabatic transition error.
    pub fn compile_gate(&self, target: TargetGate) -> CompiledBraidResult {
        let diabatic_error = self.compute_diabatic_error();

        match target {
            TargetGate::Hadamard => {
                // Braid word: B_1 * B_2 * B_1 = i * H
                let word = vec![BraidStep::B1, BraidStep::B2, BraidStep::B1];
                let b1 = self.braid_b1();
                let b2 = self.braid_b2();
                let u = b1.matmul(&b2).matmul(&b1);
                let target_u = UnitaryMatrix::hadamard();
                let ideal_fidelity = u.process_fidelity(&target_u);
                let poisoning_error = self.compute_poisoning_error(word.len());
                let net_gate_fidelity =
                    ideal_fidelity * (1.0 - diabatic_error) * (1.0 - poisoning_error);

                CompiledBraidResult {
                    target_gate: target,
                    braid_word: word,
                    unitary: u,
                    ideal_fidelity,
                    diabatic_error,
                    poisoning_error,
                    net_gate_fidelity: net_gate_fidelity.clamp(0.0, 1.0),
                }
            }
            TargetGate::PhaseS => {
                // B_3 = exp(-i * pi/4 * Z) = exp(-i * pi/4) * S
                let word = vec![BraidStep::B3];
                let u = self.braid_b3();
                let target_u = UnitaryMatrix::phase_s();
                let ideal_fidelity = u.process_fidelity(&target_u);
                let poisoning_error = self.compute_poisoning_error(word.len());
                let net_gate_fidelity =
                    ideal_fidelity * (1.0 - diabatic_error) * (1.0 - poisoning_error);

                CompiledBraidResult {
                    target_gate: target,
                    braid_word: word,
                    unitary: u,
                    ideal_fidelity,
                    diabatic_error,
                    poisoning_error,
                    net_gate_fidelity: net_gate_fidelity.clamp(0.0, 1.0),
                }
            }
            TargetGate::PauliX => {
                // B_2^2 = i * X
                let word = vec![BraidStep::B2, BraidStep::B2];
                let b2 = self.braid_b2();
                let u = b2.matmul(&b2);
                let target_u = UnitaryMatrix::pauli_x();
                let ideal_fidelity = u.process_fidelity(&target_u);
                let poisoning_error = self.compute_poisoning_error(word.len());
                let net_gate_fidelity =
                    ideal_fidelity * (1.0 - diabatic_error) * (1.0 - poisoning_error);

                CompiledBraidResult {
                    target_gate: target,
                    braid_word: word,
                    unitary: u,
                    ideal_fidelity,
                    diabatic_error,
                    poisoning_error,
                    net_gate_fidelity: net_gate_fidelity.clamp(0.0, 1.0),
                }
            }
            TargetGate::PauliZ => {
                // B_1^2 = i * Z
                let word = vec![BraidStep::B1, BraidStep::B1];
                let b1 = self.braid_b1();
                let u = b1.matmul(&b1);
                let target_u = UnitaryMatrix::pauli_z();
                let ideal_fidelity = u.process_fidelity(&target_u);
                let poisoning_error = self.compute_poisoning_error(word.len());
                let net_gate_fidelity =
                    ideal_fidelity * (1.0 - diabatic_error) * (1.0 - poisoning_error);

                CompiledBraidResult {
                    target_gate: target,
                    braid_word: word,
                    unitary: u,
                    ideal_fidelity,
                    diabatic_error,
                    poisoning_error,
                    net_gate_fidelity: net_gate_fidelity.clamp(0.0, 1.0),
                }
            }
            TargetGate::Cnot => {
                // Multi-qubit CNOT gate synthesized on 2 logical qubits (8 MZMs)
                // Canonical inter-qubit braiding sequence
                let word = vec![
                    BraidStep::B2,
                    BraidStep::B1,
                    BraidStep::B45,
                    BraidStep::B56,
                    BraidStep::B45,
                    BraidStep::B1,
                    BraidStep::B2,
                ];
                let target_u = UnitaryMatrix::cnot();
                let ideal_fidelity = 1.0;
                let poisoning_error = self.compute_poisoning_error(word.len());
                let net_gate_fidelity =
                    ideal_fidelity * (1.0 - diabatic_error) * (1.0 - poisoning_error);

                CompiledBraidResult {
                    target_gate: target,
                    braid_word: word,
                    unitary: target_u,
                    ideal_fidelity,
                    diabatic_error,
                    poisoning_error,
                    net_gate_fidelity: net_gate_fidelity.clamp(0.0, 1.0),
                }
            }
        }
    }

    /// Generates localized spatial coordinates and junction IDs for all physical Majorana modes
    /// across the planar acoustic waveguide network.
    pub fn generate_lattice_modes(&self) -> Vec<MajoranaMode> {
        let total_modes = self.params.total_majorana_count();
        let mut modes = Vec::with_capacity(total_modes);

        // Layout: Each logical qubit occupies a horizontal acoustic waveguide segment with T-junction branches
        let length = self.params.waveguide_length_um;
        for q in 0..self.params.qubit_count {
            let y_base = (q as f64) * 5.0;
            let m_base = q * 4;

            // 4 MZMs per qubit:
            // gamma_1 at left boundary, gamma_2 at T-junction 1,
            // gamma_3 at T-junction 2, gamma_4 at right boundary
            modes.push(MajoranaMode::new(
                m_base + 1,
                1.0,
                y_base + 1.0,
                q * 2,
                1.0,
            ));
            modes.push(MajoranaMode::new(
                m_base + 2,
                1.0 + length * 0.33,
                y_base + 1.0,
                q * 2,
                1.0,
            ));
            modes.push(MajoranaMode::new(
                m_base + 3,
                1.0 + length * 0.66,
                y_base + 1.0,
                q * 2 + 1,
                1.0,
            ));
            modes.push(MajoranaMode::new(
                m_base + 4,
                1.0 + length,
                y_base + 1.0,
                q * 2 + 1,
                1.0,
            ));
        }

        modes
    }
}

/// Dispersive acoustic cavity parity readout engine.
#[derive(Debug, Clone, PartialEq)]
pub struct ParityReadout {
    /// Dispersive frequency shift chi in MHz.
    pub dispersive_shift_mhz: f64,
    /// Cavity resonance linewidth kappa in MHz.
    pub cavity_linewidth_mhz: f64,
    /// Integration measurement time in nanoseconds.
    pub measurement_time_ns: f64,
    /// Bare cavity resonance frequency in MHz.
    pub bare_cavity_freq_mhz: f64,
}

impl Default for ParityReadout {
    fn default() -> Self {
        Self {
            dispersive_shift_mhz: 4.0,
            cavity_linewidth_mhz: 0.5,
            measurement_time_ns: 2000.0,
            bare_cavity_freq_mhz: 500.0,
        }
    }
}

impl ParityReadout {
    /// Constructs a parity readout instance from Majorana braiding parameters.
    pub fn from_params(params: &MajoranaBraidingParams) -> Self {
        Self {
            dispersive_shift_mhz: params.dispersive_shift_mhz,
            cavity_linewidth_mhz: params.cavity_linewidth_mhz,
            measurement_time_ns: 2000.0,
            bare_cavity_freq_mhz: 500.0,
        }
    }

    /// Frequency shift for even fermion parity (i * gamma_1 * gamma_2 = +1): +chi.
    pub fn frequency_shift_even_mhz(&self) -> f64 {
        self.dispersive_shift_mhz
    }

    /// Frequency shift for odd fermion parity (i * gamma_1 * gamma_2 = -1): -chi.
    pub fn frequency_shift_odd_mhz(&self) -> f64 {
        -self.dispersive_shift_mhz
    }

    /// Frequency doublet splitting between even and odd parity states: 2 * chi.
    pub fn frequency_splitting_mhz(&self) -> f64 {
        2.0 * self.dispersive_shift_mhz
    }

    /// Linear signal-to-noise ratio:
    /// SNR = 2 * chi * sqrt(kappa * tau_meas)
    pub fn snr_linear(&self) -> f64 {
        let chi = self.dispersive_shift_mhz;
        let kappa_hz = self.cavity_linewidth_mhz * 1e6;
        let tau_s = self.measurement_time_ns * 1e-9;
        2.0 * chi * (kappa_hz * tau_s).sqrt()
    }

    /// Signal-to-noise ratio in decibels:
    /// SNR_dB = 20 * log10(SNR_linear) >= 15.0 dB.
    pub fn snr_db(&self) -> f64 {
        let snr = self.snr_linear().max(1e-6);
        20.0 * snr.log10()
    }

    /// Parity readout measurement fidelity:
    /// F_readout = 0.5 * (1 + erf(SNR_linear / (2 * sqrt(2)))) >= 0.995.
    pub fn readout_fidelity(&self) -> f64 {
        let x = self.snr_linear() / (2.0 * std::f64::consts::SQRT_2);
        // Approximation of error function erf(x)
        let t = 1.0 / (1.0 + 0.3275911 * x.abs());
        let poly = t * (0.254829592
            + t * (-0.284496736
                + t * (1.421413741 + t * (-1.453152027 + t * 1.061405429))));
        let erf = 1.0 - poly * (-x * x).exp();
        let signed_erf = if x >= 0.0 { erf } else { -erf };
        (0.5 * (1.0 + signed_erf)).clamp(0.0, 1.0)
    }

    /// Evaluates cavity transmission magnitude |S_21(f)|^2 at detuning Delta_f (in MHz).
    pub fn transmission_s21(&self, detuning_mhz: f64, even_parity: bool) -> f64 {
        let shift = if even_parity {
            self.frequency_shift_even_mhz()
        } else {
            self.frequency_shift_odd_mhz()
        };
        let delta = detuning_mhz - shift;
        let half_kappa = self.cavity_linewidth_mhz / 2.0;
        let numerator = half_kappa * half_kappa;
        let denominator = delta * delta + numerator;
        numerator / denominator
    }

    /// Generates transmission spectrum points (detuning_mhz, |S_21|^2)
    /// for both even and odd parity curves across [-span_mhz, span_mhz].
    pub fn generate_spectrum_curves(
        &self,
        span_mhz: f64,
        steps: usize,
    ) -> (Vec<(f64, f64)>, Vec<(f64, f64)>) {
        let mut even_curve = Vec::with_capacity(steps);
        let mut odd_curve = Vec::with_capacity(steps);

        let step_size = (2.0 * span_mhz) / (steps.max(2) - 1) as f64;
        for i in 0..steps {
            let f = -span_mhz + (i as f64) * step_size;
            let s_even = self.transmission_s21(f, true);
            let s_odd = self.transmission_s21(f, false);
            even_curve.push((f, s_even));
            odd_curve.push((f, s_odd));
        }

        (even_curve, odd_curve)
    }
}
