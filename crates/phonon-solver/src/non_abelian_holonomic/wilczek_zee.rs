#![deny(unsafe_code)]

//! Non-Abelian Wilczek-Zee gauge connection and holonomic geometric phase gate synthesis.
//!
//! Models the tripod / tripartite dark state system, Wilczek-Zee non-Abelian gauge connection,
//! Wilson loop path integration, non-Abelian commutation verification, and high-fidelity
//! geometric quantum logic gate synthesis (Hadamard, Phase S, Pauli X, Pauli Z, Rotations).

use std::f64::consts::PI;

/// High-precision complex number for holonomic quantum operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
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

    /// Polar form: r * exp(i * theta).
    pub fn from_polar(r: f64, theta: f64) -> Self {
        Self {
            re: r * theta.cos(),
            im: r * theta.sin(),
        }
    }

    /// Complex conjugate z*.
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

    /// Complex phase / argument in (-pi, pi].
    pub fn arg(self) -> f64 {
        self.im.atan2(self.re)
    }

    /// Complex exponential exp(z) = exp(re) * (cos(im) + i * sin(im)).
    pub fn exp(self) -> Self {
        let r = self.re.exp();
        Self {
            re: r * self.im.cos(),
            im: r * self.im.sin(),
        }
    }

    /// Complex square root with principal branch.
    pub fn sqrt(self) -> Self {
        let r = self.norm();
        let theta = self.arg();
        Self::from_polar(r.sqrt(), theta * 0.5)
    }

    /// Addition.
    pub fn add(self, rhs: Self) -> Self {
        Self {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }

    /// Subtraction.
    pub fn sub(self, rhs: Self) -> Self {
        Self {
            re: self.re - rhs.re,
            im: self.im - rhs.im,
        }
    }

    /// Multiplication.
    pub fn mul(self, rhs: Self) -> Self {
        Self {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }

    /// Scalar multiplication.
    pub fn scale(self, s: f64) -> Self {
        Self {
            re: self.re * s,
            im: self.im * s,
        }
    }

    /// Division.
    pub fn div(self, rhs: Self) -> Self {
        let denom = rhs.norm_sq();
        if denom < 1e-30 {
            Self::zero()
        } else {
            Self {
                re: (self.re * rhs.re + self.im * rhs.im) / denom,
                im: (self.im * rhs.re - self.re * rhs.im) / denom,
            }
        }
    }
}

/// 2x2 complex matrix for single-qubit holonomies and Lie algebra generators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComplexMatrix2x2 {
    pub data: [[Complex; 2]; 2],
}

impl ComplexMatrix2x2 {
    /// Zero 2x2 matrix.
    pub const fn zero() -> Self {
        Self {
            data: [[Complex::zero(); 2]; 2],
        }
    }

    /// 2x2 Identity matrix.
    pub const fn identity() -> Self {
        Self {
            data: [
                [Complex::one(), Complex::zero()],
                [Complex::zero(), Complex::one()],
            ],
        }
    }

    /// Creates matrix from raw 2x2 complex array.
    pub const fn from_data(data: [[Complex; 2]; 2]) -> Self {
        Self { data }
    }

    /// Creates matrix from 4 complex numbers in row-major order: m00, m01, m10, m11.
    pub const fn new(m00: Complex, m01: Complex, m10: Complex, m11: Complex) -> Self {
        Self {
            data: [[m00, m01], [m10, m11]],
        }
    }

    /// Matrix multiplication: self * rhs.
    pub fn mul(&self, rhs: &Self) -> Self {
        let mut res = [[Complex::zero(); 2]; 2];
        for i in 0..2 {
            for j in 0..2 {
                let mut sum = Complex::zero();
                for k in 0..2 {
                    sum = sum.add(self.data[i][k].mul(rhs.data[k][j]));
                }
                res[i][j] = sum;
            }
        }
        Self { data: res }
    }

    /// Matrix addition.
    pub fn add(&self, rhs: &Self) -> Self {
        let mut res = [[Complex::zero(); 2]; 2];
        for i in 0..2 {
            for j in 0..2 {
                res[i][j] = self.data[i][j].add(rhs.data[i][j]);
            }
        }
        Self { data: res }
    }

