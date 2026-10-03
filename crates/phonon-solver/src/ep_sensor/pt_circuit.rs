#![deny(unsafe_code)]

//! Parity-Time (PT) symmetric coupled RLC circuit simulator, phase transition analyzer,
//! and 1D Non-Hermitian Skin Effect (NHSE) lattice solver.
//!
//! Models coupled gain-loss resonators exhibiting exact, exceptional point, and broken
//! PT symmetry phases, as well as directional boundary mode skin localization.

use super::jordan_eigensolver::Complex;
use std::f64::consts::PI;

/// PT symmetry operational regime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PtPhase {
    /// Exact PT-symmetric phase (gamma < gamma_c): real oscillation frequencies, neutral beating.
    Exact,
    /// Exceptional Point (gamma == gamma_c): coalescence of eigenfrequencies and mode vectors.
    ExceptionalPoint,
    /// Broken PT-symmetric phase (gamma > gamma_c): complex conjugate bifurcation with exponential gain/decay.
    Broken,
}

impl PtPhase {
    /// Human-readable label for telemetry and UI badges.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Exact => "Exact Phase (Neutral Beating)",
            Self::ExceptionalPoint => "Exceptional Point (Spectral Coalescence)",
            Self::Broken => "Broken Phase (Gain/Decay Bifurcation)",
        }
    }
}

/// Parameters for coupled PT-symmetric RLC circuit resonators.
#[derive(Debug, Clone, PartialEq)]
pub struct PtCircuitParams {
    /// Negative resistance magnitude for active gain resonator tank (Ohms).
    pub r_gain: f64,
    /// Positive resistance for passive loss resonator tank (Ohms).
    pub r_loss: f64,
    /// Tank inductance L (Henries).
    pub inductance_h: f64,
    /// Tank capacitance C (Farads).
    pub capacitance_f: f64,
    /// Dimensionless capacitive/inductive inter-resonator coupling coefficient.
    pub coupling_kc: f64,
    /// Bare resonance frequency omega_0 = 1 / sqrt(L * C) in rad/s.
    pub omega_0: f64,
}

impl Default for PtCircuitParams {
    fn default() -> Self {
        // Default 100 MHz resonator tank with balanced 50 Ohm gain/loss
        let l: f64 = 1.0e-7; // 100 nH
        let c: f64 = 2.533e-11; // 25.33 pF -> omega_0 ~ 2 * pi * 100 MHz
        let r: f64 = 100.0;
        let kc: f64 = 0.05;
        let omega_0: f64 = 1.0 / (l * c).sqrt();
        Self {
            r_gain: r,
            r_loss: r,
            inductance_h: l,
            capacitance_f: c,
            coupling_kc: kc,
            omega_0,
        }
    }
}

impl PtCircuitParams {
    /// Creates circuit parameters from component values.
    pub fn new(r_gain: f64, r_loss: f64, l: f64, c: f64, kc: f64) -> Self {
        let l_clamped = l.abs().max(1e-12);
        let c_clamped = c.abs().max(1e-15);
        let omega_0 = 1.0 / (l_clamped * c_clamped).sqrt();
        Self {
            r_gain: r_gain.abs().max(0.1),
            r_loss: r_loss.abs().max(0.1),
            inductance_h: l_clamped,
            capacitance_f: c_clamped,
            coupling_kc: kc.abs().max(1e-6),
            omega_0,
        }
    }

    /// Creates circuit parameters directly specifying angular rates (rad/s).
    pub fn with_rates(omega_0: f64, gamma: f64, kappa: f64) -> Self {
        let w0 = omega_0.abs().max(1e3);
        let c = 1.0e-11;
        let l = 1.0 / (w0 * w0 * c);
        let r = 1.0 / (2.0 * gamma.abs().max(1e-3) * c);
        let kc = (2.0 * kappa.abs().max(1e-3)) / w0;
        Self {
            r_gain: r,
            r_loss: r,
            inductance_h: l,
            capacitance_f: c,
            coupling_kc: kc,
            omega_0: w0,
        }
    }

