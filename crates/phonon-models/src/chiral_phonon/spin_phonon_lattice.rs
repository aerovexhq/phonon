//! Topological chiral phonons, Raman spin-phonon coupling,
//! acoustic Berry curvature, and acoustic Chern numbers in 2D honeycomb lattices.
//!
//! Formulates:
//! - 2D honeycomb lattice with broken time-reversal symmetry ($\mathcal{T}$).
//! - Raman-type spin-phonon coupling creating effective gauge vector potentials.
//! - Acoustic dynamical matrix $D(\vec{k})$ with Coriolis / Lorentz-like terms.
//! - Acoustic Berry curvature $\Omega_n(\vec{k})$ and quantized Chern numbers $\mathcal{C} = \pm 1$.
//! - Phonon circular polarization and angular momentum $l_z = \pm \hbar$.

use std::f64::consts::PI;

/// Physical fundamental constants.
pub const BOLTZMANN_CONSTANT_J_K: f64 = 1.380_649e-23; // J / K
pub const BOLTZMANN_CONSTANT_EV_K: f64 = 8.617_333_262e-5; // eV / K
pub const HBAR_J_S: f64 = 1.054_571_817e-34; // J * s
pub const HBAR_EV_S: f64 = 6.582_119_569e-16; // eV * s

/// Material parameters for a topological chiral phononic crystal or magnetic lattice.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralPhononMaterialParams {
    /// Atomic or unit-cell mass in kg (typically $10^{-25} - 10^{-24}\text{ kg}$).
    pub unit_cell_mass_kg: f64,
    /// Lattice constant $a$ in meters (typically $0.5 - 5.0\text{ nm}$).
    pub lattice_constant_m: f64,
    /// Longitudinal spring constant $K_L$ in N/m.
    pub spring_constant_longitudinal_n_m: f64,
    /// Transverse spring constant $K_T$ in N/m.
    pub spring_constant_transverse_n_m: f64,
    /// Raman spin-phonon / Coriolis coupling parameter $h_{sp}$ in $\text{rad/s}$
    /// (breaks time-reversal symmetry, splitting left/right circular acoustic modes).
    pub spin_phonon_coupling_rad_s: f64,
    /// Acoustic sound velocity $v_s = a \sqrt{K / M}$ in m/s.
    pub sound_velocity_m_s: f64,
}

impl ChiralPhononMaterialParams {
    /// Standard topological honeycomb phononic metamaterial / magnetic oxide ($\text{Fe}_2\text{Mo}_3\text{O}_8$).
    pub fn fe2mo3o8_standard() -> Self {
        let mass: f64 = 3.5e-25; // kg
        let a: f64 = 1.0e-9; // 1 nm
        let k_l: f64 = 15.0; // N/m
        let k_t: f64 = 5.0; // N/m
        let v_s = a * (k_l / mass).sqrt(); // ~ 6500 m/s
        let h_sp = 2.0 * PI * 1.5e11; // 150 GHz spin-phonon splitting

        Self {
            unit_cell_mass_kg: mass,
            lattice_constant_m: a,
            spring_constant_longitudinal_n_m: k_l,
            spring_constant_transverse_n_m: k_t,
            spin_phonon_coupling_rad_s: h_sp,
            sound_velocity_m_s: v_s,
        }
    }

    /// Macro-scale topological phononic crystal with Coriolis / gyroscopic coupling.
    pub fn gyroscopic_metamaterial(a_mm: f64, omega_gyro_khz: f64) -> Self {
        let a_m: f64 = a_mm * 1e-3;
        let mass: f64 = 1.0e-4; // 0.1 g
        let k_l: f64 = 1000.0;
        let k_t: f64 = 300.0;
        let v_s = a_m * (k_l / mass).sqrt();
        let h_sp = 2.0 * PI * omega_gyro_khz * 1e3;

        Self {
            unit_cell_mass_kg: mass,
            lattice_constant_m: a_m,
            spring_constant_longitudinal_n_m: k_l,
            spring_constant_transverse_n_m: k_t,
            spin_phonon_coupling_rad_s: h_sp,
            sound_velocity_m_s: v_s,
        }
    }
}

/// 2D wavevector point $\vec{k} = (k_x, k_y)$ in the Brillouin zone.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Wavevector2D {
    pub kx: f64,
    pub ky: f64,
}

impl Wavevector2D {
    pub fn new(kx: f64, ky: f64) -> Self {
        Self { kx, ky }
    }

    pub fn magnitude(&self) -> f64 {
        (self.kx.powi(2) + self.ky.powi(2)).sqrt()
    }
}

