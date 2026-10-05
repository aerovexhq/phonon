#![deny(unsafe_code)]

//! Floquet Drive Hamiltonian, Quasi-Energy Spectrum, and Stroboscopic Evolution Operator
//! for Topological Acoustic Discrete Time Crystals.
//!
//! Models a non-equilibrium Floquet drive over period T = T1 + T2:
//! - Step 1: Imperfect pi-pulse drive H1 = hbar * Omega_x * sum_i sigma_x with pulse angle
//!   theta = pi * (1.0 - epsilon), duration T1.
//! - Step 2: Disordered Ising interaction with on-site many-body localization (MBL) disorder:
//!   H2 = sum_i (J + delta_J_i) sigma_z_i sigma_z_{i+1} + sum_i h_i sigma_z_i, duration T2.
//! - Stroboscopic evolution operator: U_F = exp(-i H2 T2 / hbar) exp(-i H1 T1 / hbar).
//! - Discrete time-translation symmetry breaking (TTSB) exhibiting period-doubling 2T subharmonic oscillation.

use std::f64::consts::PI;
use std::ops::{Add, Div, Mul, Neg, Sub};

/// Double-precision complex number for unitary Floquet operator analysis.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub const ZERO: Self = Self { re: 0.0, im: 0.0 };
    pub const ONE: Self = Self { re: 1.0, im: 0.0 };
    pub const I: Self = Self { re: 0.0, im: 1.0 };

    pub const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    pub const fn from_real(re: f64) -> Self {
        Self { re, im: 0.0 }
    }

    pub fn from_polar(r: f64, theta: f64) -> Self {
        Self {
            re: r * theta.cos(),
            im: r * theta.sin(),
        }
    }

    pub fn norm_sq(&self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    pub fn norm(&self) -> f64 {
        self.norm_sq().sqrt()
    }

    pub fn arg(&self) -> f64 {
        self.im.atan2(self.re)
    }

    pub fn conj(&self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }

    pub fn scale(&self, s: f64) -> Self {
        Self {
            re: self.re * s,
            im: self.im * s,
        }
    }

    pub fn inv(&self) -> Self {
        let nsq = self.norm_sq();
        if nsq == 0.0 {
            Self::ZERO
        } else {
            Self {
                re: self.re / nsq,
                im: -self.im / nsq,
            }
        }
    }

    pub fn exp(&self) -> Self {
        let r = self.re.exp();
        Self {
            re: r * self.im.cos(),
            im: r * self.im.sin(),
        }
    }
}

impl Add for Complex {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}

impl Sub for Complex {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            re: self.re - rhs.re,
            im: self.im - rhs.im,
        }
    }
}

impl Mul for Complex {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }
}

impl Div for Complex {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        let nsq = rhs.norm_sq();
        if nsq == 0.0 {
            Self::ZERO
        } else {
            Self {
                re: (self.re * rhs.re + self.im * rhs.im) / nsq,
                im: (self.im * rhs.re - self.re * rhs.im) / nsq,
            }
        }
    }
}

impl Neg for Complex {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            re: -self.re,
            im: -self.im,
        }
    }
}

/// Initial spin state classification for Floquet dynamics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FloquetStateKind {
    /// All acoustic spins initialized in the up state (|00...0>).
    #[default]
    AllUp,
    /// Antiferromagnetic Neel state (|0101...01>).
    NeelState,
    /// Random spin orientation product state.
    Random,
}

/// Configuration parameters for the Floquet time crystal drive.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetTimeCrystalParams {
    /// Number of acoustic resonator sites in 1D chain (4 to 12, default 8).
    pub chain_length: usize,
    /// Drive period T in seconds (default 1.0e-6 s / 1.0 us).
    pub drive_period_t: f64,
    /// Ratio of pulse duration T1 / T (default 0.5).
    pub pulse_duration_ratio: f64,
    /// Pulse error perturbation epsilon in [-0.25, 0.25] (default 0.05).
    pub pulse_error_epsilon: f64,
    /// Ising acoustic coupling J in rad/s (default pi / (2.0 * T2)).
    pub coupling_j: f64,
    /// Coupling disorder amplitude delta_J in rad/s (default 0.2 * J).
    pub coupling_disorder_delta_j: f64,
    /// Longitudinal on-site MBL disorder strength W in rad/s (default 2.0 * J).
    pub disorder_field_w: f64,
    /// Acoustic dissipation / damping rate gamma in 1/s (default 0.001 / T).
    pub damping_gamma: f64,
}

