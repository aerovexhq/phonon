//! Periodic dispersion engineering for traveling-wave parametric amplifiers:
//! resonant phase matching, stopbands, and pump phase mismatch suppression.

/// Physical parameters for a periodic dispersion-engineered transmission line.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DispersionEngineeringParams {
    /// Bare transmission line cell series inductance L_0 in Henries (typically ~1 - 3 nH).
    pub unit_inductance_h: f64,
    /// Bare transmission line cell shunt capacitance C_0 in Farads (typically ~0.4 - 1.0 pF).
    pub unit_capacitance_f: f64,
    /// Resonant phase-matching stub resonance frequency omega_res in rad/s.
    pub stub_resonance_freq_rad_per_s: f64,
    /// Loading stub coupling capacitance C_r in Farads.
    pub stub_capacitance_f: f64,
    /// Periodic spacing of loading stubs (every N_stub unit cells, typically 8 - 16 cells).
    pub stub_period_cells: usize,
}

impl DispersionEngineeringParams {
    /// Constructs dispersion engineering parameters.
    pub fn new(
        unit_inductance_h: f64,
        unit_capacitance_f: f64,
        stub_resonance_freq_rad_per_s: f64,
        stub_capacitance_f: f64,
        stub_period_cells: usize,
    ) -> Self {
        Self {
            unit_inductance_h,
            unit_capacitance_f,
            stub_resonance_freq_rad_per_s,
            stub_capacitance_f,
            stub_period_cells,
        }
    }

    /// Standard baseline 50-Ohm Josephson transmission line with pump phase-matching stubs at 16 GHz.
    pub fn standard_50ohm_jtwpa() -> Self {
        let z0 = 50.0;
        let c0: f64 = 0.6e-12; // 0.6 pF
        let l0: f64 = z0 * z0 * c0; // 1.5 nH -> sqrt(L0/C0) = 50 Ohm
        let f_res = 16.0e9; // 16 GHz stub resonance
        Self::new(
            l0,
            c0,
            2.0 * std::f64::consts::PI * f_res,
            0.15e-12, // 0.15 pF loading stub
            12,
        )
    }

    /// Characteristic impedance Z_0 = sqrt(L_0 / C_0) in Ohms.
    pub fn characteristic_impedance_ohms(&self) -> f64 {
        (self.unit_inductance_h / self.unit_capacitance_f.max(1e-18)).sqrt()
    }

    /// Phase velocity v_p = 1 / sqrt(L_0 C_0) in m/s (or cells/s).
    pub fn phase_velocity_cells_per_s(&self) -> f64 {
        1.0 / (self.unit_inductance_h * self.unit_capacitance_f)
            .sqrt()
            .max(1e-18)
    }

    /// Linear wavenumber k_0(omega) = omega * sqrt(L_0 C_0) in rad/cell.
    pub fn linear_wavenumber_rad_per_cell(&self, omega_rad_per_s: f64) -> f64 {
        omega_rad_per_s * (self.unit_inductance_h * self.unit_capacitance_f).sqrt()
    }

    /// Engineered dispersion wavenumber k(omega) with resonant stub perturbation:
    /// $$k(\omega) \approx k_0(\omega) \left[ 1 + \frac{C_r}{2 C_0 N_{stub}} \frac{\omega^2}{\omega_{res}^2 - \omega^2} \right]$$
    pub fn engineered_wavenumber_rad_per_cell(&self, omega_rad_per_s: f64) -> f64 {
        let k0 = self.linear_wavenumber_rad_per_cell(omega_rad_per_s);
        let omega_res = self.stub_resonance_freq_rad_per_s;
        let delta_sq = omega_res.powi(2) - omega_rad_per_s.powi(2);

        // Perturbation factor
        let factor = (self.stub_capacitance_f
            / (2.0 * self.unit_capacitance_f * self.stub_period_cells as f64).max(1e-18))
            * (omega_rad_per_s.powi(2) / delta_sq.signum() * (delta_sq.abs().max(1e12)));
        k0 * (1.0 + factor.clamp(-0.5, 0.5))
    }

    /// Four-wave mixing linear phase mismatch Delta k = 2 k_p - k_s - k_i in rad/cell:
    pub fn four_wave_phase_mismatch(&self, omega_p: f64, omega_s: f64, omega_i: f64) -> f64 {
        let kp = self.engineered_wavenumber_rad_per_cell(omega_p);
        let ks = self.engineered_wavenumber_rad_per_cell(omega_s);
        let ki = self.engineered_wavenumber_rad_per_cell(omega_i);
        2.0 * kp - ks - ki
    }
}
