#![deny(unsafe_code)]

//! Floquet Chiral Magnon-Phonon Polariton Dispersion Engine.
//!
//! Models coupled acoustic phonons and precessing magnons driven by a rotating
//! microwave magnetic field:
//!   h(t) = h_0 * (cos(Omega * t) x_hat + sin(Omega * t) y_hat)
//! which dynamically breaks time-reversal symmetry (chirality / angular momentum transfer).
//! Solves the coupled Floquet-Bloch quasi-energy eigenvalue problem across momentum
//! k in [-pi/a, pi/a], verifying non-reciprocal dispersion asymmetry |k+(omega) - (-k-(omega))| > 0,
//! forward vs backward group velocity asymmetry v_g_forward != v_g_backward,
//! and avoided-crossing polariton gap Delta_omega = 2 * g_eff.

use std::f64::consts::PI;

/// Physical constants for polariton dispersion calculations.
pub const HBAR_J_S: f64 = 1.054_571_817e-34;
pub const BOLTZMANN_K_J_K: f64 = 1.380_649e-23;
pub const GYROMAGNETIC_RATIO_RAD_S_T: f64 = 1.760_859_644e11;
pub const MU_0_H_M: f64 = 1.256_637_061_4e-6;

/// Configuration parameters for Floquet chiral polariton dispersion.
#[derive(Debug, Clone, PartialEq)]
pub struct ChiralPolaritonParams {
    /// Bare acoustic phonon frequency at zone center (default ~5.0 GHz).
    pub bare_phonon_freq_ghz: f64,
    /// Bare magnon frequency at zone center (default ~5.0 GHz).
    pub bare_magnon_freq_ghz: f64,
    /// Magnetoelastic coupling rate g_0 in MHz (default ~40.0 MHz).
    pub magnetoelastic_coupling_mhz: f64,
    /// Floquet rotating drive frequency Omega in GHz (default ~0.5 GHz).
    pub floquet_drive_freq_ghz: f64,
    /// Floquet rotating microwave drive amplitude h_0 in Oersted (default ~15.0 Oe).
    pub floquet_drive_amplitude_oe: f64,
    /// Dimensionless Gilbert damping parameter alpha_G (default 1.0e-4).
    pub gilbert_damping: f64,
    /// Acoustic damping loss factor (default 5.0e-5).
    pub acoustic_damping: f64,
    /// Acoustic phase velocity v_s in m/s (default 3840.0 m/s for YIG/GGG).
    pub sound_velocity_m_s: f64,
    /// Periodic unit cell lattice constant in micrometers (default 1.0 um).
    pub lattice_constant_um: f64,
}

impl Default for ChiralPolaritonParams {
    fn default() -> Self {
        Self {
            bare_phonon_freq_ghz: 5.0,
            bare_magnon_freq_ghz: 5.0,
            magnetoelastic_coupling_mhz: 40.0,
            floquet_drive_freq_ghz: 0.5,
            floquet_drive_amplitude_oe: 15.0,
            gilbert_damping: 1.0e-4,
            acoustic_damping: 5.0e-5,
            sound_velocity_m_s: 3840.0,
            lattice_constant_um: 1.0,
        }
    }
}

/// Point on the Floquet chiral polariton dispersion curve at momentum k.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolaritonBranchPoint {
    /// Momentum / wavenumber k in rad/m.
    pub wavenumber_k: f64,
    /// Lower hybridized polariton branch quasi-energy in GHz.
    pub omega_lower_ghz: f64,
    /// Upper hybridized polariton branch quasi-energy in GHz.
    pub omega_upper_ghz: f64,
    /// Lower polariton Floquet sideband (omega - Omega) in GHz.
    pub omega_lower_sideband_minus_ghz: f64,
    /// Lower polariton Floquet sideband (omega + Omega) in GHz.
    pub omega_lower_sideband_plus_ghz: f64,
    /// Upper polariton Floquet sideband (omega - Omega) in GHz.
    pub omega_upper_sideband_minus_ghz: f64,
    /// Upper polariton Floquet sideband (omega + Omega) in GHz.
    pub omega_upper_sideband_plus_ghz: f64,
    /// Bare acoustic phonon frequency at k in GHz.
    pub bare_phonon_ghz: f64,
    /// Bare spin-wave magnon frequency at k in GHz (with Floquet chiral shift).
    pub bare_magnon_ghz: f64,
    /// Group velocity d(omega_lower)/dk in m/s.
    pub group_velocity_lower_m_s: f64,
    /// Group velocity d(omega_upper)/dk in m/s.
    pub group_velocity_upper_m_s: f64,
}

/// Solver engine for Floquet chiral polariton dispersion.
#[derive(Debug, Clone, PartialEq)]
pub struct FloquetPolaritonDispersion {
    pub params: ChiralPolaritonParams,
}

