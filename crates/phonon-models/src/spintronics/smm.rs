//! Single-Molecule Magnet (SMM) Synthesis, Giant Magnetic Anisotropy & Quantum Tunneling of Magnetization.
//!
//! Formulates the microscopic spin Hamiltonian:
//! \[\hat{H}_{SMM} = D \hat{S}_z^2 + E(\hat{S}_x^2 - \hat{S}_y^2) + g \mu_B \mathbf{B} \cdot \hat{\mathbf{S}}\]
//!
//! Physical features:
//! 1. Giant uniaxial easy-axis magnetic anisotropy ($D < 0$) and transverse rhombic parameter ($E$).
//! 2. Integer and half-integer Kramers spins ($S \in \{1/2, 5/2, 9/2, 10, 15/2\}$).
//! 3. Kramers ground-state doublets ($m_s = \pm S$) with time-reversal degeneracy at $\mathbf{B} = 0$.
//! 4. Quantum Tunneling of Magnetization (QTM) at resonance Zeeman fields:
//!    \[B_z^{(k)} = k \frac{|D|}{g \mu_B}, \quad k \in \{0, 1, 2, \dots\}\]
//! 5. Thermal Orbach relaxation lifetime $\tau(T) = \tau_0 \exp(U_{eff} / k_B T)$.

use crate::quantum::Complex;
use crate::spintronics::ciss::ComplexMatrix;
use crate::spintronics::Vec3;
use phonon_core::{BOLTZMANN_CONSTANT, ELEMENTARY_CHARGE};

/// Bohr magneton in electron-volts per Tesla ($eV/T$).
pub const BOHR_MAGNETON_EV: f64 = 5.788_381_806_0e-5;

/// Bohr magneton in Joules per Tesla ($J/T$).
pub const BOHR_MAGNETON_JOULES: f64 = 9.274_010_078_3e-24;

/// Exact representation of integer or half-integer spin quantum number $S$.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpinValue {
    /// Two times the spin quantum number ($2S$).
    pub two_s: usize,
}

impl SpinValue {
    pub fn new_half_integer(two_s: usize) -> Self {
        assert!(two_s % 2 == 1, "Expected odd 2S for half-integer spin");
        Self { two_s }
    }

    pub fn new_integer(s: usize) -> Self {
        Self { two_s: 2 * s }
    }

    /// Spin $S = 5/2$ (e.g. high-spin $\text{Fe}^{3+}$ or $\text{Mn}^{2+}$).
    pub fn s5_half() -> Self {
        Self::new_half_integer(5)
    }

    /// Spin $S = 9/2$ (e.g. dysprosium metallocene single-ion magnet).
    pub fn s9_half() -> Self {
        Self::new_half_integer(9)
    }

    /// Spin $S = 10$ (e.g. $\text{Mn}_{12}$-acetate cluster).
    pub fn s10() -> Self {
        Self::new_integer(10)
    }

    /// Spin $S = 15/2$ (e.g. $\text{Dy}^{3+}$ Kramers ground doublet with high orbital momentum).
    pub fn s15_half() -> Self {
        Self::new_half_integer(15)
    }

    #[inline(always)]
    pub fn is_kramers(&self) -> bool {
        self.two_s % 2 == 1
    }

    #[inline(always)]
    pub fn dim(&self) -> usize {
        self.two_s + 1
    }

    #[inline(always)]
    pub fn s_float(&self) -> f64 {
        self.two_s as f64 / 2.0
    }

    /// Returns the magnetic quantum number $m_s \in [-S, +S]$ for Hilbert space basis index $i \in [0, \text{dim}-1]$.
    /// Ordered descending: $i = 0 \implies m_s = +S$, $i = \text{dim}-1 \implies m_s = -S$.
    #[inline(always)]
    pub fn m_s(&self, index: usize) -> f64 {
        assert!(index <= self.two_s, "Index out of bounds for spin basis");
        self.s_float() - (index as f64)
    }
}

/// Single-Molecule Magnet (SMM) model exhibiting giant magnetic anisotropy and QTM.
#[derive(Debug, Clone, PartialEq)]
pub struct SingleMoleculeMagnet {
    /// Total spin quantum number $S$.
    pub spin: SpinValue,
    /// Uniaxial anisotropy parameter $D$ in electron-volts ($eV$). For easy-axis SMMs, $D < 0$.
    pub d_anisotropy_ev: f64,
    /// Transverse rhombic anisotropy parameter $E$ in electron-volts ($eV$).
    pub e_transverse_ev: f64,
    /// Landé $g$-factor ($g \approx 2.0$).
    pub g_factor: f64,
    /// Attempt time $\tau_0$ for Orbach thermal relaxation in seconds.
    pub attempt_time_s: f64,
}

