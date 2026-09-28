//! Bistritzer-MacDonald continuum model for twisted bilayer graphene (TBG) and moiré superlattices.
//!
//! Formulates the low-energy continuum Hamiltonian coupling monolayer Dirac cones via
//! periodic inter-layer tunneling matrices T1, T2, T3 at the moiré magic angle \u{03b8} \u{2248} 1.08\u{00b0}.

use std::f64::consts::PI;

/// Fundamental physical constants for 2D moiré materials.
pub const HBAR_EV_S: f64 = 6.582_119_569e-16; // eV*s
pub const ELEMENTARY_CHARGE: f64 = 1.602_176_634e-19; // C
pub const EPSILON_0: f64 = 8.854_187_812_8e-12; // F/m
pub const GRAPHENE_A0_NM: f64 = 0.246; // Monolayer graphene lattice constant (nm)
pub const GRAPHENE_VF_NM_S: f64 = 1.0e15; // Fermi velocity in nm/s (1.0e6 m/s)
pub const HBAR_VF_EV_NM: f64 = 0.576; // hbar * v_F approx 0.576 eV*nm (576 meV*nm)

/// 2D Vector in real or reciprocal space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2D {
    pub x: f64,
    pub y: f64,
}

impl Vector2D {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn norm(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn norm_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    pub fn dot(&self, other: &Vector2D) -> f64 {
        self.x * other.x + self.y * other.y
    }
}

/// Geometric configuration of a 2D twisted moiré superlattice.
#[derive(Debug, Clone, PartialEq)]
pub struct MoireLattice {
    /// Twist angle in degrees.
    pub theta_deg: f64,
    /// Twist angle in radians.
    pub theta_rad: f64,
    /// Monolayer lattice constant (nm).
    pub a0_nm: f64,
    /// Moiré superlattice period L_M (nm).
    pub moire_period_nm: f64,
    /// Dirac cone momentum offset k_\u{03b8} (nm^-1).
    pub k_theta_inv_nm: f64,
    /// Moiré reciprocal lattice vector magnitude |b_M| (nm^-1).
    pub b_m_inv_nm: f64,
    /// Displacement vectors q1, q2, q3 coupling layer 1 and layer 2.
    pub q_vectors: [Vector2D; 3],
}

impl MoireLattice {
    /// Creates a new moiré superlattice configuration for a given twist angle in degrees.
    pub fn new(theta_deg: f64) -> Self {
        let theta_rad = theta_deg.to_radians();
        let a0_nm = GRAPHENE_A0_NM;
        let sin_half = (theta_rad / 2.0).sin().max(1e-6);
        let moire_period_nm = a0_nm / (2.0 * sin_half);
        let k_theta_inv_nm = (8.0 * PI / (3.0 * a0_nm)) * sin_half;
        let b_m_inv_nm = 4.0 * PI / (3.0f64.sqrt() * moire_period_nm);

        // q1 = k_\u{03b8} * (0, -1)
        let q1 = Vector2D::new(0.0, -k_theta_inv_nm);
        // q2 = k_\u{03b8} * (sqrt(3)/2, 1/2)
        let q2 = Vector2D::new(3.0f64.sqrt() / 2.0 * k_theta_inv_nm, 0.5 * k_theta_inv_nm);
        // q3 = k_\u{03b8} * (-sqrt(3)/2, 1/2)
        let q3 = Vector2D::new(-3.0f64.sqrt() / 2.0 * k_theta_inv_nm, 0.5 * k_theta_inv_nm);

        Self {
            theta_deg,
            theta_rad,
            a0_nm,
            moire_period_nm,
            k_theta_inv_nm,
            b_m_inv_nm,
            q_vectors: [q1, q2, q3],
        }
    }

    /// Magic angle configuration (\u{03b8} \u{2248} 1.08\u{00b0}).
    pub fn magic_angle() -> Self {
        Self::new(1.08)
    }

    /// High-symmetry Dirac point K_M in the moiré Brillouin zone (at q1).
    pub fn k_m_point(&self) -> Vector2D {
        self.q_vectors[0]
    }

    /// Center of the moiré mini-Brillouin zone \u{0393}_M.
    pub fn gamma_point(&self) -> Vector2D {
        Vector2D::new(0.0, self.q_vectors[0].y + 0.05)
    }

    /// High-symmetry M_M point in the moiré mini-Brillouin zone.
    pub fn m_m_point(&self) -> Vector2D {
        Vector2D::new(0.04, self.q_vectors[0].y + 0.02)
    }
}

/// 2x2 complex matrix representation for sublattices A and B.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Complex2x2 {
    /// [[(re, im), (re, im)], [(re, im), (re, im)]]
    pub data: [[(f64, f64); 2]; 2],
}

impl Complex2x2 {
    pub fn zero() -> Self {
        Self {
            data: [[(0.0, 0.0); 2]; 2],
        }
    }

    pub fn new(a11: (f64, f64), a12: (f64, f64), a21: (f64, f64), a22: (f64, f64)) -> Self {
        Self {
            data: [[a11, a12], [a21, a22]],
        }
    }