impl Default for FloquetTimeCrystalParams {
    fn default() -> Self {
        let drive_period_t = 1.0e-6;
        let pulse_duration_ratio = 0.5;
        let t2 = drive_period_t * (1.0 - pulse_duration_ratio);
        let coupling_j = PI / (2.0 * t2);
        let coupling_disorder_delta_j = 0.2 * coupling_j;
        let disorder_field_w = 2.0 * coupling_j;
        let damping_gamma = 0.001 / drive_period_t;

        Self {
            chain_length: 8,
            drive_period_t,
            pulse_duration_ratio,
            pulse_error_epsilon: 0.05,
            coupling_j,
            coupling_disorder_delta_j,
            disorder_field_w,
            damping_gamma,
        }
    }
}

impl FloquetTimeCrystalParams {
    /// Creates a new FloquetTimeCrystalParams configuration.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        chain_length: usize,
        drive_period_t: f64,
        pulse_duration_ratio: f64,
        pulse_error_epsilon: f64,
        coupling_j: f64,
        coupling_disorder_delta_j: f64,
        disorder_field_w: f64,
        damping_gamma: f64,
    ) -> Self {
        Self {
            chain_length: chain_length.clamp(2, 12),
            drive_period_t: drive_period_t.max(1e-12),
            pulse_duration_ratio: pulse_duration_ratio.clamp(0.05, 0.95),
            pulse_error_epsilon: pulse_error_epsilon.clamp(-0.4, 0.4),
            coupling_j: coupling_j.max(0.0),
            coupling_disorder_delta_j: coupling_disorder_delta_j.max(0.0),
            disorder_field_w: disorder_field_w.max(0.0),
            damping_gamma: damping_gamma.max(0.0),
        }
    }

    /// Preset: Stable Discrete Time Crystal (epsilon = 0.05).
    pub fn preset_stable_dtc() -> Self {
        Self::default()
    }

    /// Preset: Zero Perturbation DTC (epsilon = 0.0).
    pub fn preset_zero_perturbation() -> Self {
        let mut p = Self::default();
        p.pulse_error_epsilon = 0.0;
        p
    }

    /// Preset: Thermal Ergodic Regime (W = 0, no MBL).
    pub fn preset_thermal_ergodic() -> Self {
        let mut p = Self::default();
        p.disorder_field_w = 0.0;
        p.coupling_disorder_delta_j = 0.0;
        p
    }

    /// Preset: Trivial Paramagnet (J = 0).
    pub fn preset_trivial_paramagnet() -> Self {
        let mut p = Self::default();
        p.coupling_j = 0.0;
        p.coupling_disorder_delta_j = 0.0;
        p.disorder_field_w = 0.0;
        p
    }

    /// Pulse duration T1 in seconds.
    pub fn t1(&self) -> f64 {
        self.drive_period_t * self.pulse_duration_ratio
    }

    /// Evolution duration T2 in seconds.
    pub fn t2(&self) -> f64 {
        self.drive_period_t * (1.0 - self.pulse_duration_ratio)
    }

    /// Fundamental Floquet angular frequency Omega = 2 * pi / T in rad/s.
    pub fn floquet_omega_rad(&self) -> f64 {
        2.0 * PI / self.drive_period_t.max(1e-15)
    }

    /// Floquet driving frequency nu_drive in MHz.
    pub fn floquet_frequency_mhz(&self) -> f64 {
        1.0e-6 / self.drive_period_t.max(1e-15)
    }

    /// Subharmonic response frequency nu_sub = nu_drive / 2 in MHz.
    pub fn subharmonic_frequency_mhz(&self) -> f64 {
        self.floquet_frequency_mhz() / 2.0
    }

    /// Rotation pulse angle theta = pi * (1.0 - epsilon) in radians.
    pub fn pulse_angle_theta(&self) -> f64 {
        PI * (1.0 - self.pulse_error_epsilon)
    }
}

/// Represents the quantum acoustic state vector of N coupled sites.
#[derive(Debug, Clone, PartialEq)]
pub struct FloquetState {
    /// Complex state vector amplitudes in computational basis (dim = 2^N).
    pub amplitudes: Vec<Complex>,
    /// Number of sites N.
    pub chain_length: usize,
}

