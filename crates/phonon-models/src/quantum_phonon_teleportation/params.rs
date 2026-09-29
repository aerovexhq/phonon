//! Multi-physics parameter structures and metric definitions for
//! quantum phonon-mediated superconducting qubit teleportation and state transfer.

/// Physical parameter configuration for quantum phonon-mediated qubit state transfer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhononTeleportationParams {
    /// Superconducting transmon qubit resonance frequency $f_q$ in GHz (default 4.8 GHz).
    pub qubit_freq_ghz: f64,
    /// Transmon negative anharmonicity $\alpha / (2\pi)$ in MHz (default -250.0 MHz).
    pub qubit_anharmonicity_mhz: f64,
    /// Transmon energy relaxation time $T_1$ in microseconds (default 65.0 us).
    pub qubit_t1_us: f64,
    /// Transmon pure dephasing time $T_\phi$ in microseconds (default 55.0 us).
    pub qubit_tphi_us: f64,
    /// Electromechanical transducer coupling rate $g / (2\pi)$ in MHz (default 30.0 MHz).
    pub electromechanical_coupling_mhz: f64,
    /// Rayleigh acoustic phase velocity $v_{\\text{saw}}$ in m/s (default 3488.0 m/s for LiNbO3).
    pub acoustic_velocity_m_s: f64,
    /// Acoustic transmission link length $L$ in micrometers (default 320.0 um).
    pub acoustic_link_length_um: f64,
    /// Cryogenic acoustic attenuation rate in dB/cm (default 0.25 dB/cm).
    pub acoustic_attenuation_db_per_cm: f64,
    /// Dilution refrigerator ambient temperature $T$ in milliKelvin (default 15.0 mK).
    pub cryogenic_temperature_mk: f64,
    /// Time-symmetric acoustic wavepacket shaping efficiency factor (default 0.996).
    pub wavepacket_shaping_efficiency: f64,
    /// Interdigital transducer (IDT) acoustic bandwidth in MHz (default 65.0 MHz).
    pub idt_bandwidth_mhz: f64,
}

impl Default for PhononTeleportationParams {
    fn default() -> Self {
        Self {
            qubit_freq_ghz: 4.8,
            qubit_anharmonicity_mhz: -250.0,
            qubit_t1_us: 65.0,
            qubit_tphi_us: 55.0,
            electromechanical_coupling_mhz: 30.0,
            acoustic_velocity_m_s: 3488.0,
            acoustic_link_length_um: 320.0,
            acoustic_attenuation_db_per_cm: 0.25,
            cryogenic_temperature_mk: 15.0,
            wavepacket_shaping_efficiency: 0.996,
            idt_bandwidth_mhz: 65.0,
        }
    }
}

impl PhononTeleportationParams {
    /// Creates a new parameter configuration for quantum phonon state transfer.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        f_q_ghz: f64,
        alpha_mhz: f64,
        t1_us: f64,
        tphi_us: f64,
        g_mhz: f64,
        v_saw: f64,
        link_um: f64,
        att_db_cm: f64,
        temp_mk: f64,
        shaping_eff: f64,
        idt_bw_mhz: f64,
    ) -> Self {
        Self {
            qubit_freq_ghz: f_q_ghz.clamp(1.0, 15.0),
            qubit_anharmonicity_mhz: alpha_mhz.clamp(-600.0, -50.0),
            qubit_t1_us: t1_us.clamp(5.0, 500.0),
            qubit_tphi_us: tphi_us.clamp(5.0, 500.0),
            electromechanical_coupling_mhz: g_mhz.clamp(1.0, 100.0),
            acoustic_velocity_m_s: v_saw.clamp(1500.0, 6000.0),
            acoustic_link_length_um: link_um.clamp(10.0, 5000.0),
            acoustic_attenuation_db_per_cm: att_db_cm.clamp(0.01, 10.0),
            cryogenic_temperature_mk: temp_mk.clamp(1.0, 500.0),
            wavepacket_shaping_efficiency: shaping_eff.clamp(0.80, 1.0),
            idt_bandwidth_mhz: idt_bw_mhz.clamp(10.0, 300.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum phonon-mediated teleportation and state transfer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhononTeleportationMetrics {
    /// Quantum state transfer fidelity $\\mathcal{F}_{\\text{trans}}$ (target >= 96.0%).
    pub state_transfer_fidelity: f64,
    /// Remote acoustic Bell state concurrence $\\mathcal{C}$ (target >= 0.92).
    pub acoustic_bell_concurrence: f64,
    /// Total single-phonon loss probability $P_{\\text{loss}}$ (target <= 0.02).
    pub phonon_loss_probability: f64,
    /// Usable quantum link bandwidth in MHz (target >= 50.0 MHz).
    pub quantum_link_bandwidth_mhz: f64,
    /// Itinerant acoustic propagation delay across link in nanoseconds.
    pub itinerant_propagation_delay_ns: f64,
    /// Cryogenic thermal phonon bath equilibrium occupancy $n_{\\text{th}}$.
    pub thermal_phonon_occupancy: f64,
    /// Total qubit decoherence rate $\\gamma_2$ in MHz.
    pub qubit_total_dephasing_rate_mhz: f64,
    /// Physical compliance flag verifying all roadmap benchmarks are met.
    pub is_physically_compliant: bool,
}
