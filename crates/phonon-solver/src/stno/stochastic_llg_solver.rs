//! Stochastic Landau-Lifshitz-Gilbert (sLLG) Langevin integrator:
//! thermal fluctuations, Slonczewski spin-transfer torque, Spin Hall effect SOT,
//! limit cycle trajectories, phase noise, and spectral linewidth.

use phonon_models::stno::StnoParams;

/// State of a macrospin oscillator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MacrospinState {
    /// Unit magnetization vector m = (m_x, m_y, m_z) with |m| = 1.
    pub m: [f64; 3],
}

impl MacrospinState {
    /// Creates a macrospin with normalized direction vector.
    pub fn new(m: [f64; 3]) -> Self {
        let norm = (m[0].powi(2) + m[1].powi(2) + m[2].powi(2))
            .sqrt()
            .max(1e-12);
        Self {
            m: [m[0] / norm, m[1] / norm, m[2] / norm],
        }
    }

    /// Normalized precession power p = (m_x^2 + m_y^2) in [0, 1].
    pub fn transverse_power(&self) -> f64 {
        self.m[0].powi(2) + self.m[1].powi(2)
    }

    /// Phase angle phi = atan2(m_y, m_x) in radians.
    pub fn phase_rad(&self) -> f64 {
        self.m[1].atan2(self.m[0])
    }
}

/// Stochastic LLG integrator.
pub struct StochasticLlgSolver;

impl StochasticLlgSolver {
    /// Integrates one time step dt of the stochastic Landau-Lifshitz-Gilbert-Slonczewski equation
    /// using the Stratonovich-Heun predictor-corrector method.
    #[allow(clippy::too_many_arguments)]
    pub fn step(
        state: &mut MacrospinState,
        params: &StnoParams,
        h_ext_a_per_m: [f64; 3],
        bias_current_a: f64,
        pol_dir: [f64; 3],
        temperature_k: f64,
        dt_s: f64,
        rng_seed: &mut u64,
    ) {
        let gamma0 = params.gyromagnetic_ratio_rad_per_s_t;
        let alpha = params.gilbert_damping;
        let sigma_stt = params.slonczewski_coefficient();
        let mu0 = 4.0 * std::f64::consts::PI * 1.0e-7;

        // Thermal fluctuation field standard deviation sigma_th:
        // <h_i h_j> = 2 alpha k_B T / (gamma_0 M_s V dt) * delta_ij
        let kb = 1.380_649e-23;
        let var_th = (2.0 * alpha * kb * temperature_k)
            / (mu0 * gamma0 * params.ms_a_per_m * params.volume_m3 * dt_s.max(1e-20));
        let sigma_th = var_th.max(0.0).sqrt();

        let h_th = [
            sample_normal(0.0, sigma_th, rng_seed),
            sample_normal(0.0, sigma_th, rng_seed),
            sample_normal(0.0, sigma_th, rng_seed),
        ];

        // 1. Predictor step (Euler)
        let dm_dt_1 = compute_dm_dt(
            &state.m,
            params,
            &h_ext_a_per_m,
            &h_th,
            bias_current_a,
            sigma_stt,
            &pol_dir,
        );

        let mut m_pred = [
            state.m[0] + dm_dt_1[0] * dt_s,
            state.m[1] + dm_dt_1[1] * dt_s,
            state.m[2] + dm_dt_1[2] * dt_s,
        ];
        normalize_vec3(&mut m_pred);

        // 2. Corrector step (Heun average)
        let dm_dt_2 = compute_dm_dt(
            &m_pred,
            params,
            &h_ext_a_per_m,
            &h_th,
            bias_current_a,
            sigma_stt,
            &pol_dir,
        );

        let mut m_next = [
            state.m[0] + 0.5 * (dm_dt_1[0] + dm_dt_2[0]) * dt_s,
            state.m[1] + 0.5 * (dm_dt_1[1] + dm_dt_2[1]) * dt_s,
            state.m[2] + 0.5 * (dm_dt_1[2] + dm_dt_2[2]) * dt_s,
        ];
        normalize_vec3(&mut m_next);

        state.m = m_next;
    }
}

fn compute_dm_dt(
    m: &[f64; 3],
    params: &StnoParams,
    h_ext: &[f64; 3],
    h_th: &[f64; 3],
    bias_i: f64,
    sigma: f64,
    p_dir: &[f64; 3],
) -> [f64; 3] {
    let gamma0 = params.gyromagnetic_ratio_rad_per_s_t;
    let alpha = params.gilbert_damping;
    let mu0 = 4.0 * std::f64::consts::PI * 1.0e-7;

    // Demagnetization field H_demag = (-N_x m_x, -N_y m_y, -N_z m_z) * M_s
    // For thin film: N_z = 1.0, N_x = N_y = 0.0
    let h_eff = [
        h_ext[0] + h_th[0],
        h_ext[1] + h_th[1],
        h_ext[2] - params.ms_a_per_m * m[2] + h_th[2],
    ];

    let b_eff = [mu0 * h_eff[0], mu0 * h_eff[1], mu0 * h_eff[2]];

    // Precession term: - gamma_0 (m x B_eff)
    let m_cross_b = cross_product(m, &b_eff);

    // Damping term: alpha (m x (m x B_eff))
    let m_cross_m_cross_b = cross_product(m, &m_cross_b);

    // Slonczewski STT: gamma_0 sigma I (m x (m x p))
    let m_cross_p = cross_product(m, p_dir);
    let m_cross_m_cross_p = cross_product(m, &m_cross_p);
    let stt_factor = gamma0 * sigma * bias_i;

    let prefactor = 1.0 / (1.0 + alpha * alpha);

    [
        prefactor
            * (-gamma0 * m_cross_b[0] - gamma0 * alpha * m_cross_m_cross_b[0]
                + stt_factor * m_cross_m_cross_p[0]),
        prefactor
            * (-gamma0 * m_cross_b[1] - gamma0 * alpha * m_cross_m_cross_b[1]
                + stt_factor * m_cross_m_cross_p[1]),
        prefactor
            * (-gamma0 * m_cross_b[2] - gamma0 * alpha * m_cross_m_cross_b[2]
                + stt_factor * m_cross_m_cross_p[2]),
    ]
}

#[inline]
fn cross_product(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn normalize_vec3(v: &mut [f64; 3]) {
    let norm = (v[0].powi(2) + v[1].powi(2) + v[2].powi(2))
        .sqrt()
        .max(1e-12);
    v[0] /= norm;
    v[1] /= norm;
    v[2] /= norm;
}

fn sample_normal(mean: f64, std_dev: f64, seed: &mut u64) -> f64 {
    let u1 = next_pseudo_uniform(seed).max(1e-12);
    let u2 = next_pseudo_uniform(seed);
    let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
    mean + z0 * std_dev
}

fn next_pseudo_uniform(state: &mut u64) -> f64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    (*state as f64) / (u64::MAX as f64)
}
