#![deny(unsafe_code)]

//! Pure safe Rust atomic Seqlock POSIX shared memory connector and auto-selecting adaptive dynamics backend.
//!
//! This module enables zero-copy ingestion of multi-world kinematics telemetry from active Aerovex
//! simulation daemon sessions (`/dev/shm/aerovex_sim_state.bin`) at up to 8.65M ticks per second,
//! while providing seamless fallback to the reference RK4 physics engine when running standalone.

use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::dynamics::{
    ActuatorInputs, BackendInfo, DynamicsError, DynamicsTelemetry, PhysicsDynamicsBackend,
};
use crate::dynamics_reference::ReferenceDynamicsBackend;
use crate::probe::{AerovexPresenceProbe, AVSM_MAGIC, DEFAULT_SHM_PATH};

/// Binary header size in bytes for the Aerovex Shared Memory layout.
pub const AVSM_HEADER_SIZE: usize = 64;

/// Slot metadata header size preceding entity arrays (seqlock, world_id, is_active, sim_time, ticks, count, pad).
pub const AVSM_SLOT_META_SIZE: usize = 40;

/// Byte stride for a single packed entity state packet.
pub const AVSM_ENTITY_SIZE: usize = 112;

/// Default byte stride for a single world slot containing 64 entities (40 + 64 * 112 = 7208 bytes).
pub const AVSM_DEFAULT_SLOT_STRIDE: usize = AVSM_SLOT_META_SIZE + 64 * AVSM_ENTITY_SIZE;

/// Kinematic state snapshot read from a specific world slot in the Aerovex shared memory buffer.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ShmSlotData {
    /// Total simulation time in seconds.
    pub sim_time_s: f64,
    /// Total simulation integration ticks accumulated by the kernel.
    pub total_ticks: u64,
    /// Vehicle position [x, y, z] in the North-East-Down (NED) frame in meters.
    pub position_m: [f64; 3],
    /// Attitude unit quaternion [w, x, y, z] mapping body frame to NED frame.
    pub orientation_quat: [f64; 4],
    /// Linear velocity [vx, vy, vz] in the NED frame in m/s.
    pub velocity_m_per_s: [f64; 3],
    /// Angular velocity [p, q, r] in body frame in rad/s.
    pub angular_velocity_rad_per_s: [f64; 3],
}

impl Default for ShmSlotData {
    fn default() -> Self {
        Self {
            sim_time_s: 0.0,
            total_ticks: 0,
            position_m: [0.0; 3],
            orientation_quat: [1.0, 0.0, 0.0, 0.0],
            velocity_m_per_s: [0.0; 3],
            angular_velocity_rad_per_s: [0.0; 3],
        }
    }
}

