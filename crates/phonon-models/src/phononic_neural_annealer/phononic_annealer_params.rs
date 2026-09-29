//! Parameters and metrics for quantum phononic neural annealers,
//! acoustic parametric oscillator networks, and adiabatic Ising solvers.

/// Parameters for acoustic parametric oscillator (APO) Ising neural annealers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhononicAnnealerParams {
    /// Number of coupled acoustic parametric oscillator spins $N_{\mathrm{spins}}$ (nominal $32 - 1024$).
    pub network_spin_count: usize,
    /// Parametric acoustic drive rate $p$ in MHz (nominal $10.0 - 150.0\text{ MHz}$).
    pub parametric_pump_rate_mhz: f64,
    /// Mean inter-oscillator acoustic coupling strength $|J_{ij}|$ in MHz (nominal $0.5 - 15.0\text{ MHz}$).
    pub acoustic_coupling_strength_mhz: f64,
    /// Non-linear Kerr elasticity saturation parameter (nominal $1.0\times 10^{-5} - 1.0\times 10^{-3}$).
    pub non_linear_elastic_kerr: f64,
    /// Adiabatic annealing pump ramp duration in microseconds (nominal $0.5 - 15.0\,\mu\text{s}$).
    pub annealing_ramp_time_us: f64,
    /// Thermal/quantum acoustic fluctuation noise floor in dBm (nominal $-110.0\text{ to } -60.0\text{ dBm}$).
    pub thermal_noise_level_dbm: f64,
}

impl Default for PhononicAnnealerParams {
    fn default() -> Self {
        Self {
            network_spin_count: 128,
            parametric_pump_rate_mhz: 48.0,
            acoustic_coupling_strength_mhz: 3.8,
            non_linear_elastic_kerr: 1.2e-4,
            annealing_ramp_time_us: 2.4,
            thermal_noise_level_dbm: -85.0,
        }
    }
}

/// Evaluated metrics for phononic neural annealers and acoustic Ising optimization.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhononicAnnealerMetrics {
    /// NP-hard combinatorial problem ground-state convergence fidelity in percent ($\ge 98.0\%$).
    pub convergence_fidelity_pct: f64,
    /// Computational speedup factor over classical simulated annealing ($\ge 100.0\times$).
    pub speedup_factor: f64,
    /// Coherent annealing energy consumption per spin flip in femtojoules ($\le 50.0\text{ fJ}$).
    pub energy_per_flip_fj: f64,
    /// Max-Cut / combinatorial graph approximation ratio ($\ge 0.95$).
    pub graph_approximation_ratio: f64,
    /// Time-to-solution latency in microseconds ($\le 10.0\,\mu\text{s}$).
    pub solution_time_us: f64,
    /// Phase bifurcation state discrimination contrast in dB ($\ge 25.0\text{ dB}$).
    pub bifurcation_contrast_db: f64,
}

impl PhononicAnnealerParams {
    /// Creates a new parameter set for phononic neural annealers.
    pub fn new(
        spins: usize,
        pump_mhz: f64,
        coupling_mhz: f64,
        ramp_us: f64,
        noise_dbm: f64,
    ) -> Self {
        Self {
            network_spin_count: spins.clamp(16, 4096),
            parametric_pump_rate_mhz: pump_mhz.clamp(1.0, 500.0),
            acoustic_coupling_strength_mhz: coupling_mhz.clamp(0.1, 50.0),
            annealing_ramp_time_us: ramp_us.clamp(0.1, 100.0),
            thermal_noise_level_dbm: noise_dbm.clamp(-140.0, -30.0),
            ..Default::default()
        }
    }
}
