//! Parameters and metrics for chiral skyrmion-phonon drag,
//! topological Hall acoustics, and magnon-assisted waveguiding.

/// Parameters for surface acoustic wave (SAW) driven magnetic skyrmion dynamics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyrmionPhononParams {
    /// Skyrmion core radius $R_{\mathrm{sk}}$ in nanometers (nominal $8.0 - 30.0\text{ nm}$).
    pub skyrmion_radius_nm: f64,
    /// Integer topological charge / winding number $Q_{\mathrm{sk}} = \pm 1$.
    pub topological_charge_q: i32,
    /// Dimensionless Gilbert damping parameter $\alpha$ (nominal $0.01 - 0.05$).
    pub gilbert_damping_alpha: f64,
    /// Saturation magnetization $M_s$ in $\text{MA/m}$ (nominal $0.6 - 1.2\text{ MA/m}$).
    pub saturation_magnetization_ma_m: f64,
    /// Magnetic thin film thickness $d$ in nanometers (nominal $1.0 - 3.0\text{ nm}$).
    pub film_thickness_nm: f64,
    /// Magnetoelastic coupling coefficient $B_{\mathrm{me}}$ in $\text{MPa}$ (nominal $5.0 - 20.0\text{ MPa}$).
    pub magnetoelastic_coupling_b_mpa: f64,
    /// SAW dynamic strain amplitude $\epsilon_0$ in parts per million (ppm, nominal $200 - 1000\text{ ppm}$).
    pub saw_strain_amplitude_ppm: f64,
    /// SAW driving frequency $f_{\mathrm{saw}}$ in $\text{GHz}$ (nominal $1.0 - 4.5\text{ GHz}$).
    pub saw_frequency_ghz: f64,
    /// Acoustic phase velocity $v_s$ in $\text{m/s}$ (nominal $3000 - 4200\text{ m/s}$).
    pub sound_velocity_m_s: f64,
    /// Operating temperature $T$ in Kelvin (nominal $4.2 - 350.0\text{ K}$).
    pub temperature_k: f64,
    /// Racetrack boundary confinement coefficient $\eta_{\mathrm{conf}} \in [0.80, 0.99]$.
    pub racetrack_confinement_factor: f64,
}

impl Default for SkyrmionPhononParams {
    fn default() -> Self {
        Self {
            skyrmion_radius_nm: 16.0,
            topological_charge_q: 1,
            gilbert_damping_alpha: 0.02,
            saturation_magnetization_ma_m: 0.85,
            film_thickness_nm: 1.5,
            magnetoelastic_coupling_b_mpa: 10.0,
            saw_strain_amplitude_ppm: 500.0,
            saw_frequency_ghz: 2.5,
            sound_velocity_m_s: 3800.0,
            temperature_k: 300.0,
            racetrack_confinement_factor: 0.94,
        }
    }
}

/// Evaluated metrics for chiral skyrmion-phonon drag and topological acoustics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyrmionDynamicsMetrics {
    /// Acoustic skyrmion steady-state drift velocity $v_{\mathrm{sk}}$ in $\text{m/s}$ ($> 100.0\text{ m/s}$).
    pub skyrmion_drift_velocity_m_s: f64,
    /// Emergent topological Skyrmion Hall deflection angle $\theta_{\mathrm{skH}}$ in degrees ($10.0^\circ - 70.0^\circ$).
    pub topological_hall_angle_deg: f64,
    /// Magnetoelastic acoustic drag force $F_{\mathrm{SAW}}$ in piconewtons (pN).
    pub acoustic_drag_force_pn: f64,
    /// Non-volatile skyrmionic logic switching contrast in decibels ($\ge 25.0\text{ dB}$).
    pub logic_switching_contrast_db: f64,
    /// Topological acoustic circulator isolation in decibels ($\ge 20.0\text{ dB}$).
    pub circulator_isolation_db: f64,
    /// Energy dissipation per bit shift in femtojoules ($< 1.0\text{ fJ}$).
    pub energy_dissipation_per_bit_fj: f64,
    /// Topological thermal stability factor $\Delta E / (k_B T) \ge 1.0$ (nominal $> 40$).
    pub topological_stability_factor: f64,
}

impl SkyrmionPhononParams {
    /// Creates a new parameter set for chiral skyrmion-phonon drag systems.
    pub fn new(radius_nm: f64, b_mpa: f64, strain_ppm: f64, freq_ghz: f64) -> Self {
        Self {
            skyrmion_radius_nm: radius_nm.max(1.0),
            magnetoelastic_coupling_b_mpa: b_mpa.max(0.1),
            saw_strain_amplitude_ppm: strain_ppm.max(1.0),
            saw_frequency_ghz: freq_ghz.max(0.1),
            ..Default::default()
        }
    }
}
