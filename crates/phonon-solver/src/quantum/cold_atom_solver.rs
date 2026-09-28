//! Split-Step Fourier Gross-Pitaevskii (GPE) & Schrödinger Wavepacket Propagator,
//! Decoherence & Dephasing Models, and Bayesian Fringe Fitting for Cold Atom Gravimetry.
//!
//! Formulates:
//! - 1D/3D Split-Step Fourier GPE & Schrödinger propagator:
//!   $$i\hbar \frac{\partial \psi}{\partial t} = \left(-\frac{\hbar^2 \nabla^2}{2m} + V_{trap}(\mathbf{r}) + m \mathbf{g}\cdot\mathbf{r} + g_{1D}|\psi|^2\right)\psi$$
//! - Pure safe-Rust Radix-2 Cooley-Tukey Fast Fourier Transform (FFT).
//! - Decoherence & dephasing: laser phase noise $\sigma_\phi$, wavepacket thermal expansion,
//!   and background gas collision losses.
//! - Sinusoidal least-squares fringe fitting and sequential Bayesian phase estimation
//!   extracting gravitational acceleration $g$ and vertical gravity gradient $T_{zz}$.

use phonon_core::constants::{BOLTZMANN_CONSTANT, H_BAR};
use phonon_models::quantum::{AtomicSpecies, Complex, RB87_MASS_KG, STANDARD_GRAVITY_M_S2};

/// Pure safe-Rust Radix-2 Cooley-Tukey Fast Fourier Transform (FFT).
pub struct SafeFourierTransform;

impl SafeFourierTransform {
    /// Computes the forward discrete Fourier transform $\tilde{X}_k = \sum_{n=0}^{N-1} x_n e^{-i 2\pi k n / N}$.
    /// Requires slice length $N = 2^m$.
    pub fn forward(buffer: &mut [Complex]) {
        Self::fft_radix2(buffer, false);
    }

    /// Computes the inverse discrete Fourier transform $x_n = \frac{1}{N} \sum_{k=0}^{N-1} \tilde{X}_k e^{+i 2\pi k n / N}$.
    /// Requires slice length $N = 2^m$.
    pub fn inverse(buffer: &mut [Complex]) {
        Self::fft_radix2(buffer, true);
        let n = buffer.len() as f64;
        if n > 0.0 {
            let inv_n = 1.0 / n;
            for val in buffer.iter_mut() {
                *val = val.scale(inv_n);
            }
        }
    }

    fn fft_radix2(buffer: &mut [Complex], is_inverse: bool) {
        let n = buffer.len();
        if n <= 1 {
            return;
        }
        assert!(
            n.is_power_of_two(),
            "FFT buffer length must be a power of two"
        );

        // 1. Bit-reversal permutation
        let mut j = 0;
        for i in 0..n {
            if i < j {
                buffer.swap(i, j);
            }
            let mut bit = n >> 1;
            while (j & bit) != 0 {
                j ^= bit;
                bit >>= 1;
            }
            j ^= bit;
        }

        // 2. Cooley-Tukey butterfly stages
        let sign = if is_inverse { 1.0 } else { -1.0 };
        let mut len = 2;
        while len <= n {
            let half = len / 2;
            let angle_step = sign * 2.0 * std::f64::consts::PI / (len as f64);
            let w_step = Complex::cis(angle_step);

            let mut i = 0;
            while i < n {
                let mut w = Complex::ONE;
                for k in 0..half {
                    let u = buffer[i + k];
                    let v = buffer[i + k + half].mul(w);
                    buffer[i + k] = u.add(v);
                    buffer[i + k + half] = u.sub(v);
                    w = w.mul(w_step);
                }
                i += len;
            }
            len <<= 1;
        }
    }
}

