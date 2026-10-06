#![deny(unsafe_code)]

//! Planar Topological Acoustic Waveguide Braiding Crossbar & Majorana Zero Mode Engine.
//!
//! Models non-Abelian Majorana zero modes (MZMs) localized at domain boundaries
//! in a 2D planar acoustic piezoelectric metamaterial crossbar network.
//! Simulates elementary braiding operators, verifies Artin non-Abelian braid relations,
//! compiles fault-tolerant single-qubit Clifford gates, and computes Landau-Zener
//! diabatic transition errors and gate fidelities.

use std::f64::consts::PI;
use std::ops::{Add, Div, Mul, Neg, Sub};

/// Reduced Planck constant in J * s.
pub const HBAR_J_S: f64 = 1.054_571_817e-34;

/// Boltzmann constant in J / K.
pub const BOLTZMANN_J_PER_K: f64 = 1.380_649e-23;

/// Elementary charge in Coulombs.
pub const ELEMENTARY_CHARGE_C: f64 = 1.602_176_634e-19;

/// High-precision complex number for unitary quantum operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BraidComplex {
    pub re: f64,
    pub im: f64,
}

impl BraidComplex {
    /// Creates a new complex number.
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

    /// Constructs a complex number from polar coordinates r * exp(i * theta).
    pub fn from_polar(r: f64, theta: f64) -> Self {
        Self {
            re: r * theta.cos(),
            im: r * theta.sin(),
        }
    }

    /// Complex conjugate.
    pub fn conj(self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }

    /// Squared Euclidean norm |z|^2.
    pub fn norm_sq(self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    /// Euclidean norm |z|.
    pub fn norm(self) -> f64 {
        self.norm_sq().sqrt()
    }
}

impl Add for BraidComplex {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}

impl Sub for BraidComplex {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            re: self.re - rhs.re,
            im: self.im - rhs.im,
        }
    }
}

impl Mul for BraidComplex {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }
}

impl Mul<f64> for BraidComplex {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self {
            re: self.re * rhs,
            im: self.im * rhs,
        }
    }
}

impl Div<f64> for BraidComplex {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        Self {
            re: self.re / rhs,
            im: self.im / rhs,
        }
    }
}

impl Div for BraidComplex {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        let d = rhs.norm_sq();
        Self {
            re: (self.re * rhs.re + self.im * rhs.im) / d,
            im: (self.im * rhs.re - self.re * rhs.im) / d,
        }
    }
}

impl Neg for BraidComplex {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            re: -self.re,
            im: -self.im,
        }
    }
}

/// 2x2 complex matrix for logical single-qubit Clifford operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat2x2 {
    pub m: [[BraidComplex; 2]; 2],
}

impl Mat2x2 {
    /// Identity 2x2 matrix.
    pub fn identity() -> Self {
        Self {
            m: [
                [BraidComplex::one(), BraidComplex::zero()],
                [BraidComplex::zero(), BraidComplex::one()],
            ],
        }
    }

    /// Matrix multiplication C = A * B.
    pub fn mul(self, rhs: Self) -> Self {
        let mut res = [[BraidComplex::zero(); 2]; 2];
        for r in 0..2 {
            for c in 0..2 {
                let mut sum = BraidComplex::zero();
                for k in 0..2 {
                    sum = sum + self.m[r][k] * rhs.m[k][c];
                }
                res[r][c] = sum;
            }
        }
        Self { m: res }
    }

    /// Conjugate transpose (Hermitian adjoint).
    pub fn dagger(self) -> Self {
        Self {
            m: [
                [self.m[0][0].conj(), self.m[1][0].conj()],
                [self.m[0][1].conj(), self.m[1][1].conj()],
            ],
        }
    }

    /// Trace of the matrix.
    pub fn trace(self) -> BraidComplex {
        self.m[0][0] + self.m[1][1]
    }