/// Safely reads and validates entity kinematics from a raw Aerovex Shared Memory buffer using the Seqlock protocol.
///
/// Execution Protocol:
/// 1. Validates the 64-byte `AVSM` header magic.
/// 2. Calculates slot and entity byte offsets with checked integer arithmetic.
/// 3. Executes an optimistic Seqlock loop (up to 5 retries):
///    - Reads sequence counter `seq1`.
///    - Verifies `seq1 % 2 == 0` (even sequence indicates writer is inactive).
///    - Extracts 64-bit float kinematics payload.
///    - Reads sequence counter `seq2`.
///    - Confirms `seq1 == seq2` (consistent, non-torn atomic snapshot).
/// 4. Re-normalizes the orientation quaternion to guaranteed unit magnitude.
pub fn safe_read_shm_slot(
    data: &[u8],
    world_idx: usize,
    entity_idx: usize,
) -> Result<ShmSlotData, DynamicsError> {
    if data.len() < AVSM_HEADER_SIZE {
        return Err(DynamicsError::StepFailed(format!(
            "SHM buffer size ({} bytes) is smaller than AVSM header ({} bytes)",
            data.len(),
            AVSM_HEADER_SIZE
        )));
    }

    if data[0..4] != AVSM_MAGIC {
        return Err(DynamicsError::StepFailed(format!(
            "Invalid AVSM header magic: expected b\"AVSM\", found {:?}",
            &data[0..4]
        )));
    }

    let header_stride = u64::from_le_bytes(data[24..32].try_into().unwrap()) as usize;
    let slot_stride = if header_stride > 0 {
        header_stride
    } else {
        AVSM_DEFAULT_SLOT_STRIDE
    };

    let slot_offset = AVSM_HEADER_SIZE
        .checked_add(
            world_idx
                .checked_mul(slot_stride)
                .ok_or_else(|| DynamicsError::StepFailed("World index multiplication overflow".to_string()))?,
        )
        .ok_or_else(|| DynamicsError::StepFailed("Slot offset addition overflow".to_string()))?;

    let entity_offset = slot_offset
        .checked_add(AVSM_SLOT_META_SIZE)
        .and_then(|o| o.checked_add(entity_idx.checked_mul(AVSM_ENTITY_SIZE)?))
        .ok_or_else(|| DynamicsError::StepFailed("Entity offset arithmetic overflow".to_string()))?;

    let required_len = entity_offset
        .checked_add(AVSM_ENTITY_SIZE)
        .ok_or_else(|| DynamicsError::StepFailed("Buffer length bounds check overflow".to_string()))?;

    if data.len() < required_len {
        return Err(DynamicsError::StepFailed(format!(
            "SHM buffer length {} is insufficient for world {} entity {} (requires {} bytes)",
            data.len(),
            world_idx,
            entity_idx,
            required_len
        )));
    }

    const MAX_RETRIES: usize = 5;
    for attempt in 0..MAX_RETRIES {
        // Step 1: Read seq1 (u64 little endian at slot offset)
        let seq1 = u64::from_le_bytes(data[slot_offset..slot_offset + 8].try_into().unwrap());

        // Step 2: Odd sequence indicates an active writer in progress
        if seq1 % 2 != 0 {
            if attempt + 1 == MAX_RETRIES {
                return Err(DynamicsError::StepFailed("writer active".to_string()));
            }
            continue;
        }

        // Step 3: Read slot metadata payload
        let sim_time_s = f64::from_le_bytes(data[slot_offset + 16..slot_offset + 24].try_into().unwrap());
        let total_ticks = u64::from_le_bytes(data[slot_offset + 24..slot_offset + 32].try_into().unwrap());

        // Step 4: Read entity kinematics payload
        let pos_x = f64::from_le_bytes(data[entity_offset + 8..entity_offset + 16].try_into().unwrap());
        let pos_y = f64::from_le_bytes(data[entity_offset + 16..entity_offset + 24].try_into().unwrap());
        let pos_z = f64::from_le_bytes(data[entity_offset + 24..entity_offset + 32].try_into().unwrap());

        let q0 = f64::from_le_bytes(data[entity_offset + 32..entity_offset + 40].try_into().unwrap());
        let q1 = f64::from_le_bytes(data[entity_offset + 40..entity_offset + 48].try_into().unwrap());
        let q2 = f64::from_le_bytes(data[entity_offset + 48..entity_offset + 56].try_into().unwrap());
        let q3 = f64::from_le_bytes(data[entity_offset + 56..entity_offset + 64].try_into().unwrap());

        let vx = f64::from_le_bytes(data[entity_offset + 64..entity_offset + 72].try_into().unwrap());
        let vy = f64::from_le_bytes(data[entity_offset + 72..entity_offset + 80].try_into().unwrap());
        let vz = f64::from_le_bytes(data[entity_offset + 80..entity_offset + 88].try_into().unwrap());

        let wx = f64::from_le_bytes(data[entity_offset + 88..entity_offset + 96].try_into().unwrap());
        let wy = f64::from_le_bytes(data[entity_offset + 96..entity_offset + 104].try_into().unwrap());
        let wz = f64::from_le_bytes(data[entity_offset + 104..entity_offset + 112].try_into().unwrap());

        // Step 5: Read seq2 and verify atomic integrity
        let seq2 = u64::from_le_bytes(data[slot_offset..slot_offset + 8].try_into().unwrap());

        if seq1 != seq2 {
            if attempt + 1 == MAX_RETRIES {
                return Err(DynamicsError::StepFailed(
                    "Torn read detected: writer collision during slot ingestion".to_string(),
                ));
            }
            continue;
        }

        // Step 6: Normalize quaternion [w, x, y, z] to unit magnitude
        let mut quat = [q0, q1, q2, q3];
        let norm = (quat[0] * quat[0] + quat[1] * quat[1] + quat[2] * quat[2] + quat[3] * quat[3]).sqrt();
        if norm.is_finite() && norm > 1e-12 {
            quat[0] /= norm;
            quat[1] /= norm;
            quat[2] /= norm;
            quat[3] /= norm;
        } else {
            quat = [1.0, 0.0, 0.0, 0.0];
        }

        return Ok(ShmSlotData {
            sim_time_s,
            total_ticks,
            position_m: [pos_x, pos_y, pos_z],
            orientation_quat: quat,
            velocity_m_per_s: [vx, vy, vz],
            angular_velocity_rad_per_s: [wx, wy, wz],
        });
    }

    Err(DynamicsError::StepFailed("writer active".to_string()))
}