impl SingleMoleculeMagnet {
    pub fn new(spin: SpinValue, d_anisotropy_ev: f64, e_transverse_ev: f64, g_factor: f64) -> Self {
        assert!(d_anisotropy_ev < 0.0, "Easy-axis SMM requires D < 0");
        assert!(
            e_transverse_ev.abs() <= d_anisotropy_ev.abs() / 3.0 + 1e-12,
            "Rhombic anisotropy must satisfy |E| <= |D|/3"
        );
        Self {
            spin,
            d_anisotropy_ev,
            e_transverse_ev,
            g_factor,
            attempt_time_s: 1.0e-10,
        }
    }

    pub fn with_attempt_time(mut self, attempt_time_s: f64) -> Self {
        self.attempt_time_s = attempt_time_s;
        self
    }

    /// Effective reversal energy barrier $U_{eff}$ in electron-volts ($eV$):
    /// - For integer spin $S$: $U_{eff} = |D| S^2$
    /// - For half-integer Kramers spin $S$: $U_{eff} = |D| (S^2 - 1/4)$.
    #[inline(always)]
    pub fn effective_barrier_ev(&self) -> f64 {
        let s = self.spin.s_float();
        let abs_d = self.d_anisotropy_ev.abs();
        if self.spin.is_kramers() {
            abs_d * (s * s - 0.25)
        } else {
            abs_d * s * s
        }
    }

    /// Effective barrier in Joules ($J$).
    #[inline(always)]
    pub fn effective_barrier_joules(&self) -> f64 {
        self.effective_barrier_ev() * ELEMENTARY_CHARGE
    }

    /// Thermal stability factor $\Delta = U_{eff} / (k_B T)$.
    #[inline(always)]
    pub fn thermal_stability_factor(&self, temp_k: f64) -> f64 {
        let kb_ev = BOLTZMANN_CONSTANT / ELEMENTARY_CHARGE;
        let kt = (kb_ev * temp_k).max(1e-9);
        self.effective_barrier_ev() / kt
    }

    /// Thermal Orbach relaxation lifetime $\tau(T) = \tau_0 \exp(U_{eff} / k_B T)$ in seconds.
    #[inline(always)]
    pub fn retention_time_s(&self, temp_k: f64) -> f64 {
        let delta = self.thermal_stability_factor(temp_k).clamp(0.0, 100.0);
        self.attempt_time_s * delta.exp()
    }

    /// Retention time in years.
    #[inline(always)]
    pub fn retention_time_years(&self, temp_k: f64) -> f64 {
        let seconds_per_year = 365.25 * 86_400.0;
        self.retention_time_s(temp_k) / seconds_per_year
    }

    /// Constructs the $\hat{S}_z$ operator matrix in the $(2S+1)$-dimensional basis.
    pub fn spin_z_operator(&self) -> ComplexMatrix {
        let dim = self.spin.dim();
        let mut sz = ComplexMatrix::zeros(dim);
        for i in 0..dim {
            let m = self.spin.m_s(i);
            sz.set(i, i, Complex::new(m, 0.0));
        }
        sz
    }

    /// Constructs the raising operator $\hat{S}_+$ matrix.
    pub fn spin_plus_operator(&self) -> ComplexMatrix {
        let dim = self.spin.dim();
        let s = self.spin.s_float();
        let mut sp = ComplexMatrix::zeros(dim);
        for i in 1..dim {
            let m = self.spin.m_s(i);
            // S+ takes |m> to |m+1>, moving from index i to index i-1:
            let val = (s * (s + 1.0) - m * (m + 1.0)).max(0.0).sqrt();
            sp.set(i - 1, i, Complex::new(val, 0.0));
        }
        sp
    }

    /// Constructs the lowering operator $\hat{S}_-$ matrix.
    pub fn spin_minus_operator(&self) -> ComplexMatrix {
        let dim = self.spin.dim();
        let s = self.spin.s_float();
        let mut sm = ComplexMatrix::zeros(dim);
        for i in 0..(dim - 1) {
            let m = self.spin.m_s(i);
            // S- takes |m> to |m-1>, moving from index i to index i+1:
            let val = (s * (s + 1.0) - m * (m - 1.0)).max(0.0).sqrt();
            sm.set(i + 1, i, Complex::new(val, 0.0));
        }
        sm
    }

    /// Constructs the $\hat{S}_x$ operator matrix: $\hat{S}_x = \frac{1}{2}(\hat{S}_+ + \hat{S}_-)$.
    pub fn spin_x_operator(&self) -> ComplexMatrix {
        let sp = self.spin_plus_operator();
        let sm = self.spin_minus_operator();
        sp.add(&sm).scale_real(0.5)
    }

