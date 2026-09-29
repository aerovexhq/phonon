//! Parameters and metrics for non-Abelian braiding of Majorana bound states
//! in chiral surface acoustic wave (SAW) phonon networks.

/// Parameters for chiral acoustic phonon-driven Majorana braiding in topological nanowire arrays.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MajoranaBraidingParams {
    /// Semiconductor nanowire arm length in micrometers (nominal $1.0 - 5.0\,\mu\text{m}$).
    pub nanowire_length_um: f64,
    /// Proximity-induced topological superconducting pairing gap $\Delta_{\mathrm{topo}}$ in $\mu\text{eV}$ (nominal $60.0 - 350.0\,\mu\text{eV}$).
    pub topological_gap_muev: f64,
    /// Rashba spin-orbit coupling energy $\alpha_{\mathrm{SO}}$ in $\text{meV}\cdot\text{nm}$ (nominal $15.0 - 45.0$).
    pub spin_orbit_coupling_mev_nm: f64,
    /// Chiral SAW driving frequency $f_{\mathrm{saw}}$ in $\text{GHz}$ (nominal $0.8 - 3.5\text{ GHz}$).
    pub saw_frequency_ghz: f64,
    /// Traveling SAW acoustic power in microwatts (nominal $25.0 - 400.0\,\mu\text{W}$).
    pub saw_acoustic_power_uw: f64,
    /// Adiabatic Majorana exchange braiding duration $T_{\mathrm{braid}}$ in nanoseconds (nominal $20.0 - 120.0\text{ ns}$).
    pub braiding_duration_ns: f64,
    /// Sub-Kelvin cryogenic operating temperature in milliKelvin (nominal $10.0 - 80.0\text{ mK}$).
    pub ambient_temperature_mk: f64,
    /// Topological charge/parity readout resonator coupling rate in $\text{MHz}$ (nominal $5.0 - 30.0\text{ MHz}$).
    pub readout_coupling_mhz: f64,
}

impl Default for MajoranaBraidingParams {
    fn default() -> Self {
        Self {
            nanowire_length_um: 2.5,
            topological_gap_muev: 180.0,
            spin_orbit_coupling_mev_nm: 25.0,
            saw_frequency_ghz: 1.8,
            saw_acoustic_power_uw: 120.0,
            braiding_duration_ns: 50.0,
            ambient_temperature_mk: 20.0,
            readout_coupling_mhz: 15.0,
        }
    }
}

/// Evaluated metrics for acoustic Majorana braiding and topological quantum gates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MajoranaBraidingMetrics {
    /// Adiabatic quantum state braiding fidelity in percent ($\ge 99.0\%$ required).
    pub braiding_fidelity_pct: f64,
    /// Acquired non-Abelian Berry geometric phase in radians (nominal $\pi/2 \approx 1.5708\text{ rad}$).
    pub non_abelian_phase_rad: f64,
    /// Deviation of non-Abelian phase from ideal $\pi/2$ in radians ($|\phi - \pi/2| \le 0.05\text{ rad}$).
    pub phase_error_rad: f64,
    /// Landau-Zener excited state leakage transition probability ($P_{\mathrm{LZ}} \le 1.0\times 10^{-3}$).
    pub landau_zener_leakage: f64,
    /// Topological fermion parity readout signal-to-noise ratio in decibels ($\ge 20.0\text{ dB}$).
    pub parity_readout_snr_db: f64,
    /// Topological qubit dephasing time $T_2^*$ in microseconds ($\ge 10.0\,\mu\text{s}$).
    pub topological_dephasing_time_us: f64,
}

impl MajoranaBraidingParams {
    /// Creates a new parameter set for chiral acoustic Majorana braiding.
    pub fn new(wire_len_um: f64, gap_muev: f64, braid_ns: f64, power_uw: f64) -> Self {
        Self {
            nanowire_length_um: wire_len_um.max(0.5),
            topological_gap_muev: gap_muev.max(10.0),
            braiding_duration_ns: braid_ns.max(5.0),
            saw_acoustic_power_uw: power_uw.max(1.0),
            ..Default::default()
        }
    }
}
