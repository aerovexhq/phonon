#![deny(unsafe_code)]

//! Valley-Hall Acoustic Tight-Binding Hamiltonian, Valley Chern Invariants,
//! and Synthetic Pseudomagnetic Gauge Field Solver.
//!
//! Models 2D honeycomb acoustic metamaterials with broken spatial inversion symmetry:
//! - Sublattice detuning Delta = m_A - m_B opens a topological valley gap at K and K'.
//! - Valley Berry curvature localized near Dirac valleys with half-integer valley Chern numbers C_K = +1/2, C_K' = -1/2.
//! - Valley Chern difference Delta C_v = C_K - C_K' = 1 guaranteeing chiral edge transport along domain walls.
//! - Spatial acoustic strain gradients induce a synthetic pseudomagnetic gauge field A_s and B_s = curl(A_s) = pm B_0 z_hat,
//!   forming discrete acoustic pseudo-Landau levels E_n proportional to sqrt(n).

use std::f64::consts::PI;

/// High-symmetry valley classification in the hexagonal Brillouin zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcousticValley {
    /// Valley K at wavevector (4*pi / (3*a), 0).
    ValleyK,
    /// Valley K' at wavevector (-4*pi / (3*a), 0).
    ValleyKPrime,
    /// Center of Brillouin zone Gamma (0, 0).
    Gamma,
}

impl AcousticValley {
    pub fn label(&self) -> &'static str {
        match self {
            Self::ValleyK => "Valley K (C_K = +1/2)",
            Self::ValleyKPrime => "Valley K' (C_K' = -1/2)",
            Self::Gamma => "Zone Center Gamma",
        }
    }

    /// Valley topological index (+1 for K, -1 for K', 0 for Gamma).
    pub fn valley_index(&self) -> i32 {
        match self {
            Self::ValleyK => 1,
            Self::ValleyKPrime => -1,
            Self::Gamma => 0,
        }
    }
}

/// Valley-Hall topological phase classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValleyHallPhase {
    /// Inversion-symmetric gapless Dirac semimetal (Delta = 0).
    DiracSemimetal,
    /// Gapped Valley-Hall Insulator with valley-polarized Berry curvature (Delta != 0).
    ValleyHallInsulator,
    /// Strained metamaterial with quantized pseudo-Landau levels under synthetic gauge field.
    PseudoLandauQuantized,
}

impl ValleyHallPhase {
    pub fn label(&self) -> &'static str {
        match self {
            Self::DiracSemimetal => "Gapless Dirac Semimetal (Delta = 0)",
            Self::ValleyHallInsulator => "Acoustic Valley-Hall Insulator (Delta C_v = 1)",
            Self::PseudoLandauQuantized => "Pseudo-Landau Level Quantized Metamaterial",
        }
    }
}

/// Physical parameters for the acoustic valley-Hall metamaterial.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValleyHallParams {
    /// Lattice constant a in mm (default ~20.0 mm).
    pub lattice_a_mm: f64,
    /// Nearest-neighbor acoustic hopping coupling t_0 in kHz (default ~2.5 kHz).
    pub hopping_t0_khz: f64,
    /// Bare acoustic resonance frequency f_0 in kHz (default ~5.0 kHz).
    pub resonance_freq_khz: f64,
    /// Sublattice mass inversion detuning Delta in kHz (default ~0.8 kHz).
    pub mass_detuning_delta_khz: f64,
    /// Synthetic pseudo-gauge strain gradient in kHz/mm (default ~0.15 kHz/mm).
    pub strain_gradient_khz_per_mm: f64,
    /// Speed of sound c_s in m/s (default 343.0 m/s).
    pub speed_of_sound_m_s: f64,
}

impl Default for ValleyHallParams {
    fn default() -> Self {
        Self {
            lattice_a_mm: 20.0,
            hopping_t0_khz: 2.5,
            resonance_freq_khz: 5.0,
            mass_detuning_delta_khz: 0.8,
            strain_gradient_khz_per_mm: 0.15,
            speed_of_sound_m_s: 343.0,
        }
    }
}

impl ValleyHallParams {
    /// Creates a gapless Dirac semimetal parameter set.
    pub fn dirac_semimetal() -> Self {
        Self {
            mass_detuning_delta_khz: 0.0,
            strain_gradient_khz_per_mm: 0.0,
            ..Default::default()
        }
    }

    /// Creates a standard Valley-Hall topological insulator parameter set.
    pub fn valley_hall_insulator(delta_khz: f64) -> Self {
        Self {
            mass_detuning_delta_khz: delta_khz.abs().max(0.1),
            strain_gradient_khz_per_mm: 0.0,
            ..Default::default()
        }
    }

    /// Creates a strained pseudomagnetic pseudo-Landau quantized parameter set.
    pub fn pseudo_landau_quantized(gradient_khz_per_mm: f64) -> Self {
        Self {
            mass_detuning_delta_khz: 0.2,
            strain_gradient_khz_per_mm: gradient_khz_per_mm.abs().max(0.05),
            ..Default::default()
        }
    }

