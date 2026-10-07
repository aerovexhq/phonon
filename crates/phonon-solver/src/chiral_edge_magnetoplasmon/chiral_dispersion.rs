#![deny(unsafe_code)]

//! Chiral Edge-Magnetoplasmon (EMP) dispersion and hybrid piezoelectric SAW coupling.
//!
//! Models 2D electron gas (2DEG) in the integer quantum Hall regime coupled to a
//! piezoelectric substrate, yielding unidirectional chiral acoustic transport.

use std::f64::consts::PI;

/// Elementary physical constants (SI units).
pub const ELEMENTARY_CHARGE: f64 = 1.602_176_634e-19; // C
pub const PLANCK_CONSTANT: f64 = 6.626_070_15e-34; // J*s
pub const HBAR: f64 = 1.054_571_817e-34; // J*s
pub const ELECTRON_MASS: f64 = 9.109_383_7015e-31; // kg
pub const EPSILON_0: f64 = 8.854_187_8128e-12; // F/m
pub const BOLTZMANN_K: f64 = 1.380_649e-23; // J/K
pub const CONDUCTANCE_QUANTUM: f64 = 3.874_046e-5; // S = e^2 / h

/// Parameters governing chiral edge-magnetoplasmon physics and piezoelectric SAW interaction.
#[derive(Debug, Clone, PartialEq)]
pub struct ChiralEmpParams {
    /// Perpendicular magnetic field B_z in Tesla.
    pub magnetic_field_t: f64,
    /// Integer Quantum Hall filling factor nu (1, 2, 3, ...).
    pub filling_factor_nu: usize,
    /// 2DEG sheet electron density in m^-2 (typically 1e15 to 3e15 m^-2).
    pub sheet_density_m2: f64,
    /// Effective electron mass ratio m* / m_0 (e.g. 0.067 for GaAs).
    pub effective_mass_ratio: f64,
    /// Relative dielectric permittivity of substrate (e.g. 12.9 for GaAs).
    pub relative_permittivity: f64,
    /// Edge depletion strip width in micrometers.
    pub edge_width_um: f64,
    /// Bare acoustic Rayleigh SAW velocity in m/s (e.g. 2865.0 m/s for GaAs).
    pub bare_acoustic_velocity_ms: f64,
    /// Electromechanical piezoelectric coupling factor K^2 (e.g. 0.045 for strong piezo coupling).
    pub piezo_coupling_k2: f64,
    /// Operating center RF frequency in GHz.
    pub center_freq_ghz: f64,
    /// Ambient operating temperature in Kelvin (cryogenic, default 4.2 K).
    pub temperature_k: f64,
}

impl Default for ChiralEmpParams {
    fn default() -> Self {
        Self {
            magnetic_field_t: 4.136,
            filling_factor_nu: 2,
            sheet_density_m2: 2.0e15,
            effective_mass_ratio: 0.067,
            relative_permittivity: 12.9,
            edge_width_um: 1.0,
            bare_acoustic_velocity_ms: 2865.0,
            piezo_coupling_k2: 0.045,
            center_freq_ghz: 3.0,
            temperature_k: 4.2,
        }
    }
}

impl ChiralEmpParams {
    /// Creates a preset tuned for nu = 2 Quantum Hall plateau in GaAs/AlGaAs.
    pub fn preset_nu2_gaas() -> Self {
        Self::default()
    }

    /// Creates a preset tuned for nu = 1 high-field regime (B_z = 8.27 T).
    pub fn preset_nu1_high_field() -> Self {
        Self {
            magnetic_field_t: 8.272,
            filling_factor_nu: 1,
            sheet_density_m2: 2.0e15,
            effective_mass_ratio: 0.067,
            relative_permittivity: 12.9,
            edge_width_um: 0.8,
            bare_acoustic_velocity_ms: 2865.0,
            piezo_coupling_k2: 0.050,
            center_freq_ghz: 3.5,
            temperature_k: 4.2,
        }
    }
}

/// A point along the EMP dispersion spectrum.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmpDispersionPoint {
    /// Wavenumber q in rad/mm.
    pub wavenumber_rad_mm: f64,
    /// Chiral EMP frequency in GHz.
    pub emp_freq_ghz: f64,
    /// Bare acoustic SAW frequency in GHz.
    pub bare_saw_freq_ghz: f64,
    /// Coupled hybrid forward frequency in GHz.
    pub hybrid_fwd_freq_ghz: f64,
    /// Coupled hybrid backward frequency in GHz.
    pub hybrid_bwd_freq_ghz: f64,
}

/// Engine for evaluating chiral edge-magnetoplasmon dispersion and SAW non-reciprocity.
#[derive(Debug, Clone)]
pub struct ChiralDispersionSolver {
    pub params: ChiralEmpParams,
}

impl ChiralDispersionSolver {
    /// Creates a new dispersion solver with the specified parameters.
    pub fn new(params: ChiralEmpParams) -> Self {
        Self { params }
    }

    /// Returns the quantized Hall conductance sigma_xy = nu * e^2 / h in Siemens (S).
    pub fn quantized_hall_conductance(&self) -> f64 {
        self.params.filling_factor_nu as f64 * CONDUCTANCE_QUANTUM
    }

    /// Computes effective electron mass m* in kg.
    pub fn effective_mass_kg(&self) -> f64 {
        self.params.effective_mass_ratio * ELECTRON_MASS
    }

    /// Computes cyclotron frequency omega_c = e * B_z / m* in rad/s.
    pub fn cyclotron_frequency_rad_s(&self) -> f64 {
        let m_eff = self.effective_mass_kg().max(1e-35);
        ELEMENTARY_CHARGE * self.params.magnetic_field_t.abs() / m_eff
    }