    /// Constructs the $\hat{S}_y$ operator matrix: $\hat{S}_y = \frac{1}{2i}(\hat{S}_+ - \hat{S}_-)$.
    pub fn spin_y_operator(&self) -> ComplexMatrix {
        let sp = self.spin_plus_operator();
        let sm = self.spin_minus_operator();
        let diff = sp.sub(&sm);
        diff.scale(Complex::new(0.0, -0.5))
    }

    /// Returns the triple $(\hat{S}_x, \hat{S}_y, \hat{S}_z)$.
    pub fn spin_operators(&self) -> (ComplexMatrix, ComplexMatrix, ComplexMatrix) {
        (
            self.spin_x_operator(),
            self.spin_y_operator(),
            self.spin_z_operator(),
        )
    }

    /// Evaluates the complete microscopic spin Hamiltonian matrix in $eV$:
    /// \[\hat{H} = D \hat{S}_z^2 + E(\hat{S}_x^2 - \hat{S}_y^2) + g \mu_B \mathbf{B} \cdot \hat{\mathbf{S}}\]
    pub fn hamiltonian(&self, b_field_tesla: Vec3) -> ComplexMatrix {
        let sz = self.spin_z_operator();
        let (sx, sy, sz_ref) = self.spin_operators();

        // 1. Uniaxial term: D * S_z^2
        let sz2 = sz.mul(&sz_ref);
        let mut h = sz2.scale_real(self.d_anisotropy_ev);

        // 2. Rhombic transverse term: E * (S_x^2 - S_y^2) = 0.5 * E * (S_+^2 + S_-^2)
        if self.e_transverse_ev.abs() > 1e-15 {
            let sx2 = sx.mul(&sx);
            let sy2 = sy.mul(&sy);
            let rhombic = sx2.sub(&sy2).scale_real(self.e_transverse_ev);
            h = h.add(&rhombic);
        }

        // 3. Zeeman interaction: g * mu_B * (Bx * Sx + By * Sy + Bz * Sz)
        let zeeman_scale = self.g_factor * BOHR_MAGNETON_EV;
        if b_field_tesla.x.abs() > 1e-15 {
            let zx = sx.scale_real(zeeman_scale * b_field_tesla.x);
            h = h.add(&zx);
        }
        if b_field_tesla.y.abs() > 1e-15 {
            let zy = sy.scale_real(zeeman_scale * b_field_tesla.y);
            h = h.add(&zy);
        }
        if b_field_tesla.z.abs() > 1e-15 {
            let zz = sz.scale_real(zeeman_scale * b_field_tesla.z);
            h = h.add(&zz);
        }

        h.hermitian_symmetrize();
        h
    }

    /// Computes the exact energy eigenspectrum in $eV$ under magnetic field $\mathbf{B}$ ($T$).
    pub fn eigenvalues(&self, b_field_tesla: Vec3) -> Vec<f64> {
        let h = self.hamiltonian(b_field_tesla);
        h.eigenvalues_hermitian()
    }

    /// Evaluates the ground-state energy in $eV$.
    pub fn ground_state_energy(&self, b_field_tesla: Vec3) -> f64 {
        let evs = self.eigenvalues(b_field_tesla);
        evs[0]
    }

    /// Computes the resonant Zeeman fields $B_z^{(k)} = k \frac{|D|}{g \mu_B}$ (in Tesla)
    /// where quantum tunneling of magnetization (QTM) is resonantly activated.
    pub fn resonance_zeeman_fields(&self, max_k: usize) -> Vec<f64> {
        let delta_b = self.d_anisotropy_ev.abs() / (self.g_factor * BOHR_MAGNETON_EV);
        (0..=max_k).map(|k| (k as f64) * delta_b).collect()
    }

    /// Computes the avoided crossing energy splitting $\Delta_{QTM}$ (in $eV$) at resonance index $k$.
    pub fn tunneling_gap_at_resonance(&self, k: usize) -> f64 {
        let b_res = self.resonance_zeeman_fields(k)[k];
        let evs = self.eigenvalues(Vec3::new(0.0, 0.0, b_res));
        // The tunnel splitting occurs between the two states coming into resonance:
        if evs.len() >= 2 {
            (evs[1] - evs[0]).abs()
        } else {
            0.0
        }
    }

    /// Returns the expectation values $\langle S_z \rangle$ for the bistable magnetic states:
    /// Returns $(+S, -S)$ corresponding to binary '1' and binary '0'.
    pub fn bistable_states(&self) -> (f64, f64) {
        let s = self.spin.s_float();
        (s, -s)
    }
}