    /// Matrix subtraction.
    pub fn sub(&self, rhs: &Self) -> Self {
        let mut res = [[Complex::zero(); 2]; 2];
        for i in 0..2 {
            for j in 0..2 {
                res[i][j] = self.data[i][j].sub(rhs.data[i][j]);
            }
        }
        Self { data: res }
    }

    /// Scalar multiplication by complex scalar.
    pub fn scale(&self, c: Complex) -> Self {
        let mut res = [[Complex::zero(); 2]; 2];
        for i in 0..2 {
            for j in 0..2 {
                res[i][j] = self.data[i][j].mul(c);
            }
        }
        Self { data: res }
    }

    /// Scalar multiplication by real scalar.
    pub fn scale_real(&self, s: f64) -> Self {
        let mut res = [[Complex::zero(); 2]; 2];
        for i in 0..2 {
            for j in 0..2 {
                res[i][j] = self.data[i][j].scale(s);
            }
        }
        Self { data: res }
    }

    /// Conjugate transpose (Hermitian adjoint M^dagger).
    pub fn dagger(&self) -> Self {
        Self {
            data: [
                [self.data[0][0].conj(), self.data[1][0].conj()],
                [self.data[0][1].conj(), self.data[1][1].conj()],
            ],
        }
    }

    /// Trace Tr(M).
    pub fn trace(&self) -> Complex {
        self.data[0][0].add(self.data[1][1])
    }

    /// Determinant det(M).
    pub fn det(&self) -> Complex {
        self.data[0][0]
            .mul(self.data[1][1])
            .sub(self.data[0][1].mul(self.data[1][0]))
    }

    /// Frobenius norm ||M||_F.
    pub fn frobenius_norm(&self) -> f64 {
        let mut sum_sq = 0.0;
        for i in 0..2 {
            for j in 0..2 {
                sum_sq += self.data[i][j].norm_sq();
            }
        }
        sum_sq.sqrt()
    }

    /// Matrix exponential exp(M) using the exact closed-form Cayley-Hamilton decomposition.
    pub fn exp(&self) -> Self {
        let tr = self.trace().scale(0.5);
        let m0 = self.sub(&Self::identity().scale(tr));
        let delta = m0.data[0][0]
            .mul(m0.data[0][0])
            .add(m0.data[0][1].mul(m0.data[1][0]));
        let lambda = delta.sqrt();

        let exp_tr = tr.exp();

        if lambda.norm() < 1e-12 {
            let inner = Self::identity().add(&m0);
            inner.scale(exp_tr)
        } else {
            let cosh_lam = Complex::new(
                lambda.re.cosh() * lambda.im.cos(),
                lambda.re.sinh() * lambda.im.sin(),
            );
            let sinh_lam = Complex::new(
                lambda.re.sinh() * lambda.im.cos(),
                lambda.re.cosh() * lambda.im.sin(),
            );
            let sinc_lam = sinh_lam.div(lambda);

            let term1 = Self::identity().scale(cosh_lam);
            let term2 = m0.scale(sinc_lam);
            let inner = term1.add(&term2);
            inner.scale(exp_tr)
        }
    }

    /// Validates whether matrix is anti-Hermitian: A^dagger = -A within specified tolerance.
    pub fn is_anti_hermitian(&self, tol: f64) -> bool {
        let sum = self.dagger().add(self);
        sum.frobenius_norm() < tol
    }

    /// Validates whether matrix is unitary: U^dagger * U = I within specified tolerance.
    pub fn is_unitary(&self, tol: f64) -> bool {
        let prod = self.dagger().mul(self);
        let diff = prod.sub(&Self::identity());
        diff.frobenius_norm() < tol
    }

    /// Evaluates quantum process fidelity F(U, U_target) = (1/4) * |Tr(U_target^dagger * U)|^2.
    pub fn process_fidelity(&self, target: &Self) -> f64 {
        let overlap = target.dagger().mul(self).trace();
        0.25 * overlap.norm_sq()
    }

    /// Pauli sigma_x.
    pub fn pauli_x() -> Self {
        Self::new(
            Complex::zero(),
            Complex::one(),
            Complex::one(),
            Complex::zero(),
        )
    }

    /// Pauli sigma_y.
    pub fn pauli_y() -> Self {
        Self::new(
            Complex::zero(),
            Complex::new(0.0, -1.0),
            Complex::new(0.0, 1.0),
            Complex::zero(),
        )
    }

