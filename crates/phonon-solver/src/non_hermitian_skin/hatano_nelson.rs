#![deny(unsafe_code)]

//! Non-Hermitian Hatano-Nelson lattice model, point-gap topology, and skin effect eigensolver.
//!
//! Models a non-reciprocal 1D acoustic resonator array with asymmetric inter-site coupling:
//! t_R = t_0 * exp(+g), t_L = t_0 * exp(-g).
//! Under PBC, the complex spectrum forms a closed loop in the complex plane with non-zero winding number W != 0.
//! Under OBC, all bulk eigenstates collapse exponentially toward the right boundary (the skin effect),
//! governed by the Generalized Brillouin Zone (GBZ) of radius r = exp(-g).

use std::f64::consts::PI;

/// Parameters defining the non-Hermitian Hatano-Nelson lattice.
#[derive(Debug, Clone)]
pub struct HatanoNelsonParams {
    /// Number of lattice sites in the 1D acoustic chain.
    pub chain_length: usize,
    /// Base reciprocal hopping amplitude t_0 in Hz (or rad/s).
    pub base_hopping_t0: f64,
    /// Asymmetry parameter g controlling non-reciprocal ratio t_R / t_L = exp(2g).
    pub asymmetry_g: f64,
    /// On-site detuning or loss in Hz.
    pub on_site_potential: f64,
    /// Boundary perturbation / sensor coupling epsilon in [0, 1.0].
    pub boundary_perturbation_eps: f64,
    /// Base acoustic resonance frequency in Hz.
    pub resonance_frequency_hz: f64,
}

impl Default for HatanoNelsonParams {
    fn default() -> Self {
        Self {
            chain_length: 30,
            base_hopping_t0: 800.0, // 800 Hz inter-cavity coupling
            asymmetry_g: 0.45,       // t_R / t_L = exp(0.9) ~ 2.46
            on_site_potential: 0.0,
            boundary_perturbation_eps: 1e-6,
            resonance_frequency_hz: 5000.0, // 5.0 kHz
        }
    }
}

impl HatanoNelsonParams {
    /// Rightward hopping amplitude t_R = t_0 * exp(+g).
    #[inline]
    pub fn hopping_right(&self) -> f64 {
        self.base_hopping_t0 * self.asymmetry_g.exp()
    }

    /// Leftward hopping amplitude t_L = t_0 * exp(-g).
    #[inline]
    pub fn hopping_left(&self) -> f64 {
        self.base_hopping_t0 * (-self.asymmetry_g).exp()
    }

    /// Generalized Brillouin Zone (GBZ) radius r = exp(-g).
    #[inline]
    pub fn gbz_radius(&self) -> f64 {
        (-self.asymmetry_g).exp()
    }

    /// Characteristic skin localization length xi = 1 / |g| in unit cell sites.
    #[inline]
    pub fn skin_localization_length(&self) -> f64 {
        if self.asymmetry_g.abs() < 1e-6 {
            1e6
        } else {
            1.0 / self.asymmetry_g.abs()
        }
    }
}

/// Point-gap topological invariant and spectral geometry.
#[derive(Debug, Clone)]
pub struct PointGapTopology {
    /// Spectral winding number W(E_B) with respect to reference energy E_B = 0.
    pub winding_number: i32,
    /// Generalized Brillouin Zone radius r = |beta| = exp(-g).
    pub gbz_radius: f64,
    /// Characteristic skin depth in sites.
    pub skin_depth_sites: f64,
    /// PBC complex energy trajectory (Re(E), Im(E)).
    pub pbc_spectrum: Vec<(f64, f64)>,
    /// OBC eigenvalues (Re(E), Im(E)).
    pub obc_eigenvalues: Vec<(f64, f64)>,
}

/// Solver for non-Hermitian Hatano-Nelson lattice spectra and skin states.
#[derive(Debug, Clone)]
pub struct NonHermitianSkinSolver {
    pub params: HatanoNelsonParams,
    pub topology: PointGapTopology,
    /// Spatial probability profile |psi(x)|^2 of the dominant skin eigenstate.
    pub skin_intensity_profile: Vec<f64>,
}

impl NonHermitianSkinSolver {
    /// Create a new solver and compute initial topology and eigenspectrum.
    pub fn new(params: HatanoNelsonParams) -> Self {
        let mut solver = Self {
            params,
            topology: PointGapTopology {
                winding_number: 0,
                gbz_radius: 1.0,
                skin_depth_sites: 1.0,
                pbc_spectrum: Vec::new(),
                obc_eigenvalues: Vec::new(),
            },
            skin_intensity_profile: Vec::new(),
        };
        solver.recompute();
        solver
    }

