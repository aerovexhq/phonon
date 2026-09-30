#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Native Engine & WebAssembly Real-Time Physics Interactive Co-Processor.

/// Physical parameter configuration for the universal multi-scale visual CAD studio
/// native engine and WebAssembly real-time interactive physics co-processor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisualStudioEngineParams {
    /// Canvas render resolution in pixels (clamp 512.0 to 8192.0, default 2048.0).
    pub canvas_render_resolution_pixels: f64,
    /// Target frame rate in FPS (clamp 30.0 to 240.0, default 60.0).
    pub target_frame_rate_fps: f64,
    /// Number of multi-physics continuum mesh nodes (clamp 1000.0 to 1000000.0, default 100000.0).
    pub multi_physics_mesh_nodes: f64,
    /// Native desktop IPC buffer size in kilobytes (clamp 64.0 to 16384.0, default 1024.0).
    pub ipc_buffer_size_kb: f64,
    /// Active realism tier index (clamp 0.0 to 6.0, default 0.0).
    pub realism_tier_active: f64,
    /// Interactive parameter update rate in Hz (clamp 10.0 to 1000.0, default 120.0).
    pub interactive_parameter_update_rate_hz: f64,
    /// WebAssembly linear memory pool size in 64 KiB pages (clamp 256.0 to 65536.0, default 4096.0).
    pub wasm_memory_pool_pages: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
}

impl Default for VisualStudioEngineParams {
    fn default() -> Self {
        Self {
            canvas_render_resolution_pixels: 2048.0,
            target_frame_rate_fps: 60.0,
            multi_physics_mesh_nodes: 100000.0,
            ipc_buffer_size_kb: 1024.0,
            realism_tier_active: 0.0,
            interactive_parameter_update_rate_hz: 120.0,
            wasm_memory_pool_pages: 4096.0,
            cryogenic_temperature_mk: 10.0,
        }
    }
}

impl VisualStudioEngineParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        canvas_render_resolution_pixels: f64,
        target_frame_rate_fps: f64,
        multi_physics_mesh_nodes: f64,
        ipc_buffer_size_kb: f64,
        realism_tier_active: f64,
        interactive_parameter_update_rate_hz: f64,
        wasm_memory_pool_pages: f64,
        cryogenic_temperature_mk: f64,
    ) -> Self {
        Self {
            canvas_render_resolution_pixels: canvas_render_resolution_pixels.clamp(512.0, 8192.0),
            target_frame_rate_fps: target_frame_rate_fps.clamp(30.0, 240.0),
            multi_physics_mesh_nodes: multi_physics_mesh_nodes.clamp(1000.0, 1000000.0),
            ipc_buffer_size_kb: ipc_buffer_size_kb.clamp(64.0, 16384.0),
            realism_tier_active: realism_tier_active.clamp(0.0, 6.0),
            interactive_parameter_update_rate_hz: interactive_parameter_update_rate_hz
                .clamp(10.0, 1000.0),
            wasm_memory_pool_pages: wasm_memory_pool_pages.clamp(256.0, 65536.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio Native Engine
/// & WebAssembly Real-Time Physics Interactive Co-Processor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisualStudioEngineMetrics {
    /// Visual engine frame fidelity across the multi-scale viewport (target >= 0.9980).
    pub engine_frame_fidelity: f64,
    /// Real-time interactive physics state retention fraction (target >= 0.9970).
    pub interactive_state_retention_fraction: f64,
    /// Multi-physics topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-tier crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_tier_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