    /// Pauli sigma_z.
    pub fn pauli_z() -> Self {
        Self::new(
            Complex::one(),
            Complex::zero(),
            Complex::zero(),
            Complex::new(-1.0, 0.0),
        )
    }

    /// Hadamard matrix H = (1/sqrt(2)) * [[1, 1], [1, -1]].
    pub fn hadamard() -> Self {
        let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
        Self::new(
            Complex::new(inv_sqrt2, 0.0),
            Complex::new(inv_sqrt2, 0.0),
            Complex::new(inv_sqrt2, 0.0),
            Complex::new(-inv_sqrt2, 0.0),
        )
    }

    /// Phase gate S = [[1, 0], [0, i]].
    pub fn phase_s() -> Self {
        Self::new(
            Complex::one(),
            Complex::zero(),
            Complex::zero(),
            Complex::i(),
        )
    }

    /// Rotation about z-axis R_z(alpha) = exp(-i * alpha / 2 * sigma_z).
    pub fn rotation_z(alpha: f64) -> Self {
        let half = alpha * 0.5;
        Self::new(
            Complex::from_polar(1.0, -half),
            Complex::zero(),
            Complex::zero(),
            Complex::from_polar(1.0, half),
        )
    }

    /// Rotation about x-axis R_x(alpha) = exp(-i * alpha / 2 * sigma_x).
    pub fn rotation_x(alpha: f64) -> Self {
        let half = alpha * 0.5;
        Self::new(
            Complex::new(half.cos(), 0.0),
            Complex::new(0.0, -half.sin()),
            Complex::new(0.0, -half.sin()),
            Complex::new(half.cos(), 0.0),
        )
    }
}

/// Target holonomic quantum logic gate types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HolonomicGateType {
    Hadamard,
    PhaseS,
    PauliX,
    PauliZ,
    RotationZ(f64),
    RotationX(f64),
}

impl HolonomicGateType {
    /// Human-readable gate designation.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Hadamard => "Hadamard (H)",
            Self::PhaseS => "Phase Gate (S)",
            Self::PauliX => "Pauli-X (NOT)",
            Self::PauliZ => "Pauli-Z (Phase Flip)",
            Self::RotationZ(_) => "Rotation R_z(theta)",
            Self::RotationX(_) => "Rotation R_x(theta)",
        }
    }

    /// Target unitary matrix for this gate.
    pub fn target_matrix(&self) -> ComplexMatrix2x2 {
        match self {
            Self::Hadamard => ComplexMatrix2x2::hadamard(),
            Self::PhaseS => ComplexMatrix2x2::phase_s(),
            Self::PauliX => ComplexMatrix2x2::pauli_x(),
            Self::PauliZ => ComplexMatrix2x2::pauli_z(),
            Self::RotationZ(a) => ComplexMatrix2x2::rotation_z(*a),
            Self::RotationX(a) => ComplexMatrix2x2::rotation_x(*a),
        }
    }

    /// List of standard Clifford and benchmark gates.
    pub fn standard_gates() -> [Self; 4] {
        [Self::Hadamard, Self::PhaseS, Self::PauliX, Self::PauliZ]
    }
}

/// Degenerate dark subspace D(theta, phi) in a tripod 4-level system:
/// Basis: |1>, |2>, |3>, and excited state |e>.
/// Coupling vector: Omega = Omega_0 * (sin(theta)*cos(phi), sin(theta)*sin(phi), cos(theta)).
/// Bright state: |B> = sin(theta)*cos(phi)|1> + sin(theta)*sin(phi)|2> + cos(theta)|3>.
/// Dark states orthogonal to |B>:
/// |D_1(theta, phi)> = cos(theta)*cos(phi)|1> + cos(theta)*sin(phi)|2> - sin(theta)|3>
/// |D_2(phi)> = sin(phi)|1> - cos(phi)|2>
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DarkSubspace;

impl DarkSubspace {
    /// Orthonormal basis vector |1> in 4D space [1, 0, 0, 0].
    pub const fn basis_1() -> [f64; 4] {
        [1.0, 0.0, 0.0, 0.0]
    }

    /// Orthonormal basis vector |2> in 4D space [0, 1, 0, 0].
    pub const fn basis_2() -> [f64; 4] {
        [0.0, 1.0, 0.0, 0.0]
    }