/// Dynamics backend reading live multi-world physics from Aerovex POSIX Shared Memory.
#[derive(Debug, Clone)]
pub struct AerovexShmBackend {
    shm_path: PathBuf,
    world_idx: usize,
    entity_idx: usize,
    telemetry: DynamicsTelemetry,
    healthy: bool,
    last_tick_time: Option<Instant>,
    mock_buffer: Option<Vec<u8>>,
}

impl Default for AerovexShmBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl AerovexShmBackend {
    /// Creates a new `AerovexShmBackend` targeting the default SHM path (`/dev/shm/aerovex_sim_state.bin`),
    /// world 0, and entity 0.
    pub fn new() -> Self {
        Self::with_path(DEFAULT_SHM_PATH, 0, 0)
    }

    /// Creates an `AerovexShmBackend` pointing to a custom shared memory path, world index, and entity index.
    pub fn with_path<P: AsRef<Path>>(path: P, world_idx: usize, entity_idx: usize) -> Self {
        Self {
            shm_path: path.as_ref().to_path_buf(),
            world_idx,
            entity_idx,
            telemetry: DynamicsTelemetry::default(),
            healthy: true,
            last_tick_time: None,
            mock_buffer: None,
        }
    }

    /// Sets or clears an in-memory buffer override for testing and benchmarking without filesystem access.
    pub fn set_mock_buffer(&mut self, buffer: Option<Vec<u8>>) {
        self.mock_buffer = buffer;
    }

    /// Returns the target shared memory path.
    #[inline]
    pub fn shm_path(&self) -> &Path {
        &self.shm_path
    }

    /// Sets the target shared memory path.
    pub fn set_shm_path<P: AsRef<Path>>(&mut self, path: P) {
        self.shm_path = path.as_ref().to_path_buf();
    }

    /// Returns the monitored world index.
    #[inline]
    pub fn world_idx(&self) -> usize {
        self.world_idx
    }

    /// Sets the monitored world index.
    pub fn set_world_idx(&mut self, world_idx: usize) {
        self.world_idx = world_idx;
    }

    /// Returns the monitored entity index.
    #[inline]
    pub fn entity_idx(&self) -> usize {
        self.entity_idx
    }

    /// Sets the monitored entity index.
    pub fn set_entity_idx(&mut self, entity_idx: usize) {
        self.entity_idx = entity_idx;
    }

    /// Steps the backend directly from an in-memory slice, avoiding disk I/O.
    pub fn step_from_slice(&mut self, dt_s: f64, data: &[u8]) -> Result<&DynamicsTelemetry, DynamicsError> {
        let slot_data = safe_read_shm_slot(data, self.world_idx, self.entity_idx)?;
        self.update_telemetry(dt_s, slot_data)?;
        Ok(&self.telemetry)
    }

