//! Superconducting nanowire geometry, critical current, kinetic inductance, and normal state properties.
//!
//! Models NbN / WSi ultra-thin meander geometries, Ginzburg-Landau depairing critical current,
//! sheet resistance, normal state resistance, and kinetic inductance.

/// Fundamental physical constants.
pub const ELEMENTARY_CHARGE: f64 = 1.602_176_634e-19; // Coulombs
pub const PLANCK_H: f64 = 6.626_070_15e-34; // J*s
pub const HBAR: f64 = 1.054_571_817e-34; // J*s
pub const SPEED_OF_LIGHT: f64 = 2.997_924_58e8; // m/s
pub const BOLTZMANN_K: f64 = 1.380_649e-23; // J/K
pub const BOLTZMANN_K_EV: f64 = 8.617_333_262e-5; // eV/K

/// Physical geometry and material parameters of a superconducting nanowire.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NanowireGeometry {
    /// Film thickness d in nanometers (typically 4.0 - 6.0 nm for NbN).
    pub thickness_nm: f64,
    /// Nanowire stripe width w in nanometers (typically 50.0 - 100.0 nm).
    pub width_nm: f64,
    /// Total meander length L in micrometers (typically 50.0 - 500.0 um).
    pub length_um: f64,
    /// Superconducting critical temperature T_c in Kelvin (e.g. 11.5 K for NbN).
    pub critical_temperature_k: f64,
    /// Substrate operating temperature T_sub in Kelvin (typically 0.8 - 2.5 K).
    pub substrate_temperature_k: f64,
    /// Normal-state sheet resistance R_sq in Ohms/square (typically 300 - 500 Ohms/sq).
    pub sheet_resistance_ohms_per_sq: f64,
    /// Kinetic inductance per square L_k,sq in picoHenries/square (typically 60 - 120 pH/sq).
    pub kinetic_inductance_per_sq_ph: f64,
    /// Zero-temperature depairing critical current I_c0 in microAmperes (typically 15 - 35 uA).
    pub critical_current_zero_t_ua: f64,
    /// Load / readout transmission line impedance R_L in Ohms (typically 50.0 Ohms).
    pub load_impedance_ohms: f64,
}

impl NanowireGeometry {
    /// Default standard NbN nanowire detector on oxidized silicon substrate.
    pub fn standard_nbn() -> Self {
        Self {
            thickness_nm: 5.0,
            width_nm: 80.0,
            length_um: 100.0,
            critical_temperature_k: 11.5,
            substrate_temperature_k: 2.0,
            sheet_resistance_ohms_per_sq: 400.0,
            kinetic_inductance_per_sq_ph: 80.0,
            critical_current_zero_t_ua: 25.0,
            load_impedance_ohms: 50.0,
        }
    }

    /// Number of squares N_sq = L / w.
    pub fn number_of_squares(&self) -> f64 {
        (self.length_um * 1000.0) / self.width_nm
    }

    /// Total normal-state resistance R_N in Ohms: R_N = R_sq * N_sq.
    pub fn normal_resistance_ohms(&self) -> f64 {
        self.sheet_resistance_ohms_per_sq * self.number_of_squares()
    }

    /// Total kinetic inductance L_k in Henries: L_k = L_k,sq * N_sq.
    pub fn kinetic_inductance_henries(&self) -> f64 {
        let n_sq = self.number_of_squares();
        (self.kinetic_inductance_per_sq_ph * 1e-12) * n_sq
    }

    /// Ginzburg-Landau depairing critical current I_c(T) in microAmperes:
    /// I_c(T) = I_c0 * [1 - (T / T_c)^2]^(3/2) for T < T_c, and 0 for T >= T_c.
    pub fn critical_current_at_temp_ua(&self, temperature_k: f64) -> f64 {
        if temperature_k >= self.critical_temperature_k {
            return 0.0;
        }
        let t_ratio = temperature_k / self.critical_temperature_k;
        let diff = (1.0 - t_ratio * t_ratio).max(0.0);
        self.critical_current_zero_t_ua * diff.powf(1.5)
    }

    /// Operating critical current I_c at substrate temperature T_sub.
    pub fn operational_critical_current_ua(&self) -> f64 {
        self.critical_current_at_temp_ua(self.substrate_temperature_k)
    }

    /// Operating bias current I_b in microAmperes for a given bias ratio alpha_bias = I_b / I_c.
    pub fn bias_current_ua(&self, alpha_bias: f64) -> f64 {
        self.operational_critical_current_ua() * alpha_bias.clamp(0.0, 1.0)
    }

    /// Inductive recovery / reset time constant tau_rec = L_k / R_L in seconds.
    pub fn inductive_reset_time_s(&self) -> f64 {
        self.kinetic_inductance_henries() / self.load_impedance_ohms
    }

    /// Cross-sectional area A = w * d in m^2.
    pub fn cross_sectional_area_m2(&self) -> f64 {
        (self.width_nm * 1e-9) * (self.thickness_nm * 1e-9)
    }

    /// Nanowire volume V = L * w * d in m^3.
    pub fn active_volume_m3(&self) -> f64 {
        (self.length_um * 1e-6) * (self.width_nm * 1e-9) * (self.thickness_nm * 1e-9)
    }

    /// Zero-temperature superconducting gap Delta(0) approx 1.764 * k_B * T_c in Joules.
    pub fn superconducting_gap_zero_t_joules(&self) -> f64 {
        1.764 * BOLTZMANN_K * self.critical_temperature_k
    }

    /// Superconducting gap in milli-electronVolts (meV).
    pub fn superconducting_gap_zero_t_mev(&self) -> f64 {
        (self.superconducting_gap_zero_t_joules() / ELEMENTARY_CHARGE) * 1000.0
    }
}
