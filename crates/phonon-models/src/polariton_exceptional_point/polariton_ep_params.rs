//! Parameters and metrics for non-Hermitian exceptional points in topological
//! exciton-polariton phonon condensates, chiral mode switching, and EP gyroscopes.

/// Parameters for driven-dissipative non-Hermitian polariton condensates near exceptional points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolaritonEpParams {
    /// Bare cavity photon decay rate $\gamma_{\mathrm{cav}}$ in $\text{MHz}$ (nominal $10.0 - 60.0\text{ MHz}$).
    pub cavity_photon_decay_mhz: f64,
    /// Exciton-photon Rabi coupling strength $\Omega_R$ in $\text{meV}$ (nominal $4.0 - 16.0\text{ meV}$).
    pub exciton_coupling_rabi_mev: f64,
    /// Coupled acoustic phonon breathing mode frequency $f_{\mathrm{ph}}$ in $\text{GHz}$ (nominal $1.0 - 4.5\text{ GHz}$).
    pub acoustic_phonon_frequency_ghz: f64,
    /// Inter-cavity coherent evanescent coupling rate $J$ in $\text{MHz}$ (nominal $20.0 - 120.0\text{ MHz}$).
    pub inter_cavity_coupling_j_mhz: f64,
    /// Gain/loss non-Hermitian contrast $\gamma_{\mathrm{net}}$ in $\text{MHz}$ (nominal $30.0 - 240.0\text{ MHz}$).
    pub gain_loss_contrast_gamma_mhz: f64,
    /// Optical/acoustic pump power in milliwatts (nominal $1.0 - 15.0\text{ mW}$).
    pub pump_power_mw: f64,
    /// Dynamic EP parameter encirclement period in nanoseconds (nominal $20.0 - 200.0\text{ ns}$).
    pub encirclement_period_ns: f64,
    /// Perturbation strain amplitude $\delta\epsilon$ in parts per million (nominal $0.5 - 50.0\text{ ppm}$).
    pub perturbation_strain_ppm: f64,
}

impl Default for PolaritonEpParams {
    fn default() -> Self {
        Self {
            cavity_photon_decay_mhz: 25.0,
            exciton_coupling_rabi_mev: 8.5,
            acoustic_phonon_frequency_ghz: 2.2,
            inter_cavity_coupling_j_mhz: 50.0,
            gain_loss_contrast_gamma_mhz: 100.0, // Exactly at EP2 when gamma_net = 2 * J
            pump_power_mw: 4.0,
            encirclement_period_ns: 60.0,
            perturbation_strain_ppm: 5.0,
        }
    }
}

/// Evaluated metrics for non-Hermitian polariton condensates and exceptional point dynamics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolaritonEpMetrics {
    /// Non-Hermitian square-root sensitivity enhancement in decibels ($\ge 30.0\text{ dB}$).
    pub exceptional_sensitivity_db: f64,
    /// Chiral state selection purity under EP encirclement in percent ($\ge 99.0\%$).
    pub chiral_mode_purity_pct: f64,
    /// Polariton macroscopic condensation threshold power in milliwatts ($\le 5.0\text{ mW}$).
    pub condensation_threshold_mw: f64,
    /// Polariton laser emission spectral linewidth in $\text{MHz}$ ($\le 50.0\text{ MHz}$).
    pub polariton_laser_linewidth_mhz: f64,
    /// Gyroscopic Sagnac scale-factor sensitivity enhancement ratio ($\ge 10.0\times$).
    pub gyro_scale_factor_enhancement: f64,
    /// Non-Hermitian topological charge / winding number ($W = 1/2 = 0.5$).
    pub topological_winding_charge: f64,
}

impl PolaritonEpParams {
    /// Creates a new parameter set for non-Hermitian polariton EP condensates.
    pub fn new(j_mhz: f64, gamma_mhz: f64, pump_mw: f64, strain_ppm: f64) -> Self {
        Self {
            inter_cavity_coupling_j_mhz: j_mhz.max(1.0),
            gain_loss_contrast_gamma_mhz: gamma_mhz.max(1.0),
            pump_power_mw: pump_mw.max(0.1),
            perturbation_strain_ppm: strain_ppm.max(0.01),
            ..Default::default()
        }
    }
}
