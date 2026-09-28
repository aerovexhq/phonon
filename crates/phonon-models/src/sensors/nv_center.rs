//! Diamond Nitrogen-Vacancy (NV) Center Quantum Sensor Model
//!
//! Provides the ground-state spin-triplet ($S=1$) Hamiltonian for the diamond NV center:
//! \[\hat{H}_{NV} = D \hat{S}_z^2 + E(\hat{S}_x^2 - \hat{S}_y^2) + \gamma_e \mathbf{B} \cdot \hat{\mathbf{S}} + \hat{\mathbf{S}} \cdot \mathbf{A} \cdot \hat{\mathbf{I}} + P_n \hat{I}_z^2\]
//! with zero-field splitting $D \approx 2.87\text{ GHz}$, transverse strain $E$, electronic gyromagnetic
//! ratio $\gamma_e \approx 28.024\text{ GHz/T}$ ($28.024\text{ MHz/mT}$), and nuclear spin hyperfine coupling
//! ($^{14}\text{N}$ with $I=1$ and $P_n \approx -4.95\text{ MHz}$, or $^{15}\text{N}$ with $I=1/2$).
//!
//! Key capabilities:
//! 1. 4 crystallographic $\langle 111 \rangle$ diamond NV orientations allowing full 3D vector
//!    magnetic field reconstruction $\mathbf{B} = (B_x, B_y, B_z)$ from multi-resonance ODMR spectra.
//! 2. Optical spin polarization via green laser ($532\text{ nm}$) excitation and non-radiative
//!    intersystem crossing (ISC) through intermediate singlet states ($^1A_1, ^1E$), initializing
//!    the spin state into $|m_s = 0\rangle$ with high fidelity ($> 85\%$).
//! 3. Optically Detected Magnetic Resonance (ODMR) continuous-wave (CW) spectrum calculation across
//!    microwave frequencies with Lorentzian line broadening and photoluminescence contrast dips $C_\pm$.
//! 4. DC magnetic field sensitivity calculation:
//!    \[\eta_{DC} \approx \frac{\hbar}{g_e \mu_B} \frac{\Delta \nu}{C \sqrt{I_0 T_2^*}}\]

use crate::quantum::Complex;
use crate::spintronics::ciss::ComplexMatrix;
use crate::spintronics::Vec3;
use phonon_core::H_BAR;

/// Zero-field splitting $D$ at room temperature ($300\text{ K}$) in Hertz ($2.870\text{ GHz}$).
pub const NV_ZERO_FIELD_SPLITTING_D_HZ: f64 = 2.870e9;

/// Temperature coefficient of zero-field splitting $dD/dT$ in $\text{Hz/K}$ ($-74.2\text{ kHz/K}$).
pub const NV_D_TEMP_COEFFICIENT_HZ_PER_K: f64 = -74.2e3;

/// Electronic gyromagnetic ratio $\gamma_e = \frac{g_e \mu_B}{h}$ in $\text{Hz/T}$ ($28.024\text{ GHz/T}$ or $28.024\text{ MHz/mT}$).
pub const NV_ELECTRON_GYROMAGNETIC_RATIO_HZ_PER_T: f64 = 28.024e9;

/// Electron Landé g-factor for the diamond NV center ($g_e \approx 2.0028$).
pub const NV_ELECTRON_G_FACTOR: f64 = 2.0028;

/// Bohr magneton in Joules per Tesla ($J/T$).
pub const BOHR_MAGNETON_JOULES: f64 = 9.274_010_078_3e-24;

/// $^{14}\text{N}$ nuclear quadrupole interaction parameter $P_n$ in Hertz ($-4.95\text{ MHz}$).
pub const N14_QUADRUPOLE_SPLITTING_HZ: f64 = -4.95e6;

/// $^{14}\text{N}$ parallel hyperfine parameter $A_{||}$ in Hertz ($-2.16\text{ MHz}$).
pub const N14_HYPERFINE_PARALLEL_HZ: f64 = -2.16e6;

/// $^{14}\text{N}$ perpendicular hyperfine parameter $A_\perp$ in Hertz ($-2.70\text{ MHz}$).
pub const N14_HYPERFINE_PERP_HZ: f64 = -2.70e6;

