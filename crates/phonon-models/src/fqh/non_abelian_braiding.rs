//! Non-Abelian Ising anyon fusion rules, braid matrices R and B_23, and topological entanglement entropy.
//!
//! Formulates the 2D topological qubit Hilbert space spanned by 4 Majorana vortices \u{03c3},
//! non-commutative braid transformations, and quantum dimension \u{03ba} = 2.

use std::f64::consts::PI;

/// Complex number represented as (real, imag).
pub type Complex = (f64, f64);

/// Multiplies two complex numbers.
pub fn c_mul(a: Complex, b: Complex) -> Complex {
    (a.0 * b.0 - a.1 * b.1, a.0 * b.1 + a.1 * b.0)
}

/// Adds two complex numbers.
pub fn c_add(a: Complex, b: Complex) -> Complex {
    (a.0 + b.0, a.1 + b.1)
}

/// Conjugate of a complex number.
pub fn c_conj(a: Complex) -> Complex {
    (a.0, -a.1)
}

/// Modulus squared |z|^2.
pub fn c_abs_sq(a: Complex) -> f64 {
    a.0 * a.0 + a.1 * a.1
}

/// 2x2 complex unitary matrix [[m00, m01], [m10, m11]].
pub type Unitary2x2 = [Complex; 4];

/// Multiplies two 2x2 complex matrices: C = A * B.
pub fn mat2_mul(a: &Unitary2x2, b: &Unitary2x2) -> Unitary2x2 {
    [
        c_add(c_mul(a[0], b[0]), c_mul(a[1], b[2])),
        c_add(c_mul(a[0], b[1]), c_mul(a[1], b[3])),
        c_add(c_mul(a[2], b[0]), c_mul(a[3], b[2])),
        c_add(c_mul(a[2], b[1]), c_mul(a[3], b[3])),
    ]
}

/// Conjugate transpose (Hermitian adjoint) of a 2x2 matrix: A\u{2020}.
pub fn mat2_dagger(a: &Unitary2x2) -> Unitary2x2 {
    [c_conj(a[0]), c_conj(a[2]), c_conj(a[1]), c_conj(a[3])]
}

/// Non-Abelian Ising Anyon Braiding Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NonAbelianBraidingModel;

impl NonAbelianBraidingModel {
    /// Quantum dimension of the vacuum / identity anyon: d_I = 1.
    pub fn dim_identity() -> f64 {
        1.0
    }

    /// Quantum dimension of the neutral fermion anyon: d_\u{03c8} = 1.
    pub fn dim_fermion() -> f64 {
        1.0
    }

    /// Quantum dimension of the non-Abelian Majorana vortex: d_\u{03c3} = sqrt(2).
    pub fn dim_sigma() -> f64 {
        (2.0_f64).sqrt()
    }

    /// Total quantum dimension \u{03ba} = sqrt(\u{2211} d_i^2) = sqrt(1^2 + 1^2 + 2) = 2.
    pub fn total_quantum_dimension() -> f64 {
        let d_i = Self::dim_identity();
        let d_psi = Self::dim_fermion();
        let d_sig = Self::dim_sigma();
        (d_i * d_i + d_psi * d_psi + d_sig * d_sig).sqrt()
    }

    /// Topological entanglement entropy S_topo = ln(\u{03ba}) = ln(2) \u{2248} 0.69315.
    pub fn topological_entanglement_entropy() -> f64 {
        Self::total_quantum_dimension().ln()
    }

    /// Elementary braid matrix R_12 exchanging anyons 1 and 2 in the fusion basis {|0\u{27e9}, |1\u{27e9}}:
    /// R = e^{-i\u{03c0}/8} * diag(1, i).
    pub fn braid_r12() -> Unitary2x2 {
        let phase = -PI / 8.0;
        let c = phase.cos();
        let s = phase.sin();
        let e_minus_i_pi_8 = (c, s);

        // e^{-i\u{03c0}/8} * 1 = (c, s)
        let r00 = e_minus_i_pi_8;
        let r01 = (0.0, 0.0);
        let r10 = (0.0, 0.0);
        // e^{-i\u{03c0}/8} * i = (-s, c)
        let r11 = (-s, c);

        [r00, r01, r10, r11]
    }

    /// F-matrix changing fusion basis from (12)3 to 1(23):
    /// F = (1 / sqrt(2)) * [[1, 1], [1, -1]].
    pub fn f_matrix() -> Unitary2x2 {
        let inv_sqrt2 = 1.0 / (2.0_f64).sqrt();
        [
            (inv_sqrt2, 0.0),
            (inv_sqrt2, 0.0),
            (inv_sqrt2, 0.0),
            (-inv_sqrt2, 0.0),
        ]
    }

    /// Braid matrix B_23 exchanging anyons 2 and 3:
    /// B_23 = F^\u{2020} * R * F = (e^{-i\u{03c0}/8} / sqrt(2)) * [[1, -i], [-i, 1]].
    pub fn braid_b23() -> Unitary2x2 {
        let phase = -PI / 8.0;
        let c = phase.cos();
        let s = phase.sin();
        let inv_s2 = 1.0 / (2.0_f64).sqrt();

        // (c + i s) * inv_s2
        let diag = (c * inv_s2, s * inv_s2);
        // (c + i s) * (-i) * inv_s2 = (s - i c) * inv_s2
        let off_diag = (s * inv_s2, -c * inv_s2);

        [diag, off_diag, off_diag, diag]
    }

    /// Evaluates the commutator norm || [R, B_23] || = || R * B_23 - B_23 * R ||.
    /// Non-zero commutator mathematically proves the non-Abelian braiding algebra!
    pub fn commutator_norm() -> f64 {
        let r = Self::braid_r12();
        let b = Self::braid_b23();

        let rb = mat2_mul(&r, &b);
        let br = mat2_mul(&b, &r);

        let mut sum_sq = 0.0;
        for i in 0..4 {
            let diff_re = rb[i].0 - br[i].0;
            let diff_im = rb[i].1 - br[i].1;
            sum_sq += diff_re * diff_re + diff_im * diff_im;
        }

        sum_sq.sqrt()
    }

    /// Evolves a density matrix \u{03c1} under a unitary braid transformation U:
    /// \u{03c1}\u{2032} = U * \u{03c1} * U\u{2020}.
    pub fn evolve_density_matrix(rho: &Unitary2x2, u: &Unitary2x2) -> Unitary2x2 {
        let u_rho = mat2_mul(u, rho);
        let u_dag = mat2_dagger(u);
        mat2_mul(&u_rho, &u_dag)
    }

    /// Evaluates state fidelity F = \u{27e8}\u{03c8}_target | \u{03c1} | \u{03c8}_target\u{27e9}.
    pub fn state_fidelity(rho: &Unitary2x2, target_state: [Complex; 2]) -> f64 {
        // \u{27e8}\u{03c8}| \u{03c1} |\u{03c8}\u{27e9} = c0* (\u{03c1}00 c0 + \u{03c1}01 c1) + c1* (\u{03c1}10 c0 + \u{03c1}11 c1)
        let c0 = target_state[0];
        let c1 = target_state[1];

        let term0 = c_add(c_mul(rho[0], c0), c_mul(rho[1], c1));
        let term1 = c_add(c_mul(rho[2], c0), c_mul(rho[3], c1));

        let res0 = c_mul(c_conj(c0), term0);
        let res1 = c_mul(c_conj(c1), term1);

        (res0.0 + res1.0).clamp(0.0, 1.0)
    }
}
