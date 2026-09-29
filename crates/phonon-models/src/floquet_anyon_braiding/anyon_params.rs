//! Parameters and metrics for Floquet-engineered non-Abelian anyon lattices
//! and topological phonon braiding in driven acoustic crystals.

/// Parameters for Floquet-engineered acoustic lattices generating synthetic non-Abelian gauge fields.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetAnyonParams {
    /// Number of acoustic lattice sites per dimension (nominal $16 - 64$).
    pub lattice_dimension: usize,
    /// High-frequency Floquet drive modulation frequency $\Omega / (2\pi)$ in $\text{MHz}$ (nominal $15.0 - 60.0\text{ MHz}$).
    pub floquet_drive_freq_mhz: f64,
    /// Spatio-temporal acoustic hopping modulation depth $\mu_{\mathrm{mod}} \in [0.20, 0.75]$.
    pub modulation_depth: f64,
    /// Effective Floquet topological quasi-energy gap $\Delta_F$ in $\text{kHz}$ (nominal $100.0 - 500.0\text{ kHz}$).
    pub floquet_gap_khz: f64,
    /// Single braid sequence duration $T_{\mathrm{braid}}$ in microseconds (nominal $1.5 - 15.0\,\mu\text{s}$).
    pub braid_duration_us: f64,
    /// Physical anyon vortex core separation in micrometers (nominal $2.0 - 10.0\,\mu\text{m}$).
    pub anyon_core_separation_um: f64,
    /// Cryogenic sub-Kelvin temperature in milliKelvin (nominal $10.0 - 80.0\text{ mK}$).
    pub ambient_temperature_mk: f64,
}

impl Default for FloquetAnyonParams {
    fn default() -> Self {
        Self {
            lattice_dimension: 32,
            floquet_drive_freq_mhz: 30.0,
            modulation_depth: 0.45,
            floquet_gap_khz: 220.0,
            braid_duration_us: 4.5,
            anyon_core_separation_um: 4.0,
            ambient_temperature_mk: 20.0,
        }
    }
}

/// Evaluated metrics for Floquet-engineered non-Abelian anyon lattices and quantum gates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetAnyonMetrics {
    /// Fault-tolerant topological braiding gate fidelity in percent ($\ge 99.5\%$ required).
    pub braiding_gate_fidelity_pct: f64,
    /// Non-adiabatic Landau-Zener-Floquet leakage error rate ($P_{\mathrm{leak}} \le 1.0\times 10^{-4}$).
    pub leakage_error_rate: f64,
    /// Norm of synthetic non-Abelian gauge field commutator $\|[A_x, A_y]\|$ ($\ge 0.50$).
    pub synthetic_gauge_commutator_norm: f64,
    /// Floquet topological bandgap in $\text{kHz}$ ($\ge 50.0\text{ kHz}$).
    pub floquet_gap_khz: f64,
    /// Logical topological state readout SNR in decibels ($\ge 25.0\text{ dB}$).
    pub logical_readout_snr_db: f64,
    /// Quantum logical qubit Hilbert space dimension ($d = 2$).
    pub anyon_qubit_dimension: usize,
}

impl FloquetAnyonParams {
    /// Creates a new parameter set for Floquet anyon braiding lattices.
    pub fn new(dim: usize, drive_mhz: f64, gap_khz: f64, braid_us: f64) -> Self {
        Self {
            lattice_dimension: dim.max(8),
            floquet_drive_freq_mhz: drive_mhz.max(1.0),
            floquet_gap_khz: gap_khz.max(10.0),
            braid_duration_us: braid_us.max(0.5),
            ..Default::default()
        }
    }
}