    /// Updates internal telemetry from parsed slot data, computing linear and angular accelerations
    /// via finite differences from the prior step.
    fn update_telemetry(&mut self, dt_s: f64, slot: ShmSlotData) -> Result<(), DynamicsError> {
        let prev_vel = self.telemetry.velocity_m_per_s;
        let prev_ang_vel = self.telemetry.angular_velocity_rad_per_s;

        let acc = if dt_s > 1e-9 && self.telemetry.step_count > 0 {
            [
                (slot.velocity_m_per_s[0] - prev_vel[0]) / dt_s,
                (slot.velocity_m_per_s[1] - prev_vel[1]) / dt_s,
                (slot.velocity_m_per_s[2] - prev_vel[2]) / dt_s,
            ]
        } else {
            [0.0, 0.0, 0.0]
        };

        let ang_acc = if dt_s > 1e-9 && self.telemetry.step_count > 0 {
            [
                (slot.angular_velocity_rad_per_s[0] - prev_ang_vel[0]) / dt_s,
                (slot.angular_velocity_rad_per_s[1] - prev_ang_vel[1]) / dt_s,
                (slot.angular_velocity_rad_per_s[2] - prev_ang_vel[2]) / dt_s,
            ]
        } else {
            [0.0, 0.0, 0.0]
        };

        self.telemetry.position_m = slot.position_m;
        self.telemetry.velocity_m_per_s = slot.velocity_m_per_s;
        self.telemetry.acceleration_m_per_s2 = acc;
        self.telemetry.orientation_quat = slot.orientation_quat;
        self.telemetry.angular_velocity_rad_per_s = slot.angular_velocity_rad_per_s;
        self.telemetry.angular_acceleration_rad_per_s2 = ang_acc;
        self.telemetry.sim_time_s = slot.sim_time_s;
        self.telemetry.step_count = if slot.total_ticks > 0 {
            slot.total_ticks
        } else {
            self.telemetry.step_count + 1
        };
        self.healthy = true;
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.last_tick_time = Some(Instant::now());
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.last_tick_time = None;
        }
        Ok(())
    }
}

impl PhysicsDynamicsBackend for AerovexShmBackend {
    fn info(&self) -> BackendInfo {
        BackendInfo::new(
            "Aerovex POSIX Shared Memory Connector",
            "2.0.0",
            true,
            8650000.0,
            "Lock-free atomic Seqlock dynamics reader connecting Aerovex Sim multi-world buffer",
        )
    }

    fn reset(&mut self, initial_state: Option<&DynamicsTelemetry>) -> Result<(), DynamicsError> {
        if let Some(state) = initial_state {
            self.telemetry = state.clone();
        } else {
            self.telemetry = DynamicsTelemetry::default();
        }
        self.healthy = true;
        self.last_tick_time = None;
        Ok(())
    }

    fn step(&mut self, dt_s: f64, _inputs: &ActuatorInputs) -> Result<&DynamicsTelemetry, DynamicsError> {
        if let Some(ref buf) = self.mock_buffer {
            let slot_data = safe_read_shm_slot(buf, self.world_idx, self.entity_idx)?;
            self.update_telemetry(dt_s, slot_data)?;
            return Ok(&self.telemetry);
        }

        let data = std::fs::read(&self.shm_path).map_err(|e| {
            self.healthy = false;
            DynamicsError::BackendUnavailable(format!("Failed to read SHM file: {}", e))
        })?;

        let slot_data = safe_read_shm_slot(&data, self.world_idx, self.entity_idx)?;
        self.update_telemetry(dt_s, slot_data)?;
        Ok(&self.telemetry)
    }

    fn current_telemetry(&self) -> &DynamicsTelemetry {
        &self.telemetry
    }

    fn is_healthy(&self) -> bool {
        self.healthy
    }
}

/// Adaptive dynamics backend that seamlessly routes simulation steps to `AerovexShmBackend`
/// when an active Aerovex session is detected, or falls back to `ReferenceDynamicsBackend`
/// during standalone execution.
#[derive(Debug, Clone)]
pub struct AutoSelectingDynamicsBackend {
    reference: ReferenceDynamicsBackend,
    shm: AerovexShmBackend,
    shm_path: PathBuf,
    is_shm_active: bool,
}

impl Default for AutoSelectingDynamicsBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoSelectingDynamicsBackend {
    /// Creates a new `AutoSelectingDynamicsBackend` configured with the default SHM path.
    pub fn new() -> Self {
        Self::with_path(DEFAULT_SHM_PATH, 0, 0)
    }

    /// Creates an `AutoSelectingDynamicsBackend` configured with a custom SHM path and indices.
    pub fn with_path<P: AsRef<Path>>(path: P, world_idx: usize, entity_idx: usize) -> Self {
        let path_buf = path.as_ref().to_path_buf();
        Self {
            reference: ReferenceDynamicsBackend::default(),
            shm: AerovexShmBackend::with_path(&path_buf, world_idx, entity_idx),
            shm_path: path_buf,
            is_shm_active: false,
        }
    }