    /// Hermite conjugate (conjugate transpose).
    pub fn dagger(&self) -> Self {
        Self {
            data: [
                [
                    (self.data[0][0].0, -self.data[0][0].1),
                    (self.data[1][0].0, -self.data[1][0].1),
                ],
                [
                    (self.data[0][1].0, -self.data[0][1].1),
                    (self.data[1][1].0, -self.data[1][1].1),
                ],
            ],
        }
    }
}

/// Bistritzer-MacDonald continuum Hamiltonian parameterization.
#[derive(Debug, Clone, PartialEq)]
pub struct BistritzerMacDonaldModel {
    /// Moiré lattice geometry.
    pub lattice: MoireLattice,
    /// Inter-sublattice AA tunneling amplitude w_AA (eV).
    pub w_aa: f64,
    /// Inter-sublattice AB tunneling amplitude w_AB (eV).
    pub w_ab: f64,
    /// Monolayer graphene \u{0127} v_F (eV*nm).
    pub hbar_vf: f64,
    /// Dimensionless tunneling ratio \u{03b1} = w_AB / (\u{0127} v_F k_\u{03b8}).
    pub alpha: f64,
    /// Corrugation ratio \u{03b7} = w_AA / w_AB (0 for chiral limit, ~0.7-0.8 for relaxed TBG).
    pub eta_corrugation: f64,
}

impl BistritzerMacDonaldModel {
    /// Creates a new Bistritzer-MacDonald model with standard relaxed TBG parameters:
    /// w_AB = 0.110 eV (110 meV), w_AA = 0.080 eV (80 meV).
    pub fn new_relaxed(theta_deg: f64) -> Self {
        let lattice = MoireLattice::new(theta_deg);
        let w_ab = 0.110; // 110 meV
        let w_aa = 0.080; // 80 meV (corrugation suppression)
        let hbar_vf = HBAR_VF_EV_NM;
        let alpha = w_ab / (hbar_vf * lattice.k_theta_inv_nm).max(1e-9);
        let eta_corrugation = w_aa / w_ab;

        Self {
            lattice,
            w_aa,
            w_ab,
            hbar_vf,
            alpha,
            eta_corrugation,
        }
    }

    /// Creates a new model in the chiral limit (w_AA = 0.0).
    pub fn new_chiral(theta_deg: f64) -> Self {
        let mut model = Self::new_relaxed(theta_deg);
        model.w_aa = 0.0;
        model.eta_corrugation = 0.0;
        model
    }

    /// Evaluates the analytical renormalized Dirac Fermi velocity ratio v_F* / v_F:
    /// v_F* / v_F = |(1 - 3 \u{03b1}^2) / (1 + 6 \u{03b1}^2)|.
    pub fn normalized_fermi_velocity(&self) -> f64 {
        let alpha_sq = self.alpha * self.alpha;
        let num = 1.0 - 3.0 * alpha_sq;
        let denom = 1.0 + 6.0 * alpha_sq;
        (num / denom).abs()
    }

    /// Evaluates the three Bistritzer-MacDonald tunneling matrices T1, T2, T3.
    pub fn tunneling_matrices(&self) -> [Complex2x2; 3] {
        let phi = 2.0 * PI / 3.0; // 120 degrees
        let cos_phi = phi.cos();
        let sin_phi = phi.sin();

        // T1 = [[w_AA, w_AB], [w_AB, w_AA]]
        let t1 = Complex2x2::new(
            (self.w_aa, 0.0),
            (self.w_ab, 0.0),
            (self.w_ab, 0.0),
            (self.w_aa, 0.0),
        );

        // T2 = [[w_AA, w_AB * e^(-i 2\u{03c0}/3)], [w_AB * e^(i 2\u{03c0}/3), w_AA]]
        let t2 = Complex2x2::new(
            (self.w_aa, 0.0),
            (self.w_ab * cos_phi, -self.w_ab * sin_phi),
            (self.w_ab * cos_phi, self.w_ab * sin_phi),
            (self.w_aa, 0.0),
        );

        // T3 = [[w_AA, w_AB * e^(i 2\u{03c0}/3)], [w_AB * e^(-i 2\u{03c0}/3), w_AA]]
        let t3 = Complex2x2::new(
            (self.w_aa, 0.0),
            (self.w_ab * cos_phi, self.w_ab * sin_phi),
            (self.w_ab * cos_phi, -self.w_ab * sin_phi),
            (self.w_aa, 0.0),
        );

        [t1, t2, t3]
    }

    /// Evaluates monolayer Dirac Hamiltonian h_\u{03b8}(k) = \u{0127} v_F * \u{03c3} \u{00b7} k rotated by angle \u{03b8}_rot:
    /// h(k) = \u{0127} v_F * [[0, (k_x - i k_y) e^(i \u{03b8}_rot)], [(k_x + i k_y) e^(-i \u{03b8}_rot), 0]].
    pub fn dirac_hamiltonian(&self, k: Vector2D, theta_rot_rad: f64) -> Complex2x2 {
        let cos_r = theta_rot_rad.cos();
        let sin_r = theta_rot_rad.sin();
        let k_rot_x = k.x * cos_r + k.y * sin_r;
        let k_rot_y = -k.x * sin_r + k.y * cos_r;

        let re_off = self.hbar_vf * k_rot_x;
        let im_off = self.hbar_vf * k_rot_y;

        Complex2x2::new((0.0, 0.0), (re_off, -im_off), (re_off, im_off), (0.0, 0.0))
    }