/// $^{15}\text{N}$ parallel hyperfine parameter $A_{||}$ in Hertz ($3.03\text{ MHz}$).
pub const N15_HYPERFINE_PARALLEL_HZ: f64 = 3.03e6;

/// $^{15}\text{N}$ perpendicular hyperfine parameter $A_\perp$ in Hertz ($3.65\text{ MHz}$).
pub const N15_HYPERFINE_PERP_HZ: f64 = 3.65e6;

/// Diamond NV center crystallographic orientation along one of the 4 $\langle 111 \rangle$ diamond bond axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NvOrientation {
    /// Bond orientation along $[1, 1, 1] / \sqrt{3}$.
    V0,
    /// Bond orientation along $[1, -1, -1] / \sqrt{3}$.
    V1,
    /// Bond orientation along $[-1, 1, -1] / \sqrt{3}$.
    V2,
    /// Bond orientation along $[-1, -1, 1] / \sqrt{3}$.
    V3,
}

impl NvOrientation {
    /// Returns all 4 crystallographic $\langle 111 \rangle$ orientations.
    pub const fn all() -> [Self; 4] {
        [Self::V0, Self::V1, Self::V2, Self::V3]
    }

    /// Returns the unit vector $\mathbf{u}_i$ for this orientation in the diamond crystal Cartesian frame.
    pub fn unit_vector(self) -> Vec3 {
        let inv_sqrt3 = 1.0 / 3.0_f64.sqrt();
        match self {
            Self::V0 => Vec3::new(inv_sqrt3, inv_sqrt3, inv_sqrt3),
            Self::V1 => Vec3::new(inv_sqrt3, -inv_sqrt3, -inv_sqrt3),
            Self::V2 => Vec3::new(-inv_sqrt3, inv_sqrt3, -inv_sqrt3),
            Self::V3 => Vec3::new(-inv_sqrt3, -inv_sqrt3, inv_sqrt3),
        }
    }

    /// Projects an external 3D vector magnetic field $\mathbf{B}$ onto this NV defect axis:
    /// \[B_{NV} = \mathbf{B} \cdot \mathbf{u}_i\]
    pub fn project_field(self, b_field: Vec3) -> f64 {
        b_field.dot(self.unit_vector())
    }

    /// Constructs an orthonormal local coordinate frame $(\hat{x}_{loc}, \hat{y}_{loc}, \hat{z}_{loc})$
    /// where $\hat{z}_{loc}$ is aligned with the NV defect axis.
    pub fn local_frame(self) -> (Vec3, Vec3, Vec3) {
        let z_axis = self.unit_vector();
        let ref_vec = if z_axis.x.abs() < 0.9 {
            Vec3::X
        } else {
            Vec3::Y
        };
        let x_axis = ref_vec.cross(z_axis).normalize();
        let y_axis = z_axis.cross(x_axis).normalize();
        (x_axis, y_axis, z_axis)
    }

    /// Transforms a vector from the global diamond crystal Cartesian frame into this NV's local frame.
    pub fn to_local(self, v: Vec3) -> Vec3 {
        let (x_axis, y_axis, z_axis) = self.local_frame();
        Vec3::new(v.dot(x_axis), v.dot(y_axis), v.dot(z_axis))
    }
}

/// Nitrogen isotope coupled to the NV center.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NitrogenIsotope {
    /// $^{14}\text{N}$ isotope: nuclear spin $I=1$, quadrupole $P_n \approx -4.95\text{ MHz}$,
    /// hyperfine $A_{||} \approx -2.16\text{ MHz}$, $A_\perp \approx -2.70\text{ MHz}$.
    N14,
    /// $^{15}\text{N}$ isotope: nuclear spin $I=1/2$, quadrupole $P_n = 0$,
    /// hyperfine $A_{||} \approx 3.03\text{ MHz}$, $A_\perp \approx 3.65\text{ MHz}$.
    N15,
}

impl NitrogenIsotope {
    /// Nuclear spin quantum number $I$.
    pub fn nuclear_spin(self) -> f64 {
        match self {
            Self::N14 => 1.0,
            Self::N15 => 0.5,
        }
    }

    /// Hilbert space dimension of nuclear spin $(2I+1)$.
    pub fn nuclear_dim(self) -> usize {
        match self {
            Self::N14 => 3,
            Self::N15 => 2,
        }
    }