impl FloquetState {
    /// Creates an All-Up ferromagnetic state |00...0>.
    pub fn new_all_up(chain_length: usize) -> Self {
        let n = chain_length.clamp(2, 12);
        let dim = 1 << n;
        let mut amplitudes = vec![Complex::ZERO; dim];
        amplitudes[0] = Complex::ONE;
        Self {
            amplitudes,
            chain_length: n,
        }
    }

    /// Creates an alternating antiferromagnetic Neel state |0101...>.
    pub fn new_neel(chain_length: usize) -> Self {
        let n = chain_length.clamp(2, 12);
        let dim = 1 << n;
        let mut neel_index = 0usize;
        for k in 0..n {
            if k % 2 == 1 {
                neel_index |= 1 << (n - 1 - k);
            }
        }
        let mut amplitudes = vec![Complex::ZERO; dim];
        amplitudes[neel_index] = Complex::ONE;
        Self {
            amplitudes,
            chain_length: n,
        }
    }

    /// Creates a deterministic pseudo-random product state from a seed.
    pub fn new_random(chain_length: usize, seed: u64) -> Self {
        let n = chain_length.clamp(2, 12);
        let dim = 1 << n;
        let mut rng_state = seed.wrapping_add(0x9E3779B97F4A7C15);
        let mut next_u64 = || {
            rng_state ^= rng_state << 13;
            rng_state ^= rng_state >> 7;
            rng_state ^= rng_state << 17;
            rng_state
        };

        // Random product state: for each site choose up or down
        let mut state_index = 0usize;
        for k in 0..n {
            if (next_u64() & 1) == 1 {
                state_index |= 1 << (n - 1 - k);
            }
        }

        let mut amplitudes = vec![Complex::ZERO; dim];
        amplitudes[state_index] = Complex::ONE;
        Self {
            amplitudes,
            chain_length: n,
        }
    }

    /// Initializes a state from the specified kind.
    pub fn from_kind(kind: FloquetStateKind, chain_length: usize, seed: u64) -> Self {
        match kind {
            FloquetStateKind::AllUp => Self::new_all_up(chain_length),
            FloquetStateKind::NeelState => Self::new_neel(chain_length),
            FloquetStateKind::Random => Self::new_random(chain_length, seed),
        }
    }

    /// Dimension of Hilbert space 2^N.
    pub fn dim(&self) -> usize {
        self.amplitudes.len()
    }

    /// Computes squared norm sum_s |psi_s|^2.
    pub fn norm_sq(&self) -> f64 {
        self.amplitudes.iter().map(|c| c.norm_sq()).sum()
    }

    /// Computes vector Euclidean norm.
    pub fn norm(&self) -> f64 {
        self.norm_sq().sqrt()
    }

    /// Normalizes state vector in place.
    pub fn normalize(&mut self) {
        let n = self.norm();
        if n > 1e-15 {
            let inv_n = 1.0 / n;
            for c in &mut self.amplitudes {
                *c = c.scale(inv_n);
            }
        }
    }

    /// Computes local expectation value <sigma^z_i> at site `site` in [-1.0, 1.0].
    pub fn site_polarization(&self, site: usize) -> f64 {
        if site >= self.chain_length {
            return 0.0;
        }
        let bit_shift = self.chain_length - 1 - site;
        let mut exp_val = 0.0;
        for (idx, amp) in self.amplitudes.iter().enumerate() {
            let p = amp.norm_sq();
            let bit = (idx >> bit_shift) & 1;
            let sz = if bit == 0 { 1.0 } else { -1.0 };
            exp_val += p * sz;
        }
        exp_val
    }

    /// Computes all individual site polarizations [<sigma^z_0>, ..., <sigma^z_{N-1}>].
    pub fn all_site_polarizations(&self) -> Vec<f64> {
        (0..self.chain_length)
            .map(|i| self.site_polarization(i))
            .collect()
    }

    /// Computes average magnetization / acoustic polarization M_z = (1/N) sum_i <sigma^z_i>.
    pub fn average_magnetization(&self) -> f64 {
        let sum: f64 = self.all_site_polarizations().iter().sum();
        sum / (self.chain_length as f64)
    }
}