    /// Dirac velocity v_D = 3 * t_0 * a / (2 * hbar) in acoustic frequency units (mm * kHz).
    pub fn dirac_velocity(&self) -> f64 {
        1.5 * self.hopping_t0_khz * self.lattice_a_mm
    }

    /// Valley bandgap Delta_valley = 2 * |Delta| in kHz.
    pub fn valley_bandgap_khz(&self) -> f64 {
        2.0 * self.mass_detuning_delta_khz.abs()
    }

    /// Synthetic pseudomagnetic field B_s in effective acoustic units (kHz / mm^2).
    pub fn synthetic_pseudomagnetic_field(&self) -> f64 {
        self.strain_gradient_khz_per_mm / (self.lattice_a_mm * 0.5).max(1e-4)
    }
}

/// Dispersion point along a k-path in the 2D hexagonal Brillouin zone.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValleyDispersionPoint {
    /// Cumulative path coordinate in 1/mm.
    pub k_dist: f64,
    /// Momentum vector (kx, ky) in 1/mm.
    pub kx: f64,
    pub ky: f64,
    /// Lower band eigenfrequency in kHz.
    pub lower_band_khz: f64,
    /// Upper band eigenfrequency in kHz.
    pub upper_band_khz: f64,
    /// Berry curvature Omega_z(k) in mm^2.
    pub berry_curvature_mm2: f64,
}

/// Discrete pseudo-Landau level properties under synthetic gauge field.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PseudoLandauLevel {
    /// Landau level index n (0, 1, 2, ...).
    pub index_n: usize,
    /// Eigenfrequency in kHz: f_n = f_0 pm v_D * sqrt(2 * B_s * n).
    pub frequency_khz: f64,
    /// Energy offset relative to resonance f_0 in kHz.
    pub energy_offset_khz: f64,
    /// Degeneracy factor or density scaling.
    pub degeneracy_weight: f64,
}

/// Valley-Hall acoustic Hamiltonian solver.
#[derive(Debug, Clone, PartialEq)]
pub struct ValleyHamiltonian {
    pub params: ValleyHallParams,
}

impl ValleyHamiltonian {
    /// Creates a new ValleyHamiltonian solver.
    pub fn new(params: ValleyHallParams) -> Self {
        Self { params }
    }

    /// Current topological phase of the metamaterial.
    pub fn phase(&self) -> ValleyHallPhase {
        if self.params.strain_gradient_khz_per_mm > 0.05 {
            ValleyHallPhase::PseudoLandauQuantized
        } else if self.params.mass_detuning_delta_khz.abs() > 1e-4 {
            ValleyHallPhase::ValleyHallInsulator
        } else {
            ValleyHallPhase::DiracSemimetal
        }
    }

    /// Computes the 2x2 Dirac-type effective valley Hamiltonian eigenvalues at momentum q = k - K_xi:
    /// H_xi(q) = f_0 * I + v_D * (xi * q_x * sigma_x + q_y * sigma_y) + Delta * sigma_z.
    ///
    /// Eigenvalues: E_pm(q) = f_0 pm sqrt((v_D * |q|)^2 + Delta^2).
    pub fn valley_dispersion_at(&self, qx: f64, qy: f64, valley: AcousticValley) -> (f64, f64) {
        let v_d = self.params.dirac_velocity();
        let q_mag_sq = qx * qx + qy * qy;
        let delta = self.params.mass_detuning_delta_khz;
        let gap_term = (v_d * v_d * q_mag_sq + delta * delta).sqrt();
        let f0 = self.params.resonance_freq_khz;

        // Sign of xi is encoded in valley index (+1 for K, -1 for K')
        let _xi = valley.valley_index();
        (f0 - gap_term, f0 + gap_term)
    }

    /// Evaluates the Berry curvature Omega_z(q) around valley xi:
    /// Omega_z(q) = -xi * (v_D^2 * Delta) / (2 * [(v_D * |q|)^2 + Delta^2]^(3/2)).
    pub fn berry_curvature_at(&self, qx: f64, qy: f64, valley: AcousticValley) -> f64 {
        let xi = valley.valley_index() as f64;
        let delta = self.params.mass_detuning_delta_khz;
        if delta.abs() < 1e-6 {
            return 0.0;
        }

        let v_d = self.params.dirac_velocity();
        let q_mag_sq = qx * qx + qy * qy;
        let denom = (v_d * v_d * q_mag_sq + delta * delta).powf(1.5);
        if denom < 1e-12 {
            return 0.0;
        }

        -xi * (v_d * v_d * delta) / (2.0 * denom)
    }

    /// Evaluates the valley Chern number C_xi = (1 / 2*pi) * integral Omega_z d^2q.
    /// Analytical integration over the valley pocket yields C_xi = -0.5 * xi * sign(Delta).
    pub fn valley_chern_number(&self, valley: AcousticValley) -> f64 {
        let xi = valley.valley_index() as f64;
        let delta = self.params.mass_detuning_delta_khz;
        if delta.abs() < 1e-6 {
            0.0
        } else {
            -0.5 * xi * delta.signum()
        }
    }