/// 1D Split-Step Fourier Gross-Pitaevskii (GPE) and Schrödinger wavepacket propagator.
#[derive(Debug, Clone)]
pub struct Gpe1DPropagator {
    /// Target atomic species.
    pub species: AtomicSpecies,
    /// Spatial grid size $N = 2^k$.
    pub grid_points: usize,
    /// Total spatial window size $L$ in meters ($m$).
    pub spatial_window_m: f64,
    /// Spatial coordinate array $x_j$ in meters ($m$).
    pub x_coords: Vec<f64>,
    /// Momentum coordinate array $k_j$ in $\text{rad/m}$.
    pub k_coords: Vec<f64>,
    /// Static external trap potential $V_{trap}(x_j)$ in Joules ($J$).
    pub trap_potential: Vec<f64>,
    /// Gravitational acceleration $g$ along the coordinate axis in $\text{m/s}^2$.
    pub gravity_acceleration: f64,
    /// 1D contact interaction strength $g_{1D}$ in $\text{J}\cdot\text{m}$.
    pub g_1d: f64,
    /// Complex atomic wavefunction $\psi(x_j)$.
    pub wavefunction: Vec<Complex>,
}

impl Gpe1DPropagator {
    /// Creates a 1D GPE propagator on domain $[-L/2, L/2]$ with $N$ points.
    pub fn new(species: AtomicSpecies, grid_points: usize, spatial_window_m: f64) -> Self {
        assert!(
            grid_points.is_power_of_two(),
            "Grid points must be a power of two"
        );
        let n = grid_points;
        let dx = spatial_window_m / (n as f64);
        let x_min = -0.5 * spatial_window_m;

        let mut x_coords = Vec::with_capacity(n);
        for j in 0..n {
            x_coords.push(x_min + (j as f64) * dx);
        }

        // Standard FFT frequency ordering
        let dk = 2.0 * std::f64::consts::PI / spatial_window_m;
        let mut k_coords = Vec::with_capacity(n);
        let half = n / 2;
        for j in 0..n {
            let k = if j < half {
                (j as f64) * dk
            } else {
                ((j as isize - n as isize) as f64) * dk
            };
            k_coords.push(k);
        }

        let trap_potential = vec![0.0; n];
        let wavefunction = vec![Complex::ZERO; n];

        Self {
            species,
            grid_points,
            spatial_window_m,
            x_coords,
            k_coords,
            trap_potential,
            gravity_acceleration: STANDARD_GRAVITY_M_S2,
            g_1d: 0.0,
            wavefunction,
        }
    }

    /// Spatial grid spacing $\Delta x = L / N$ in meters.
    #[inline]
    pub fn dx(&self) -> f64 {
        self.spatial_window_m / (self.grid_points as f64)
    }

    /// Initializes a normalized Gaussian wavepacket:
    /// $$\psi_0(x) = \frac{1}{(\pi \sigma_0^2)^{1/4}} \exp\left( -\frac{(x - x_0)^2}{2\sigma_0^2} \right) \exp(i k_0 x)$$
    pub fn initialize_gaussian(&mut self, x0: f64, sigma0: f64, k0: f64) {
        let norm_const = 1.0 / (std::f64::consts::PI * sigma0 * sigma0).powf(0.25);
        for (j, x) in self.x_coords.iter().enumerate() {
            let diff = *x - x0;
            let envelope = norm_const * (-0.5 * diff * diff / (sigma0 * sigma0)).exp();
            let phase = k0 * *x;
            self.wavefunction[j] = Complex::cis(phase).scale(envelope);
        }
        self.normalize();
    }

    /// Sets up a harmonic oscillator confinement potential $V_{trap}(x) = \frac{1}{2} m \omega_x^2 x^2$.
    pub fn set_harmonic_trap(&mut self, trap_freq_rad_s: f64) {
        let m = self.species.mass_kg();
        let half_m_omega2 = 0.5 * m * trap_freq_rad_s * trap_freq_rad_s;
        for (j, x) in self.x_coords.iter().enumerate() {
            self.trap_potential[j] = half_m_omega2 * x * x;
        }
    }

