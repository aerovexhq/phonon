//! Parameters and metrics for fractional quantum Hall acoustic interferometers,
//! fractional charge shot noise, and non-Abelian anyon braiding probes.

/// Parameters for SAW-coupled fractional quantum Hall interferometers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FqhInterferometerParams {
    /// FQH Landau level filling factor $\\nu$ (nominal $2.5 = 5/2$ Moore-Read Pfaffian).
    pub filling_factor_nu: f64,
    /// Quasiparticle fractional charge ratio $e^* / e$ (nominal $0.25$).
    pub quasiparticle_charge_ratio: f64,
    /// Cryogenic sub-Kelvin base temperature in millikelvin (nominal $10.0 - 60.0\text{ mK}$).
    pub temperature_mk: f64,
    /// Chiral Luttinger liquid edge drift velocity in m/s (nominal $2.0\times 10^4 - 8.0\times 10^4\text{ m/s}$).
    pub edge_velocity_m_s: f64,
    /// Edge state phase dephasing time $\\tau_\\phi$ in nanoseconds (nominal $0.5 - 4.0\text{ ns}$).
    pub dephasing_time_ns: f64,
    /// Surface acoustic wave (SAW) dynamic potential amplitude in millivolts (nominal $5.0 - 40.0\text{ mV}$).
    pub saw_drive_amplitude_mv: f64,
}

impl Default for FqhInterferometerParams {
    fn default() -> Self {
        Self {
            filling_factor_nu: 2.5,
            quasiparticle_charge_ratio: 0.25,
            temperature_mk: 20.0,
            edge_velocity_m_s: 45_000.0,
            dephasing_time_ns: 1.2,
            saw_drive_amplitude_mv: 15.0,
        }
    }
}

/// Evaluated metrics for fractional quantum Hall acoustic interferometers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FqhInterferometerMetrics {
    /// Fractional charge measurement precision error $|e^* / e - 0.25|$ ($\\le 1.0\times 10^{-4}$).
    pub charge_precision_error: f64,
    /// Evaluated Fano factor $F = S_I / (2 e I_B)$ ($0.25 \pm 1.0\times 10^{-4}$).
    pub fano_factor: f64,
    /// Interferometric fringe visibility in percent ($\ge 90.0\%$).
    pub fringe_visibility_pct: f64,
    /// Chiral edge phase coherence length in micrometers ($\ge 25.0\,\mu\text{m}$).
    pub coherence_length_um: f64,
    /// Shot noise cross-correlation suppression in dB ($\\le -20.0\text{ dB}$).
    pub cross_correlation_db: f64,
    /// Sub-Kelvin cryogenic readout signal-to-noise ratio in dB ($\ge 25.0\text{ dB}$).
    pub readout_snr_db: f64,
}

impl FqhInterferometerParams {
    /// Creates a new parameter set for FQH acoustic interferometers.
    pub fn new(nu: f64, t_mk: f64, v_edge: f64, tau_ns: f64, v_saw_mv: f64) -> Self {
        Self {
            filling_factor_nu: nu.clamp(1.0, 5.0),
            quasiparticle_charge_ratio: 0.25,
            temperature_mk: t_mk.clamp(1.0, 200.0),
            edge_velocity_m_s: v_edge.clamp(1000.0, 500_000.0),
            dephasing_time_ns: tau_ns.clamp(0.1, 50.0),
            saw_drive_amplitude_mv: v_saw_mv.clamp(0.5, 200.0),
        }
    }
}