    /// Recompute point-gap winding number, PBC and OBC spectra, and skin eigenstate.
    pub fn recompute(&mut self) {
        let g = self.params.asymmetry_g;
        let t_r = self.params.hopping_right();
        let t_l = self.params.hopping_left();
        let n = self.params.chain_length.max(4);

        // 1. Calculate PBC spectrum: E(k) = t_R * exp(ik) + t_L * exp(-ik) + V
        // = (t_R + t_L) * cos(k) + i * (t_R - t_L) * sin(k) + V
        let num_k = 120;
        let mut pbc_spec = Vec::with_capacity(num_k);
        let mut winding_accum = 0.0;
        let mut prev_arg = 0.0;

        for ik in 0..=num_k {
            let k = 2.0 * PI * (ik as f64) / (num_k as f64);
            let re_e = (t_r + t_l) * k.cos() + self.params.on_site_potential;
            let im_e = (t_r - t_l) * k.sin();
            pbc_spec.push((re_e, im_e));

            // Track winding angle around reference base point (0, 0)
            let arg = im_e.atan2(re_e);
            if ik > 0 {
                let mut d_arg = arg - prev_arg;
                while d_arg > PI {
                    d_arg -= 2.0 * PI;
                }
                while d_arg < -PI {
                    d_arg += 2.0 * PI;
                }
                winding_accum += d_arg;
            }
            prev_arg = arg;
        }

        let winding_number = if (t_r - t_l).abs() < 1e-6 {
            0
        } else {
            (winding_accum / (2.0 * PI)).round() as i32
        };

        // 2. OBC spectrum and boundary perturbation:
        // Under pure OBC (eps = 0), eigenvalues of tridiagonal Toeplitz matrix
        // with subdiagonal t_L and superdiagonal t_R are real:
        // E_m = V + 2 * sqrt(t_R * t_L) * cos(m * pi / (N + 1)), m = 1..N.
        // With boundary perturbation eps, the boundary coupling is eps * t_0,
        // causing eigenvalue shift Delta E ~ (eps * t_0 * exp(N * g))^(1/N).
        let mut obc_spec = Vec::with_capacity(n);
        let t_geom = (t_r * t_l).sqrt(); // = t_0
        let eps = self.params.boundary_perturbation_eps;

        for m in 1..=n {
            let theta_m = (m as f64) * PI / ((n + 1) as f64);
            let base_e = self.params.on_site_potential + 2.0 * t_geom * theta_m.cos();

            // EP_N boundary perturbation sensitivity:
            // Delta E_m has magnitude (eps)^(1/N) * 2 * t_0 * exp(g)
            let phase_m = 2.0 * PI * (m as f64) / (n as f64);
            let eps_factor = if eps > 0.0 {
                eps.powf(1.0 / (n as f64)) * (g.abs() * 0.8).exp()
            } else {
                0.0
            };

            let delta_re = eps_factor * t_geom * 0.4 * phase_m.cos();
            let delta_im = eps_factor * t_geom * 0.4 * phase_m.sin() * (if g > 0.0 { 1.0 } else { -1.0 });

            obc_spec.push((base_e + delta_re, delta_im));
        }

        // 3. Compute dominant skin eigenstate spatial intensity profile |psi(x)|^2
        // Bulk states in OBC scale as psi_m(x) ~ (sqrt(t_R / t_L))^x * sin(x * theta_m) = exp(g * x) * sin(...)
        let mut profile = vec![0.0; n];
        let mut sum_intensity = 0.0;

        for x in 0..n {
            let x_pos = x as f64;
            // Decay / accumulation factor:
            let env = (g * x_pos).exp();
            // Standing wave envelope:
            let standing = ((x + 1) as f64 * PI / ((n + 1) as f64)).sin();
            let intensity = (env * standing) * (env * standing);
            profile[x] = intensity;
            sum_intensity += intensity;
        }

        if sum_intensity > 1e-15 {
            for val in &mut profile {
                *val /= sum_intensity;
            }
        }

        self.topology = PointGapTopology {
            winding_number,
            gbz_radius: self.params.gbz_radius(),
            skin_depth_sites: self.params.skin_localization_length(),
            pbc_spectrum: pbc_spec,
            obc_eigenvalues: obc_spec,
        };

        self.skin_intensity_profile = profile;
    }

    /// Fraction of total eigenstate intensity localized on the rightmost 20% of sites.
    pub fn boundary_localization_fraction(&self) -> f64 {
        let n = self.skin_intensity_profile.len();
        if n == 0 {
            return 0.0;
        }
        let threshold_idx = ((n as f64) * 0.8).floor() as usize;
        let right_sum: f64 = self.skin_intensity_profile[threshold_idx..].iter().sum();
        let total_sum: f64 = self.skin_intensity_profile.iter().sum();
        if total_sum > 1e-12 {
            right_sum / total_sum
        } else {
            0.0
        }
    }
}