    /// Normalizes the wavefunction so that $\int |\psi(x)|^2 dx = 1$.
    pub fn normalize(&mut self) {
        let dx = self.dx();
        let total_norm_sq: f64 = self.wavefunction.iter().map(|c| c.norm_sq()).sum::<f64>() * dx;
        if total_norm_sq > 1e-30 {
            let scale = 1.0 / total_norm_sq.sqrt();
            for val in self.wavefunction.iter_mut() {
                *val = val.scale(scale);
            }
        }
    }

    /// Total integrated norm $\int |\psi|^2 dx$.
    pub fn total_norm(&self) -> f64 {
        self.wavefunction.iter().map(|c| c.norm_sq()).sum::<f64>() * self.dx()
    }

    /// Expectation value of position $\langle x \rangle = \int x |\psi|^2 dx$ in meters.
    pub fn expectation_position(&self) -> f64 {
        let dx = self.dx();
        let mut sum = 0.0;
        for (j, x) in self.x_coords.iter().enumerate() {
            sum += *x * self.wavefunction[j].norm_sq() * dx;
        }
        sum
    }

    /// Spatial wavepacket root-mean-square width $\sigma_x = \sqrt{\langle x^2 \rangle - \langle x \rangle^2}$ in meters.
    pub fn position_width(&self) -> f64 {
        let dx = self.dx();
        let mean_x = self.expectation_position();
        let mut mean_x2 = 0.0;
        for (j, x) in self.x_coords.iter().enumerate() {
            mean_x2 += *x * *x * self.wavefunction[j].norm_sq() * dx;
        }
        (mean_x2 - mean_x * mean_x).max(0.0).sqrt()
    }

    /// Applies a two-photon laser momentum kick:
    /// $$\psi(x) \leftarrow \psi(x) e^{i k_{kick} x}$$
    pub fn apply_momentum_kick(&mut self, k_kick: f64) {
        for (j, x) in self.x_coords.iter().enumerate() {
            let kick_phase = Complex::cis(k_kick * *x);
            self.wavefunction[j] = self.wavefunction[j].mul(kick_phase);
        }
    }

    /// Applies an ideal $\pi/2$ beam splitter splitting the cloud into equal unkicked and kicked branches:
    /// $$\psi(x) \leftarrow \frac{1}{\sqrt{2}} \psi(x) + \frac{i}{\sqrt{2}} \psi(x) e^{i k_{kick} x}$$
    pub fn apply_beam_splitter(&mut self, k_kick: f64) {
        let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
        let i_over_sqrt2 = Complex::new(0.0, inv_sqrt2);
        for (j, x) in self.x_coords.iter().enumerate() {
            let orig = self.wavefunction[j];
            let kicked = orig.mul(Complex::cis(k_kick * *x));
            self.wavefunction[j] = orig.scale(inv_sqrt2).add(kicked.mul(i_over_sqrt2));
        }
        self.normalize();
    }

    /// Executes a single time step $\Delta t$ using 2nd-order Strang Split-Step Fourier method:
    /// 1. Position half-step: $e^{-i \hat{V} \Delta t / (2\hbar)}$
    /// 2. Momentum full-step: $e^{-i \hat{T} \Delta t / \hbar}$ via FFT
    /// 3. Position half-step: $e^{-i \hat{V} \Delta t / (2\hbar)}$ via IFFT
    pub fn step_time(&mut self, dt: f64) {
        let m = self.species.mass_kg();
        let half_dt = 0.5 * dt;

        // Stage 1: Half-step position evolution
        self.apply_potential_step(half_dt);

        // Stage 2: Full-step kinetic evolution in momentum space
        SafeFourierTransform::forward(&mut self.wavefunction);
        let kinetic_factor = -dt / H_BAR;
        for (j, k) in self.k_coords.iter().enumerate() {
            let kinetic_energy = (H_BAR * H_BAR * *k * *k) / (2.0 * m);
            let phase = kinetic_factor * kinetic_energy;
            let propagator = Complex::cis(phase);
            self.wavefunction[j] = self.wavefunction[j].mul(propagator);
        }
        SafeFourierTransform::inverse(&mut self.wavefunction);

        // Stage 3: Second half-step position evolution
        self.apply_potential_step(half_dt);
    }

