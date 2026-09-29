//! Transient superconducting pairing enhancement, non-adiabatic Eliashberg coupling,
//! and dynamic pair-density wave (PDW) nucleation.

use super::chiral_drive_params::ChiralPhononDriveParams;

/// Parameters for transient electron-phonon pairing enhancement.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransientPairingParams {
    /// Equilibrium zero-temperature superconducting gap $\Delta_0$ in $\text{meV}$ (nominal $5.0 - 25.0\text{ meV}$).
    pub equilibrium_gap_mev: f64,
    /// Equilibrium dimensionless electron-phonon coupling constant $\lambda_0$ (nominal $0.30 - 0.60$).
    pub equilibrium_lambda: f64,
    /// Dynamic pairing enhancement sensitivity coefficient $\eta_{\mathrm{pair}}$ (nominal $0.15 - 0.25$).
    pub pairing_sensitivity: f64,
    /// Fermi velocity $v_F$ in $\text{m/s}$ (nominal $2.0 \times 10^5\text{ m/s}$).
    pub fermi_velocity_m_s: f64,
}

impl Default for TransientPairingParams {
    fn default() -> Self {
        Self {
            equilibrium_gap_mev: 15.0,
            equilibrium_lambda: 0.45,
            pairing_sensitivity: 0.18,
            fermi_velocity_m_s: 2.0e5,
        }
    }
}

/// Evaluated metrics for the transient non-equilibrium superconducting state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransientPairingMetrics {
    /// Non-equilibrium transient superconducting gap $\Delta_{\mathrm{transient}}$ in $\text{meV}$.
    pub transient_gap_mev: f64,
    /// Transient pairing enhancement percentage $\frac{\Delta_{\mathrm{transient}} - \Delta_0}{\Delta_0} \times 100\%$ ($> 50\%$).
    pub pairing_enhancement_fraction: f64,
    /// Renormalized dynamic electron-phonon coupling $\lambda_{\mathrm{eff}}$.
    pub effective_coupling_lambda: f64,
    /// Dynamic pair-density wave (PDW) wavevector $q_{\mathrm{PDW}}$ in $\text{m}^{-1}$.
    pub pdw_wavevector_inv_m: f64,
}

impl TransientPairingParams {
    /// Creates new transient pairing parameters.
    pub fn new(
        equilibrium_gap_mev: f64,
        equilibrium_lambda: f64,
        pairing_sensitivity: f64,
    ) -> Self {
        Self {
            equilibrium_gap_mev: equilibrium_gap_mev.max(1.0),
            equilibrium_lambda: equilibrium_lambda.max(0.05),
            pairing_sensitivity: pairing_sensitivity.clamp(0.05, 0.50),
            fermi_velocity_m_s: 2.0e5,
        }
    }

    /// Evaluates transient pairing metrics under a given chiral phonon drive.
    pub fn evaluate_pairing(
        &self,
        drive_params: &ChiralPhononDriveParams,
    ) -> TransientPairingMetrics {
        let q_ratio = drive_params.normalized_amplitude_q0;
        let drive_factor = q_ratio * q_ratio;

        // Effective coupling lambda_eff = lambda_0 * [1 + eta * (Q0/Qzpf)^2]
        let lambda_eff = self.equilibrium_lambda * (1.0 + self.pairing_sensitivity * drive_factor);

        // Transient gap enhancement: Delta_transient = Delta_0 * [1 + eta_pair * (Q0/Qzpf)^2]
        // For Q0/Qzpf in [1.8, 3.2] and eta_pair = 0.18:
        // enhancement = 0.18 * (2.2)^2 = 0.18 * 4.84 = 0.87 (87% > 50%)
        let enhancement = self.pairing_sensitivity * drive_factor;
        let delta_transient = self.equilibrium_gap_mev * (1.0 + enhancement);

        // PDW wavevector q_PDW = Omega_ph / v_F
        let omega_rad_s = drive_params.phonon_frequency_thz * 1.0e12 * 2.0 * std::f64::consts::PI;
        let q_pdw = omega_rad_s / self.fermi_velocity_m_s;

        TransientPairingMetrics {
            transient_gap_mev: delta_transient,
            pairing_enhancement_fraction: enhancement,
            effective_coupling_lambda: lambda_eff,
            pdw_wavevector_inv_m: q_pdw,
        }
    }
}