    /// Resonant frequency in Hertz.
    pub fn f0_hz(&self) -> f64 {
        self.omega_0 / (2.0 * PI)
    }

    /// Active gain rate gamma_gain = 1 / (2 * R_gain * C) in rad/s.
    pub fn gain_rate(&self) -> f64 {
        1.0 / (2.0 * self.r_gain * self.capacitance_f)
    }

    /// Passive loss rate gamma_loss = 1 / (2 * R_loss * C) in rad/s.
    pub fn loss_rate(&self) -> f64 {
        1.0 / (2.0 * self.r_loss * self.capacitance_f)
    }

    /// Effective balanced gain/loss rate gamma = (gamma_gain + gamma_loss) / 2.
    pub fn balanced_gamma(&self) -> f64 {
        0.5 * (self.gain_rate() + self.loss_rate())
    }

    /// Inter-cavity coupling rate kappa = kc * omega_0 / 2 in rad/s.
    pub fn coupling_rate(&self) -> f64 {
        0.5 * self.coupling_kc * self.omega_0
    }

    /// Critical gain/loss threshold for exceptional point coalescence gamma_c = kappa.
    pub fn critical_gamma(&self) -> f64 {
        self.coupling_rate()
    }

    /// Identifies the PT symmetry regime of the coupled circuit.
    pub fn pt_phase(&self) -> PtPhase {
        let gamma = self.balanced_gamma();
        let kappa = self.coupling_rate();
        let diff = (gamma - kappa).abs();
        let tol = 1e-3 * kappa.max(1.0);

        if diff <= tol {
            PtPhase::ExceptionalPoint
        } else if gamma < kappa {
            PtPhase::Exact
        } else {
            PtPhase::Broken
        }
    }

    /// Evaluates the coupled supermode eigenfrequencies omega_+ and omega_-.
    ///
    /// omega_pm = omega_0 pm sqrt(kappa^2 - gamma^2)
    pub fn eigenfrequencies(&self) -> (Complex, Complex) {
        let w0 = self.omega_0;
        let gamma = self.balanced_gamma();
        let kappa = self.coupling_rate();

        let rad = kappa * kappa - gamma * gamma;
        if rad >= 0.0 {
            let split = rad.sqrt();
            (Complex::new(w0 + split, 0.0), Complex::new(w0 - split, 0.0))
        } else {
            let bifurc = (-rad).sqrt();
            (
                Complex::new(w0, bifurc),
                Complex::new(w0, -bifurc),
            )
        }
    }

    /// Evaluates eigenfrequency splitting Delta omega in rad/s.
    pub fn frequency_splitting(&self) -> f64 {
        let (w1, w2) = self.eigenfrequencies();
        (w1 - w2).norm()
    }