    /// Propagates the wavefunction over duration $t_{total}$ with step size $\Delta t$.
    pub fn propagate(&mut self, total_time: f64, dt: f64) {
        let steps = (total_time / dt).round().max(1.0) as usize;
        let actual_dt = total_time / (steps as f64);
        for _ in 0..steps {
            self.step_time(actual_dt);
        }
    }

    fn apply_potential_step(&mut self, dt_half: f64) {
        let m = self.species.mass_kg();
        let factor = -dt_half / H_BAR;
        for (j, x) in self.x_coords.iter().enumerate() {
            let v_trap = self.trap_potential[j];
            let v_grav = m * self.gravity_acceleration * *x;
            let v_int = self.g_1d * self.wavefunction[j].norm_sq();
            let v_total = v_trap + v_grav + v_int;

            let phase = factor * v_total;
            let propagator = Complex::cis(phase);
            self.wavefunction[j] = self.wavefunction[j].mul(propagator);
        }
    }
}

/// Decoherence and dephasing mechanisms attenuating matter-wave fringe visibility.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColdAtomDecoherenceModel {
    /// Root-mean-square laser phase noise $\sigma_\phi$ in radians.
    pub laser_phase_noise_rad: f64,
    /// Vacuum chamber residual background gas pressure in Pascals ($Pa$) (e.g. $10^{-8} - 10^{-10}\text{ Pa}$).
    pub background_pressure_pa: f64,
    /// Atom cloud kinetic temperature $T_{atom}$ in Kelvin ($K$) (e.g. $1\,\mu\text{K}$).
    pub atom_temperature_kelvin: f64,
    /// Total interferometer baseline interrogation time $2T$ in seconds ($s$).
    pub interrogation_duration_s: f64,
}

impl ColdAtomDecoherenceModel {
    /// Constructs a standard high-vacuum decoherence profile.
    pub fn standard_high_vacuum(interrogation_duration_s: f64) -> Self {
        Self {
            laser_phase_noise_rad: 0.05,     // 50 mrad phase noise
            background_pressure_pa: 1.0e-8,  // 1e-8 Pa ultra-high vacuum
            atom_temperature_kelvin: 2.0e-6, // 2.0 uK cold cloud
            interrogation_duration_s,
        }
    }

    /// Contrast loss factor from laser phase fluctuations:
    /// $$C_{laser} = \exp\left( -\frac{\sigma_\phi^2}{2} \right)$$
    #[inline]
    pub fn laser_phase_contrast_factor(&self) -> f64 {
        let s = self.laser_phase_noise_rad;
        (-0.5 * s * s).exp()
    }

    /// Atom survival fraction after background gas collision loss over duration $t = 2T$:
    /// $$\Gamma_{bg} = n_{bg} \sigma_{coll} \bar{v}_{bg}$$
    /// $$\eta_{bg} = \exp(-\Gamma_{bg} t)$$
    pub fn background_gas_survival_fraction(&self) -> f64 {
        let t_bg = 300.0; // 300 K background gas temperature
        let n_bg = self.background_pressure_pa / (BOLTZMANN_CONSTANT * t_bg);
        let m_h2 = 2.0 * 1.66e-27; // Hydrogen gas molecules
        let v_avg = (8.0 * BOLTZMANN_CONSTANT * t_bg / (std::f64::consts::PI * m_h2)).sqrt();
        let sigma_coll = 3.0e-18; // ~3e-18 m^2 collisional cross section
        let gamma_bg = n_bg * sigma_coll * v_avg;

        (-gamma_bg * self.interrogation_duration_s)
            .exp()
            .clamp(0.0, 1.0)
    }