impl FloquetPolaritonDispersion {
    /// Creates a new dispersion engine instance.
    pub fn new(params: ChiralPolaritonParams) -> Self {
        Self { params }
    }

    /// Lattice constant a in meters.
    #[inline]
    pub fn lattice_constant_m(&self) -> f64 {
        self.params.lattice_constant_um * 1.0e-6
    }

    /// Brillouin zone boundary k_max = pi / a in rad/m.
    #[inline]
    pub fn k_max(&self) -> f64 {
        PI / self.lattice_constant_m()
    }

    /// Effective magnetoelastic coupling rate g_eff in rad/s, enhanced by Floquet drive.
    #[inline]
    pub fn effective_coupling_rad_s(&self) -> f64 {
        let g0 = 2.0 * PI * self.params.magnetoelastic_coupling_mhz * 1.0e6;
        let drive_enhancement = 1.0 + 0.015 * self.params.floquet_drive_amplitude_oe;
        g0 * drive_enhancement
    }

    /// Effective avoided-crossing polariton gap Delta_omega = 2 * g_eff in rad/s.
    #[inline]
    pub fn avoided_crossing_gap_rad_s(&self) -> f64 {
        2.0 * self.effective_coupling_rad_s()
    }

    /// Avoided-crossing polariton gap in MHz.
    #[inline]
    pub fn avoided_crossing_gap_mhz(&self) -> f64 {
        self.avoided_crossing_gap_rad_s() / (2.0 * PI * 1.0e6)
    }

    /// Avoided-crossing polariton gap in GHz.
    #[inline]
    pub fn avoided_crossing_gap_ghz(&self) -> f64 {
        self.avoided_crossing_gap_rad_s() / (2.0 * PI * 1.0e9)
    }

    /// Bare acoustic phonon frequency in GHz at wavenumber k.
    ///
    /// Reciprocal dispersion symmetric about k = 0: omega_ph(k) = omega_p0 + Delta_ph * (1 - cos(k * a)).
    #[inline]
    pub fn bare_phonon_freq_ghz_at_k(&self, k: f64) -> f64 {
        let a = self.lattice_constant_m();
        let delta_ph = 0.25; // 250 MHz dispersive bandwidth across Brillouin zone
        self.params.bare_phonon_freq_ghz + delta_ph * (1.0 - (k * a).cos())
    }

    /// Bare magnon frequency in GHz at wavenumber k, driven by chiral rotating field.
    ///
    /// The rotating drive h(t) = h_0 * (cos(Omega*t) x_hat + sin(Omega*t) y_hat) imparts angular
    /// momentum, adding an odd chiral velocity / synthetic momentum term delta_F * sin(k * a)
    /// that dynamically breaks time-reversal symmetry: omega_m(k) != omega_m(-k).
    #[inline]
    pub fn bare_magnon_freq_ghz_at_k(&self, k: f64) -> f64 {
        let a = self.lattice_constant_m();
        let delta_m = 0.25; // 250 MHz magnon bandwidth
        // Chiral Floquet shift coefficient proportional to drive amplitude and drive frequency
        let delta_floquet_chiral = (self.params.floquet_drive_amplitude_oe / 15.0)
            * (self.params.floquet_drive_freq_ghz / 0.5)
            * 0.035; // 35 MHz chiral shift at 15 Oe and 0.5 GHz drive
        self.params.bare_magnon_freq_ghz
            + delta_m * (1.0 - (k * a).cos())
            + delta_floquet_chiral * (k * a).sin()
    }

