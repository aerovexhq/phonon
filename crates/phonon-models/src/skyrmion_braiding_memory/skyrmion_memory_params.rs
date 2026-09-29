//! Parameters and metrics for chiral phonon-driven skyrmion braiding,
//! non-volatile racetrack acoustic memory registers, and topological logic.

/// Parameters for acoustic surface wave skyrmion dragging, braiding, and memory.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyrmionMemoryParams {
    /// Surface acoustic wave (SAW) wavelength in nanometers (nominal $50.0 - 400.0\text{ nm}$).
    pub acoustic_wavelength_nm: f64,
    /// Peak acoustic dynamic strain tensor amplitude (nominal $5.0\times 10^{-4} - 5.0\times 10^{-3}$).
    pub acoustic_strain_amplitude: f64,
    /// Magnetoelastic coupling energy density $B_{\mathrm{me}}$ in $\text{J/m}^3$ (nominal $2.0\times 10^6 - 2.0\times 10^7\text{ J/m}^3$).
    pub magnetoelastic_coupling_j_m3: f64,
    /// Dimensionless magnetic Gilbert damping parameter $\alpha$ (nominal $0.005 - 0.05$).
    pub gilbert_damping: f64,
    /// Nanowire racetrack conduit width in nanometers (nominal $30.0 - 150.0\text{ nm}$).
    pub nanowire_track_width_nm: f64,
    /// Thermal stability factor $\Delta E / (k_B T)$ (nominal $50.0 - 90.0$).
    pub thermal_stability_factor: f64,
}

impl Default for SkyrmionMemoryParams {
    fn default() -> Self {
        Self {
            acoustic_wavelength_nm: 120.0,
            acoustic_strain_amplitude: 1.8e-3,
            magnetoelastic_coupling_j_m3: 8.5e6,
            gilbert_damping: 0.015,
            nanowire_track_width_nm: 60.0,
            thermal_stability_factor: 62.0,
        }
    }
}

/// Evaluated metrics for chiral phonon-magnon skyrmion braiding and racetrack memory.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkyrmionMemoryMetrics {
    /// Steady-state acoustic skyrmion drift velocity along the conduit in m/s ($\ge 250.0\text{ m/s}$).
    pub drift_velocity_m_s: f64,
    /// Non-volatile racetrack memory bit error rate ($\le 1.0\times 10^{-12}$).
    pub bit_error_rate: f64,
    /// Non-volatile memory retention time in years ($\ge 15.0\text{ years}$).
    pub retention_years: f64,
    /// Synaptic / memory bit write energy in femtojoules ($\le 0.5\text{ fJ}$).
    pub write_energy_fj: f64,
    /// Non-commutative topological skyrmion braiding gate fidelity in percent ($\ge 99.5\%$).
    pub braiding_fidelity_pct: f64,
    /// Skyrmion Hall effect deflection angle suppression efficiency in percent ($\ge 90.0\%$).
    pub hall_angle_suppression_pct: f64,
}

impl SkyrmionMemoryParams {
    /// Creates a new parameter set for chiral phonon-magnon skyrmion memory.
    pub fn new(
        lambda_nm: f64,
        strain: f64,
        coupling: f64,
        damping: f64,
        track_nm: f64,
        stability: f64,
    ) -> Self {
        Self {
            acoustic_wavelength_nm: lambda_nm.clamp(20.0, 1000.0),
            acoustic_strain_amplitude: strain.clamp(1e-4, 0.02),
            magnetoelastic_coupling_j_m3: coupling.clamp(1e5, 1e8),
            gilbert_damping: damping.clamp(0.001, 0.2),
            nanowire_track_width_nm: track_nm.clamp(10.0, 500.0),
            thermal_stability_factor: stability.clamp(20.0, 150.0),
        }
    }
}