/// Recorded stroboscopic dynamics trajectory across driving cycles.
#[derive(Debug, Clone, PartialEq)]
pub struct StroboscopicTrajectory {
    /// Recorded timestamps t_n = n * T in seconds.
    pub timestamps: Vec<f64>,
    /// Average magnetization M_z(n * T) in [-1.0, 1.0].
    pub average_magnetization: Vec<f64>,
    /// Individual site polarizations m_i(n * T) for each cycle and site.
    pub site_polarizations: Vec<Vec<f64>>,
    /// Number of recorded cycles.
    pub cycle_count: usize,
    /// Floquet driving period T in seconds.
    pub drive_period_t: f64,
}

impl StroboscopicTrajectory {
    /// Checks for robust period-doubling 2T oscillation:
    /// M_z(2n * T) has opposite sign to M_z((2n+1) * T).
    pub fn is_period_doubled(&self) -> bool {
        let n = self.average_magnetization.len().min(20);
        if n < 4 {
            return false;
        }
        let mut sign_flips = 0;
        for i in 0..(n - 1) {
            let m1 = self.average_magnetization[i];
            let m2 = self.average_magnetization[i + 1];
            if m1 * m2 < 0.0 && m1.abs() > 0.05 && m2.abs() > 0.05 {
                sign_flips += 1;
            }
        }
        // At least 70% of transitions alternate sign
        (sign_flips as f64) / ((n - 1) as f64) >= 0.70
    }

    /// Expected subharmonic period 2 * T.
    pub fn subharmonic_period(&self) -> f64 {
        2.0 * self.drive_period_t
    }
}

/// Stroboscopic Floquet Unitary Propagation Operator:
/// U_F = exp(-i H2 T2 / hbar) exp(-i H1 T1 / hbar).
#[derive(Debug, Clone, PartialEq)]
pub struct FloquetUnitaryOperator {
    /// Physical drive parameters.
    pub params: FloquetTimeCrystalParams,
    /// Coupling disorder realizations delta_J_i.
    pub delta_j: Vec<f64>,
    /// On-site disorder field realizations h_i.
    pub disorder_h: Vec<f64>,
    /// Hilbert space dimension D = 2^N.
    pub dim: usize,
    /// Full unitary matrix U_F in row-major order (dim x dim).
    pub matrix: Vec<Complex>,
    /// Diagonal phase entries of U2.
    pub diag_u2: Vec<Complex>,
}

impl FloquetUnitaryOperator {
    /// Constructs a new FloquetUnitaryOperator using default deterministic disorder.
    pub fn new(params: &FloquetTimeCrystalParams) -> Self {
        let n = params.chain_length.clamp(2, 12);
        // Deterministic pseudo-random disorder generation
        let mut rng = 42_u64;
        let mut next_f64 = || {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            ((rng & 0x001F_FFFF_FFFF_FFFF_u64) as f64) / ((0x001F_FFFF_FFFF_FFFF_u64 as f64) + 1.0)
        };

        let delta_j: Vec<f64> = (0..(n.saturating_sub(1)))
            .map(|_| {
                let r = next_f64() * 2.0 - 1.0;
                r * params.coupling_disorder_delta_j
            })
            .collect();

        let disorder_h: Vec<f64> = (0..n)
            .map(|_| {
                let r = next_f64() * 2.0 - 1.0;
                r * params.disorder_field_w
            })
            .collect();

        Self::new_with_disorder(params, &delta_j, &disorder_h)
    }