    /// Quadrupole splitting $P_n$ in Hertz.
    pub fn quadrupole_splitting_hz(self) -> f64 {
        match self {
            Self::N14 => N14_QUADRUPOLE_SPLITTING_HZ,
            Self::N15 => 0.0,
        }
    }

    /// Hyperfine parameters $(A_{||}, A_\perp)$ in Hertz.
    pub fn hyperfine_parameters_hz(self) -> (f64, f64) {
        match self {
            Self::N14 => (N14_HYPERFINE_PARALLEL_HZ, N14_HYPERFINE_PERP_HZ),
            Self::N15 => (N15_HYPERFINE_PARALLEL_HZ, N15_HYPERFINE_PERP_HZ),
        }
    }
}

/// Reconstructs the 3D vector magnetic field $\mathbf{B} = (B_x, B_y, B_z)$ in Tesla from
/// the 4 ODMR axial field projections $b_i = \mathbf{B} \cdot \mathbf{u}_i$ ($i \in \{0, 1, 2, 3\}$).
///
/// Exploits the tetrahedral completeness identity:
/// \[\sum_{i=0}^3 \mathbf{u}_i \mathbf{u}_i^T = \frac{4}{3} \mathbf{I}_{3\times 3} \implies \mathbf{B} = \frac{3}{4} \sum_{i=0}^3 b_i \mathbf{u}_i\]
pub fn reconstruct_vector_magnetic_field(projections: &[f64; 4]) -> Vec3 {
    let orientations = NvOrientation::all();
    let mut bx = 0.0;
    let mut by = 0.0;
    let mut bz = 0.0;
    for i in 0..4 {
        let u = orientations[i].unit_vector();
        let b_i = projections[i];
        bx += b_i * u.x;
        by += b_i * u.y;
        bz += b_i * u.z;
    }
    Vec3::new(bx * 0.75, by * 0.75, bz * 0.75)
}

/// Diamond Nitrogen-Vacancy (NV) Center Quantum Sensor.
#[derive(Debug, Clone, PartialEq)]
pub struct NvCenter {
    /// Defect crystallographic axis orientation.
    pub orientation: NvOrientation,
    /// Zero-field splitting $D$ in Hertz.
    pub d_splitting_hz: f64,
    /// Transverse rhombic strain / electric field splitting $E$ in Hertz.
    pub e_strain_hz: f64,
    /// Nitrogen isotope coupling.
    pub isotope: NitrogenIsotope,
    /// Optical spin polarization fidelity into $|m_s = 0\rangle$ under $532\text{ nm}$ laser (typically $> 0.85$).
    pub polarization_fidelity: f64,
    /// Longitudinal spin-lattice relaxation time $T_1$ in seconds (typically $\sim 5\text{ ms}$).
    pub t1_relaxation_time_s: f64,
    /// Inhomogeneous dephasing time $T_2^*$ in seconds (typically $\sim 1\,\mu\text{s}$).
    pub t2_star_time_s: f64,
    /// Coherence time under dynamical decoupling $T_2$ in seconds (typically $> 500\,\mu\text{s}$).
    pub t2_coherence_time_s: f64,
    /// Operating temperature in Kelvin.
    pub temperature_k: f64,
}

impl Default for NvCenter {
    fn default() -> Self {
        Self::new(NvOrientation::V0, NitrogenIsotope::N14)
    }
}

/// Configuration parameters for continuous-wave ODMR spectrum calculation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OdmrConfig {
    /// Starting frequency in Hertz.
    pub freq_start_hz: f64,
    /// Ending frequency in Hertz.
    pub freq_end_hz: f64,
    /// Number of sampled frequency points.
    pub num_points: usize,
    /// Excitation laser optical power in milliwatts.
    pub laser_power_mw: f64,
    /// Maximum fractional contrast dip depth ($0.01$ to $0.30$).
    pub contrast_fraction: f64,
    /// Full-width at half-maximum (FWHM) linewidth in Hertz.
    pub linewidth_fwhm_hz: f64,
}

impl Default for OdmrConfig {
    fn default() -> Self {
        Self {
            freq_start_hz: 2.80e9,
            freq_end_hz: 2.94e9,
            num_points: 300,
            laser_power_mw: 5.0,
            contrast_fraction: 0.15,
            linewidth_fwhm_hz: 10.0e6,
        }
    }
}

