#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Reinforcement Learning Co-Pilot
//! & Neural Circuit Synthesizer.

/// Physical parameter configuration for the universal multi-scale visual CAD studio
/// autonomous reinforcement learning co-pilot and neural circuit synthesizer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NeuralCircuitCopilotParams {
    /// Reinforcement learning policy coupling energy in meV (clamp 1.0 to 35.0, default 18.0).
    pub rl_policy_coupling_mev: f64,
    /// Topological policy gap energy in meV (clamp 2.0 to 45.0, default 24.0).
    pub topological_policy_gap_mev: f64,
    /// Acoustic drive carrier frequency in GHz (clamp 1.0 to 12.0, default 6.5).
    pub acoustic_drive_frequency_ghz: f64,
    /// Neural inference dispatch speed in m/s (clamp 200.0 to 3000.0, default 1500.0).
    pub neural_inference_dispatch_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave critic diagnostic power in microwatts (clamp 0.5 to 30.0, default 6.5).
    pub microwave_critic_power_uw: f64,
    /// Synthetic actor depth factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_actor_depth_factor: f64,
    /// Synaptic routing spatial pitch in micrometers (clamp 0.5 to 20.0, default 5.5).
    pub synaptic_routing_pitch_um: f64,
}

impl Default for NeuralCircuitCopilotParams {
    fn default() -> Self {
        Self {
            rl_policy_coupling_mev: 18.0,
            topological_policy_gap_mev: 24.0,
            acoustic_drive_frequency_ghz: 6.5,
            neural_inference_dispatch_speed_m_per_s: 1500.0,
            cryogenic_temperature_mk: 10.0,
            microwave_critic_power_uw: 6.5,
            synthetic_actor_depth_factor: 4.0,
            synaptic_routing_pitch_um: 5.5,
        }
    }
}

impl NeuralCircuitCopilotParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        rl_policy_coupling_mev: f64,
        topological_policy_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        neural_inference_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_critic_power_uw: f64,
        synthetic_actor_depth_factor: f64,
        synaptic_routing_pitch_um: f64,
    ) -> Self {
        Self {
            rl_policy_coupling_mev: rl_policy_coupling_mev.clamp(1.0, 35.0),
            topological_policy_gap_mev: topological_policy_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            neural_inference_dispatch_speed_m_per_s: neural_inference_dispatch_speed_m_per_s
                .clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_critic_power_uw: microwave_critic_power_uw.clamp(0.5, 30.0),
            synthetic_actor_depth_factor: synthetic_actor_depth_factor.clamp(1.0, 8.0),
            synaptic_routing_pitch_um: synaptic_routing_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Reinforcement Learning Co-Pilot & Neural Circuit Synthesizer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NeuralCircuitCopilotMetrics {
    /// Co-pilot autonomous circuit synthesis fidelity (target >= 0.9980).
    pub copilot_synthesis_fidelity: f64,
    /// Neural state retention fraction across reinforcement learning inference cycles (target >= 0.9970).
    pub neural_state_retention_fraction: f64,
    /// Multi-physics topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-layer crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_layer_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