    /// Frobenius difference norm ||A - B||_F.
    pub fn diff_norm(self, rhs: Self) -> f64 {
        let mut sum_sq = 0.0;
        for r in 0..2 {
            for c in 0..2 {
                sum_sq += (self.m[r][c] - rhs.m[r][c]).norm_sq();
            }
        }
        sum_sq.sqrt()
    }
}

/// Physical parameters for the acoustic Majorana braiding crossbar.
#[derive(Debug, Clone, PartialEq)]
pub struct MajoranaBraidingCrossbarParams {
    /// Number of logical qubits (1 to 4, with 4 MZMs per qubit).
    pub qubit_count: usize,
    /// Topological superconducting bulk gap in MHz.
    pub topological_gap_mhz: f64,
    /// Adiabatic braid traversal duration in nanoseconds.
    pub braid_duration_ns: f64,
    /// Waveguide crossbar segment length in micrometers.
    pub waveguide_length_um: f64,
    /// Quasiparticle poisoning rate in Hz.
    pub quasiparticle_poisoning_rate_hz: f64,
    /// Operating cryogenic temperature in milliKelvin.
    pub temperature_mk: f64,
}

impl Default for MajoranaBraidingCrossbarParams {
    fn default() -> Self {
        Self {
            qubit_count: 1,
            topological_gap_mhz: 4.5,
            braid_duration_ns: 120.0,
            waveguide_length_um: 12.0,
            quasiparticle_poisoning_rate_hz: 8.0,
            temperature_mk: 20.0,
        }
    }
}

/// A localized Majorana Zero Mode (MZM) on the acoustic crossbar.
#[derive(Debug, Clone, PartialEq)]
pub struct MajoranaZeroMode {
    /// Zero-based identifier (0 .. 2N - 1).
    pub id: usize,
    /// Label (e.g., "gamma_1").
    pub label: String,
    /// Planar crossbar position X in micrometers.
    pub x: f64,
    /// Planar crossbar position Y in micrometers.
    pub y: f64,
    /// Home arm index (0: West, 1: North, 2: East, 3: South, etc.).
    pub arm_id: usize,
    /// Paired Majorana index forming a Dirac fermion mode.
    pub parity_partner: usize,
}

/// Target single-qubit Clifford gate compiled from Majorana braiding sequences.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetCliffordGate {
    Identity,
    PhaseS,
    Hadamard,
    PauliX,
    PauliZ,
}

/// Compilation result for a target Clifford gate.
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledBraidGate {
    /// Target gate requested.
    pub target: TargetCliffordGate,
    /// Braid word sequence (e.g. vec![1, 2, 1] for B_1 B_2 B_1).
    pub braid_sequence: Vec<usize>,
    /// Unitary 2x2 matrix implemented by the braid word.
    pub unitary: Mat2x2,
    /// Target ideal 2x2 unitary matrix.
    pub ideal_unitary: Mat2x2,
    /// Gate process fidelity F = |Tr(U_ideal^\dagger U_braid)|^2 / 4.
    pub process_fidelity: f64,
    /// Adiabatic transition diabatic error probability.
    pub diabatic_error: f64,
}

/// Planar waveguide crossbar network geometry.
#[derive(Debug, Clone, PartialEq)]
pub struct CrossbarGeometry {
    /// Segments connecting junctions: (x1, y1) to (x2, y2).
    pub segments: Vec<((f64, f64), (f64, f64))>,
    /// Junction center points (e.g., T-junction, cross-junction).
    pub junctions: Vec<(f64, f64)>,
}

/// Trajectory step during a visual/simulated braid.
#[derive(Debug, Clone, PartialEq)]
pub struct BraidTrajectoryStep {
    /// Step index (0 .. total_steps - 1).
    pub step_index: usize,
    /// Normalized progress in [0.0, 1.0].
    pub progress: f64,
    /// Current positions of all MZMs at this step.
    pub mzm_positions: Vec<(f64, f64)>,
    /// Active braiding pair (e.g., (1, 2)).
    pub active_pair: (usize, usize),
    /// Description of the physical shuttling motion.
    pub description: String,
}