impl NvCenter {
    /// Creates a new NV center with standard diamond room-temperature physical parameters.
    pub fn new(orientation: NvOrientation, isotope: NitrogenIsotope) -> Self {
        Self {
            orientation,
            d_splitting_hz: NV_ZERO_FIELD_SPLITTING_D_HZ,
            e_strain_hz: 0.0,
            isotope,
            polarization_fidelity: 0.90,
            t1_relaxation_time_s: 5.0e-3,
            t2_star_time_s: 1.0e-6,
            t2_coherence_time_s: 500.0e-6,
            temperature_k: 300.0,
        }
    }

    /// Sets operating temperature in Kelvin and adjusts zero-field splitting $D(T)$.
    pub fn with_temperature(mut self, temp_k: f64) -> Self {
        self.temperature_k = temp_k;
        let delta_t = temp_k - 300.0;
        self.d_splitting_hz =
            NV_ZERO_FIELD_SPLITTING_D_HZ + NV_D_TEMP_COEFFICIENT_HZ_PER_K * delta_t;
        self
    }

    /// Sets transverse strain parameter $E$ in Hertz.
    pub fn with_strain_hz(mut self, e_strain_hz: f64) -> Self {
        self.e_strain_hz = e_strain_hz;
        self
    }

    /// Sets dephasing times $T_2^*$ and coherence time $T_2$ in seconds.
    pub fn with_coherence_times(mut self, t2_star_s: f64, t2_s: f64) -> Self {
        self.t2_star_time_s = t2_star_s;
        self.t2_coherence_time_s = t2_s;
        self
    }

    /// Spin operators $(\hat{S}_x, \hat{S}_y, \hat{S}_z)$ for spin-triplet $S=1$ in the
    /// basis $\{|m_s = +1\rangle, |m_s = 0\rangle, |m_s = -1\rangle\}$.
    pub fn spin_operators() -> (ComplexMatrix, ComplexMatrix, ComplexMatrix) {
        let mut sz = ComplexMatrix::zeros(3);
        sz.set(0, 0, Complex::new(1.0, 0.0));
        sz.set(1, 1, Complex::new(0.0, 0.0));
        sz.set(2, 2, Complex::new(-1.0, 0.0));

        let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
        let mut sx = ComplexMatrix::zeros(3);
        sx.set(0, 1, Complex::new(inv_sqrt2, 0.0));
        sx.set(1, 0, Complex::new(inv_sqrt2, 0.0));
        sx.set(1, 2, Complex::new(inv_sqrt2, 0.0));
        sx.set(2, 1, Complex::new(inv_sqrt2, 0.0));

        let mut sy = ComplexMatrix::zeros(3);
        sy.set(0, 1, Complex::new(0.0, -inv_sqrt2));
        sy.set(1, 0, Complex::new(0.0, inv_sqrt2));
        sy.set(1, 2, Complex::new(0.0, -inv_sqrt2));
        sy.set(2, 1, Complex::new(0.0, inv_sqrt2));

        (sx, sy, sz)
    }

    /// Spin operators $(\hat{I}_x, \hat{I}_y, \hat{I}_z)$ for nuclear spin $I$.
    pub fn nuclear_spin_operators(
        isotope: NitrogenIsotope,
    ) -> (ComplexMatrix, ComplexMatrix, ComplexMatrix) {
        match isotope {
            NitrogenIsotope::N14 => {
                // I = 1, same algebraic matrix form as S = 1
                Self::spin_operators()
            }
            NitrogenIsotope::N15 => {
                // I = 1/2, Pauli matrices / 2 in basis {|+1/2>, |-1/2>}
                let mut iz = ComplexMatrix::zeros(2);
                iz.set(0, 0, Complex::new(0.5, 0.0));
                iz.set(1, 1, Complex::new(-0.5, 0.0));

                let mut ix = ComplexMatrix::zeros(2);
                ix.set(0, 1, Complex::new(0.5, 0.0));
                ix.set(1, 0, Complex::new(0.5, 0.0));

                let mut iy = ComplexMatrix::zeros(2);
                iy.set(0, 1, Complex::new(0.0, -0.5));
                iy.set(1, 0, Complex::new(0.0, 0.5));

                (ix, iy, iz)
            }
        }
    }

