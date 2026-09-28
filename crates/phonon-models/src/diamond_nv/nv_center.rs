//! Diamond Nitrogen-Vacancy (NV) Center Spin Hamiltonian & Magnetometry Models.
//!
//! Formulates ground-state Spin-1 triplet Hamiltonians, zero-field splitting (ZFS),
//! 4 crystallographic <111> diamond orientations, Zeeman shifts, and 3D vector
//! magnetic field reconstruction.

use std::f64::consts::PI;

/// Canonical crystallographic unit vectors for the 4 NV orientations in diamond:
/// $\mathbf{u}_1 = \frac{1}{\sqrt{3}}(1, 1, 1)$,
/// $\mathbf{u}_2 = \frac{1}{\sqrt{3}}(1, -1, -1)$,
/// $\mathbf{u}_3 = \frac{1}{\sqrt{3}}(-1, 1, -1)$,
/// $\mathbf{u}_4 = \frac{1}{\sqrt{3}}(-1, -1, 1)$.
pub const NV_AXES: [[f64; 3]; 4] = [
    [
        0.577_350_269_189_625_7,
        0.577_350_269_189_625_7,
        0.577_350_269_189_625_7,
    ],
    [
        0.577_350_269_189_625_7,
        -0.577_350_269_189_625_7,
        -0.577_350_269_189_625_7,
    ],
    [
        -0.577_350_269_189_625_7,
        0.577_350_269_189_625_7,
        -0.577_350_269_189_625_7,
    ],
    [
        -0.577_350_269_189_625_7,
        -0.577_350_269_189_625_7,
        0.577_350_269_189_625_7,
    ],
];

/// Diamond Nitrogen-Vacancy center physical configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct NvCenterConfig {
    /// Zero-field splitting (axial parameter) $D$ in Hertz (default 2.870 GHz).
    pub zero_field_splitting_hz: f64,
    /// Transverse strain splitting $E$ in Hertz (default 2.0 MHz).
    pub transverse_strain_hz: f64,
    /// Electron gyromagnetic ratio $\gamma_e$ in Hz/Tesla (default 28.024 GHz/T).
    pub electron_gyromagnetic_ratio_hz_per_t: f64,
    /// Optical spin polarization fidelity into $|m_s = 0\rangle$ (default 0.90).
    pub spin_polarization_fidelity: f64,
    /// Inhomogeneous spin dephasing time $T_2^*$ in seconds (default 2.5 us).
    pub dephasing_time_t2_star_s: f64,
    /// Hahn echo coherence time $T_2$ in seconds (default 250 us).
    pub coherence_time_t2_echo_s: f64,
    /// Longitudinal spin-lattice relaxation time $T_1$ in seconds (default 5 ms).
    pub relaxation_time_t1_s: f64,
    /// Shallow NV implantation depth from diamond surface in meters (default 5 nm).
    pub sensor_depth_m: f64,
}

impl Default for NvCenterConfig {
    fn default() -> Self {
        Self {
            zero_field_splitting_hz: 2.870e9,
            transverse_strain_hz: 2.0e6,
            electron_gyromagnetic_ratio_hz_per_t: 28.024e9,
            spin_polarization_fidelity: 0.90,
            dephasing_time_t2_star_s: 2.5e-6,
            coherence_time_t2_echo_s: 250.0e-6,
            relaxation_time_t1_s: 5.0e-3,
            sensor_depth_m: 5.0e-9,
        }
    }
}

impl NvCenterConfig {
    /// Creates a new NV center configuration with customized parameters.
    pub fn new(
        zero_field_splitting_hz: f64,
        transverse_strain_hz: f64,
        dephasing_time_t2_star_s: f64,
        coherence_time_t2_echo_s: f64,
        sensor_depth_m: f64,
    ) -> Self {
        Self {
            zero_field_splitting_hz,
            transverse_strain_hz,
            dephasing_time_t2_star_s,
            coherence_time_t2_echo_s,
            sensor_depth_m,
            ..Self::default()
        }
    }

    /// Computes the projected magnetic field along crystallographic axis $i \in \{0, 1, 2, 3\}$:
    /// $B_{||, i} = \mathbf{B} \cdot \hat{\mathbf{u}}_i$.
    #[inline]
    pub fn projected_field_tesla(&self, b_vector: [f64; 3], orientation_idx: usize) -> f64 {
        let u = NV_AXES[orientation_idx % 4];
        b_vector[0] * u[0] + b_vector[1] * u[1] + b_vector[2] * u[2]
    }

    /// Evaluates the two Zeeman transition frequencies $f_\pm = D \pm \sqrt{(\gamma_e B_{||})^2 + E^2}$
    /// for crystallographic axis $i$ under external field $\mathbf{B}$.
    pub fn transition_frequencies_hz(
        &self,
        b_vector: [f64; 3],
        orientation_idx: usize,
    ) -> (f64, f64) {
        let b_parallel = self.projected_field_tesla(b_vector, orientation_idx);
        let zeeman = self.electron_gyromagnetic_ratio_hz_per_t * b_parallel;
        let delta = (zeeman.powi(2) + self.transverse_strain_hz.powi(2)).sqrt();
        (
            self.zero_field_splitting_hz + delta,
            self.zero_field_splitting_hz - delta,
        )
    }

    /// Evaluates the frequency splitting $\Delta f = 2 \sqrt{(\gamma_e B_{||})^2 + E^2}$.
    #[inline]
    pub fn zeeman_splitting_hz(&self, b_parallel_tesla: f64) -> f64 {
        let zeeman = self.electron_gyromagnetic_ratio_hz_per_t * b_parallel_tesla;
        2.0 * (zeeman.powi(2) + self.transverse_strain_hz.powi(2)).sqrt()
    }

    /// Reconstructs the 3D vector magnetic field $\mathbf{B} = (B_x, B_y, B_z)$
    /// from the 4 projected parallel fields $B_{||, 0..3}$ using the exact Moore-Penrose pseudo-inverse:
    /// $\mathbf{B} = \frac{3}{4} \sum_{i=0}^3 B_{||, i} \hat{\mathbf{u}}_i$.
    pub fn reconstruct_vector_field(&self, projected_fields: [f64; 4]) -> [f64; 3] {
        let mut bx = 0.0;
        let mut by = 0.0;
        let mut bz = 0.0;
        for (i, &b_parallel) in projected_fields.iter().enumerate() {
            let u = NV_AXES[i];
            bx += b_parallel * u[0];
            by += b_parallel * u[1];
            bz += b_parallel * u[2];
        }
        let scale = 3.0 / 4.0;
        [bx * scale, by * scale, bz * scale]
    }

    /// Computes the shot-noise-limited magnetic sensitivity in $\text{T}/\sqrt{\text{Hz}}$:
    /// $\eta_B = \frac{1}{2\pi \gamma_e C \sqrt{I_{ph} T_2}}$.
    pub fn magnetic_sensitivity_t_per_rt_hz(&self, contrast: f64, photon_rate_cps: f64) -> f64 {
        let gamma_rad = 2.0 * PI * self.electron_gyromagnetic_ratio_hz_per_t;
        let c = contrast.clamp(0.01, 1.0);
        let n_ph = photon_rate_cps.max(1.0);
        let t2 = self.coherence_time_t2_echo_s.max(1e-9);
        1.0 / (gamma_rad * c * (n_ph * t2).sqrt())
    }
}