/// Master solver for the Majorana Braiding Crossbar.
#[derive(Debug, Clone, PartialEq)]
pub struct MajoranaBraidingCrossbar {
    pub params: MajoranaBraidingCrossbarParams,
    pub modes: Vec<MajoranaZeroMode>,
    pub geometry: CrossbarGeometry,
}

impl MajoranaBraidingCrossbar {
    /// Constructs a new crossbar with default or custom parameters.
    pub fn new(params: MajoranaBraidingCrossbarParams) -> Self {
        let total_mzms = params.qubit_count * 4;
        let l = params.waveguide_length_um;

        let mut modes = Vec::with_capacity(total_mzms);
        let mut segments = Vec::new();
        let mut junctions = Vec::new();

        // Standard T-junction crossbar layout for qubit 0 (modes gamma_1 .. gamma_4)
        // Center junction at (0.0, 0.0)
        junctions.push((0.0, 0.0));
        segments.push(((-l, 0.0), (l, 0.0))); // West - East trunk
        segments.push(((0.0, 0.0), (0.0, l))); // North tuning stub
        segments.push(((0.0, 0.0), (0.0, -l))); // South ground stub

        // Place initial modes along the arms
        let initial_coords = [
            (-l * 0.85, 0.0, 0, 1), // gamma_1: West arm, paired with gamma_2
            (0.0, l * 0.85, 1, 0),  // gamma_2: North arm, paired with gamma_1
            (l * 0.85, 0.0, 2, 3),  // gamma_3: East arm, paired with gamma_4
            (0.0, -l * 0.85, 3, 2), // gamma_4: South arm, paired with gamma_3
        ];

        for i in 0..total_mzms {
            let base_idx = i % 4;
            let qubit_offset_x = (i / 4) as f64 * (2.5 * l);
            let (x, y, arm, partner_offset) = initial_coords[base_idx];
            modes.push(MajoranaZeroMode {
                id: i,
                label: format!("gamma_{}", i + 1),
                x: x + qubit_offset_x,
                y,
                arm_id: arm,
                parity_partner: (i / 4) * 4 + partner_offset,
            });
        }

        Self {
            params,
            modes,
            geometry: CrossbarGeometry {
                segments,
                junctions,
            },
        }
    }

    /// Evaluates the Landau-Zener diabatic transition error probability:
    /// P_diabatic = exp(-pi * Delta_topo * tau_braid / (2 * hbar)).
    pub fn compute_diabatic_error(&self) -> f64 {
        // Delta in Joules = h * delta_hz = 2 * pi * hbar * (topological_gap_mhz * 1e6)
        // Delta / hbar = 2 * pi * (topological_gap_mhz * 1e6)
        let omega_topo = 2.0 * PI * self.params.topological_gap_mhz * 1.0e6;
        let tau_s = self.params.braid_duration_ns * 1.0e-9;

        // Effective multi-turn adiabatic exponent including crossbar geometry factor ~1.8
        let exponent = (PI / 2.0) * omega_topo * tau_s * 1.8;
        (-exponent).exp().clamp(1.0e-15, 1.0)
    }