    /// Constructs a FloquetUnitaryOperator with specified disorder vectors.
    pub fn new_with_disorder(
        params: &FloquetTimeCrystalParams,
        delta_j: &[f64],
        disorder_h: &[f64],
    ) -> Self {
        let n = params.chain_length.clamp(2, 12);
        let dim = 1 << n;

        // Step 1: Imperfect pi-pulse drive operator U1
        // Pulse angle theta = pi * (1.0 - epsilon)
        let theta = params.pulse_angle_theta();
        let c = (theta * 0.5).cos();
        let s = -(theta * 0.5).sin();
        let u1_single = [Complex::new(c, 0.0), Complex::new(0.0, s)];

        // Compute full U1 matrix via tensor product
        let mut u1_mat = vec![Complex::ZERO; dim * dim];
        for row in 0..dim {
            for col in 0..dim {
                let mut elem = Complex::ONE;
                for k in 0..n {
                    let r_bit = (row >> (n - 1 - k)) & 1;
                    let c_bit = (col >> (n - 1 - k)) & 1;
                    let single_val = if r_bit == c_bit {
                        u1_single[0]
                    } else {
                        u1_single[1]
                    };
                    elem = elem * single_val;
                }
                u1_mat[row * dim + col] = elem;
            }
        }

        // Step 2: Disordered Ising interaction operator U2
        // U2 is diagonal in the computational z basis:
        // E_s = sum_i (J + delta_J_i) z_i z_{i+1} + sum_i h_i z_i
        let t2 = params.t2();
        let mut diag_u2 = vec![Complex::ZERO; dim];
        for state in 0..dim {
            let mut energy = 0.0;
            // Nearest-neighbor Ising interaction
            for k in 0..(n - 1) {
                let z1 = if ((state >> (n - 1 - k)) & 1) == 0 {
                    1.0
                } else {
                    -1.0
                };
                let z2 = if ((state >> (n - 2 - k)) & 1) == 0 {
                    1.0
                } else {
                    -1.0
                };
                let dj = if k < delta_j.len() { delta_j[k] } else { 0.0 };
                energy += (params.coupling_j + dj) * z1 * z2;
            }
            // On-site longitudinal disorder field
            for k in 0..n {
                let z = if ((state >> (n - 1 - k)) & 1) == 0 {
                    1.0
                } else {
                    -1.0
                };
                let h = if k < disorder_h.len() {
                    disorder_h[k]
                } else {
                    0.0
                };
                energy += h * z;
            }
            // exp(-i * E_s * T2)
            let phase = -energy * t2;
            diag_u2[state] = Complex::from_polar(1.0, phase);
        }

        // Composite Floquet operator U_F = U2 * U1:
        // (U_F)_{r, c} = (U2)_{r, r} * (U1)_{r, c}
        let mut u_f = vec![Complex::ZERO; dim * dim];
        for r in 0..dim {
            let u2_r = diag_u2[r];
            for c in 0..dim {
                u_f[r * dim + c] = u2_r * u1_mat[r * dim + c];
            }
        }

        Self {
            params: *params,
            delta_j: delta_j.to_vec(),
            disorder_h: disorder_h.to_vec(),
            dim,
            matrix: u_f,
            diag_u2,
        }
    }

    /// Applies one Floquet period evolution to state vector: psi_{n+1} = U_F * psi_n.
    pub fn apply_step(&self, state: &FloquetState) -> FloquetState {
        let dim = self.dim;
        let mut next_amplitudes = vec![Complex::ZERO; dim];
        for r in 0..dim {
            let mut sum = Complex::ZERO;
            for c in 0..dim {
                sum = sum + self.matrix[r * dim + c] * state.amplitudes[c];
            }
            next_amplitudes[r] = sum;
        }

        let mut res = FloquetState {
            amplitudes: next_amplitudes,
            chain_length: self.params.chain_length,
        };
        res.normalize();
        res
    }

    /// Evaluates stroboscopic evolution over `n_cycles` Floquet periods.
    pub fn evolve(&self, initial_state: &FloquetState, n_cycles: usize) -> StroboscopicTrajectory {
        let cycles = n_cycles.max(2);
        let mut timestamps = Vec::with_capacity(cycles);
        let mut average_magnetization = Vec::with_capacity(cycles);
        let mut site_polarizations = Vec::with_capacity(cycles);

        let mut current_state = initial_state.clone();
        let dt = self.params.drive_period_t;
        let gamma = self.params.damping_gamma;

        for n in 0..cycles {
            let t_n = (n as f64) * dt;
            let damping_factor = (-gamma * t_n).exp();

            let unattenuated_m = current_state.all_site_polarizations();
            let attenuated_m: Vec<f64> = unattenuated_m
                .iter()
                .map(|&m| (m * damping_factor).clamp(-1.0, 1.0))
                .collect();

            let avg_m = (attenuated_m.iter().sum::<f64>() / (attenuated_m.len() as f64))
                .clamp(-1.0, 1.0);

            timestamps.push(t_n);
            average_magnetization.push(avg_m);
            site_polarizations.push(attenuated_m);

            current_state = self.apply_step(&current_state);
        }

        StroboscopicTrajectory {
            timestamps,
            average_magnetization,
            site_polarizations,
            cycle_count: cycles,
            drive_period_t: dt,
        }
    }

