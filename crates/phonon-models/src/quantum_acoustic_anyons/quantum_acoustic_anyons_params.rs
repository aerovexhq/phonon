//! Parameters and metrics for non-Abelian anyon braiding in quantum acoustic surface networks.

/// Parameters for SAW dynamic nanoconstrictions and Majorana anyon braiding networks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumAcousticAnyonParams {
    /// Topological superconducting minigap $\Delta_{\mathrm{topo}} / (2\pi)$ in $\text{MHz}$ (nominal $15.0 - 90.0\text{ MHz}$).
    pub topological_gap_mhz: f64,
    /// Surface acoustic wave drive frequency $f_{\mathrm{SAW}}$ in $\text{GHz}$ (nominal $1.0 - 6.0\text{ GHz}$).
    pub saw_frequency_ghz: f64,
    /// Adiabatic braiding cycle duration $\tau_{\mathrm{braid}}$ in nanoseconds (nominal $50.0 - 400.0\text{ ns}$).
    pub braiding_duration_ns: f64,
    /// SAW piezoelectric confinement potential amplitude $V_0$ in $\text{meV}$ (nominal $5.0 - 40.0\text{ meV}$).
    pub saw_potential_amplitude_mev: f64,
    /// Nanowire tri-junction branch length in micrometers (nominal $1.0 - 5.0\,\mu\text{m}$).
    pub junction_arm_length_um: f64,
    /// Effective induced Rashba spin-orbit coupling rate $\alpha_{\mathrm{SO}}$ in $\text{eV}\cdot\text{\AA}$ (nominal $0.5 - 2.5\text{ eV}\cdot\text{\AA}$).
    pub spin_orbit_coupling_ev_ang: f64,
    /// In-plane Zeeman magnetic field energy $E_Z$ in $\text{meV}$ (nominal $0.5 - 4.0\text{ meV}$).
    pub zeeman_energy_mev: f64,
    /// Dilution cryostat thermal temperature in millikelvin (nominal $15.0 - 60.0\text{ mK}$).
    pub temperature_mk: f64,
}

impl Default for QuantumAcousticAnyonParams {
    fn default() -> Self {
        Self {
            topological_gap_mhz: 42.0,
            saw_frequency_ghz: 2.5,
            braiding_duration_ns: 160.0,
            saw_potential_amplitude_mev: 18.0,
            junction_arm_length_um: 2.5,
            spin_orbit_coupling_ev_ang: 1.2,
            zeeman_energy_mev: 1.8,
            temperature_mk: 25.0,
        }
    }
}

/// Evaluated metrics for quantum acoustic non-Abelian anyon braiding networks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumAcousticAnyonMetrics {
    /// Fault-tolerant braiding quantum gate fidelity in percent ($\ge 99.90\%$).
    pub braiding_fidelity_pct: f64,
    /// Non-adiabatic Landau-Zener transition leakage probability ($\le 1.0\times 10^{-5}$).
    pub non_adiabatic_leakage: f64,
    /// Effective protected topological minigap in $\text{MHz}$ ($\ge 15.0\text{ MHz}$).
    pub effective_topological_gap_mhz: f64,
    /// Geometric Berry phase holonomy error in radians ($\le 0.005\text{ rad}$).
    pub braiding_phase_error_rad: f64,
    /// Non-local topological fermion parity readout SNR in decibels ($\ge 30.0\text{ dB}$).
    pub parity_readout_snr_db: f64,
    /// Topological quantum memory dephasing coherence time $T_2^*$ in microseconds ($\ge 50.0\,\mu\text{s}$).
    pub coherence_time_us: f64,
}

impl QuantumAcousticAnyonParams {
    /// Creates a new parameter set for quantum acoustic anyon braiding.
    pub fn new(
        gap_mhz: f64,
        f_saw_ghz: f64,
        duration_ns: f64,
        potential_mev: f64,
        t_mk: f64,
    ) -> Self {
        Self {
            topological_gap_mhz: gap_mhz.clamp(5.0, 300.0),
            saw_frequency_ghz: f_saw_ghz.clamp(0.2, 20.0),
            braiding_duration_ns: duration_ns.clamp(10.0, 2000.0),
            saw_potential_amplitude_mev: potential_mev.clamp(1.0, 100.0),
            temperature_mk: t_mk.clamp(1.0, 500.0),
            ..Default::default()
        }
    }
}