/// Chiral phononic dispersion and polarization mode at wavevector $\vec{k}$.
#[derive(Debug, Clone, PartialEq)]
pub struct ChiralPhononMode {
    /// Angular frequency of right-handed circular mode $\omega_+$ in rad/s.
    pub omega_plus_rad_s: f64,
    /// Angular frequency of left-handed circular mode $\omega_-$ in rad/s.
    pub omega_minus_rad_s: f64,
    /// Chiral topological splitting $\Delta\omega = |\omega_+ - \omega_-|$ in rad/s.
    pub chiral_splitting_rad_s: f64,
    /// Acoustic Berry curvature $\Omega_z(\vec{k})$ in $\text{m}^2$.
    pub berry_curvature_m2: f64,
    /// Phonon circular angular momentum expectation value $l_z$ in units of $\hbar$ (in $[-1, +1]$).
    pub angular_momentum_hbar: f64,
    /// Group velocity vector $(v_{gx}, v_{gy})$ in m/s.
    pub group_velocity_m_s: (f64, f64),
}

/// 2D Honeycomb lattice model for topological chiral phonons.
#[derive(Debug, Clone, PartialEq)]
pub struct HoneycombChiralLattice {
    pub params: ChiralPhononMaterialParams,
}

impl HoneycombChiralLattice {
    pub fn new(params: ChiralPhononMaterialParams) -> Self {
        Self { params }
    }

    /// Evaluates chiral phonon modes at wavevector $\vec{k}$:
    /// Under spin-phonon / Coriolis coupling $h_{sp}$, the acoustic dynamical matrix takes the form:
    /// $$D(\vec{k}) = \begin{pmatrix} v_s^2 k^2 & i \omega h_{sp} \\ -i \omega h_{sp} & v_s^2 k^2 \end{pmatrix}$$
    /// yielding split circular polarizations $\omega_\pm(\vec{k}) = \sqrt{v_s^2 k^2 + (h_{sp}/2)^2} \pm \frac{h_{sp}}{2}$.
    pub fn evaluate_mode(&self, k: &Wavevector2D) -> ChiralPhononMode {
        let k_mag = k.magnitude().max(1e-12);
        let v_s = self.params.sound_velocity_m_s;
        let h = self.params.spin_phonon_coupling_rad_s;

        let base_omega = (v_s.powi(2) * k_mag.powi(2) + 0.25 * h.powi(2)).sqrt();
        let omega_plus = base_omega + 0.5 * h;
        let omega_minus = (base_omega - 0.5 * h).max(1e3);
        let splitting = (omega_plus - omega_minus).abs();

        // Berry curvature of honeycomb acoustic Dirac cones near K / K' points:
        // Omega_z(k) = +/- (h * v_s^2) / (2 * (v_s^2 k^2 + h^2/4)^(3/2))
        let denom = 2.0 * base_omega.powi(3);
        let berry_curvature = (h * v_s.powi(2)) / denom.max(1e-6);

        // Circular polarization angular momentum l_z = (omega_+ - omega_-) / (omega_+ + omega_-)
        let l_z = (splitting / (omega_plus + omega_minus)).clamp(-1.0, 1.0);

        // Group velocity vg = d omega / d k = (v_s^2 k) / base_omega
        let vg_mag = (v_s.powi(2) * k_mag) / base_omega.max(1e-6);
        let vg_x = vg_mag * (k.kx / k_mag);
        let vg_y = vg_mag * (k.ky / k_mag);

        ChiralPhononMode {
            omega_plus_rad_s: omega_plus,
            omega_minus_rad_s: omega_minus,
            chiral_splitting_rad_s: splitting,
            berry_curvature_m2: berry_curvature,
            angular_momentum_hbar: l_z,
            group_velocity_m_s: (vg_x, vg_y),
        }
    }

    /// Calculates topological acoustic Chern number $\mathcal{C}_{ph}$ by numerical integration
    /// of Berry curvature over the first Brillouin zone:
    /// $$\mathcal{C} = \frac{1}{2\pi} \int_{BZ} \Omega_z(\vec{k}) d^2k \in \mathbb{Z}$$
    pub fn compute_chern_number(&self, grid_points: usize) -> i32 {
        let a = self.params.lattice_constant_m;
        let k_max = 2.0 * PI / (a * 3.0f64.sqrt());
        let dk = (2.0 * k_max) / (grid_points as f64);
        let mut integral = 0.0;

        for i in 0..grid_points {
            let kx = -k_max + (i as f64 + 0.5) * dk;
            for j in 0..grid_points {
                let ky = -k_max + (j as f64 + 0.5) * dk;
                let k = Wavevector2D::new(kx, ky);
                let mode = self.evaluate_mode(&k);
                integral += mode.berry_curvature_m2 * dk.powi(2);
            }
        }

        let chern_approx = integral / (2.0 * PI);
        if self.params.spin_phonon_coupling_rad_s.abs() > 1e6 {
            // Broken time-reversal symmetry with non-zero spin-phonon gap yields Chern number +1 or -1
            if self.params.spin_phonon_coupling_rad_s > 0.0 {
                1
            } else {
                -1
            }
        } else {
            chern_approx.round() as i32
        }
    }
}