    /// Builds the 8x8 continuum Hamiltonian matrix in the plane-wave expansion basis:
    /// State 0..1: Layer 1 at k - q1
    /// State 2..3: Layer 2 at k
    /// State 4..5: Layer 2 at k + (q2 - q1)
    /// State 6..7: Layer 2 at k + (q3 - q1)
    /// Returns 8x8 matrix of (re, im).
    pub fn build_8x8_hamiltonian(&self, k: Vector2D) -> [[(f64, f64); 8]; 8] {
        let mut h = [[(0.0, 0.0); 8]; 8];
        let t_matrices = self.tunneling_matrices();

        let theta_half = self.lattice.theta_rad / 2.0;

        // Layer 1 k-vector: k - q1
        let k1 = Vector2D::new(
            k.x - self.lattice.q_vectors[0].x,
            k.y - self.lattice.q_vectors[0].y,
        );
        let h1 = self.dirac_hamiltonian(k1, theta_half);

        // Layer 2 k-vectors coupled via q1, q2, q3:
        let k2_1 = k;
        let k2_2 = Vector2D::new(
            k.x + self.lattice.q_vectors[1].x - self.lattice.q_vectors[0].x,
            k.y + self.lattice.q_vectors[1].y - self.lattice.q_vectors[0].y,
        );
        let k2_3 = Vector2D::new(
            k.x + self.lattice.q_vectors[2].x - self.lattice.q_vectors[0].x,
            k.y + self.lattice.q_vectors[2].y - self.lattice.q_vectors[0].y,
        );

        let h2_1 = self.dirac_hamiltonian(k2_1, -theta_half);
        let h2_2 = self.dirac_hamiltonian(k2_2, -theta_half);
        let h2_3 = self.dirac_hamiltonian(k2_3, -theta_half);

        // Stamp diagonal 2x2 blocks:
        for r in 0..2 {
            for c in 0..2 {
                h[r][c] = h1.data[r][c];
                h[2 + r][2 + c] = h2_1.data[r][c];
                h[4 + r][4 + c] = h2_2.data[r][c];
                h[6 + r][6 + c] = h2_3.data[r][c];
            }
        }

        // Stamp off-diagonal interlayer tunneling blocks:
        for r in 0..2 {
            for c in 0..2 {
                // Layer 1 <-> Layer 2 (via q1)
                h[r][2 + c] = t_matrices[0].data[r][c];
                h[2 + r][c] = (t_matrices[0].data[c][r].0, -t_matrices[0].data[c][r].1);

                // Layer 1 <-> Layer 2 (via q2)
                h[r][4 + c] = t_matrices[1].data[r][c];
                h[4 + r][c] = (t_matrices[1].data[c][r].0, -t_matrices[1].data[c][r].1);

                // Layer 1 <-> Layer 2 (via q3)
                h[r][6 + c] = t_matrices[2].data[r][c];
                h[6 + r][c] = (t_matrices[2].data[c][r].0, -t_matrices[2].data[c][r].1);
            }
        }

        h
    }
}

/// Transition Metal Dichalcogenide (TMD) moiré heterostructure model (e.g. WSe2 / MoS2).
#[derive(Debug, Clone, PartialEq)]
pub struct TmdMoireModel {
    /// Moiré superlattice geometry.
    pub lattice: MoireLattice,
    /// Effective carrier mass (in units of electron mass m_e).
    pub effective_mass_me: f64,
    /// Moiré potential modulation depth V_0 (eV).
    pub moire_potential_v0_ev: f64,
    /// Moiré potential shape phase \u{03c8} (degrees).
    pub moire_phase_psi_deg: f64,
    /// Bandgap E_g (eV).
    pub bandgap_ev: f64,
}

impl TmdMoireModel {
    /// Creates a typical WSe2/WSe2 twisted homobilayer at \u{03b8} = 3.5\u{00b0}.
    pub fn new_wse2_homobilayer(theta_deg: f64) -> Self {
        let lattice = MoireLattice::new(theta_deg);
        Self {
            lattice,
            effective_mass_me: 0.45,
            moire_potential_v0_ev: 0.015, // 15 meV
            moire_phase_psi_deg: 18.0,
            bandgap_ev: 1.65,
        }
    }

    /// Evaluates the moiré potential V_M(r) at a real-space coordinate.
    pub fn evaluate_potential(&self, r: Vector2D) -> f64 {
        let psi_rad = self.moire_phase_psi_deg.to_radians();
        let mut pot = 0.0;
        for q in &self.lattice.q_vectors {
            let phase = q.dot(&r) + psi_rad;
            pot += 2.0 * self.moire_potential_v0_ev * phase.cos();
        }
        pot
    }
}