    /// Returns the 2x2 unitary matrix in the logical qubit basis {|0>_L, |1>_L}
    /// for elementary braid B_k exchanging gamma_k and gamma_{k+1}.
    ///
    /// In the 4-MZM single-qubit representation:
    /// - c = (gamma_1 + i * gamma_2) / 2
    /// - d = (gamma_3 + i * gamma_4) / 2
    /// - |0>_L has even parity (c^\dagger c = 0, d^\dagger d = 0).
    /// - |1>_L has even parity (c^\dagger c = 1, d^\dagger d = 1).
    ///
    /// - B_1 exchanges gamma_1, gamma_2:
    ///   B_1 = exp(pi/4 * gamma_2 * gamma_1) = diag(e^{-i*pi/4}, e^{i*pi/4}) (up to global phase = Phase gate S).
    /// - B_2 exchanges gamma_2, gamma_3:
    ///   mixes c and d fermions, implementing a beam-splitter / Hadamard operation.
    /// - B_3 exchanges gamma_3, gamma_4:
    ///   acts on the second pair with phase rotation.
    pub fn elementary_braid_matrix(&self, k: usize) -> Mat2x2 {
        let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
        match k {
            1 => {
                // Phase rotation S = diag(1, i) up to global phase e^{-i*pi/4}
                let e_minus_ipi4 = BraidComplex::new(inv_sqrt2, -inv_sqrt2);
                let e_plus_ipi4 = BraidComplex::new(inv_sqrt2, inv_sqrt2);
                Mat2x2 {
                    m: [
                        [e_minus_ipi4, BraidComplex::zero()],
                        [BraidComplex::zero(), e_plus_ipi4],
                    ],
                }
            }
            2 => {
                // Non-Abelian mixing braid between gamma_2 and gamma_3
                // U_2 = 1/sqrt(2) * [[1, -i], [-i, 1]]
                let c_one = BraidComplex::new(inv_sqrt2, 0.0);
                let c_minus_i = BraidComplex::new(0.0, -inv_sqrt2);
                Mat2x2 {
                    m: [[c_one, c_minus_i], [c_minus_i, c_one]],
                }
            }
            3 => {
                // B_3 = diag(e^{-i*pi/4}, e^{i*pi/4})
                let e_minus_ipi4 = BraidComplex::new(inv_sqrt2, -inv_sqrt2);
                let e_plus_ipi4 = BraidComplex::new(inv_sqrt2, inv_sqrt2);
                Mat2x2 {
                    m: [
                        [e_minus_ipi4, BraidComplex::zero()],
                        [BraidComplex::zero(), e_plus_ipi4],
                    ],
                }
            }
            _ => Mat2x2::identity(),
        }
    }

    /// Verifies the Artin non-Abelian braid relation:
    /// B_1 * B_2 * B_1 == B_2 * B_1 * B_2.
    /// Returns the Frobenius difference norm ||B_1 B_2 B_1 - B_2 B_1 B_2||_F.
    pub fn verify_artin_braid_relation(&self) -> (bool, f64) {
        let b1 = self.elementary_braid_matrix(1);
        let b2 = self.elementary_braid_matrix(2);

        let lhs = b1.mul(b2).mul(b1);
        let rhs = b2.mul(b1).mul(b2);

        let diff = lhs.diff_norm(rhs);
        (diff < 1.0e-10, diff)
    }

    /// Verifies distant commutativity: B_1 * B_3 == B_3 * B_1 for |1 - 3| >= 2.
    pub fn verify_distant_commutativity(&self) -> (bool, f64) {
        let b1 = self.elementary_braid_matrix(1);
        let b3 = self.elementary_braid_matrix(3);

        let lhs = b1.mul(b3);
        let rhs = b3.mul(b1);

        let diff = lhs.diff_norm(rhs);
        (diff < 1.0e-10, diff)
    }

