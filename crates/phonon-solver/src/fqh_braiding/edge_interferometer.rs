#![deny(unsafe_code)]

//! Fractional Quantum Hall (FQH) Edge Interferometer Simulator.
//!
//! Models Fabry-Perot and Mach-Zehnder edge interferometers in fractional quantum Hall regimes:
//! - Aharonov-Bohm phase accumulation: theta_AB = 2 * pi * e* * B * A / h.
//! - Topological braiding phase: theta_topo = 2 * pi * n_bulk * e* / e,
//!   with non-Abelian bulk anyon parity modulation (even vs odd quasiparticles).
//! - Quasiparticle tunneling transmission P_tun(V_g) and thermal dephasing.
//! - Fractional shot noise S_I = 2 * e* * I * P_tun * (1 - P_tun) and Fano factor F = e* / e.

use std::f64::consts::PI;

/// Fundamental physical constants (SI units).
pub const PLANCK_CONSTANT_H: f64 = 6.62607015e-34; // J * s
pub const ELEMENTARY_CHARGE_E: f64 = 1.602176634e-19; // C
pub const FLUX_QUANTUM_PHI0: f64 = PLANCK_CONSTANT_H / ELEMENTARY_CHARGE_E; // Wb = T * m^2 (~4.1356677e-15)
pub const CONDUCTANCE_QUANTUM_G0: f64 = ELEMENTARY_CHARGE_E * ELEMENTARY_CHARGE_E / PLANCK_CONSTANT_H; // S (~3.874e-5)
pub const BOLTZMANN_CONSTANT_KB: f64 = 1.380649e-23; // J / K

/// Fractional quantum Hall filling fraction state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillingFraction {
    /// Moore-Read Pfaffian state at nu = 5/2 with quasiparticle charge e* = e / 4 (0.25 e).
    Nu5_2,
    /// Laughlin state at nu = 1/3 with quasiparticle charge e* = e / 3 (~0.3333 e).
    Nu1_3,
    /// Particle-hole conjugate state at nu = 2/3 with quasiparticle charge e* = e / 3 (~0.3333 e).
    Nu2_3,
    /// Jain sequence state at nu = 2/5 with quasiparticle charge e* = e / 5 (0.20 e).
    Nu2_5,
}

impl FillingFraction {
    /// Quasiparticle fractional charge in units of elementary charge e (e* / e).
    pub fn quasiparticle_charge_e(&self) -> f64 {
        match self {
            Self::Nu5_2 => 0.25,
            Self::Nu1_3 => 1.0 / 3.0,
            Self::Nu2_3 => 1.0 / 3.0,
            Self::Nu2_5 => 0.20,
        }
    }

    /// Theoretical shot noise Fano factor F = e* / e.
    pub fn fano_factor(&self) -> f64 {
        self.quasiparticle_charge_e()
    }

    /// Numerical bulk filling factor nu.
    pub fn filling_factor(&self) -> f64 {
        match self {
            Self::Nu5_2 => 2.5,
            Self::Nu1_3 => 1.0 / 3.0,
            Self::Nu2_3 => 2.0 / 3.0,
            Self::Nu2_5 => 0.40,
        }
    }

    /// Whether the topological order hosts non-Abelian anyons.
    pub fn is_non_abelian(&self) -> bool {
        matches!(self, Self::Nu5_2)
    }

    /// User-friendly label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Nu5_2 => "nu = 5/2 (e* = 0.25 e, Non-Abelian)",
            Self::Nu1_3 => "nu = 1/3 (e* = 0.33 e, Laughlin)",
            Self::Nu2_3 => "nu = 2/3 (e* = 0.33 e, Conjugate)",
            Self::Nu2_5 => "nu = 2/5 (e* = 0.20 e, Jain)",
        }
    }
}

/// Type of electronic edge state interferometer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterferometerType {
    /// Fabry-Perot interferometer formed by two quantum point contacts (QPCs) enclosing area A.
    FabryPerot,
    /// Mach-Zehnder interferometer formed by two chiral edge arms of lengths L1, L2.
    MachZehnder,
}

impl InterferometerType {
    /// User-friendly label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::FabryPerot => "Fabry-Perot (Two QPCs)",
            Self::MachZehnder => "Mach-Zehnder (Chiral Arms)",
        }
    }
}