    /// Wavepacket thermal wavefront curvature and expansion dephasing factor:
    /// $$C_{thermal} = \frac{1}{\sqrt{1 + (t / \tau_{exp})^2}}$$
    pub fn thermal_expansion_contrast_factor(&self) -> f64 {
        let m = RB87_MASS_KG;
        let v_th = (BOLTZMANN_CONSTANT * self.atom_temperature_kelvin / m).sqrt();
        let beam_waist = 0.015; // 15 mm laser beam waist
        let tau_exp = beam_waist / v_avg_clamped(v_th);
        let ratio = self.interrogation_duration_s / tau_exp;
        (1.0 / (1.0 + ratio * ratio).sqrt()).clamp(0.0, 1.0)
    }

    /// Overall combined fringe contrast $C_{net} = C_0 \cdot C_{laser} \cdot \eta_{bg} \cdot C_{thermal}$.
    pub fn net_fringe_contrast(&self, intrinsic_contrast: f64) -> f64 {
        intrinsic_contrast
            * self.laser_phase_contrast_factor()
            * self.background_gas_survival_fraction()
            * self.thermal_expansion_contrast_factor()
    }
}

#[inline(always)]
fn v_avg_clamped(v: f64) -> f64 {
    v.max(1e-6)
}

/// Extracted matter-wave fringe parameters from sinusoidal regression.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FringeFitResult {
    /// Extracted fringe contrast $C \in [0, 1]$.
    pub contrast: f64,
    /// Extracted matter-wave phase $\Delta\Phi$ in radians.
    pub phase_rad: f64,
    /// Mean baseline population offset $y_0$.
    pub baseline_offset: f64,
    /// Root-mean-square fit residual error.
    pub rmse_residual: f64,
}

/// Sinusoidal least-squares matter-wave fringe fitting engine.
pub struct ColdAtomFringeFitter;

impl ColdAtomFringeFitter {
    /// Fits a scan of measured excited state populations $P_i$ at scan phases $\theta_i$ to:
    /// $$P(\theta) = y_0 - \frac{C}{2} \cos(\Delta\Phi + \theta) = y_0 + A \cos(\theta) + B \sin(\theta)$$
    /// where $A = -\frac{C}{2}\cos(\Delta\Phi)$ and $B = \frac{C}{2}\sin(\Delta\Phi)$.
    pub fn fit_sinusoid(scan_phases: &[f64], populations: &[f64]) -> FringeFitResult {
        assert_eq!(scan_phases.len(), populations.len());
        let n = scan_phases.len();
        assert!(
            n >= 3,
            "At least 3 scan phase points required for sinusoidal fit"
        );

        // Set up 3x3 normal equations for linear least squares [1, cos(theta), sin(theta)]
        let mut s_1 = 0.0;
        let mut s_c = 0.0;
        let mut s_s = 0.0;
        let mut s_cc = 0.0;
        let mut s_ss = 0.0;
        let mut s_cs = 0.0;
        let mut s_y = 0.0;
        let mut s_yc = 0.0;
        let mut s_ys = 0.0;

        for i in 0..n {
            let theta = scan_phases[i];
            let y = populations[i];
            let c = theta.cos();
            let s = theta.sin();

            s_1 += 1.0;
            s_c += c;
            s_s += s;
            s_cc += c * c;
            s_ss += s * s;
            s_cs += c * s;
            s_y += y;
            s_yc += y * c;
            s_ys += y * s;
        }

        // Solve 3x3 linear system M * beta = Y via Cramer's rule
        let det = s_1 * (s_cc * s_ss - s_cs * s_cs) - s_c * (s_c * s_ss - s_cs * s_s)
            + s_s * (s_c * s_cs - s_cc * s_s);

        let (y0, a, b) = if det.abs() > 1e-12 {
            let det_y0 = s_y * (s_cc * s_ss - s_cs * s_cs) - s_c * (s_yc * s_ss - s_cs * s_ys)
                + s_s * (s_yc * s_cs - s_cc * s_ys);
            let det_a = s_1 * (s_yc * s_ss - s_cs * s_ys) - s_y * (s_c * s_ss - s_cs * s_s)
                + s_s * (s_c * s_ys - s_yc * s_s);
            let det_b = s_1 * (s_cc * s_ys - s_yc * s_cs) - s_c * (s_c * s_ys - s_s * s_yc)
                + s_y * (s_c * s_cs - s_cc * s_s);

            (det_y0 / det, det_a / det, det_b / det)
        } else {
            (s_y / (n as f64), 0.0, 0.0)
        };

        // Extract contrast C and phase DeltaPhi
        let r = (a * a + b * b).sqrt();
        let contrast = (2.0 * r).clamp(0.0, 1.0);
        // A = -C/2 cos(Phi), B = C/2 sin(Phi) => -A = C/2 cos(Phi), B = C/2 sin(Phi)
        let phase_rad = b.atan2(-a);

        // Compute RMSE residual
        let mut s_res = 0.0;
        for i in 0..n {
            let theta = scan_phases[i];
            let model = y0 + a * theta.cos() + b * theta.sin();
            let err = populations[i] - model;
            s_res += err * err;
        }
        let rmse_residual = (s_res / (n as f64)).sqrt();

        FringeFitResult {
            contrast,
            phase_rad,
            baseline_offset: y0,
            rmse_residual,
        }
    }