    /// Simulates transient dynamics of coupled voltages v_1(t) and v_2(t) using 4th-order Runge-Kutta.
    ///
    /// Implements coupled-mode equations:
    ///   d a_1 / dt = (gamma_g - i * omega_0) * a_1 - i * kappa * a_2
    ///   d a_2 / dt = (-gamma_l - i * omega_0) * a_2 - i * kappa * a_1
    /// where v_1(t) = Re(a_1(t)), v_2(t) = Re(a_2(t)), and envelope(t) = sqrt(|a_1|^2 + |a_2|^2).
    pub fn simulate_transient(
        &self,
        v1_init: f64,
        v2_init: f64,
        t_end_s: f64,
        num_steps: usize,
    ) -> PtCircuitState {
        let n_steps = num_steps.max(50);
        let dt = t_end_s / (n_steps as f64);

        let gamma_g = self.gain_rate();
        let gamma_l = self.loss_rate();
        let w0 = self.omega_0;
        let kappa = self.coupling_rate();

        let mut time = Vec::with_capacity(n_steps + 1);
        let mut v1 = Vec::with_capacity(n_steps + 1);
        let mut v2 = Vec::with_capacity(n_steps + 1);
        let mut envelope = Vec::with_capacity(n_steps + 1);

        // State vector s = [u1, w1, u2, w2] where a1 = u1 + i*w1, a2 = u2 + i*w2
        let mut state = [v1_init, 0.0, v2_init, 0.0];

        time.push(0.0);
        v1.push(state[0]);
        v2.push(state[2]);
        envelope.push((state[0] * state[0] + state[1] * state[1] + state[2] * state[2] + state[3] * state[3]).sqrt());

        let deriv = |s: &[f64; 4]| -> [f64; 4] {
            [
                gamma_g * s[0] + w0 * s[1] + kappa * s[3],
                gamma_g * s[1] - w0 * s[0] - kappa * s[2],
                -gamma_l * s[2] + w0 * s[3] + kappa * s[1],
                -gamma_l * s[3] - w0 * s[2] - kappa * s[0],
            ]
        };

        for step in 1..=n_steps {
            let t = (step as f64) * dt;

            // RK4 integration step
            let k1 = deriv(&state);

            let mut s2 = [0.0; 4];
            for i in 0..4 {
                s2[i] = state[i] + 0.5 * dt * k1[i];
            }
            let k2 = deriv(&s2);

            let mut s3 = [0.0; 4];
            for i in 0..4 {
                s3[i] = state[i] + 0.5 * dt * k2[i];
            }
            let k3 = deriv(&s3);

            let mut s4 = [0.0; 4];
            for i in 0..4 {
                s4[i] = state[i] + dt * k3[i];
            }
            let k4 = deriv(&s4);

            for i in 0..4 {
                state[i] += (dt / 6.0) * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
                state[i] = state[i].clamp(-1e9, 1e9);
            }

            time.push(t);
            v1.push(state[0]);
            v2.push(state[2]);
            envelope.push((state[0] * state[0] + state[1] * state[1] + state[2] * state[2] + state[3] * state[3]).sqrt());
        }

        PtCircuitState {
            time,
            v1,
            v2,
            envelope,
        }
    }
}

/// Simulated transient state waveforms of PT-symmetric coupled circuit.
#[derive(Debug, Clone, PartialEq)]
pub struct PtCircuitState {
    /// Simulation time points in seconds.
    pub time: Vec<f64>,
    /// Voltage waveform across gain resonator tank v_1(t).
    pub v1: Vec<f64>,
    /// Voltage waveform across loss resonator tank v_2(t).
    pub v2: Vec<f64>,
    /// Combined envelope amplitude sqrt(v_1^2 + v_2^2).
    pub envelope: Vec<f64>,
}

/// 1D Non-Hermitian Skin Effect (NHSE) lattice model with asymmetric directional hopping.
#[derive(Debug, Clone, PartialEq)]
pub struct NhseLattice {
    /// Number of lattice sites M.
    pub num_sites: usize,
    /// Hopping amplitude toward left neighbor (site j+1 -> site j).
    pub t_left: f64,
    /// Hopping amplitude toward right neighbor (site j -> site j+1).
    pub t_right: f64,
    /// On-site resonant potential.
    pub on_site_potential: f64,
}

impl Default for NhseLattice {
    fn default() -> Self {
        Self {
            num_sites: 20,
            t_left: 1.0,
            t_right: 2.0,
            on_site_potential: 0.0,
        }
    }
}

