#![deny(unsafe_code)]

use std::f64::consts::PI;

/// Parameters defining the valley-Hall acoustic honeycomb lattice.
#[derive(Debug, Clone)]
pub struct ValleyLatticeParams {
    /// Lattice constant a_0 in meters (default 20 mm).
    pub lattice_constant_a: f64,
    /// Background acoustic sound speed c_0 in m/s (default 343 m/s for airborne or 1500 m/s).
    pub speed_of_sound: f64,
    /// Cylinder A radius in meters.
    pub cylinder_radius_a: f64,
    /// Cylinder B radius in meters.
    pub cylinder_radius_b: f64,
    /// Normalized inversion symmetry breaking parameter Delta = (r_A - r_B) / (r_A + r_B).
    pub asymmetry_delta: f64,
    /// Dirac center resonance frequency in Hz.
    pub dirac_frequency_hz: f64,
}

impl Default for ValleyLatticeParams {
    fn default() -> Self {
        Self {
            lattice_constant_a: 0.020, // 20 mm
            speed_of_sound: 343.0,     // 343 m/s
            cylinder_radius_a: 0.0055, // 5.5 mm
            cylinder_radius_b: 0.0035, // 3.5 mm
            asymmetry_delta: 0.22,     // (5.5 - 3.5) / (5.5 + 3.5) = 0.222
            dirac_frequency_hz: 4800.0, // 4.8 kHz
        }
    }
}

impl ValleyLatticeParams {
    /// Dirac velocity v_D = sqrt(3)/2 * c_0 in m/s.
    #[inline]
    pub fn dirac_velocity(&self) -> f64 {
        0.5 * 3.0_f64.sqrt() * self.speed_of_sound
    }

    /// Dirac mass term m = Delta * 2 * pi * f0 in rad/s.
    #[inline]
    pub fn dirac_mass(&self) -> f64 {
        self.asymmetry_delta * 2.0 * PI * self.dirac_frequency_hz * 0.15
    }

    /// Topological valley bandgap in Hz: Delta_gap = 2 * |m| / (2 * pi).
    #[inline]
    pub fn valley_bandgap_hz(&self) -> f64 {
        2.0 * self.dirac_mass().abs() / (2.0 * PI)
    }

    /// K-valley wavevector magnitude |K| = 4 * pi / (3 * sqrt(3) * a).
    #[inline]
    pub fn k_valley_magnitude(&self) -> f64 {
        4.0 * PI / (3.0 * 3.0_f64.sqrt() * self.lattice_constant_a)
    }
}

/// Valley index identifying the Dirac valley.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValleyIndex {
    /// K valley (tau = +1).
    K,
    /// K' valley (tau = -1).
    KPrime,
}

impl ValleyIndex {
    #[inline]
    pub fn tau(&self) -> f64 {
        match self {
            ValleyIndex::K => 1.0,
            ValleyIndex::KPrime => -1.0,
        }
    }
}

/// Berry curvature and valley topological metrics.
#[derive(Debug, Clone)]
pub struct ValleyBerryCurvature {
    /// Valley Chern number for K valley (+0.5 for Delta > 0).
    pub c_k: f64,
    /// Valley Chern number for K' valley (-0.5 for Delta > 0).
    pub c_k_prime: f64,
    /// Total Chern number across full Brillouin zone (strictly 0.0).
    pub total_chern_number: f64,
    /// Valley Chern number difference C_v = 0.5 * (C_K - C_K').
    pub valley_chern_number: f64,
    /// Interface topological valley index jump Delta C_v = C_v(A) - C_v(B) = +1.0.
    pub delta_cv_interface: f64,
}

/// Evaluator for valley Dirac dispersion and Berry curvature.
#[derive(Debug, Clone)]
pub struct ValleyLatticeSolver {
    pub params: ValleyLatticeParams,
}

impl ValleyLatticeSolver {
    pub fn new(params: ValleyLatticeParams) -> Self {
        Self { params }
    }

    /// Calculate Dirac energy dispersion E_pm(delta_k) around valley tau.
    pub fn dispersion_at(&self, delta_k: [f64; 2], valley: ValleyIndex) -> (f64, f64) {
        let vd = self.params.dirac_velocity();
        let m = self.params.dirac_mass();
        let dk_sq = delta_k[0] * delta_k[0] + delta_k[1] * delta_k[1];
        let kinetic_sq = vd * vd * dk_sq;
        let mass_sq = m * m;
        let delta_e = (kinetic_sq + mass_sq).sqrt();

        let _tau = valley.tau();
        let f0 = self.params.dirac_frequency_hz;
        let upper_freq = f0 + delta_e / (2.0 * PI);
        let lower_freq = f0 - delta_e / (2.0 * PI);

        (lower_freq, upper_freq)
    }

    /// Calculate Berry curvature Omega_z(delta_k) in m^2 around valley tau.
    pub fn berry_curvature_at(&self, delta_k: [f64; 2], valley: ValleyIndex) -> f64 {
        let vd = self.params.dirac_velocity();
        let m = self.params.dirac_mass();
        let tau = valley.tau();

        if m.abs() < 1e-12 {
            return 0.0;
        }

        let dk_sq = delta_k[0] * delta_k[0] + delta_k[1] * delta_k[1];
        let denom = (vd * vd * dk_sq + m * m).powf(1.5);
        tau * (vd * vd * m) / (2.0 * denom)
    }

    /// Calculate integrated valley topological numbers.
    pub fn compute_berry_metrics(&self) -> ValleyBerryCurvature {
        let sign_m = if self.params.dirac_mass() >= 0.0 {
            1.0
        } else {
            -1.0
        };

        let c_k = 0.5 * sign_m;
        let c_k_prime = -0.5 * sign_m;
        let total_chern = c_k + c_k_prime; // Exactly 0.0
        let valley_chern = 0.5 * (c_k - c_k_prime); // +0.5 or -0.5
        let delta_cv = 2.0 * valley_chern; // Jump across opposite domain wall = +1.0

        ValleyBerryCurvature {
            c_k,
            c_k_prime,
            total_chern_number: total_chern,
            valley_chern_number: valley_chern,
            delta_cv_interface: delta_cv,
        }
    }

    /// Calculate gapless kink edge dispersion omega(k_parallel) along domain wall.
    pub fn kink_edge_dispersion(&self, k_par: f64, valley: ValleyIndex) -> f64 {
        let f0 = self.params.dirac_frequency_hz;
        let vd = self.params.dirac_velocity();
        let tau = valley.tau();
        // Gapless linear chiral dispersion traversing bandgap: omega = omega_0 + tau * v_D * k_par
        let f_kink = f0 + (tau * vd * k_par) / (2.0 * PI);
        f_kink
    }
}
