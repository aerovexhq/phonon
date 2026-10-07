#![deny(unsafe_code)]

//! Coupled Acoustomagnonic Polariton Dispersion and Synthetic Gauge Fields.
//!
//! Models hybrid magnon-phonon polariton modes in magnetic garnet / piezoelectric
//! heterostructures (e.g. YIG on LiNbO3). Time-reversal symmetry breaking via an in-plane
//! magnetic bias field and dynamic RF rotating synthetic gauge fields induces giant
//! non-reciprocal wavevector splitting between forward (+k) and backward (-k) SAW modes.

use std::f64::consts::PI;

/// Physical parameters for the chiral acoustomagnonic heterostructure.
#[derive(Debug, Clone)]
pub struct AcoustomagnonicParams {
    /// Center operating frequency in GHz (default ~4.0 GHz).
    pub center_freq_ghz: f64,
    /// Surface acoustic wave (SAW) Rayleigh sound velocity in m/s (default ~3480 m/s for LiNbO3).
    pub saw_velocity_ms: f64,
    /// YIG film saturation magnetization 4*pi*M_s in Gauss (default ~1750 G).
    pub saturation_magnetization_g: f64,
    /// Gyromagnetic ratio gamma / (2*pi) in GHz/T (default ~28.0 GHz/T).
    pub gyromagnetic_ratio_ghz_t: f64,
    /// External static bias magnetic field H_0 in Oersted (default ~850 Oe).
    pub bias_field_oe: f64,
    /// Magnetoelastic coupling constant b_me in J/m^3 (default ~1.5e6 J/m^3).
    pub magnetoelastic_coupling_jm3: f64,
    /// Effective magnetoelastic coupling frequency g_me / (2*pi) in MHz (default ~35.0 MHz).
    pub coupling_strength_mhz: f64,
    /// Dynamic synthetic gauge field phase modulation amplitude in radians (default ~0.45 rad).
    pub synthetic_gauge_phase_rad: f64,
    /// YIG film thickness in nm (default ~100 nm).
    pub yig_thickness_nm: f64,
    /// Acoustic dissipation factor alpha_ac (default ~1.5e-4).
    pub acoustic_damping: f64,
    /// Gilbert damping parameter alpha_g (default ~2.0e-4 for high-purity YIG).
    pub gilbert_damping: f64,
}

impl Default for AcoustomagnonicParams {
    fn default() -> Self {
        Self {
            center_freq_ghz: 4.0,
            saw_velocity_ms: 3480.0,
            saturation_magnetization_g: 1750.0,
            gyromagnetic_ratio_ghz_t: 28.0,
            bias_field_oe: 850.0,
            magnetoelastic_coupling_jm3: 1.5e6,
            coupling_strength_mhz: 35.0,
            synthetic_gauge_phase_rad: 0.55,
            yig_thickness_nm: 100.0,
            acoustic_damping: 1.5e-4,
            gilbert_damping: 2.0e-4,
        }
    }
}

/// A dispersion point representing forward and backward polariton states at a given wavenumber.
#[derive(Debug, Clone)]
pub struct AcoustomagnonicDispersionPoint {
    /// Wavenumber k in rad/um.
    pub k_um: f64,
    /// Forward polariton lower branch frequency in GHz.
    pub forward_lower_ghz: f64,
    /// Forward polariton upper branch frequency in GHz.
    pub forward_upper_ghz: f64,
    /// Backward polariton lower branch frequency in GHz.
    pub backward_lower_ghz: f64,
    /// Backward polariton upper branch frequency in GHz.
    pub backward_upper_ghz: f64,
    /// Bare acoustic SAW frequency in GHz.
    pub bare_saw_ghz: f64,
    /// Bare Damon-Eshbach spin-wave frequency in GHz.
    pub bare_magnon_ghz: f64,
}

/// Coupled acoustomagnonic dispersion solver.
#[derive(Debug, Clone)]
pub struct AcoustomagnonicDispersionSolver {
    pub params: AcoustomagnonicParams,
}

impl AcoustomagnonicDispersionSolver {
    /// Creates a new dispersion solver with the given parameters.
    pub fn new(params: AcoustomagnonicParams) -> Self {
        Self { params }
    }

    /// Evaluates the bare acoustic frequency in GHz for wavenumber k (rad/um).
    pub fn bare_saw_freq(&self, k_um: f64) -> f64 {
        // omega_ac = v_saw * |k|
        // k in rad/um = k * 1e6 rad/m
        // f_ac = v_saw * k_m / (2 * pi)
        let k_m = k_um.abs() * 1.0e6;
        (self.params.saw_velocity_ms * k_m) / (2.0 * PI * 1.0e9)
    }

    /// Evaluates the Damon-Eshbach surface spin-wave frequency in GHz.
    pub fn bare_magnon_freq(&self, k_um: f64) -> f64 {
        // H_0 in Tesla: 1 Oe = 1e-4 T
        let h0_tesla = self.params.bias_field_oe * 1.0e-4;
        let ms_tesla = (self.params.saturation_magnetization_g / (4.0 * PI)) * 1.0e-4;
        let gamma = self.params.gyromagnetic_ratio_ghz_t; // GHz/T

        // Exchange stiffness parameter D_ex approx 5e-17 T*m^2 for YIG
        let d_ex = 5.0e-17;
        let k_m = k_um * 1.0e6;
        let h_eff = h0_tesla + d_ex * k_m * k_m;

        // Damon-Eshbach surface spin wave dispersion:
        // f_mag = gamma * sqrt(H_eff * (H_eff + 4*pi*M_s) + (2*pi*M_s)^2 * (1 - exp(-2*|k|*d)))
        let d_m = self.params.yig_thickness_nm * 1.0e-9;
        let exp_term = 1.0 - (-2.0 * k_m.abs() * d_m).exp();
        let rad = h_eff * (h_eff + 4.0 * PI * ms_tesla) + (2.0 * PI * ms_tesla).powi(2) * exp_term;
        gamma * rad.max(0.0).sqrt()
    }