    /// Computes the $3 \times 3$ ground-state electronic spin Hamiltonian in frequency units ($\text{Hz}$):
    /// \[\hat{H}_{elec} = D \hat{S}_z^2 + E (\hat{S}_x^2 - \hat{S}_y^2) + \gamma_e (B_x \hat{S}_x + B_y \hat{S}_y + B_z \hat{S}_z)\]
    /// where $\mathbf{B}_{loc} = (B_x, B_y, B_z)$ is the magnetic field evaluated in the NV local defect frame.
    pub fn electronic_hamiltonian(&self, b_field_global: Vec3) -> ComplexMatrix {
        let b_loc = self.orientation.to_local(b_field_global);
        let (sx, sy, sz) = Self::spin_operators();

        let sz2 = sz.mul(&sz);
        let sx2 = sx.mul(&sx);
        let sy2 = sy.mul(&sy);
        let s_transverse = sx2.sub(&sy2);

        // H = D * Sz^2 + E * (Sx^2 - Sy^2) + gamma_e * (Bx*Sx + By*Sy + Bz*Sz)
        let term_d = sz2.scale_real(self.d_splitting_hz);
        let term_e = s_transverse.scale_real(self.e_strain_hz);
        let zeeman_x = sx.scale_real(NV_ELECTRON_GYROMAGNETIC_RATIO_HZ_PER_T * b_loc.x);
        let zeeman_y = sy.scale_real(NV_ELECTRON_GYROMAGNETIC_RATIO_HZ_PER_T * b_loc.y);
        let zeeman_z = sz.scale_real(NV_ELECTRON_GYROMAGNETIC_RATIO_HZ_PER_T * b_loc.z);

        term_d
            .add(&term_e)
            .add(&zeeman_x)
            .add(&zeeman_y)
            .add(&zeeman_z)
    }

    /// Computes the sorted real energy eigenvalues ($E_0 \le E_1 \le E_2$) of the electronic Hamiltonian in Hertz.
    pub fn electronic_eigenvalues(&self, b_field_global: Vec3) -> Vec<f64> {
        let h = self.electronic_hamiltonian(b_field_global);
        h.eigenvalues_hermitian()
    }

    /// Computes the full coupled electron-nuclear spin Hamiltonian in Hertz ($9\times 9$ for $^{14}\text{N}$, $6\times 6$ for $^{15}\text{N}$):
    /// \[\hat{H} = \hat{H}_{elec} \otimes \mathbf{I}_n + \mathbf{I}_e \otimes (P_n \hat{I}_z^2) + A_{||} \hat{S}_z \otimes \hat{I}_z + A_\perp (\hat{S}_x \otimes \hat{I}_x + \hat{S}_y \otimes \hat{I}_y)\]
    pub fn coupled_hamiltonian(&self, b_field_global: Vec3) -> ComplexMatrix {
        let h_elec = self.electronic_hamiltonian(b_field_global);
        let n_dim = self.isotope.nuclear_dim();
        let tot_dim = 3 * n_dim;

        let (sx, sy, sz) = Self::spin_operators();
        let (ix, iy, iz) = Self::nuclear_spin_operators(self.isotope);

        let (a_par, a_perp) = self.isotope.hyperfine_parameters_hz();
        let p_n = self.isotope.quadrupole_splitting_hz();

        let mut h_tot = ComplexMatrix::zeros(tot_dim);

        // Term 1: H_elec (x) I_n
        for es1 in 0..3 {
            for es2 in 0..3 {
                let elem = h_elec.get(es1, es2);
                if elem.norm_sq() > 1e-30 {
                    for ns in 0..n_dim {
                        let r = es1 * n_dim + ns;
                        let c = es2 * n_dim + ns;
                        h_tot.add_at(r, c, elem);
                    }
                }
            }
        }

        // Term 2: I_e (x) (P_n * Iz^2)
        if p_n.abs() > 1e-12 {
            let iz2 = iz.mul(&iz);
            for es in 0..3 {
                for ns1 in 0..n_dim {
                    for ns2 in 0..n_dim {
                        let elem = iz2.get(ns1, ns2).scale(p_n);
                        if elem.norm_sq() > 1e-30 {
                            let r = es * n_dim + ns1;
                            let c = es * n_dim + ns2;
                            h_tot.add_at(r, c, elem);
                        }
                    }
                }
            }
        }

        // Term 3: A_par * Sz (x) Iz
        for es1 in 0..3 {
            for es2 in 0..3 {
                let sz_val = sz.get(es1, es2);
                if sz_val.norm_sq() > 1e-30 {
                    for ns1 in 0..n_dim {
                        for ns2 in 0..n_dim {
                            let iz_val = iz.get(ns1, ns2);
                            let elem = (sz_val * iz_val).scale(a_par);
                            if elem.norm_sq() > 1e-30 {
                                let r = es1 * n_dim + ns1;
                                let c = es2 * n_dim + ns2;
                                h_tot.add_at(r, c, elem);
                            }
                        }
                    }
                }
            }
        }

        // Term 4: A_perp * (Sx (x) Ix + Sy (x) Iy)
        for es1 in 0..3 {
            for es2 in 0..3 {
                let sx_val = sx.get(es1, es2);
                let sy_val = sy.get(es1, es2);
                for ns1 in 0..n_dim {
                    for ns2 in 0..n_dim {
                        let ix_val = ix.get(ns1, ns2);
                        let iy_val = iy.get(ns1, ns2);
                        let elem = (sx_val * ix_val + sy_val * iy_val).scale(a_perp);
                        if elem.norm_sq() > 1e-30 {
                            let r = es1 * n_dim + ns1;
                            let c = es2 * n_dim + ns2;
                            h_tot.add_at(r, c, elem);
                        }
                    }
                }
            }
        }

        h_tot.hermitian_symmetrize();
        h_tot
    }

