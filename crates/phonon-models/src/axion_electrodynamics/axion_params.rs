//! Quantum axion electrodynamics, modified Maxwell-Chern-Simons electrodynamics,
//! Witten effect anomalous Hall conductance, and dark-matter haloscopes.

/// Physical constants for quantum axion electrodynamics.
pub mod constants {
    /// Reduced Planck constant $\hbar$ in $\text{J}\cdot\text{s}$.
    pub const HBAR: f64 = 1.054_571_817e-34;
    /// Planck constant $h$ in $\text{J}\cdot\text{s}$.
    pub const H_PLANCK: f64 = 6.626_070_15e-34;
    /// Speed of light $c$ in $\text{m/s}$.
    pub const SPEED_OF_LIGHT: f64 = 2.997_924_58e8;
    /// Elementary electric charge $e$ in Coulombs.
    pub const ELEMENTARY_CHARGE: f64 = 1.602_176_634e-19;
    /// Boltzmann constant $k_B$ in $\text{J/K}$.
    pub const K_BOLTZMANN: f64 = 1.380_649e-23;
    /// Vacuum magnetic permeability $\mu_0$ in $\text{H/m}$.
    pub const MU_0: f64 = 1.256_637_062_12e-6;
    /// Vacuum electric permittivity $\epsilon_0$ in $\text{F/m}$.
    pub const EPSILON_0: f64 = 8.854_187_812_8e-12;
    /// Quantum of conductance $G_0 = e^2 / h$ in Siemens ($\text{S}$).
    pub const QUANTUM_CONDUCTANCE: f64 = (ELEMENTARY_CHARGE * ELEMENTARY_CHARGE) / H_PLANCK;
}

/// Parameters for quantum axion electrodynamics and resonant haloscope transducers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxionElectrodynamicsParams {
    /// Dark matter axion rest mass $m_a$ in micro-electronvolts ($\mu\text{eV}$) (nominal $10.0 - 50.0\,\mu\text{eV}$).
    pub axion_mass_uev: f64,
    /// Axion-photon coupling constant $g_{a\gamma\gamma}$ in $\text{GeV}^{-1}$ (nominal $10^{-14} - 10^{-12}\,\text{GeV}^{-1}$).
    pub axion_photon_coupling_gev_inv: f64,
    /// Background static magnetic field $B_0$ in Tesla (nominal $4.0 - 14.0\text{ T}$).
    pub magnetic_field_tesla: f64,
    /// Microwave cavity volume $V$ in cubic meters ($\text{m}^3$) (nominal $0.002 - 0.05\text{ m}^3$).
    pub cavity_volume_m3: f64,
    /// Cavity loaded quality factor $Q_L$ (nominal $20,000 - 250,000$).
    pub cavity_q_factor: f64,
    /// Cavity antenna coupling coefficient $\beta_c$ (nominal $1.0 - 2.0$, $1.0 =$ critical coupling).
    pub cavity_coupling_beta: f64,
    /// Geometric cavity mode form factor $C_{010}$ for $\text{TM}_{010}$ resonance (nominal $0.50 - 0.65$).
    pub form_factor_c010: f64,
    /// System total effective noise temperature $T_{\mathrm{sys}}$ in Kelvin (nominal $0.05 - 1.2\text{ K}$).
    pub system_noise_temp_k: f64,
    /// Haloscope integration time $\tau$ in seconds (nominal $0.1 - 5.0\text{ s}$).
    pub integration_time_s: f64,
    /// Local galactic dark matter mass density $\rho_{\mathrm{DM}}$ in $\text{GeV/cm}^3$ (standard $0.45\text{ GeV/cm}^3$).
    pub local_dark_matter_density_gev_cm3: f64,
    /// Dynamic axion angle $\theta(\mathbf{r}, t)$ in radians (nominal $\pi$ for topological insulator, or dynamic value).
    pub theta_angle_rad: f64,
    /// Topological axion polariton resonant enhancement factor $\mathcal{M}_{\mathrm{topo}}$ (nominal $1.0\times 10^3 - 5.0\times 10^4$).
    pub topological_polariton_enhancement: f64,
}

impl Default for AxionElectrodynamicsParams {
    fn default() -> Self {
        Self {
            axion_mass_uev: 20.0,
            axion_photon_coupling_gev_inv: 1.0e-13,
            magnetic_field_tesla: 8.0,
            cavity_volume_m3: 0.010,
            cavity_q_factor: 80_000.0,
            cavity_coupling_beta: 1.2,
            form_factor_c010: 0.58,
            system_noise_temp_k: 0.25,
            integration_time_s: 1.0,
            local_dark_matter_density_gev_cm3: 0.45,
            theta_angle_rad: std::f64::consts::PI,
            topological_polariton_enhancement: 5_000.0,
        }
    }
}

/// Evaluated metrics for quantum axion electrodynamics and dark matter haloscopes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxionConversionMetrics {
    /// Converted signal power $P_{\mathrm{conv}}$ in Watts ($\text{W}$).
    pub conversion_power_watts: f64,
    /// Converted signal power in $\text{dBm}$.
    pub conversion_power_dbm: f64,
    /// Haloscope signal-to-noise ratio in decibels ($\text{dB}$) ($\mathrm{SNR} \ge 15.0\text{ dB}$).
    pub snr_db: f64,
    /// Witten effect anomalous Hall conductance $\sigma_{xy} = \frac{e^2}{h} \frac{\theta}{2\pi}$ in Siemens ($\text{S}$).
    pub witten_anomalous_hall_conductance_siemens: f64,
    /// Witten induced monopole fractional electric charge $q / e = \theta / (2\pi)$.
    pub witten_charge_fraction: f64,
    /// Axion-polariton anti-crossing gap in $\text{GHz}$.
    pub polariton_gap_ghz: f64,
    /// Haloscope resonance frequency $\nu_a = m_a c^2 / h$ in $\text{GHz}$.
    pub resonance_frequency_ghz: f64,
    /// Galactic halo axion line virial bandwidth $\Delta \nu_a$ in $\text{kHz}$.
    pub axion_line_bandwidth_khz: f64,
}

impl AxionElectrodynamicsParams {
    /// Creates a new parameter set for quantum axion electrodynamics.
    pub fn new(
        axion_mass_uev: f64,
        magnetic_field_tesla: f64,
        cavity_volume_m3: f64,
        cavity_q_factor: f64,
    ) -> Self {
        Self {
            axion_mass_uev: axion_mass_uev.max(1.0),
            magnetic_field_tesla: magnetic_field_tesla.max(0.1),
            cavity_volume_m3: cavity_volume_m3.max(1.0e-5),
            cavity_q_factor: cavity_q_factor.max(100.0),
            ..Default::default()
        }
    }
}