/// Fractional Quantum Hall Edge Interferometer Simulator.
#[derive(Debug, Clone, PartialEq)]
pub struct FqhEdgeInterferometer {
    /// Filling fraction of the fractional quantum Hall fluid.
    pub filling: FillingFraction,
    /// Interferometer geometry.
    pub interferometer_type: InterferometerType,
    /// Enclosed magnetic flux area in um^2.
    pub area_um2: f64,
    /// Arm length L1 in um (Mach-Zehnder arm 1).
    pub arm_length_1_um: f64,
    /// Arm length L2 in um (Mach-Zehnder arm 2).
    pub arm_length_2_um: f64,
    /// Applied perpendicular magnetic field B in Tesla.
    pub magnetic_field_tesla: f64,
    /// Quantum point contact gate voltage V_g in Volts.
    pub gate_voltage_v: f64,
    /// Cryogenic electron temperature T in mK.
    pub temperature_mk: f64,
    /// Number of localized bulk quasiparticles enclosed within the interferometer loop.
    pub bulk_anyon_count: usize,
    /// Injected bias current I in nA.
    pub bias_current_na: f64,
}

impl Default for FqhEdgeInterferometer {
    fn default() -> Self {
        Self {
            filling: FillingFraction::Nu5_2,
            interferometer_type: InterferometerType::FabryPerot,
            area_um2: 2.0,
            arm_length_1_um: 5.0,
            arm_length_2_um: 5.5,
            magnetic_field_tesla: 4.0,
            gate_voltage_v: -0.5,
            temperature_mk: 20.0,
            bulk_anyon_count: 0,
            bias_current_na: 1.0,
        }
    }
}

impl FqhEdgeInterferometer {
    /// Creates a new FQH edge interferometer with specified parameters.
    pub fn new(
        filling: FillingFraction,
        interferometer_type: InterferometerType,
        area_um2: f64,
        arm_length_1_um: f64,
        arm_length_2_um: f64,
        magnetic_field_tesla: f64,
        gate_voltage_v: f64,
        temperature_mk: f64,
        bulk_anyon_count: usize,
        bias_current_na: f64,
    ) -> Self {
        Self {
            filling,
            interferometer_type,
            area_um2: area_um2.max(0.01),
            arm_length_1_um: arm_length_1_um.max(0.1),
            arm_length_2_um: arm_length_2_um.max(0.1),
            magnetic_field_tesla: magnetic_field_tesla.max(0.0),
            gate_voltage_v,
            temperature_mk: temperature_mk.max(0.1),
            bulk_anyon_count,
            bias_current_na: bias_current_na.max(0.0),
        }
    }

    /// Effective enclosed loop area in m^2.
    pub fn effective_area_m2(&self) -> f64 {
        self.area_um2 * 1.0e-12
    }

    /// Quasiparticle fractional charge in Coulombs e* = (e* / e) * e.
    pub fn quasiparticle_charge_coulombs(&self) -> f64 {
        self.filling.quasiparticle_charge_e() * ELEMENTARY_CHARGE_E
    }

    /// Aharonov-Bohm phase accumulated around the loop: theta_AB = 2 * pi * e* * B * A / h.
    pub fn aharonov_bohm_phase(&self, b_field_tesla: f64) -> f64 {
        let e_star_ratio = self.filling.quasiparticle_charge_e();
        let area_m2 = self.effective_area_m2();
        2.0 * PI * e_star_ratio * (b_field_tesla * area_m2) / FLUX_QUANTUM_PHI0
    }

    /// Aharonov-Bohm oscillation periodicity in magnetic field: Delta B = h / (e* * A).
    pub fn oscillation_period_delta_b(&self) -> f64 {
        let e_star_ratio = self.filling.quasiparticle_charge_e();
        let area_m2 = self.effective_area_m2();
        FLUX_QUANTUM_PHI0 / (e_star_ratio * area_m2)
    }

    /// Topological braiding phase accumulated from enclosed bulk anyons:
    /// theta_topo = 2 * pi * n_bulk * e* / e.
    pub fn topological_phase(&self) -> f64 {
        let e_star_ratio = self.filling.quasiparticle_charge_e();
        2.0 * PI * (self.bulk_anyon_count as f64) * e_star_ratio
    }

    /// Bulk anyon parity visibility factor V(n_bulk).
    /// For non-Abelian Moore-Read Pfaffian (nu = 5/2):
    /// - Even bulk anyons (n_bulk % 2 == 0): full interference visibility V = 1.0.
    /// - Odd bulk anyons (n_bulk % 2 == 1): complete suppression of interference contrast V = 0.0
    ///   due to topological degeneracy of bulk Majorana zero modes.
    /// For Abelian states (nu = 1/3, 2/3, 2/5):
    /// - Visibility is preserved V = 1.0, with a pure topological phase shift.
    pub fn bulk_parity_visibility(&self) -> f64 {
        if self.filling.is_non_abelian() {
            if self.bulk_anyon_count % 2 == 1 {
                0.0 // Complete non-Abelian interference extinction
            } else {
                1.0 // Unsuppressed Abelian paired parity
            }
        } else {
            1.0 // Abelian fluids preserve full visibility across parity
        }
    }