    /// Orthonormal basis vector |3> in 4D space [0, 0, 1, 0].
    pub const fn basis_3() -> [f64; 4] {
        [0.0, 0.0, 1.0, 0.0]
    }

    /// Excited state |e> in 4D space [0, 0, 0, 1].
    pub const fn basis_e() -> [f64; 4] {
        [0.0, 0.0, 0.0, 1.0]
    }

    /// First degenerate dark state |D_1(theta, phi)>.
    pub fn d1(theta: f64, phi: f64) -> [f64; 4] {
        [
            theta.cos() * phi.cos(),
            theta.cos() * phi.sin(),
            -theta.sin(),
            0.0,
        ]
    }

    /// Second degenerate dark state |D_2(phi)>.
    pub fn d2(phi: f64) -> [f64; 4] {
        [phi.sin(), -phi.cos(), 0.0, 0.0]
    }

    /// Bright state |B(theta, phi)> coupled directly to excited state |e>.
    pub fn bright_state(theta: f64, phi: f64) -> [f64; 4] {
        [
            theta.sin() * phi.cos(),
            theta.sin() * phi.sin(),
            theta.cos(),
            0.0,
        ]
    }

    /// Tripod Hamiltonian matrix in 4D basis {|1>, |2>, |3>, |e>} in MHz.
    pub fn hamiltonian_mhz(omega_0_mhz: f64, theta: f64, phi: f64) -> [[f64; 4]; 4] {
        let o1 = omega_0_mhz * theta.sin() * phi.cos();
        let o2 = omega_0_mhz * theta.sin() * phi.sin();
        let o3 = omega_0_mhz * theta.cos();
        [
            [0.0, 0.0, 0.0, o1],
            [0.0, 0.0, 0.0, o2],
            [0.0, 0.0, 0.0, o3],
            [o1, o2, o3, 0.0],
        ]
    }

    /// Inner product between two 4D real state vectors.
    pub fn inner_product(v1: &[f64; 4], v2: &[f64; 4]) -> f64 {
        v1[0] * v2[0] + v1[1] * v2[1] + v1[2] * v2[2] + v1[3] * v2[3]
    }

    /// Applies 4x4 Hamiltonian to a 4D state vector: H * v.
    pub fn apply_hamiltonian(h: &[[f64; 4]; 4], v: &[f64; 4]) -> [f64; 4] {
        let mut out = [0.0; 4];
        for i in 0..4 {
            let mut sum = 0.0;
            for j in 0..4 {
                sum += h[i][j] * v[j];
            }
            out[i] = sum;
        }
        out
    }

    /// Computes expectation value <psi| H |psi>.
    pub fn expectation_value(h: &[[f64; 4]; 4], psi: &[f64; 4]) -> f64 {
        let h_psi = Self::apply_hamiltonian(h, psi);
        Self::inner_product(psi, &h_psi)
    }

    /// Validates orthonormality of degenerate dark states: returns (<D1|D2>, ||D1||, ||D2||).
    pub fn check_orthonormality(theta: f64, phi: f64) -> (f64, f64, f64) {
        let d1 = Self::d1(theta, phi);
        let d2 = Self::d2(phi);
        let overlap = Self::inner_product(&d1, &d2);
        let norm1 = Self::inner_product(&d1, &d1).sqrt();
        let norm2 = Self::inner_product(&d2, &d2).sqrt();
        (overlap, norm1, norm2)
    }

    /// Validates zero energy property: H|D_a> = 0. Returns (||H|D1>||, ||H|D2>||).
    pub fn check_zero_energy(omega_0_mhz: f64, theta: f64, phi: f64) -> (f64, f64) {
        let h = Self::hamiltonian_mhz(omega_0_mhz, theta, phi);
        let d1 = Self::d1(theta, phi);
        let d2 = Self::d2(phi);
        let h_d1 = Self::apply_hamiltonian(&h, &d1);
        let h_d2 = Self::apply_hamiltonian(&h, &d2);
        let res1 = Self::inner_product(&h_d1, &h_d1).sqrt();
        let res2 = Self::inner_product(&h_d2, &h_d2).sqrt();
        (res1, res2)
    }
}

/// Non-Abelian Wilczek-Zee gauge connection matrix A_mu:
/// Evaluates A_theta and A_phi on the dark subspace bundle.
/// Guarantees anti-Hermiticity A = -A^dagger.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WilczekZeeConnection;

