//! Giant Spin Hamiltonian for Single-Molecule Magnets (SMMs):
//! uniaxial anisotropy, transverse crystal fields, Zeeman splitting,
//! and resonant quantum tunneling of magnetization (QTM).

/// Bohr magneton in Joules per Tesla (J/T).
pub const BOHR_MAGNETON: f64 = 9.274_010_078_3e-24;
/// Planck constant over 2*pi in Joule-seconds (J*s).
pub const HBAR: f64 = 1.054_571_817e-34;
/// Boltzmann constant in Joules per Kelvin (J/K).
pub const BOLTZMANN_K: f64 = 1.380_649e-23;

/// Giant spin Hamiltonian parameters for a single-molecule magnet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GiantSpinParams {
    /// Total spin quantum number S (e.g., 10.0 for Fe8 and Mn12).
    pub spin_s: f64,
    /// Second-order axial (uniaxial) anisotropy constant D / k_B in Kelvin (D > 0 indicates easy-axis).
    pub d_uniaxial_k: f64,
    /// Second-order rhombic (transverse) anisotropy constant E / k_B in Kelvin (E != 0 mixes m states with dm = +-2).
    pub e_rhombic_k: f64,
    /// Fourth-order tetragonal crystal field parameter B_4^4 / k_B in Kelvin (mixes m states with dm = +-4).
    pub b44_crystal_field_k: f64,
    /// Landé g-factor (typically ~2.00 for transition metals).
    pub g_factor: f64,
}

impl GiantSpinParams {
    /// Constructs giant spin parameters.
    pub fn new(
        spin_s: f64,
        d_uniaxial_k: f64,
        e_rhombic_k: f64,
        b44_crystal_field_k: f64,
        g_factor: f64,
    ) -> Self {
        Self {
            spin_s,
            d_uniaxial_k,
            e_rhombic_k,
            b44_crystal_field_k,
            g_factor,
        }
    }

    /// Canonical Fe8 molecular magnet: S = 10, D/k_B = 0.275 K, E/k_B = 0.046 K, B_4^4/k_B = -2.5e-5 K, g = 2.0.
    pub fn fe8_molecular_magnet() -> Self {
        Self::new(10.0, 0.275, 0.046, -2.5e-5, 2.0)
    }

    /// Canonical Mn12-acetate molecular magnet: S = 10, D/k_B = 0.55 K, E/k_B = 0.0 K (tetragonal symmetry), B_4^4/k_B = -2.2e-5 K, g = 1.93.
    pub fn mn12_molecular_magnet() -> Self {
        Self::new(10.0, 0.55, 0.0, -2.2e-5, 1.93)
    }

    /// Classical spin reversal energy barrier U = D * S^2 in Kelvin (for integer spin).
    pub fn classical_barrier_k(&self) -> f64 {
        self.d_uniaxial_k * self.spin_s.powi(2)
    }

    /// Classical spin reversal energy barrier U in Joules.
    pub fn classical_barrier_joules(&self) -> f64 {
        self.classical_barrier_k() * BOLTZMANN_K
    }

    /// Resonant QTM longitudinal magnetic field for step index k:
    /// $$B_z^{(k)} = k \cdot rac{D}{g \mu_B}$$
    pub fn resonant_field_tesla(&self, step_k: i32) -> f64 {
        let d_joules = self.d_uniaxial_k * BOLTZMANN_K;
        (step_k as f64 * d_joules) / (self.g_factor * BOHR_MAGNETON)
    }

    /// Dimension of the spin Hilbert space (2S + 1).
    pub fn hilbert_dim(&self) -> usize {
        (2.0 * self.spin_s).round() as usize + 1
    }

    /// Blocking temperature estimate T_B = U / (k_B * ln(tau_exp / tau_0)) ~ U / 25 in Kelvin.
    pub fn blocking_temperature_estimate_k(&self) -> f64 {
        self.classical_barrier_k() / 25.0
    }
}