    /// Verifies unitarity of U_F: evaluates max |(U_F^dagger * U_F)_{r, c} - delta_{r, c}|.
    pub fn verify_unitarity(&self) -> f64 {
        let dim = self.dim;
        let mut max_err = 0.0_f64;

        for r in 0..dim {
            for c in 0..dim {
                let mut sum = Complex::ZERO;
                for k in 0..dim {
                    // (U^dagger)_{r, k} = (U_{k, r})^*
                    let u_kr_conj = self.matrix[k * dim + r].conj();
                    let u_kc = self.matrix[k * dim + c];
                    sum = sum + u_kr_conj * u_kc;
                }
                let delta = if r == c {
                    Complex::ONE
                } else {
                    Complex::ZERO
                };
                let err = (sum - delta).norm();
                if err > max_err {
                    max_err = err;
                }
            }
        }
        max_err
    }

    /// Computes Floquet quasi-energies epsilon_k in [-pi / T, pi / T].
    ///
    /// For efficiency in interactive GUI and high chain lengths, computes the exact
    /// quasi-energies on the representative subspace (up to N=6, dim=64).
    pub fn compute_quasi_energies(&self) -> Vec<f64> {
        let n_sub = self.params.chain_length.min(6);
        let sub_op = if n_sub == self.params.chain_length {
            self.clone()
        } else {
            let mut sub_p = self.params;
            sub_p.chain_length = n_sub;
            Self::new(&sub_p)
        };

        let dim = sub_op.dim;
        let t = sub_op.params.drive_period_t;

        // Diagonalize normal operator U_F via Hermitian reduction:
        // A = cos(alpha) * (U + U^dagger)/2 + sin(alpha) * (U - U^dagger)/(2i)
        let alpha = 0.73_f64;
        let cos_a = alpha.cos();
        let sin_a = alpha.sin();

        let mut a_re = vec![0.0; dim * dim];
        let mut a_im = vec![0.0; dim * dim];

        for r in 0..dim {
            for c in 0..dim {
                let u_rc = sub_op.matrix[r * dim + c];
                let u_cr_conj = sub_op.matrix[c * dim + r].conj();

                // H = (U + U^dagger)/2
                let h_rc = (u_rc + u_cr_conj).scale(0.5);
                // K = (U - U^dagger)/(2i)
                let diff = (u_rc - u_cr_conj).scale(0.5);
                // diff / i = diff * (-i) = (diff.re + i diff.im) * (-i) = diff.im - i diff.re
                let k_rc = Complex::new(diff.im, -diff.re);

                let a_elem = h_rc.scale(cos_a) + k_rc.scale(sin_a);
                a_re[r * dim + c] = a_elem.re;
                a_im[r * dim + c] = a_elem.im;
            }
        }

        // Real 2D x 2D symmetric mapping
        let dim2 = 2 * dim;
        let mut m_mat = vec![0.0; dim2 * dim2];
        for r in 0..dim {
            for c in 0..dim {
                let re = a_re[r * dim + c];
                let im = a_im[r * dim + c];
                m_mat[r * dim2 + c] = re;
                m_mat[r * dim2 + (dim + c)] = -im;
                m_mat[(dim + r) * dim2 + c] = im;
                m_mat[(dim + r) * dim2 + (dim + c)] = re;
            }
        }

        let (_evals, v2_flat) = jacobi_real_symmetric(&m_mat, dim2);

        // Extract eigenvectors and Rayleigh quotient with U_F
        let mut quasi_energies = Vec::with_capacity(dim);
        let mut idx = 0;
        while idx < dim2 && quasi_energies.len() < dim {
            let mut w = Vec::with_capacity(dim);
            let mut norm_sq = 0.0_f64;
            for r in 0..dim {
                let u = v2_flat[r * dim2 + idx];
                let v = v2_flat[(dim + r) * dim2 + idx];
                norm_sq += u * u + v * v;
                w.push(Complex::new(u, v));
            }
            let norm = norm_sq.sqrt().max(1e-15);
            for c in &mut w {
                *c = c.scale(1.0 / norm);
            }

            // Rayleigh quotient: lambda = w^dagger * U_F * w
            let mut lambda = Complex::ZERO;
            for r in 0..dim {
                let mut row_sum = Complex::ZERO;
                for c in 0..dim {
                    row_sum = row_sum + sub_op.matrix[r * dim + c] * w[c];
                }
                lambda = lambda + w[r].conj() * row_sum;
            }

            let phase = lambda.arg(); // in [-pi, pi]
            let eps_k = -phase / t;
            quasi_energies.push(eps_k);

            idx += 2;
        }

        quasi_energies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        quasi_energies
    }