    /// Computes the exact coupled electron-nuclear eigenfrequencies in Hertz.
    pub fn coupled_eigenvalues(&self, b_field_global: Vec3) -> Vec<f64> {
        let h = self.coupled_hamiltonian(b_field_global);
        h.eigenvalues_hermitian()
    }

    /// Analytical first-order resonance frequencies $f_\pm$ in Hertz for continuous-wave microwave transitions:
    /// \[f_\pm \approx D \pm \sqrt{(\gamma_e B_{NV})^2 + E^2}\]
    /// where $B_{NV} = \mathbf{B} \cdot \mathbf{u}$ is the axial magnetic field.
    pub fn odmr_resonance_frequencies(&self, b_field_global: Vec3) -> (f64, f64) {
        let b_nv = self.orientation.project_field(b_field_global);
        let zeeman_hz = NV_ELECTRON_GYROMAGNETIC_RATIO_HZ_PER_T * b_nv;
        let delta = (zeeman_hz * zeeman_hz + self.e_strain_hz * self.e_strain_hz).sqrt();
        (self.d_splitting_hz - delta, self.d_splitting_hz + delta)
    }

    /// Simulates optical spin polarization under continuous $532\text{ nm}$ green laser excitation.
    ///
    /// The optical cycle excites triplet $^3A_2 \to ^3E$, followed by spin-dependent non-radiative
    /// intersystem crossing (ISC) via intermediate singlet states ($^1A_1, ^1E$).
    /// Returns the steady-state ground state spin populations $[P(|+1\rangle), P(|0\rangle), P(|-1\rangle)]$.
    pub fn simulate_optical_pumping(&self, laser_power_mw: f64) -> [f64; 3] {
        // Optical excitation saturation at 532 nm (saturation power Psat ~ 0.5 mW)
        let sat_factor = (1.0 - (-laser_power_mw.max(0.0) / 0.8).exp()).clamp(0.0, 1.0);
        // Polarization into |m_s = 0> transitions from thermal 1/3 to polarization_fidelity
        let p0 = (0.3333333333333333
            + (self.polarization_fidelity - 0.3333333333333333) * sat_factor)
            .clamp(0.3333333333333333, 0.98);
        let p_pm = (1.0 - p0) * 0.5;
        [p_pm, p0, p_pm]
    }