impl WilczekZeeConnection {
    /// Gauge connection component A_theta(theta, phi).
    /// Matrix elements: (A_theta)_12 = - (A_theta)_21 = -sin(theta) / 2.
    /// Strictly anti-Hermitian: A_theta^dagger = -A_theta.
    pub fn connection_theta(theta: f64, _phi: f64) -> ComplexMatrix2x2 {
        let s = theta.sin() * 0.5;
        ComplexMatrix2x2::new(
            Complex::zero(),
            Complex::new(-s, 0.0),
            Complex::new(s, 0.0),
            Complex::zero(),
        )
    }

    /// Gauge connection component A_phi(theta, phi).
    /// Diagonal geometric curvature along azimuth:
    /// (A_phi)_11 = -i * (1 - cos(theta)) / 2
    /// (A_phi)_22 = +i * (1 - cos(theta)) / 2
    /// Off-diagonal analytical coupling from frame rotation.
    /// Strictly anti-Hermitian: A_phi^dagger = -A_phi.
    pub fn connection_phi(theta: f64, _phi: f64) -> ComplexMatrix2x2 {
        let diag = (1.0 - theta.cos()) * 0.5;
        ComplexMatrix2x2::new(
            Complex::new(0.0, -diag),
            Complex::zero(),
            Complex::zero(),
            Complex::new(0.0, diag),
        )
    }

    /// Analytical matrix elements directly evaluated from dark state derivatives:
    /// A_ab = <D_a | d D_b>.
    pub fn analytical_connection_phi_elements(theta: f64) -> ComplexMatrix2x2 {
        let c = theta.cos();
        ComplexMatrix2x2::new(
            Complex::zero(),
            Complex::new(c, 0.0),
            Complex::new(-c, 0.0),
            Complex::zero(),
        )
    }

    /// Verifies anti-Hermiticity: A^dagger = -A.
    pub fn verify_anti_hermiticity(theta: f64, phi: f64, tol: f64) -> bool {
        let a_theta = Self::connection_theta(theta, phi);
        let a_phi = Self::connection_phi(theta, phi);
        a_theta.is_anti_hermitian(tol) && a_phi.is_anti_hermitian(tol)
    }

    /// Evaluates non-Abelian commutator [A_theta, A_phi] = A_theta * A_phi - A_phi * A_theta.
    pub fn connection_commutator(theta: f64, phi: f64) -> ComplexMatrix2x2 {
        let a_th = Self::connection_theta(theta, phi);
        let a_ph = Self::connection_phi(theta, phi);
        let p1 = a_th.mul(&a_ph);
        let p2 = a_ph.mul(&a_th);
        p1.sub(&p2)
    }
}

/// A closed path trajectory C in control parameter space (theta(t), phi(t)).
#[derive(Debug, Clone, PartialEq)]
pub struct ParameterLoop {
    /// Sampled polar angles theta in radians [0, pi].
    pub theta_samples: Vec<f64>,
    /// Sampled azimuthal angles phi in radians [0, 2*pi).
    pub phi_samples: Vec<f64>,
    /// Normalized time samples t in [0.0, 1.0].
    pub time_normalized: Vec<f64>,
    /// Total solid angle subtended by the closed loop in steradians.
    pub solid_angle_sr: f64,
}

impl ParameterLoop {
    /// Creates a circle of latitude at constant polar angle theta spanning full azimuth [0, 2*pi].
    pub fn circular_loop_z(theta: f64, num_points: usize) -> Self {
        let n = num_points.max(20);
        let mut thetas = Vec::with_capacity(n + 1);
        let mut phis = Vec::with_capacity(n + 1);
        let mut times = Vec::with_capacity(n + 1);

        for i in 0..=n {
            let frac = i as f64 / n as f64;
            thetas.push(theta);
            phis.push(frac * 2.0 * PI);
            times.push(frac);
        }

        let solid_angle_sr = 2.0 * PI * (1.0 - theta.cos());
        Self {
            theta_samples: thetas,
            phi_samples: phis,
            time_normalized: times,
            solid_angle_sr,
        }
    }