    /// Compiles a target single-qubit Clifford gate into a minimal braid sequence.
    pub fn compile_clifford_gate(&self, target: TargetCliffordGate) -> CompiledBraidGate {
        let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
        let (sequence, ideal) = match target {
            TargetCliffordGate::Identity => (
                vec![],
                Mat2x2::identity(),
            ),
            TargetCliffordGate::PhaseS => (
                vec![1],
                Mat2x2 {
                    m: [
                        [BraidComplex::one(), BraidComplex::zero()],
                        [BraidComplex::zero(), BraidComplex::i()],
                    ],
                },
            ),
            TargetCliffordGate::Hadamard => (
                vec![1, 2, 1], // B_1 * B_2 * B_1 = Hadamard (up to global phase)
                Mat2x2 {
                    m: [
                        [BraidComplex::new(inv_sqrt2, 0.0), BraidComplex::new(inv_sqrt2, 0.0)],
                        [BraidComplex::new(inv_sqrt2, 0.0), BraidComplex::new(-inv_sqrt2, 0.0)],
                    ],
                },
            ),
            TargetCliffordGate::PauliX => (
                vec![2, 2], // Two successive B_2 braids implement bit-flip X
                Mat2x2 {
                    m: [
                        [BraidComplex::zero(), BraidComplex::one()],
                        [BraidComplex::one(), BraidComplex::zero()],
                    ],
                },
            ),
            TargetCliffordGate::PauliZ => (
                vec![1, 1], // Two successive B_1 braids implement phase-flip Z (S^2 = Z)
                Mat2x2 {
                    m: [
                        [BraidComplex::one(), BraidComplex::zero()],
                        [BraidComplex::zero(), BraidComplex::new(-1.0, 0.0)],
                    ],
                },
            ),
        };

        // Evaluate composite braid unitary
        let mut u_braid = Mat2x2::identity();
        for &k in &sequence {
            u_braid = u_braid.mul(self.elementary_braid_matrix(k));
        }

        // Process fidelity F = |Tr(U_ideal^\dagger * U_braid)|^2 / 4 (accounting for global phase)
        let prod = ideal.dagger().mul(u_braid);
        let tr = prod.trace();
        let fidelity = (tr.norm() / 2.0).powi(2).clamp(0.0, 1.0);

        let diabatic = self.compute_diabatic_error();

        CompiledBraidGate {
            target,
            braid_sequence: sequence,
            unitary: u_braid,
            ideal_unitary: ideal,
            process_fidelity: (fidelity * (1.0 - diabatic)).clamp(0.0, 1.0),
            diabatic_error: diabatic,
        }
    }

    /// Generates multi-step trajectory coordinates for a visual adiabatic braid exchange
    /// between gamma_1 and gamma_2 via the T-junction center.
    pub fn generate_braid_trajectory(&self, steps_count: usize) -> Vec<BraidTrajectoryStep> {
        let n = steps_count.max(8);
        let mut steps = Vec::with_capacity(n);
        let l = self.params.waveguide_length_um;

        let start_pos_1 = (-l * 0.85, 0.0);
        let start_pos_2 = (0.0, l * 0.85);
        let stub_pos = (0.0, -l * 0.85); // South stub used for non-colliding passing

        for step in 0..n {
            let p = step as f64 / (n - 1) as f64;
            let (pos1, pos2, desc) = if p < 0.333 {
                // Phase 1: Mode 1 moves from West arm into South stub (clearing trunk)
                let sub = p / 0.333;
                let x1 = start_pos_1.0 * (1.0 - sub);
                let y1 = stub_pos.1 * sub;
                (
                    (x1, y1),
                    start_pos_2,
                    format!("Phase 1: Shuttle gamma_1 into South stub ({:.0}%)", sub * 100.0),
                )
            } else if p < 0.666 {
                // Phase 2: Mode 2 moves from North arm through center into West arm
                let sub = (p - 0.333) / 0.333;
                let x2 = start_pos_1.0 * sub;
                let y2 = start_pos_2.1 * (1.0 - sub);
                (
                    (0.0, stub_pos.1),
                    (x2, y2),
                    format!("Phase 2: Shuttle gamma_2 to West arm ({:.0}%)", sub * 100.0),
                )
            } else {
                // Phase 3: Mode 1 moves from South stub through center to North arm
                let sub = (p - 0.666) / 0.334;
                let x1 = 0.0;
                let y1 = stub_pos.1 * (1.0 - sub) + start_pos_2.1 * sub;
                (
                    (x1, y1),
                    (start_pos_1.0, 0.0),
                    format!("Phase 3: Shuttle gamma_1 to North arm ({:.0}%)", sub * 100.0),
                )
            };

            let mut all_positions = Vec::with_capacity(self.modes.len());
            for m in &self.modes {
                if m.id == 0 {
                    all_positions.push(pos1);
                } else if m.id == 1 {
                    all_positions.push(pos2);
                } else {
                    all_positions.push((m.x, m.y));
                }
            }

            steps.push(BraidTrajectoryStep {
                step_index: step,
                progress: p,
                mzm_positions: all_positions,
                active_pair: (0, 1),
                description: desc,
            });
        }

        steps
    }
}
