//! Multi-physics parameter structures and metric definitions for
//! quantum acoustic waveguide QED and chiral phonon-atom bound states.

/// Physical parameter configuration for quantum acoustic waveguide QED.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaveguideQedParams {
    /// Superconducting artificial atom (transmon) transition frequency $f_a$ in GHz (default 4.5 GHz).
    pub atom_freq_ghz: f64,
    /// 1D phononic crystal waveguide acoustic bandgap center in GHz (default 4.5 GHz).
    pub bandgap_center_ghz: f64,
    /// Waveguide phononic stopband fractional bandwidth (default 0.18).
    pub bandgap_fractional_width: f64,
    /// Rayleigh acoustic phase velocity $v_{\text{saw}}$ in m/s (default 3488.0 m/s).
    pub acoustic_velocity_m_s: f64,
    /// Giant atom inter-coupling port separation distance $d$ in micrometers (default 3.87 um ~ 5 lambda/4).
    pub coupling_port_spacing_um: f64,
    /// Number of discrete acoustic coupling fingers per giant atom (default 12).
    pub coupling_finger_count: usize,
    /// Single-port acoustic emission decay rate $\Gamma_0 / (2\pi)$ in MHz (default 8.0 MHz).
    pub base_emission_rate_mhz: f64,
    /// Relative phase shift between coupling ports in radians (default pi/2 for maximal chirality).
    pub coupling_phase_shift_rad: f64,
    /// Multi-qubit separation distance $L$ along the waveguide in micrometers (default 180.0 um).
    pub inter_qubit_distance_um: f64,
    /// Qubit intrinsic non-radiative energy relaxation time $T_1$ in microseconds (default 80.0 us).
    pub qubit_intrinsic_t1_us: f64,
    /// Qubit pure dephasing time $T_\phi$ in microseconds (default 70.0 us).
    pub qubit_tphi_us: f64,
    /// Cryogenic bath temperature in milliKelvin (default 15.0 mK).
    pub cryogenic_temperature_mk: f64,
}

impl Default for WaveguideQedParams {
    fn default() -> Self {
        Self {
            atom_freq_ghz: 4.5,
            bandgap_center_ghz: 4.5,
            bandgap_fractional_width: 0.18,
            acoustic_velocity_m_s: 3488.0,
            coupling_port_spacing_um: 3.87,
            coupling_finger_count: 12,
            base_emission_rate_mhz: 8.0,
            coupling_phase_shift_rad: std::f64::consts::FRAC_PI_2,
            inter_qubit_distance_um: 180.0,
            qubit_intrinsic_t1_us: 80.0,
            qubit_tphi_us: 70.0,
            cryogenic_temperature_mk: 15.0,
        }
    }
}

impl WaveguideQedParams {
    /// Creates a new parameter configuration with bounds clamping.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        f_a_ghz: f64,
        f_bg_ghz: f64,
        bg_width: f64,
        v_saw: f64,
        d_um: f64,
        fingers: usize,
        gamma_0_mhz: f64,
        phase_rad: f64,
        l_um: f64,
        t1_us: f64,
        tphi_us: f64,
        temp_mk: f64,
    ) -> Self {
        Self {
            atom_freq_ghz: f_a_ghz.clamp(1.0, 15.0),
            bandgap_center_ghz: f_bg_ghz.clamp(1.0, 15.0),
            bandgap_fractional_width: bg_width.clamp(0.01, 0.50),
            acoustic_velocity_m_s: v_saw.clamp(1500.0, 6000.0),
            coupling_port_spacing_um: d_um.clamp(0.1, 50.0),
            coupling_finger_count: fingers.clamp(2, 64),
            base_emission_rate_mhz: gamma_0_mhz.clamp(0.1, 100.0),
            coupling_phase_shift_rad: phase_rad.clamp(-std::f64::consts::PI, std::f64::consts::PI),
            inter_qubit_distance_um: l_um.clamp(10.0, 2000.0),
            qubit_intrinsic_t1_us: t1_us.clamp(5.0, 500.0),
            qubit_tphi_us: tphi_us.clamp(5.0, 500.0),
            cryogenic_temperature_mk: temp_mk.clamp(1.0, 500.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic waveguide QED.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaveguideQedMetrics {
    /// Chiral directional emission ratio $D = (\Gamma_R - \Gamma_L) / (\Gamma_R + \Gamma_L)$ (target >= 95.0%).
    pub chiral_acoustic_directionality: f64,
    /// Waveguide Purcell emission enhancement factor $F_P = \Gamma_{wg} / \Gamma_{free}$ (target >= 80.0).
    pub waveguide_purcell_factor: f64,
    /// Bound state in continuum (BIC) atom lifetime extension factor $\tau_{\text{bound}} / \tau_0$ (target >= 50.0x).
    pub bound_state_lifetime_extension: f64,
    /// Multi-qubit coherent acoustic entanglement concurrence $\mathcal{C}$ (target >= 0.90).
    pub acoustic_entanglement_concurrence: f64,
    /// Retardation non-Markovian travel delay $\tau = L / v$ in nanoseconds.
    pub acoustic_retardation_delay_ns: f64,
    /// Rightward directional decay rate $\Gamma_R / (2\pi)$ in MHz.
    pub rightward_emission_rate_mhz: f64,
    /// Leftward suppressed decay rate $\Gamma_L / (2\pi)$ in MHz.
    pub leftward_emission_rate_mhz: f64,
    /// Physical compliance verification flag.
    pub is_physically_compliant: bool,
}