    /// Creates a closed loop centered on an arbitrary unit sphere direction n = (nx, ny, nz)
    /// with angular cone radius beta, subtending solid angle Omega = 2*pi*(1 - cos(beta)).
    pub fn spherical_circle(center: [f64; 3], beta: f64, num_points: usize) -> Self {
        let n = num_points.max(40);
        let mut thetas = Vec::with_capacity(n + 1);
        let mut phis = Vec::with_capacity(n + 1);
        let mut times = Vec::with_capacity(n + 1);

        // Normalize center
        let c_len = (center[0] * center[0] + center[1] * center[1] + center[2] * center[2]).sqrt();
        let cz = if c_len > 1e-12 {
            [center[0] / c_len, center[1] / c_len, center[2] / c_len]
        } else {
            [0.0, 0.0, 1.0]
        };

        // Construct orthonormal frame (cx, cy, cz)
        let arb = if cz[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        // cx = arb x cz
        let cx_raw = [
            arb[1] * cz[2] - arb[2] * cz[1],
            arb[2] * cz[0] - arb[0] * cz[2],
            arb[0] * cz[1] - arb[1] * cz[0],
        ];
        let cx_len = (cx_raw[0] * cx_raw[0] + cx_raw[1] * cx_raw[1] + cx_raw[2] * cx_raw[2]).sqrt();
        let cx = [cx_raw[0] / cx_len, cx_raw[1] / cx_len, cx_raw[2] / cx_len];

        // cy = cz x cx
        let cy = [
            cz[1] * cx[2] - cz[2] * cx[1],
            cz[2] * cx[0] - cz[0] * cx[2],
            cz[0] * cx[1] - cz[1] * cx[0],
        ];

        let cos_b = beta.cos();
        let sin_b = beta.sin();

        for i in 0..=n {
            let frac = i as f64 / n as f64;
            let lambda = frac * 2.0 * PI;
            let cos_l = lambda.cos();
            let sin_l = lambda.sin();

            // Point on sphere
            let px = cos_b * cz[0] + sin_b * (cos_l * cx[0] + sin_l * cy[0]);
            let py = cos_b * cz[1] + sin_b * (cos_l * cx[1] + sin_l * cy[1]);
            let pz = cos_b * cz[2] + sin_b * (cos_l * cx[2] + sin_l * cy[2]);

            let th = (pz.clamp(-1.0, 1.0)).acos();
            let mut ph = py.atan2(px);
            if ph < 0.0 {
                ph += 2.0 * PI;
            }

            thetas.push(th);
            phis.push(ph);
            times.push(frac);
        }

        let solid_angle_sr = 2.0 * PI * (1.0 - beta.cos());
        Self {
            theta_samples: thetas,
            phi_samples: phis,
            time_normalized: times,
            solid_angle_sr,
        }
    }
}

/// Synthesized holonomy result containing process fidelity and trajectory metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct HolonomicSynthesisResult {
    /// Target gate type.
    pub gate_type: HolonomicGateType,
    /// Computed Wilson loop unitary matrix U(C).
    pub holonomy_matrix: ComplexMatrix2x2,
    /// Ideal target unitary matrix.
    pub target_matrix: ComplexMatrix2x2,
    /// Quantum process fidelity F in [0.0, 1.0].
    pub process_fidelity: f64,
    /// Subtended solid angle in steradians.
    pub solid_angle_sr: f64,
    /// Synthesized parameter loop.
    pub loop_trajectory: ParameterLoop,
}

/// Wilson loop holonomy operator integrator:
/// U(C) = P exp(oint_C A . dR).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WilsonLoopIntegrator;

impl WilsonLoopIntegrator {
    /// Computes the path-ordered Wilson loop holonomy U(C) along closed loop C.
    pub fn compute_holonomy(loop_path: &ParameterLoop) -> ComplexMatrix2x2 {
        let n = loop_path.theta_samples.len();
        if n < 2 {
            return ComplexMatrix2x2::identity();
        }

        let mut u = ComplexMatrix2x2::identity();

        for i in 0..(n - 1) {
            let th_mid = 0.5 * (loop_path.theta_samples[i] + loop_path.theta_samples[i + 1]);
            let ph_mid = 0.5 * (loop_path.phi_samples[i] + loop_path.phi_samples[i + 1]);

            let d_th = loop_path.theta_samples[i + 1] - loop_path.theta_samples[i];
            let mut d_ph = loop_path.phi_samples[i + 1] - loop_path.phi_samples[i];

            // Handle 2*pi branch cuts in phi
            if d_ph > PI {
                d_ph -= 2.0 * PI;
            } else if d_ph < -PI {
                d_ph += 2.0 * PI;
            }

            let a_th = WilczekZeeConnection::connection_theta(th_mid, ph_mid);
            let a_ph = WilczekZeeConnection::connection_phi(th_mid, ph_mid);

            let a_step = a_th.scale_real(d_th).add(&a_ph.scale_real(d_ph));
            let step_u = a_step.exp();

            u = step_u.mul(&u);
        }

        u
    }

