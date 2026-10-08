#![deny(unsafe_code)]

//! Phase 447: Read-Rezayi Fractional Quantum Hall State & Topological Quasiparticle Spectrum.
//!
//! Models the non-Abelian Read-Rezayi state at filling factor nu = 12/5 (and 2 + 2/3),
//! corresponding to the Z_3 parafermion conformal field theory whose quasiparticles
//! realize Fibonacci anyons with golden ratio quantum dimension.

/// Filling factor regime for the Read-Rezayi topological phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadRezayiFilling {
    /// nu = 12/5 = 2 + 2/5 (Second Landau level non-Abelian Fibonacci state).
    Nu12Over5,
    /// nu = 8/3 = 2 + 2/3 (Conjugate/particle-hole partner Fibonacci state).
    Nu2Plus2Over3,
}

impl ReadRezayiFilling {
    #[inline]
    pub fn value(&self) -> f64 {
        match self {
            Self::Nu12Over5 => 2.40,
            Self::Nu2Plus2Over3 => 8.0 / 3.0,
        }
    }

    #[inline]
    pub fn quasiparticle_charge_fraction(&self) -> f64 {
        match self {
            Self::Nu12Over5 => 0.20, // e* = e / 5
            Self::Nu2Plus2Over3 => 0.3333333333333333, // e* = e / 3
        }
    }

    #[inline]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Nu12Over5 => "nu = 12/5 (k = 3 Read-Rezayi)",
            Self::Nu2Plus2Over3 => "nu = 2 + 2/3 (Read-Rezayi Conjugate)",
        }
    }
}

/// Simulation parameters for the Read-Rezayi state.
#[derive(Debug, Clone)]
pub struct ReadRezayiStateParams {
    pub filling_factor: ReadRezayiFilling,
    pub magnetic_field_t: f64,
    pub electron_density_m2: f64,
    pub topological_gap_kelvin: f64,
    pub effective_mass_ratio: f64,
    pub dielectric_constant: f64,
}

impl Default for ReadRezayiStateParams {
    fn default() -> Self {
        Self {
            filling_factor: ReadRezayiFilling::Nu12Over5,
            magnetic_field_t: 5.80,
            electron_density_m2: 3.35e15,
            topological_gap_kelvin: 0.048, // 48 mK
            effective_mass_ratio: 0.067,   // GaAs electron effective mass
            dielectric_constant: 12.8,
        }
    }
}

/// Evaluated physical metrics for the Read-Rezayi state.
#[derive(Debug, Clone)]
pub struct ReadRezayiMetrics {
    pub filling_factor_val: f64,
    pub fibonacci_quantum_dimension: f64,
    pub topological_entropy_s_topo: f64,
    pub neutral_mode_velocity_ms: f64,
    pub charge_mode_velocity_ms: f64,
    pub quasiparticle_charge_fraction: f64,
    pub magnetic_length_nm: f64,
    pub coulomb_energy_scale_k: f64,
}

/// Point along the Read-Rezayi edge mode dispersion curve.
#[derive(Debug, Clone)]
pub struct ReadRezayiDispersionPoint {
    pub momentum_k_nm_inv: f64,
    pub charge_branch_ghz: f64,
    pub neutral_fibonacci_branch_ghz: f64,
}

/// Point along the pair correlation function g(r).
#[derive(Debug, Clone)]
pub struct ReadRezayiCorrelationPoint {
    pub radius_r_over_l_b: f64,
    pub pair_correlation_g_r: f64,
}

/// Numerical solver for Read-Rezayi state physics and edge excitations.
#[derive(Debug, Clone)]
pub struct ReadRezayiStateSolver {
    pub params: ReadRezayiStateParams,
}

impl ReadRezayiStateSolver {
    pub fn new(params: ReadRezayiStateParams) -> Self {
        Self { params }
    }