    /// Calculates the continuous-wave (CW) ODMR photoluminescence (PL) spectrum across microwave frequencies.
    ///
    /// Photoluminescence dips occur at the resonance frequencies $f_\pm$:
    /// \[PL(f) = PL_0 \left[1 - \sum_{\pm} C_\pm \frac{(\Delta \nu / 2)^2}{(f - f_\pm)^2 + (\Delta \nu / 2)^2}\right]\]
    pub fn compute_odmr_spectrum(&self, config: &OdmrConfig, b_field_global: Vec3) -> OdmrSpectrum {
        assert!(
            config.num_points >= 2,
            "Requires at least 2 frequency sample points"
        );
        let (f_minus, f_plus) = self.odmr_resonance_frequencies(b_field_global);

        let half_gamma = config.linewidth_fwhm_hz * 0.5;
        let half_gamma_sq = half_gamma * half_gamma;

        let pops = self.simulate_optical_pumping(config.laser_power_mw);
        let base_contrast = config.contrast_fraction * (pops[1] - pops[0]).max(0.05);

        let df = (config.freq_end_hz - config.freq_start_hz) / (config.num_points - 1) as f64;
        let mut frequencies = Vec::with_capacity(config.num_points);
        let mut pl_signal = Vec::with_capacity(config.num_points);

        for i in 0..config.num_points {
            let f = config.freq_start_hz + i as f64 * df;
            frequencies.push(f);

            let dip_minus = base_contrast * half_gamma_sq / ((f - f_minus).powi(2) + half_gamma_sq);
            let dip_plus = base_contrast * half_gamma_sq / ((f - f_plus).powi(2) + half_gamma_sq);
            let pl = 1.0 - (dip_minus + dip_plus);
            pl_signal.push(pl);
        }

        OdmrSpectrum {
            frequencies_hz: frequencies,
            photoluminescence: pl_signal,
            resonance_frequencies_hz: vec![f_minus, f_plus],
            contrast_dips: vec![base_contrast, base_contrast],
            linewidth_hz: config.linewidth_fwhm_hz,
        }
    }

    /// Evaluates DC magnetic field sensitivity $\eta_{DC}$ in $\text{T}/\sqrt{\text{Hz}}$:
    /// \[\eta_{DC} \approx \frac{\hbar}{g_e \mu_B} \frac{\Delta \nu}{C \sqrt{I_0 T_2^*}}\]
    ///
    /// - `linewidth_hz`: Resonance linewidth $\Delta \nu$ (typically $1$ to $10\text{ MHz}$).
    /// - `contrast`: Optical contrast $C$ ($0.01$ to $0.20$).
    /// - `photon_count_rate_cps`: Photoluminescence collection rate $I_0$ in counts per second (e.g. $10^5$ to $10^7\text{ cps}$).
    pub fn dc_magnetic_sensitivity(
        &self,
        linewidth_hz: f64,
        contrast: f64,
        photon_count_rate_cps: f64,
    ) -> f64 {
        let c_eff = contrast.clamp(1e-4, 1.0);
        let i0 = photon_count_rate_cps.max(1.0);
        let t2_star = self.t2_star_time_s.max(1e-9);

        // prefactor = hbar / (g_e * mu_B)
        let prefactor = H_BAR / (NV_ELECTRON_G_FACTOR * BOHR_MAGNETON_JOULES);
        (prefactor * linewidth_hz) / (c_eff * (i0 * t2_star).sqrt())
    }
}

/// Continuous-wave Optically Detected Magnetic Resonance (ODMR) spectrum.
#[derive(Debug, Clone, PartialEq)]
pub struct OdmrSpectrum {
    /// Sampled microwave frequencies in Hertz.
    pub frequencies_hz: Vec<f64>,
    /// Normalized photoluminescence (PL) intensity values.
    pub photoluminescence: Vec<f64>,
    /// Peak resonance dip frequencies in Hertz.
    pub resonance_frequencies_hz: Vec<f64>,
    /// Contrast of resonance dips.
    pub contrast_dips: Vec<f64>,
    /// Full-width at half-maximum (FWHM) linewidth in Hertz.
    pub linewidth_hz: f64,
}

impl OdmrSpectrum {
    /// Minimum photoluminescence (deepest resonance dip).
    pub fn min_pl(&self) -> f64 {
        self.photoluminescence
            .iter()
            .copied()
            .fold(1.0_f64, |acc, x| acc.min(x))
    }

    /// Maximum contrast dip depth ($1 - \min(PL)$).
    pub fn max_contrast(&self) -> f64 {
        1.0 - self.min_pl()
    }
}