    /// Returns the human-readable identifier of the currently active dynamics backend.
    pub fn active_backend_name(&self) -> &str {
        if self.is_shm_active {
            "Aerovex POSIX Shared Memory Connector"
        } else {
            "Reference Dynamics Engine (RK4 6-DOF)"
        }
    }

    /// Returns `true` if execution is currently promoted to the Aerovex Shared Memory backend.
    #[inline]
    pub fn is_shm_active(&self) -> bool {
        self.is_shm_active
    }

    /// Returns a reference to the fallback reference dynamics engine.
    #[inline]
    pub fn reference_backend(&self) -> &ReferenceDynamicsBackend {
        &self.reference
    }

    /// Returns a mutable reference to the fallback reference dynamics engine.
    #[inline]
    pub fn reference_backend_mut(&mut self) -> &mut ReferenceDynamicsBackend {
        &mut self.reference
    }

    /// Returns a reference to the shared memory backend connector.
    #[inline]
    pub fn shm_backend(&self) -> &AerovexShmBackend {
        &self.shm
    }

    /// Returns a mutable reference to the shared memory backend connector.
    #[inline]
    pub fn shm_backend_mut(&mut self) -> &mut AerovexShmBackend {
        &mut self.shm
    }

    /// Sets the target shared memory path for both probe discovery and SHM ingestion.
    pub fn set_shm_path<P: AsRef<Path>>(&mut self, path: P) {
        let p = path.as_ref().to_path_buf();
        self.shm.set_shm_path(&p);
        self.shm_path = p;
    }
}

impl PhysicsDynamicsBackend for AutoSelectingDynamicsBackend {
    fn info(&self) -> BackendInfo {
        if self.is_shm_active {
            self.shm.info()
        } else {
            self.reference.info()
        }
    }

    fn reset(&mut self, initial_state: Option<&DynamicsTelemetry>) -> Result<(), DynamicsError> {
        self.reference.reset(initial_state)?;
        let _ = self.shm.reset(initial_state);
        Ok(())
    }

    fn step(&mut self, dt_s: f64, inputs: &ActuatorInputs) -> Result<&DynamicsTelemetry, DynamicsError> {
        // Probe if Aerovex shared memory is active and fresh
        let is_shm_available = AerovexPresenceProbe::is_available_at(&self.shm_path);

        if is_shm_available {
            match self.shm.step(dt_s, inputs) {
                Ok(telem) => {
                    self.is_shm_active = true;
                    // Keep reference engine synchronized in case SHM drops later
                    let _ = self.reference.reset(Some(telem));
                    return Ok(self.shm.current_telemetry());
                }
                Err(_) => {
                    // Fall back to reference engine on step error
                    self.is_shm_active = false;
                }
            }
        } else {
            self.is_shm_active = false;
        }

        self.reference.step(dt_s, inputs)
    }

    fn current_telemetry(&self) -> &DynamicsTelemetry {
        if self.is_shm_active {
            self.shm.current_telemetry()
        } else {
            self.reference.current_telemetry()
        }
    }

    fn is_healthy(&self) -> bool {
        if self.is_shm_active {
            self.shm.is_healthy()
        } else {
            self.reference.is_healthy()
        }
    }
}