/// Result of non-Hermitian skin effect eigenmode decomposition.
#[derive(Debug, Clone, PartialEq)]
pub struct NhseResult {
    /// Eigenvalues under Open Boundary Conditions (OBC).
    pub eigenvalues: Vec<Complex>,
    /// Normalized spatial probability density |psi_m(j)|^2 for each eigenmode.
    pub eigenmode_profiles: Vec<Vec<f64>>,
    /// Ratio of boundary intensity to opposite edge intensity.
    pub boundary_localization_ratio: f64,
    /// Characteristic skin localization depth in unit cells.
    pub skin_depth: f64,
    /// True if bulk modes localize toward the right boundary.
    pub is_localized_right: bool,
    /// True if bulk modes localize toward the left boundary.
    pub is_localized_left: bool,
}

impl NhseLattice {
    /// Creates a new NHSE lattice with asymmetric directional coupling.
    pub fn new(num_sites: usize, t_left: f64, t_right: f64) -> Self {
        Self {
            num_sites: num_sites.max(3),
            t_left: t_left.abs().max(1e-6),
            t_right: t_right.abs().max(1e-6),
            on_site_potential: 0.0,
        }
    }

    /// Assembles non-Hermitian Hamiltonian matrix under Open Boundary Conditions.
    pub fn assemble_hamiltonian(&self) -> Vec<Vec<Complex>> {
        let m = self.num_sites;
        let mut mat = vec![vec![Complex::ZERO; m]; m];
        for i in 0..m {
            mat[i][i] = Complex::from_real(self.on_site_potential);
            if i + 1 < m {
                mat[i][i + 1] = Complex::from_real(self.t_right);
                mat[i + 1][i] = Complex::from_real(self.t_left);
            }
        }
        mat
    }

    /// Solves bulk eigenmodes and demonstrates directional skin localization under OBC.
    ///
    /// Via similarity transform S = diag(1, r, r^2, ..., r^{M-1}) with r = sqrt(t_R / t_L):
    ///   E_m = on_site + 2 * sqrt(t_L * t_R) * cos(m * pi / (M + 1))
    ///   psi_m(j) = r^j * sin((j + 1) * m * pi / (M + 1))
    pub fn solve_eigenmodes(&self) -> NhseResult {
        let m = self.num_sites;
        let tl = self.t_left;
        let tr = self.t_right;
        let r = (tr / tl).sqrt();

        let mut eigenvalues = Vec::with_capacity(m);
        let mut eigenmode_profiles = Vec::with_capacity(m);

        let geo_coupling = (tl * tr).sqrt();

        for mode in 1..=m {
            let angle = (mode as f64) * PI / ((m + 1) as f64);
            let energy = self.on_site_potential + 2.0 * geo_coupling * angle.cos();
            eigenvalues.push(Complex::from_real(energy));

            let mut profile = Vec::with_capacity(m);
            let mut sum_sq = 0.0f64;

            for j in 0..m {
                let envelope = r.powi(j as i32);
                let standing = (((j + 1) as f64) * angle).sin();
                let psi = envelope * standing;
                let prob = psi * psi;
                profile.push(prob);
                sum_sq += prob;
            }

            let norm = sum_sq.max(1e-18);
            for p in profile.iter_mut() {
                *p /= norm;
            }

            eigenmode_profiles.push(profile);
        }

        // Evaluate boundary localization ratio
        let mut total_ratio = 0.0f64;
        let count = eigenmode_profiles.len();
        for p in &eigenmode_profiles {
            let p_left = p[0].max(1e-15);
            let p_right = p[m - 1].max(1e-15);
            if tr >= tl {
                total_ratio += p_right / p_left;
            } else {
                total_ratio += p_left / p_right;
            }
        }
        let avg_ratio = total_ratio / (count as f64);

        let ln_r = r.ln().abs();
        let skin_depth = if ln_r <= 1e-12 {
            f64::INFINITY
        } else {
            1.0 / ln_r
        };

        let is_localized_right = tr > tl;
        let is_localized_left = tl > tr;

        NhseResult {
            eigenvalues,
            eigenmode_profiles,
            boundary_localization_ratio: avg_ratio,
            skin_depth,
            is_localized_right,
            is_localized_left,
        }
    }
}
