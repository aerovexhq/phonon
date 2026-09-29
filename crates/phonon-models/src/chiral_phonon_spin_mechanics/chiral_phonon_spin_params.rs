//! Parameters and metrics for chiral phonon spin-mechanics
//! and quantum acoustical angular momentum multiplexers.

/// Parameters for chiral phonon spin-orbit coupling and OAM multiplexers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralPhononSpinParams {
    /// Acoustic central carrier frequency $f_0$ in $\text{GHz}$ (nominal $1.0 - 10.0\text{ GHz}$).
    pub acoustic_frequency_ghz: f64,
    /// Phase acoustic velocity $v_{\mathrm{SAW}}$ in $\text{m/s}$ (nominal $2500 - 4500\text{ m/s}$).
    pub acoustic_velocity_m_per_s: f64,
    /// Acoustic spin-orbit coupling rate $\xi_{\mathrm{SOC}} / (2\pi)$ in $\text{MHz}$ (nominal $10.0 - 80.0\text{ MHz}$).
    pub spin_orbit_coupling_mhz: f64,
    /// Orbital angular momentum (OAM) topological charge $\ell \in \{-4, \dots, +4\}$.
    pub topological_oam_charge: i32,
    /// Transducer interdigital finger pairs count $N_{\mathrm{pairs}}$ (nominal $20 - 120$).
    pub transducer_finger_pairs: usize,
    /// Piezoelectric electromechanical coupling coefficient $K^2$ (nominal $0.03 - 0.09$).
    pub electromechanical_coupling_k2: f64,
    /// Acoustic waveguide / vortex beam radius in micrometers (nominal $5.0 - 50.0\,\mu\text{m}$).
    pub waveguide_radius_um: f64,
    /// Spiral Archimedean transducer finger pitch $\Lambda$ in micrometers (nominal $0.4 - 3.0\,\mu\text{m}$).
    pub spiral_pitch_um: f64,
    /// Continuous-wave RF input drive power in milliwatts (nominal $0.5 - 20.0\text{ mW}$).
    pub pump_rf_power_mw: f64,
}

impl Default for ChiralPhononSpinParams {
    fn default() -> Self {
        Self {
            acoustic_frequency_ghz: 3.5,
            acoustic_velocity_m_per_s: 3488.0, // LiNbO3 Rayleigh SAW velocity
            spin_orbit_coupling_mhz: 35.0,
            topological_oam_charge: 1,
            transducer_finger_pairs: 50,
            electromechanical_coupling_k2: 0.055,
            waveguide_radius_um: 15.0,
            spiral_pitch_um: 1.0,
            pump_rf_power_mw: 5.0,
        }
    }
}

/// Evaluated metrics for chiral phonon spin-mechanics and OAM multiplexing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralPhononSpinMetrics {
    /// OAM mode isolation in decibels ($\ge 25.0\text{ dB}$).
    pub oam_mode_isolation_db: f64,
    /// Inter-channel modal crosstalk in decibels ($\le -20.0\text{ dB}$).
    pub channel_crosstalk_db: f64,
    /// Transducer electromechanical power conversion efficiency in percent ($\ge 70.0\%$).
    pub transduction_efficiency_pct: f64,
    /// Forward insertion loss in decibels ($\le 2.0\text{ dB}$).
    pub insertion_loss_db: f64,
    /// Phonon spin-to-orbital angular momentum conversion purity in percent ($\ge 90.0\%$).
    pub spin_orbit_purity_pct: f64,
    /// Topological OAM router / sorter extinction ratio in decibels ($\ge 25.0\text{ dB}$).
    pub router_extinction_ratio_db: f64,
    /// High-dimensional multi-channel aggregate transmission capacity in Gbps ($\ge 10.0\text{ Gbps}$).
    pub multiplexed_capacity_gbps: f64,
}

impl ChiralPhononSpinParams {
    /// Creates a new parameter set for chiral phonon spin-mechanics.
    pub fn new(freq_ghz: f64, charge: i32, pairs: usize, k2: f64, power_mw: f64) -> Self {
        Self {
            acoustic_frequency_ghz: freq_ghz.clamp(0.5, 20.0),
            topological_oam_charge: charge.clamp(-4, 4),
            transducer_finger_pairs: pairs.clamp(10, 200),
            electromechanical_coupling_k2: k2.clamp(0.01, 0.20),
            pump_rf_power_mw: power_mw.max(0.1),
            ..Default::default()
        }
    }
}