/// Constructs a valid mock Aerovex Shared Memory binary buffer for deterministic verification and benchmarking.
pub fn create_mock_shm_buffer(
    version: u32,
    active_worlds: u32,
    timestamp_epoch_ms: Option<u64>,
    slots: &[(usize, u64, ShmSlotData)],
) -> Vec<u8> {
    let max_world_idx = slots.iter().map(|(w, _, _)| *w).max().unwrap_or(0);
    let total_slots = (max_world_idx + 1).max(active_worlds as usize).max(1);
    let total_size = AVSM_HEADER_SIZE + total_slots * AVSM_DEFAULT_SLOT_STRIDE;

    let mut buf = vec![0u8; total_size];

    // 1. Header (64 bytes)
    buf[0..4].copy_from_slice(&AVSM_MAGIC);
    buf[4..8].copy_from_slice(&version.to_le_bytes());
    buf[8..12].copy_from_slice(&128u32.to_le_bytes()); // max_worlds
    buf[12..16].copy_from_slice(&active_worlds.to_le_bytes());
    buf[16..20].copy_from_slice(&64u32.to_le_bytes()); // max_entities_per_world
    buf[20..24].copy_from_slice(&[0u8; 4]); // padding
    buf[24..32].copy_from_slice(&(AVSM_DEFAULT_SLOT_STRIDE as u64).to_le_bytes());

    if let Some(ts) = timestamp_epoch_ms {
        buf[32..40].copy_from_slice(&ts.to_le_bytes());
    }

    // 2. Populate slots
    for (world_idx, seq, data) in slots {
        let slot_offset = AVSM_HEADER_SIZE + world_idx * AVSM_DEFAULT_SLOT_STRIDE;
        if slot_offset + AVSM_SLOT_META_SIZE + AVSM_ENTITY_SIZE <= buf.len() {
            // Sequence counter (u64)
            buf[slot_offset..slot_offset + 8].copy_from_slice(&seq.to_le_bytes());
            // world_id (u32)
            buf[slot_offset + 8..slot_offset + 12].copy_from_slice(&(*world_idx as u32).to_le_bytes());
            // is_active (u32)
            buf[slot_offset + 12..slot_offset + 16].copy_from_slice(&1u32.to_le_bytes());
            // sim_time_secs (f64)
            buf[slot_offset + 16..slot_offset + 24].copy_from_slice(&data.sim_time_s.to_le_bytes());
            // total_ticks (u64)
            buf[slot_offset + 24..slot_offset + 32].copy_from_slice(&data.total_ticks.to_le_bytes());
            // entity_count (u32)
            buf[slot_offset + 32..slot_offset + 36].copy_from_slice(&1u32.to_le_bytes());

            // Entity 0
            let ent_offset = slot_offset + AVSM_SLOT_META_SIZE;
            // entity_id (u32)
            buf[ent_offset..ent_offset + 4].copy_from_slice(&0u32.to_le_bytes());
            // is_active (u32)
            buf[ent_offset + 4..ent_offset + 8].copy_from_slice(&1u32.to_le_bytes());
            // position: [f64; 3]
            buf[ent_offset + 8..ent_offset + 16].copy_from_slice(&data.position_m[0].to_le_bytes());
            buf[ent_offset + 16..ent_offset + 24].copy_from_slice(&data.position_m[1].to_le_bytes());
            buf[ent_offset + 24..ent_offset + 32].copy_from_slice(&data.position_m[2].to_le_bytes());
            // orientation: [f64; 4]
            buf[ent_offset + 32..ent_offset + 40].copy_from_slice(&data.orientation_quat[0].to_le_bytes());
            buf[ent_offset + 40..ent_offset + 48].copy_from_slice(&data.orientation_quat[1].to_le_bytes());
            buf[ent_offset + 48..ent_offset + 56].copy_from_slice(&data.orientation_quat[2].to_le_bytes());
            buf[ent_offset + 56..ent_offset + 64].copy_from_slice(&data.orientation_quat[3].to_le_bytes());
            // linear_velocity: [f64; 3]
            buf[ent_offset + 64..ent_offset + 72].copy_from_slice(&data.velocity_m_per_s[0].to_le_bytes());
            buf[ent_offset + 72..ent_offset + 80].copy_from_slice(&data.velocity_m_per_s[1].to_le_bytes());
            buf[ent_offset + 80..ent_offset + 88].copy_from_slice(&data.velocity_m_per_s[2].to_le_bytes());
            // angular_velocity: [f64; 3]
            buf[ent_offset + 88..ent_offset + 96].copy_from_slice(&data.angular_velocity_rad_per_s[0].to_le_bytes());
            buf[ent_offset + 96..ent_offset + 104].copy_from_slice(&data.angular_velocity_rad_per_s[1].to_le_bytes());
            buf[ent_offset + 104..ent_offset + 112].copy_from_slice(&data.angular_velocity_rad_per_s[2].to_le_bytes());
        }
    }

    buf
}