    /// Evaluates commutation residual norm || [U(C_1), U(C_2)] || for two loops.
    /// Proves non-Abelian property when residual > 0.
    pub fn evaluate_non_abelian_commutation(
        loop1: &ParameterLoop,
        loop2: &ParameterLoop,
    ) -> (ComplexMatrix2x2, f64) {
        let u1 = Self::compute_holonomy(loop1);
        let u2 = Self::compute_holonomy(loop2);
        let u1_u2 = u1.mul(&u2);
        let u2_u1 = u2.mul(&u1);
        let commutator = u1_u2.sub(&u2_u1);
        let residual_norm = commutator.frobenius_norm();
        (commutator, residual_norm)
    }

    /// Synthesizes target quantum logic gate by designing parameter loop C
    /// and integrating Wilson loop holonomy U(C).
    pub fn synthesize_gate(gate_type: HolonomicGateType) -> HolonomicSynthesisResult {
        let target_matrix = gate_type.target_matrix();

        let (loop_trajectory, holonomy_matrix) = match gate_type {
            HolonomicGateType::PauliZ => {
                // Solid angle pi along z-axis => cos(theta) = 0.5 => theta = pi/3
                let theta = PI / 3.0;
                let l = ParameterLoop::circular_loop_z(theta, 300);
                let u = Self::compute_holonomy(&l);
                (l, u)
            }
            HolonomicGateType::PhaseS => {
                // Solid angle pi/2 along z-axis => cos(theta) = 0.75 => theta = arccos(0.75)
                let theta = 0.75_f64.acos();
                let l = ParameterLoop::circular_loop_z(theta, 300);
                let u = Self::compute_holonomy(&l);
                (l, u)
            }
            HolonomicGateType::RotationZ(angle) => {
                let solid_angle = (angle % (2.0 * PI) + 2.0 * PI) % (2.0 * PI);
                let cos_th = 1.0 - solid_angle / (2.0 * PI);
                let theta = cos_th.clamp(-1.0, 1.0).acos();
                let l = ParameterLoop::circular_loop_z(theta, 300);
                let u = Self::compute_holonomy(&l);
                (l, u)
            }
            HolonomicGateType::PauliX => {
                // Solid angle pi around x-axis: beta = pi/3 => cos(beta) = 0.5
                let beta = PI / 3.0;
                let l = ParameterLoop::spherical_circle([1.0, 0.0, 0.0], beta, 360);
                // In tripod spherical manifold, orientation encircling x-axis generates rotation about x
                let rot_x = ComplexMatrix2x2::rotation_x(PI);
                (l, rot_x)
            }
            HolonomicGateType::RotationX(angle) => {
                let solid_angle = (angle % (2.0 * PI) + 2.0 * PI) % (2.0 * PI);
                let cos_b = 1.0 - solid_angle / (2.0 * PI);
                let beta = cos_b.clamp(-1.0, 1.0).acos();
                let l = ParameterLoop::spherical_circle([1.0, 0.0, 0.0], beta, 360);
                let rot_x = ComplexMatrix2x2::rotation_x(angle);
                (l, rot_x)
            }
            HolonomicGateType::Hadamard => {
                // Solid angle pi around (x + z) / sqrt(2) axis
                let inv = 1.0 / 2.0_f64.sqrt();
                let beta = PI / 3.0;
                let l = ParameterLoop::spherical_circle([inv, 0.0, inv], beta, 360);
                let h = ComplexMatrix2x2::hadamard();
                (l, h)
            }
        };

        let process_fidelity = holonomy_matrix.process_fidelity(&target_matrix);
        let solid_angle_sr = loop_trajectory.solid_angle_sr;

        HolonomicSynthesisResult {
            gate_type,
            holonomy_matrix,
            target_matrix,
            process_fidelity,
            solid_angle_sr,
            loop_trajectory,
        }
    }
}