    /// Solves coupled Floquet-Bloch quasi-energies and group velocities at momentum k.
    pub fn solve_point(&self, k: f64) -> PolaritonBranchPoint {
        let omega_ph = self.bare_phonon_freq_ghz_at_k(k);
        let omega_m = self.bare_magnon_freq_ghz_at_k(k);
        let g_ghz = self.effective_coupling_rad_s() / (2.0 * PI * 1.0e9);

        // Eigenvalues of coupled Hamiltonian:
        // H = [[omega_ph, g_eff], [g_eff, omega_m]]
        let avg = 0.5 * (omega_ph + omega_m);
        let diff = 0.5 * (omega_ph - omega_m);
        let radical = (diff * diff + g_ghz * g_ghz).sqrt();

        let omega_upper = avg + radical;
        let omega_lower = avg - radical;

        let omega_drive = self.params.floquet_drive_freq_ghz;

        // Group velocity evaluation d(omega)/dk via finite difference
        let dk = 100.0; // small wavenumber step in rad/m
        let k_plus = k + dk;
        let k_minus = k - dk;

        let ph_p = self.bare_phonon_freq_ghz_at_k(k_plus);
        let m_p = self.bare_magnon_freq_ghz_at_k(k_plus);
        let up_p = 0.5 * (ph_p + m_p) + (0.25 * (ph_p - m_p).powi(2) + g_ghz * g_ghz).sqrt();
        let lo_p = 0.5 * (ph_p + m_p) - (0.25 * (ph_p - m_p).powi(2) + g_ghz * g_ghz).sqrt();

        let ph_m = self.bare_phonon_freq_ghz_at_k(k_minus);
        let m_m = self.bare_magnon_freq_ghz_at_k(k_minus);
        let up_m = 0.5 * (ph_m + m_m) + (0.25 * (ph_m - m_m).powi(2) + g_ghz * g_ghz).sqrt();
        let lo_m = 0.5 * (ph_m + m_m) - (0.25 * (ph_m - m_m).powi(2) + g_ghz * g_ghz).sqrt();

        // 2 * pi * 1e9 factor converts GHz/(rad/m) to m/s
        let v_g_upper = (up_p - up_m) / (2.0 * dk) * 2.0 * PI * 1.0e9;
        let v_g_lower = (lo_p - lo_m) / (2.0 * dk) * 2.0 * PI * 1.0e9;

        PolaritonBranchPoint {
            wavenumber_k: k,
            omega_lower_ghz: omega_lower,
            omega_upper_ghz: omega_upper,
            omega_lower_sideband_minus_ghz: omega_lower - omega_drive,
            omega_lower_sideband_plus_ghz: omega_lower + omega_drive,
            omega_upper_sideband_minus_ghz: omega_upper - omega_drive,
            omega_upper_sideband_plus_ghz: omega_upper + omega_drive,
            bare_phonon_ghz: omega_ph,
            bare_magnon_ghz: omega_m,
            group_velocity_lower_m_s: v_g_lower,
            group_velocity_upper_m_s: v_g_upper,
        }
    }

    /// Computes full dispersion curves across the First Brillouin Zone [-pi/a, pi/a].
    pub fn compute_dispersion(&self, num_points: usize) -> Vec<PolaritonBranchPoint> {
        let n = num_points.max(3);
        let k_max = self.k_max();
        let mut points = Vec::with_capacity(n);

        for i in 0..n {
            let frac = i as f64 / (n - 1) as f64;
            let k = -k_max + 2.0 * k_max * frac;
            points.push(self.solve_point(k));
        }

        points
    }

    /// Solves forward wavenumber k_plus(omega) > 0 and backward wavenumber k_minus(omega) < 0
    /// on the upper polariton branch for a target frequency, returning (k_plus, k_minus, delta_k).
    ///
    /// Verifies non-reciprocal dispersion asymmetry: |k_plus(omega) - (-k_minus(omega))| > 0.
    pub fn forward_backward_wavenumbers(&self, target_freq_ghz: f64) -> (f64, f64, f64) {
        let k_max = self.k_max();
        let g_ghz = self.effective_coupling_rad_s() / (2.0 * PI * 1.0e9);
        let target = if target_freq_ghz <= self.params.bare_phonon_freq_ghz + g_ghz {
            self.params.bare_phonon_freq_ghz + g_ghz + 0.05
        } else {
            target_freq_ghz
        };

        // Bisection for forward root in (0, k_max]
        let mut k_lo = 0.0;
        let mut k_hi = k_max;
        for _ in 0..50 {
            let mid = 0.5 * (k_lo + k_hi);
            let pt = self.solve_point(mid);
            if pt.omega_upper_ghz < target {
                k_lo = mid;
            } else {
                k_hi = mid;
            }
        }
        let k_plus = 0.5 * (k_lo + k_hi);

        // Bisection for backward root in [-k_max, 0)
        let mut k_lo_bwd = -k_max;
        let mut k_hi_bwd = 0.0;
        for _ in 0..50 {
            let mid = 0.5 * (k_lo_bwd + k_hi_bwd);
            let pt = self.solve_point(mid);
            if pt.omega_upper_ghz < target {
                k_hi_bwd = mid;
            } else {
                k_lo_bwd = mid;
            }
        }
        let k_minus = 0.5 * (k_lo_bwd + k_hi_bwd);

        // Non-reciprocity asymmetry |k_plus - (-k_minus)| = |k_plus + k_minus|
        let delta_k = (k_plus + k_minus).abs();

        (k_plus, k_minus, delta_k)
    }

    /// Evaluates forward vs backward group velocity asymmetry at a target frequency.
    ///
    /// Returns (v_g_forward, v_g_backward, delta_vg).
    pub fn forward_backward_group_velocities(&self, target_freq_ghz: f64) -> (f64, f64, f64) {
        let (k_plus, k_minus, _) = self.forward_backward_wavenumbers(target_freq_ghz);
        let pt_fwd = self.solve_point(k_plus);
        let pt_bwd = self.solve_point(k_minus);

        let v_g_forward = pt_fwd.group_velocity_upper_m_s.abs();
        let v_g_backward = pt_bwd.group_velocity_upper_m_s.abs();
        let delta_vg = (v_g_forward - v_g_backward).abs();

        (v_g_forward, v_g_backward, delta_vg)
    }
}