    /// Evaluates macroscopic and topological invariants.
    pub fn evaluate_metrics(&self) -> ReadRezayiMetrics {
        let p = &self.params;
        let hbar = 1.054571817e-34;
        let e_charge = 1.602176634e-19;
        let eps_0 = 8.8541878128e-12;
        let k_b = 1.380649e-23;

        // Golden Ratio phi = (1 + sqrt(5)) / 2
        let phi = (1.0 + 5.0_f64.sqrt()) / 2.0;

        // Total quantum dimension D = sqrt(1 + phi^2)
        let total_d = (1.0 + phi * phi).sqrt();
        let s_topo = total_d.ln();

        // Magnetic length l_B = sqrt(hbar / (e * B))
        let l_b = (hbar / (e_charge * p.magnetic_field_t)).sqrt();
        let l_b_nm = l_b * 1.0e9;

        // Coulomb energy scale E_C = e^2 / (4 * pi * eps_r * eps_0 * l_B)
        let e_c_joules = (e_charge * e_charge) / (4.0 * std::f64::consts::PI * p.dielectric_constant * eps_0 * l_b);
        let e_c_kelvin = e_c_joules / k_b;

        // Charged edge velocity v_c ~ 1.1e5 m/s, Neutral Fibonacci Majorana/parafermion velocity v_n ~ 1.6e4 m/s
        let v_c = 1.15e5 * (p.magnetic_field_t / 5.80).sqrt();
        let v_n = 1.62e4 * (p.topological_gap_kelvin / 0.048).sqrt();

        ReadRezayiMetrics {
            filling_factor_val: p.filling_factor.value(),
            fibonacci_quantum_dimension: phi,
            topological_entropy_s_topo: s_topo,
            neutral_mode_velocity_ms: v_n,
            charge_mode_velocity_ms: v_c,
            quasiparticle_charge_fraction: p.filling_factor.quasiparticle_charge_fraction(),
            magnetic_length_nm: l_b_nm,
            coulomb_energy_scale_k: e_c_kelvin,
        }
    }

    /// Computes chiral edge excitation dispersion.
    pub fn compute_edge_dispersion(&self, points_count: usize) -> Vec<ReadRezayiDispersionPoint> {
        let metrics = self.evaluate_metrics();
        let mut result = Vec::with_capacity(points_count);

        let k_max = 0.35; // nm^-1
        let step = if points_count > 1 { k_max / (points_count - 1) as f64 } else { 0.1 };

        for i in 0..points_count {
            let k_nm_inv = i as f64 * step;
            let k_m_inv = k_nm_inv * 1.0e9;

            // omega_c(k) = v_c * k / (2 * pi)
            let f_charge_ghz = (metrics.charge_mode_velocity_ms * k_m_inv) / (2.0 * std::f64::consts::PI * 1.0e9);
            // omega_n(k) = v_n * k / (2 * pi) + gap_offset
            let gap_ghz = self.params.topological_gap_kelvin * 1.380649e-23 / (6.62607015e-34 * 1.0e9);
            let f_neutral_ghz = (metrics.neutral_mode_velocity_ms * k_m_inv) / (2.0 * std::f64::consts::PI * 1.0e9) + gap_ghz;

            result.push(ReadRezayiDispersionPoint {
                momentum_k_nm_inv: k_nm_inv,
                charge_branch_ghz: f_charge_ghz,
                neutral_fibonacci_branch_ghz: f_neutral_ghz,
            });
        }

        result
    }

    /// Computes the characteristic pair-correlation function g(r / l_B) showing the 3-cluster clustering.
    pub fn compute_pair_correlation(&self, points_count: usize) -> Vec<ReadRezayiCorrelationPoint> {
        let mut result = Vec::with_capacity(points_count);
        let r_max = 6.0;
        let step = if points_count > 1 { r_max / (points_count - 1) as f64 } else { 0.5 };

        for i in 0..points_count {
            let r = (i as f64 * step).max(0.01);
            // For k=3 Read-Rezayi, the wave function vanishes when 4 particles coincide,
            // producing short-range suppression g(r) ~ (r/l_B)^4 at small r and oscillating to 1.0 at large r.
            let envelope = 1.0 - (-0.6 * r * r).exp() * (1.0 + 0.3 * (2.0 * std::f64::consts::PI * r / 2.2).cos());
            let g_r = (envelope.max(0.0)).min(1.4);

            result.push(ReadRezayiCorrelationPoint {
                radius_r_over_l_b: r,
                pair_correlation_g_r: g_r,
            });
        }

        result
    }
}