    /// Extracts gravitational acceleration $g$ from fitted phase shift $\Delta\Phi$:
    /// $$g = \frac{\Delta\Phi}{k_{eff} T^2}$$
    pub fn extract_gravitational_acceleration(
        phase_rad: f64,
        k_eff: f64,
        t_interrogation: f64,
    ) -> f64 {
        let t2 = t_interrogation * t_interrogation;
        phase_rad / (k_eff * t2)
    }

    /// Extracts vertical gravity gradient $T_{zz} = \frac{\partial g_z}{\partial z}$ from dual fringe fits:
    /// $$T_{zz} = \frac{\Delta\Phi_{upper} - \Delta\Phi_{lower}}{k_{eff} d T^2}$$
    pub fn extract_gravity_gradient(
        phase_upper: f64,
        phase_lower: f64,
        k_eff: f64,
        baseline_d: f64,
        t_interrogation: f64,
    ) -> f64 {
        let t2 = t_interrogation * t_interrogation;
        (phase_upper - phase_lower) / (k_eff * baseline_d * t2)
    }
}

/// Sequential Bayesian phase estimator for cold atom quantum interferometers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BayesianPhaseEstimator {
    /// Current estimated phase mean $\mu_\Phi$ in radians.
    pub mean_phase: f64,
    /// Current phase estimation variance $\sigma^2_\Phi$ in $\text{rad}^2$.
    pub variance: f64,
    /// Total measurements integrated into estimate.
    pub measurement_count: usize,
}

impl BayesianPhaseEstimator {
    /// Initializes a Gaussian prior $\Phi \sim \mathcal{N}(\mu_0, \sigma_0^2)$.
    pub fn with_prior(prior_mean: f64, prior_variance: f64) -> Self {
        Self {
            mean_phase: prior_mean,
            variance: prior_variance.max(1e-12),
            measurement_count: 0,
        }
    }

    /// Updates the posterior estimate with a new noisy phase measurement $\phi_{obs}$
    /// with observation variance $\sigma_{obs}^2 = \frac{1}{C^2 N_{atoms}}$:
    /// $$\frac{1}{\sigma_{post}^2} = \frac{1}{\sigma_{prior}^2} + \frac{1}{\sigma_{obs}^2}$$
    /// $$\mu_{post} = \sigma_{post}^2 \left( \frac{\mu_{prior}}{\sigma_{prior}^2} + \frac{\phi_{obs}}{\sigma_{obs}^2} \right)$$
    pub fn update(&mut self, observed_phase: f64, observation_variance: f64) {
        let var_obs = observation_variance.max(1e-12);
        let precision_prior = 1.0 / self.variance;
        let precision_obs = 1.0 / var_obs;

        let precision_post = precision_prior + precision_obs;
        let var_post = 1.0 / precision_post;

        let mean_post =
            var_post * (self.mean_phase * precision_prior + observed_phase * precision_obs);

        self.mean_phase = mean_post;
        self.variance = var_post;
        self.measurement_count += 1;
    }