    /// Quasiparticle tunneling transmission probability P_tun(V_g) in [0.001, 0.999].
    pub fn tunneling_probability(&self, gate_voltage: f64) -> f64 {
        let v_th = -0.5; // Threshold voltage in V
        let v_0 = 0.1; // Transition width in V
        let arg = ((gate_voltage - v_th) / v_0).clamp(-20.0, 20.0);
        let p = 1.0 / (1.0 + (-arg).exp());
        p.clamp(0.001, 0.999)
    }

    /// Differential conductance G(B, V_g) in Siemens.
    /// G(B, V_g) = G0 + Delta_G * V(n_bulk) * cos(theta_AB + theta_topo) * exp(-T / T_0).
    pub fn conductance(&self, b_field_tesla: f64, gate_voltage: f64) -> f64 {
        let p_tun = self.tunneling_probability(gate_voltage);
        let g_base = self.filling.filling_factor() * CONDUCTANCE_QUANTUM_G0 * p_tun;
        let delta_g = 0.10 * CONDUCTANCE_QUANTUM_G0 * (p_tun * (1.0 - p_tun)).sqrt();

        let theta_ab = self.aharonov_bohm_phase(b_field_tesla);
        let theta_topo = self.topological_phase();
        let visibility = self.bulk_parity_visibility();

        // Thermal dephasing factor exp(-T / T0) with T0 = 50 mK
        let t_0_mk = 50.0;
        let thermal_factor = (-self.temperature_mk / t_0_mk).exp();

        let oscillation = (theta_ab + theta_topo).cos();
        g_base + delta_g * visibility * oscillation * thermal_factor
    }

    /// Differential conductance G(B, V_g) normalized in units of e^2 / h.
    pub fn conductance_in_e2_h(&self, b_field_tesla: f64, gate_voltage: f64) -> f64 {
        self.conductance(b_field_tesla, gate_voltage) / CONDUCTANCE_QUANTUM_G0
    }

    /// Quasiparticle shot noise power spectral density S_I in A^2 / Hz:
    /// S_I = 2 * e* * I * P_tun * (1 - P_tun).
    pub fn shot_noise(&self) -> f64 {
        let e_star = self.quasiparticle_charge_coulombs();
        let current_a = self.bias_current_na * 1.0e-9;
        let p_tun = self.tunneling_probability(self.gate_voltage_v);
        2.0 * e_star * current_a * p_tun * (1.0 - p_tun)
    }

    /// Measured Fano factor F = S_I / (2 * e * I * P_tun * (1 - P_tun)) = e* / e.
    pub fn fano_factor(&self) -> f64 {
        let s_i = self.shot_noise();
        let current_a = self.bias_current_na * 1.0e-9;
        let p_tun = self.tunneling_probability(self.gate_voltage_v);
        let denom = 2.0 * ELEMENTARY_CHARGE_E * current_a * p_tun * (1.0 - p_tun);
        if denom == 0.0 {
            self.filling.fano_factor()
        } else {
            s_i / denom
        }
    }

    /// Johnson-Nyquist equilibrium thermal noise floor in A^2 / Hz: S_th = 4 * k_B * T * G.
    pub fn thermal_noise_floor(&self) -> f64 {
        let t_kelvin = self.temperature_mk * 1.0e-3;
        let g = self.conductance(self.magnetic_field_tesla, self.gate_voltage_v);
        4.0 * BOLTZMANN_CONSTANT_KB * t_kelvin * g
    }

    /// Simulates a sweep of the perpendicular magnetic field B and returns (B, G in e^2/h) curve.
    pub fn simulate_b_field_sweep(
        &self,
        b_min: f64,
        b_max: f64,
        num_points: usize,
    ) -> Vec<[f64; 2]> {
        let n = num_points.max(2);
        let step = (b_max - b_min) / (n - 1) as f64;
        let mut curve = Vec::with_capacity(n);

        for i in 0..n {
            let b = b_min + (i as f64) * step;
            let g = self.conductance_in_e2_h(b, self.gate_voltage_v);
            curve.push([b, g]);
        }
        curve
    }
}
