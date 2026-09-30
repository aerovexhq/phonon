#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio GPU WebGPU / Metal Accelerators & Real-Time
//! Tensor Mesh Solvers.

/// Physical parameter configuration for the universal multi-scale visual CAD studio
/// GPU WebGPU / Metal accelerators and real-time tensor mesh solvers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GpuTensorMeshParams {
    /// GPU compute kernel coupling energy in meV (clamp 1.0 to 35.0, default 16.5).
    pub gpu_compute_coupling_mev: f64,
    /// Topological tensor mesh bandgap energy in meV (clamp 2.0 to 45.0, default 22.0).
    pub topological_mesh_gap_mev: f64,
    /// Acoustic drive carrier frequency in GHz (clamp 1.0 to 12.0, default 5.8).
    pub acoustic_drive_frequency_ghz: f64,
    /// Tensor kernel dispatch propagation speed in m/s (clamp 200.0 to 3000.0, default 1400.0).
    pub tensor_kernel_dispatch_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave probe diagnostic power in microwatts (clamp 0.5 to 30.0, default 5.8).
    pub microwave_probe_power_uw: f64,
    /// Synthetic GPU compute workgroup size factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_workgroup_size_factor: f64,
    /// Tensor mesh node spatial pitch in micrometers (clamp 0.5 to 20.0, default 4.8).
    pub mesh_node_pitch_um: f64,
}

impl Default for GpuTensorMeshParams {
    fn default() -> Self {
        Self {
            gpu_compute_coupling_mev: 16.5,
            topological_mesh_gap_mev: 22.0,
            acoustic_drive_frequency_ghz: 5.8,
            tensor_kernel_dispatch_speed_m_per_s: 1400.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 5.8,
            synthetic_workgroup_size_factor: 4.0,
            mesh_node_pitch_um: 4.8,
        }
    }
}

impl GpuTensorMeshParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        gpu_compute_coupling_mev: f64,
        topological_mesh_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        tensor_kernel_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_workgroup_size_factor: f64,
        mesh_node_pitch_um: f64,
    ) -> Self {
        Self {
            gpu_compute_coupling_mev: gpu_compute_coupling_mev.clamp(1.0, 35.0),
            topological_mesh_gap_mev: topological_mesh_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            tensor_kernel_dispatch_speed_m_per_s: tensor_kernel_dispatch_speed_m_per_s
                .clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_workgroup_size_factor: synthetic_workgroup_size_factor.clamp(1.0, 8.0),
            mesh_node_pitch_um: mesh_node_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio GPU WebGPU / Metal
/// Accelerators & Real-Time Tensor Mesh Solvers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GpuTensorMeshMetrics {
    /// Real-time tensor mesh solver numerical fidelity (target >= 0.9980).
    pub solver_fidelity: f64,
    /// Tensor state retention fraction across GPU dispatch cycles (target >= 0.9970).
    pub tensor_state_retention_fraction: f64,
    /// Multi-physics topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-channel crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_channel_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