    /// Phase estimation standard uncertainty $\sigma_\Phi = \sqrt{\sigma^2_\Phi}$ in radians.
    #[inline]
    pub fn uncertainty_rad(&self) -> f64 {
        self.variance.sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_radix2_fft_roundtrip() {
        let mut buffer = vec![
            Complex::new(1.0, 0.0),
            Complex::new(2.0, 0.0),
            Complex::new(3.0, 0.0),
            Complex::new(4.0, 0.0),
            Complex::new(0.0, 1.0),
            Complex::new(-1.0, 0.0),
            Complex::new(0.0, -2.0),
            Complex::new(1.5, 0.5),
        ];
        let original = buffer.clone();

        SafeFourierTransform::forward(&mut buffer);
        SafeFourierTransform::inverse(&mut buffer);

        for (orig, recon) in original.iter().zip(buffer.iter()) {
            assert!((orig.re - recon.re).abs() < 1e-12);
            assert!((orig.im - recon.im).abs() < 1e-12);
        }
    }

    #[test]
    fn test_gpe_1d_wavepacket_propagation_and_norm_conservation() {
        let mut gpe = Gpe1DPropagator::new(AtomicSpecies::Rubidium87, 128, 20e-6);
        gpe.initialize_gaussian(0.0, 2e-6, 0.0);
        let w_initial = gpe.position_width();
        let initial_norm = gpe.total_norm();
        assert!((initial_norm - 1.0).abs() < 1e-6);

        // Propagate for 100 microseconds in zero trap
        gpe.gravity_acceleration = 0.0;
        gpe.propagate(100e-6, 1e-6);

        let final_norm = gpe.total_norm();
        assert!((final_norm - 1.0).abs() < 1e-5);

        // Wavepacket expands under quantum kinetic dispersion
        let w_final = gpe.position_width();
        assert!(w_final >= w_initial);
    }

    #[test]
    fn test_cold_atom_decoherence_contrast_decay() {
        let deco = ColdAtomDecoherenceModel::standard_high_vacuum(0.1);
        let net_c = deco.net_fringe_contrast(0.95);
        assert!(net_c > 0.70);
        assert!(net_c <= 0.95);
    }

    #[test]
    fn test_fringe_fitting_and_gravitational_acceleration() {
        let true_phase = 1.25; // radians
        let true_contrast = 0.88;
        let mut scan_phases = Vec::new();
        let mut populations = Vec::new();

        for i in 0..16 {
            let theta = (i as f64) * 2.0 * std::f64::consts::PI / 16.0;
            let pop = 0.5 * (1.0 - true_contrast * (true_phase + theta).cos());
            scan_phases.push(theta);
            populations.push(pop);
        }

        let fit = ColdAtomFringeFitter::fit_sinusoid(&scan_phases, &populations);
        assert!((fit.contrast - true_contrast).abs() < 1e-6);
        assert!((fit.phase_rad - true_phase).abs() < 1e-6);
        assert!(fit.rmse_residual < 1e-6);

        // Reconstruct g
        let k_eff = 1.61e7;
        let t = 0.05;
        let g = ColdAtomFringeFitter::extract_gravitational_acceleration(fit.phase_rad, k_eff, t);
        assert!(g > 0.0);
    }

    #[test]
    fn test_bayesian_phase_estimator_convergence() {
        let mut estimator = BayesianPhaseEstimator::with_prior(0.0, 1.0);
        let true_phase = 0.75;
        let obs_var = 0.04; // sigma = 0.2 rad

        for _ in 0..50 {
            estimator.update(true_phase, obs_var);
        }

        assert!((estimator.mean_phase - true_phase).abs() < 0.02);
        assert!(estimator.uncertainty_rad() < 0.04);
    }
}