    /// Evaluates the mean pi-quasienergy pairing gap in radians:
    /// For every quasienergy phase phi_k, finds closest phase to phi_k + pi (mod 2*pi).
    /// Returns mean gap in radians (~ pi = 3.14159 rad in DTC regime).
    pub fn compute_pi_quasienergy_pairing_gap(&self) -> f64 {
        let qe = self.compute_quasi_energies();
        if qe.is_empty() {
            return PI;
        }
        let t = self.params.drive_period_t;
        let phases: Vec<f64> = qe.iter().map(|&e| e * t).collect();

        let mut gaps = Vec::with_capacity(phases.len());
        for &p in &phases {
            let mut target = p + PI;
            while target > PI {
                target -= 2.0 * PI;
            }
            while target < -PI {
                target += 2.0 * PI;
            }

            let mut min_diff = 2.0 * PI;
            for &other in &phases {
                let diff = (other - target).abs();
                let wrapped_diff = diff.min(2.0 * PI - diff);
                if wrapped_diff < min_diff {
                    min_diff = wrapped_diff;
                }
            }
            gaps.push(PI - min_diff);
        }

        let mean_gap = gaps.iter().sum::<f64>() / (gaps.len() as f64);
        mean_gap.clamp(0.0, PI)
    }
}

/// Computes eigenvalues and eigenvectors of a real symmetric N x N matrix via cyclic Jacobi rotations.
fn jacobi_real_symmetric(a_in: &[f64], n: usize) -> (Vec<f64>, Vec<f64>) {
    let mut a = a_in.to_vec();
    let mut d = vec![0.0; n];
    let mut v = vec![0.0; n * n];

    for i in 0..n {
        for j in 0..n {
            v[i * n + j] = if i == j { 1.0 } else { 0.0 };
        }
        d[i] = a[i * n + i];
    }

    let max_sweeps = 35;
    for _sweep in 0..max_sweeps {
        let mut sm = 0.0;
        for p in 0..n {
            for q in (p + 1)..n {
                sm += a[p * n + q].abs();
            }
        }
        if sm <= 1e-12 {
            break;
        }

        for p in 0..n {
            for q in (p + 1)..n {
                let apq = a[p * n + q];
                if apq.abs() > 1e-15 {
                    let h = d[q] - d[p];
                    let theta = 0.5 * h / apq;
                    let mut t = 1.0 / (theta.abs() + (1.0 + theta * theta).sqrt());
                    if theta < 0.0 {
                        t = -t;
                    }
                    let c = 1.0 / (1.0 + t * t).sqrt();
                    let s = t * c;
                    let tau = s / (1.0 + c);
                    let h_rot = t * apq;
                    d[p] -= h_rot;
                    d[q] += h_rot;
                    a[p * n + q] = 0.0;

                    for j in 0..p {
                        let g1 = a[j * n + p];
                        let h1 = a[j * n + q];
                        a[j * n + p] = g1 - s * (h1 + g1 * tau);
                        a[j * n + q] = h1 + s * (g1 - h1 * tau);
                    }
                    for j in (p + 1)..q {
                        let g1 = a[p * n + j];
                        let h1 = a[j * n + q];
                        a[p * n + j] = g1 - s * (h1 + g1 * tau);
                        a[j * n + q] = h1 + s * (g1 - h1 * tau);
                    }
                    for j in (q + 1)..n {
                        let g1 = a[p * n + j];
                        let h1 = a[q * n + j];
                        a[p * n + j] = g1 - s * (h1 + g1 * tau);
                        a[q * n + j] = h1 + s * (g1 - h1 * tau);
                    }
                    for j in 0..n {
                        let g1 = v[j * n + p];
                        let h1 = v[j * n + q];
                        v[j * n + p] = g1 - s * (h1 + g1 * tau);
                        v[j * n + q] = h1 + s * (g1 - h1 * tau);
                    }
                }
            }
        }
    }

    (d, v)
}
