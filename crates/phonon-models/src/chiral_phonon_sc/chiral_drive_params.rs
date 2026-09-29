//! Circularly polarized optical phonon drive, non-linear phononics,
//! and dynamic crystal inversion symmetry breaking.

use phonon_core::constants::H_BAR;

/// Parameters for circularly polarized optical phonon drive.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralPhononDriveParams {
    /// Optical phonon resonant frequency in Terahertz ($\text{THz}$) (nominal $15.0 - 25.0\text{ THz}$, $60 - 100\text{ meV}$).
    pub phonon_frequency_thz: f64,
    /// Dimensionless coherent phonon drive amplitude $Q_0 / Q_{\mathrm{zpf}}$ (nominal $1.5 - 3.5$).
    pub normalized_amplitude_q0: f64,
    /// Non-linear phononics coupling constant $g_{12}$ in $\text{meV}$ (nominal $10.0 - 30.0\text{ meV}$).
    pub nonlinear_coupling_g12_mev: f64,
    /// Effective Born dynamical charge circulation factor in units of elementary charge $e$.
    pub effective_born_charge: f64,
    /// Drive laser pulse duration in picoseconds ($\text{ps}$) (nominal $0.2 - 1.0\text{ ps}$).
    pub pulse_duration_ps: f64,
}

impl Default for ChiralPhononDriveParams {
    fn default() -> Self {
        Self {
            phonon_frequency_thz: 20.0,
            normalized_amplitude_q0: 2.2,
            nonlinear_coupling_g12_mev: 18.0,
            effective_born_charge: 3.2,
            pulse_duration_ps: 0.5,
        }
    }
}

/// Evaluated metrics for the chiral phonon driven state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralDriveMetrics {
    /// Chiral phonon angular momentum $L_{\mathrm{ph}} = \pm \hbar$ in Joule-seconds.
    pub angular_momentum_j_s: f64,
    /// Induced orbital magnetic moment in Bohr magnetons $\mu_B$ ($1.0 - 5.0\,\mu_B$).
    pub induced_magnetic_moment_bohr: f64,
    /// Dynamic structural inversion breaking order parameter $\eta_{\mathrm{inv}} \propto Q_0^2$.
    pub inversion_breaking_parameter: f64,
    /// Rectified static coordinate displacement $X_{\mathrm{coupled}}$ in picometers ($\text{pm}$).
    pub rectified_displacement_pm: f64,
}

impl ChiralPhononDriveParams {
    /// Creates new chiral phonon drive parameters.
    pub fn new(
        phonon_frequency_thz: f64,
        normalized_amplitude_q0: f64,
        nonlinear_coupling_g12_mev: f64,
    ) -> Self {
        Self {
            phonon_frequency_thz: phonon_frequency_thz.max(1.0),
            normalized_amplitude_q0: normalized_amplitude_q0.max(0.1),
            nonlinear_coupling_g12_mev: nonlinear_coupling_g12_mev.max(1.0),
            effective_born_charge: 3.2,
            pulse_duration_ps: 0.5,
        }
    }

    /// Evaluates the driven phonon state metrics.
    pub fn evaluate_drive_metrics(&self) -> ChiralDriveMetrics {
        let q_ratio = self.normalized_amplitude_q0;
        let l_ph = H_BAR; // Circularly polarized chiral mode carries +/- hbar

        // Induced orbital magnetic moment: mu_ph = (Z* e / 2 M) L_ph ~ 0.5 * Z* * (Q0/Qzpf)^2 mu_B
        let mu_bohr = 0.40 * self.effective_born_charge * q_ratio * q_ratio;

        // Inversion breaking parameter eta_inv = (Q0/Qzpf)^2 / 10.0
        let eta_inv = (q_ratio * q_ratio) * 0.10;

        // Rectified static displacement via cubic/quartic non-linear phononics:
        // X = g12 * Q0^2 / (2 omega_R^2) ~ 0.25 * (Q0/Qzpf)^2 pm
        let disp_pm = 0.25 * q_ratio * q_ratio;

        ChiralDriveMetrics {
            angular_momentum_j_s: l_ph,
            induced_magnetic_moment_bohr: mu_bohr,
            inversion_breaking_parameter: eta_inv,
            rectified_displacement_pm: disp_pm,
        }
    }
}