    /// Evaluates the valley Chern number difference Delta C_v = C_K - C_K'.
    /// For topological edge states along a domain wall, Delta C_v = pm 1.
    pub fn valley_chern_difference(&self) -> f64 {
        self.valley_chern_number(AcousticValley::ValleyK)
            - self.valley_chern_number(AcousticValley::ValleyKPrime)
    }

    /// Computes full tight-binding dispersion along high-symmetry path:
    /// Gamma (0, 0) -> K (4*pi / 3a, 0) -> M (pi / a, pi / (sqrt(3)*a)) -> Gamma (0, 0).
    pub fn band_structure_path(&self, num_points_per_segment: usize) -> Vec<ValleyDispersionPoint> {
        let a = self.params.lattice_a_mm;
        let n = num_points_per_segment.max(6);
        let mut path = Vec::with_capacity(3 * n);

        // High symmetry points in 1/mm
        let gamma = [0.0, 0.0];
        let k_pt = [4.0 * PI / (3.0 * a), 0.0];
        let m_pt = [PI / a, PI / (3.0_f64.sqrt() * a)];

        let segments = [(gamma, k_pt), (k_pt, m_pt), (m_pt, gamma)];
        let mut total_dist = 0.0;

        for (start, end) in segments {
            let dx = end[0] - start[0];
            let dy = end[1] - start[1];
            let seg_len = (dx * dx + dy * dy).sqrt();

            for i in 0..n {
                let frac = (i as f64) / ((n - 1) as f64);
                let kx = start[0] + frac * dx;
                let ky = start[1] + frac * dy;
                let dist = total_dist + frac * seg_len;

                // Evaluate tight-binding structure factor f(k):
                // delta_1 = (a / sqrt(3)) * [0, 1]
                // delta_2 = (a / sqrt(3)) * [sqrt(3)/2, -0.5]
                // delta_3 = (a / sqrt(3)) * [-sqrt(3)/2, -0.5]
                let d = a / 3.0_f64.sqrt();
                let p1 = ky * d;
                let p2 = 0.5 * 3.0_f64.sqrt() * kx * d - 0.5 * ky * d;
                let p3 = -0.5 * 3.0_f64.sqrt() * kx * d - 0.5 * ky * d;

                let f_re = p1.cos() + p2.cos() + p3.cos();
                let f_im = p1.sin() + p2.sin() + p3.sin();
                let f_mag = (f_re * f_re + f_im * f_im).sqrt();

                let t0 = self.params.hopping_t0_khz;
                let delta = self.params.mass_detuning_delta_khz;
                let f0 = self.params.resonance_freq_khz;

                let energy_split = (t0 * t0 * f_mag * f_mag + delta * delta).sqrt();
                let lower = f0 - energy_split;
                let upper = f0 + energy_split;

                // Near K point evaluate Berry curvature
                let qx = kx - k_pt[0];
                let qy = ky - k_pt[1];
                let bc = self.berry_curvature_at(qx, qy, AcousticValley::ValleyK);

                path.push(ValleyDispersionPoint {
                    k_dist: dist,
                    kx,
                    ky,
                    lower_band_khz: lower,
                    upper_band_khz: upper,
                    berry_curvature_mm2: bc,
                });
            }
            total_dist += seg_len;
        }

        path
    }

    /// Evaluates discrete acoustic pseudo-Landau levels E_n under synthetic pseudomagnetic field B_s:
    /// E_n = f_0 pm v_D * sqrt(2 * B_s * n).
    pub fn pseudo_landau_levels(&self, max_n: usize) -> Vec<PseudoLandauLevel> {
        let f0 = self.params.resonance_freq_khz;
        let v_d = self.params.dirac_velocity();
        let b_s = self.params.synthetic_pseudomagnetic_field().abs().max(1e-4);

        let count = max_n.clamp(3, 15);
        let mut levels = Vec::with_capacity(count * 2 + 1);

        // n = 0 level (pinned near f0 by delta)
        let delta = self.params.mass_detuning_delta_khz;
        levels.push(PseudoLandauLevel {
            index_n: 0,
            frequency_khz: f0 + delta,
            energy_offset_khz: delta,
            degeneracy_weight: 1.0,
        });

        // n > 0 levels
        for n in 1..=count {
            let offset = v_d * (2.0 * b_s * (n as f64)).sqrt();
            // Positive energy branch
            levels.push(PseudoLandauLevel {
                index_n: n,
                frequency_khz: f0 + offset,
                energy_offset_khz: offset,
                degeneracy_weight: 1.0 / (n as f64).sqrt(),
            });
            // Negative energy branch
            levels.push(PseudoLandauLevel {
                index_n: n,
                frequency_khz: f0 - offset,
                energy_offset_khz: -offset,
                degeneracy_weight: 1.0 / (n as f64).sqrt(),
            });
        }

        levels.sort_by(|a, b| a.frequency_khz.partial_cmp(&b.frequency_khz).unwrap_or(std::cmp::Ordering::Equal));
        levels
    }
}