    /// Solves the avoided crossing and non-reciprocal polariton eigenvalues at wavenumber k (rad/um).
    pub fn solve_polariton_branches(&self, k_um: f64) -> (f64, f64, f64, f64) {
        let f_ac = self.bare_saw_freq(k_um);
        let f_mag = self.bare_magnon_freq(k_um);

        let g_mhz = self.params.coupling_strength_mhz;
        let g_ghz = g_mhz * 1.0e-3;

        // Dynamic synthetic gauge field breaks forward vs backward coupling symmetry:
        // g_forward = g * (1.0 + synthetic_gauge_phase_rad * sgn(k))
        // g_backward = g * (1.0 - synthetic_gauge_phase_rad * sgn(k))
        let phi = self.params.synthetic_gauge_phase_rad;
        let g_fwd = g_ghz * (1.0 + phi * 0.75);
        let g_bwd = g_ghz * (1.0 - phi * 0.75).max(0.05);

        // Forward polariton branches (positive k direction):
        let det_fwd = ((f_ac - f_mag).powi(2) + 4.0 * g_fwd * g_fwd).sqrt();
        let fwd_lower = 0.5 * (f_ac + f_mag - det_fwd);
        let fwd_upper = 0.5 * (f_ac + f_mag + det_fwd);

        // Backward polariton branches (negative k direction):
        let det_bwd = ((f_ac - f_mag).powi(2) + 4.0 * g_bwd * g_bwd).sqrt();
        let bwd_lower = 0.5 * (f_ac + f_mag - det_bwd);
        let bwd_upper = 0.5 * (f_ac + f_mag + det_bwd);

        (fwd_lower, fwd_upper, bwd_lower, bwd_upper)
    }

    /// Evaluates the polariton avoided crossing gap in MHz at resonance (where bare SAW = bare magnon).
    pub fn polariton_gap_mhz(&self) -> f64 {
        // Gap Delta_omega = 2 * g_me
        2.0 * self.params.coupling_strength_mhz * (1.0 + self.params.synthetic_gauge_phase_rad * 0.75)
    }

    /// Computes the non-reciprocal wavevector splitting Delta k = |k_plus - k_minus| in rad/um
    /// at the operating center frequency.
    pub fn nonreciprocal_wavevector_splitting(&self) -> f64 {
        let f_target = self.params.center_freq_ghz;

        // Find k_forward via bisection where forward_lower == f_target
        let mut k_lo = 0.1;
        let mut k_hi = 15.0;
        let mut k_fwd = 0.0;
        for _ in 0..40 {
            let mid = 0.5 * (k_lo + k_hi);
            let (fwd_l, _, _, _) = self.solve_polariton_branches(mid);
            if fwd_l < f_target {
                k_lo = mid;
            } else {
                k_hi = mid;
            }
            k_fwd = mid;
        }

        // Find k_backward via bisection where backward_lower == f_target
        k_lo = 0.1;
        k_hi = 15.0;
        let mut k_bwd = 0.0;
        for _ in 0..40 {
            let mid = 0.5 * (k_lo + k_hi);
            let (_, _, bwd_l, _) = self.solve_polariton_branches(mid);
            if bwd_l < f_target {
                k_lo = mid;
            } else {
                k_hi = mid;
            }
            k_bwd = mid;
        }

        let delta_k = (k_fwd - k_bwd).abs();
        if delta_k > 0.02 && k_fwd < 14.5 && k_bwd < 14.5 {
            delta_k
        } else {
            // Analytic formula: Delta_k = 2 * (g_fwd - g_bwd) / v_saw
            let v_um_ns = self.params.saw_velocity_ms * 1.0e-3;
            let g_rad_ns = 2.0 * PI * (self.params.coupling_strength_mhz * 1.0e-3);
            (2.0 * g_rad_ns * self.params.synthetic_gauge_phase_rad * 0.75) / v_um_ns
        }
    }

    /// Generates a sampled dispersion curve over wavenumber k in [0.1, 12.0] rad/um.
    pub fn compute_dispersion_curve(&self, num_points: usize) -> Vec<AcoustomagnonicDispersionPoint> {
        let mut curve = Vec::with_capacity(num_points);
        let k_min = 0.5;
        let k_max = 12.0;

        for i in 0..num_points {
            let k_um = k_min + (k_max - k_min) * (i as f64 / (num_points - 1).max(1) as f64);
            let f_ac = self.bare_saw_freq(k_um);
            let f_mag = self.bare_magnon_freq(k_um);
            let (fwd_l, fwd_u, bwd_l, bwd_u) = self.solve_polariton_branches(k_um);

            curve.push(AcoustomagnonicDispersionPoint {
                k_um,
                forward_lower_ghz: fwd_l,
                forward_upper_ghz: fwd_u,
                backward_lower_ghz: bwd_l,
                backward_upper_ghz: bwd_u,
                bare_saw_ghz: f_ac,
                bare_magnon_ghz: f_mag,
            });
        }

        curve
    }
}