    /// Computes cyclotron frequency f_c in GHz.
    pub fn cyclotron_frequency_ghz(&self) -> f64 {
        self.cyclotron_frequency_rad_s() / (2.0 * PI * 1.0e9)
    }

    /// Computes cyclotron energy gap hbar * omega_c in meV.
    pub fn cyclotron_energy_mev(&self) -> f64 {
        (HBAR * self.cyclotron_frequency_rad_s() / ELEMENTARY_CHARGE) * 1.0e3
    }

    /// Computes thermal energy k_B * T in meV.
    pub fn thermal_energy_mev(&self) -> f64 {
        (BOLTZMANN_K * self.params.temperature_k / ELEMENTARY_CHARGE) * 1.0e3
    }

    /// Verifies if cyclotron gap significantly exceeds thermal energy: hbar*omega_c >> k_B*T.
    pub fn is_quantum_hall_regime_valid(&self) -> bool {
        self.cyclotron_energy_mev() > 5.0 * self.thermal_energy_mev()
    }

    /// Computes bare chiral EMP group velocity v_emp(q) in m/s.
    ///
    /// Based on Volkov-Mikhailov chiral edge-magnetoplasmon formulation:
    /// v_emp approx (2 * sigma_xy) / (pi * epsilon_0 * (1 + epsilon_r)) * ln(2 / (|q| * w))
    pub fn emp_velocity_ms(&self, q_rad_m: f64) -> f64 {
        let sigma_xy = self.quantized_hall_conductance();
        let eps_eff = EPSILON_0 * (1.0 + self.params.relative_permittivity);
        let w_m = (self.params.edge_width_um * 1.0e-6).max(1e-9);
        let q_mag = q_rad_m.abs().max(1.0);

        let arg = (2.0 / (q_mag * w_m)).max(1.001);
        let log_term = arg.ln() + 1.0;
        let v0 = (2.0 * sigma_xy) / (PI * eps_eff);
        (v0 * log_term).max(1.0e4)
    }

    /// Evaluates bare chiral EMP frequency in GHz for wavenumber q in rad/m.
    pub fn emp_frequency_ghz(&self, q_rad_m: f64) -> f64 {
        let v = self.emp_velocity_ms(q_rad_m);
        let omega = v * q_rad_m.abs();
        omega / (2.0 * PI * 1.0e9)
    }

    /// Computes non-reciprocal piezoelectric acoustic forward velocity v_fwd in m/s.
    ///
    /// For forward propagation (co-propagating with chiral edge current),
    /// coupling to the chiral edge state shifts velocity by:
    /// Delta v_fwd / v_0 approx + K^2 / 2 * chi_coupling
    pub fn acoustic_forward_velocity_ms(&self) -> f64 {
        let v0 = self.params.bare_acoustic_velocity_ms;
        let k2 = self.params.piezo_coupling_k2;
        // Constructive forward hybrid phase velocity enhancement
        v0 * (1.0 + 0.5 * k2 * 0.95)
    }

    /// Computes non-reciprocal piezoelectric acoustic backward velocity v_bwd in m/s.
    ///
    /// For backward propagation (counter-propagating against chiral edge current),
    /// velocity remains screened without resonant chiral enhancement:
    /// Delta v_bwd / v_0 approx - K^2 / 2 * chi_screen
    pub fn acoustic_backward_velocity_ms(&self) -> f64 {
        let v0 = self.params.bare_acoustic_velocity_ms;
        let k2 = self.params.piezo_coupling_k2;
        v0 * (1.0 - 0.5 * k2 * 0.85)
    }

    /// Evaluates velocity non-reciprocity ratio eta_nr = |v_fwd - v_bwd| / (v_fwd + v_bwd).
    pub fn velocity_non_reciprocity_ratio(&self) -> f64 {
        let vf = self.acoustic_forward_velocity_ms();
        let vb = self.acoustic_backward_velocity_ms();
        let diff = (vf - vb).abs();
        let sum = vf + vb;
        if sum > 1e-12 {
            diff / sum
        } else {
            0.0
        }
    }

    /// Generates dispersion spectrum points across a wavenumber span in rad/mm.
    pub fn compute_dispersion_curve(
        &self,
        q_min_rad_mm: f64,
        q_max_rad_mm: f64,
        points: usize,
    ) -> Vec<EmpDispersionPoint> {
        let n = points.max(2);
        let step = (q_max_rad_mm - q_min_rad_mm) / (n - 1) as f64;
        let mut result = Vec::with_capacity(n);

        let v_fwd = self.acoustic_forward_velocity_ms();
        let v_bwd = self.acoustic_backward_velocity_ms();
        let v_bare = self.params.bare_acoustic_velocity_ms;

        for i in 0..n {
            let q_rad_mm = q_min_rad_mm + i as f64 * step;
            let q_rad_m = q_rad_mm * 1.0e3; // conversion to rad/m

            let emp_f = self.emp_frequency_ghz(q_rad_m);
            let bare_saw_f = (v_bare * q_rad_m.abs()) / (2.0 * PI * 1.0e9);
            let fwd_saw_f = (v_fwd * q_rad_m.abs()) / (2.0 * PI * 1.0e9);
            let bwd_saw_f = (v_bwd * q_rad_m.abs()) / (2.0 * PI * 1.0e9);

            result.push(EmpDispersionPoint {
                wavenumber_rad_mm: q_rad_mm,
                emp_freq_ghz: emp_f,
                bare_saw_freq_ghz: bare_saw_f,
                hybrid_fwd_freq_ghz: fwd_saw_f,
                hybrid_bwd_freq_ghz: bwd_saw_f,
            });
        }

        result
    }
}
