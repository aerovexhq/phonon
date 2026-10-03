#![deny(unsafe_code)]

//! Distributed RPC protocol and serialization structures for Phonon Cluster Sweep Engine.
//!
//! Provides zero-unsafe binary wire protocols for worker registration, heartbeat telemetry,
//! batch dispatching, and aggregated execution reporting.

use std::collections::HashMap;

/// Operational status of a cluster worker node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeStatus {
    /// Worker is online, responsive, and available for job assignment.
    Online,
    /// Worker is actively computing simulation samples.
    Busy,
    /// Worker is registered and waiting for new work batches.
    Idle,
    /// Worker timed out or dropped connection.
    Offline,
}

impl NodeStatus {
    /// Returns static string representation of node status.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Online => "Online",
            Self::Busy => "Busy",
            Self::Idle => "Idle",
            Self::Offline => "Offline",
        }
    }

    /// Returns true if node can accept new batch assignments.
    pub const fn is_available(&self) -> bool {
        matches!(self, Self::Online | Self::Idle)
    }

    /// Converts status to internal byte discriminant for serialization.
    pub const fn to_discriminant(&self) -> u8 {
        match self {
            Self::Online => 0,
            Self::Busy => 1,
            Self::Idle => 2,
            Self::Offline => 3,
        }
    }

    /// Decodes status from internal byte discriminant.
    pub fn from_discriminant(d: u8) -> Result<Self, ProtocolError> {
        match d {
            0 => Ok(Self::Online),
            1 => Ok(Self::Busy),
            2 => Ok(Self::Idle),
            3 => Ok(Self::Offline),
            other => Err(ProtocolError::InvalidDiscriminant(other)),
        }
    }
}

/// Simulation solver category distributed across cluster workers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimulationType {
    /// Latin Hypercube or Pseudo-Random Monte Carlo parameter harvesting.
    MonteCarlo,
    /// Multi-port S-parameter frequency domain sweeping.
    SParameterSweep,
    /// Multi-corner DC operating point sweep.
    DcCornerSweep,
}

impl SimulationType {
    /// Returns static string representation of simulation category.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::MonteCarlo => "Monte Carlo",
            Self::SParameterSweep => "S-Parameter Sweep",
            Self::DcCornerSweep => "DC Corner Sweep",
        }
    }

    /// Converts simulation type to byte discriminant.
    pub const fn to_discriminant(&self) -> u8 {
        match self {
            Self::MonteCarlo => 0,
            Self::SParameterSweep => 1,
            Self::DcCornerSweep => 2,
        }
    }

    /// Decodes simulation type from byte discriminant.
    pub fn from_discriminant(d: u8) -> Result<Self, ProtocolError> {
        match d {
            0 => Ok(Self::MonteCarlo),
            1 => Ok(Self::SParameterSweep),
            2 => Ok(Self::DcCornerSweep),
            other => Err(ProtocolError::InvalidDiscriminant(other)),
        }
    }
}

/// Individual parameter sweep sample defining parameter override dictionary.
#[derive(Debug, Clone, PartialEq)]
pub struct SweepSample {
    /// Monotonically increasing unique sample sequence index.
    pub sample_id: u64,
    /// Key-value mapping of component parameters overridden for this sample point.
    pub parameter_overrides: HashMap<String, f64>,
}

impl SweepSample {
    /// Creates a new sweep sample with empty overrides.
    pub fn new(sample_id: u64) -> Self {
        Self {
            sample_id,
            parameter_overrides: HashMap::new(),
        }
    }

    /// Creates a new sweep sample with provided overrides map.
    pub fn with_overrides(sample_id: u64, parameter_overrides: HashMap<String, f64>) -> Self {
        Self {
            sample_id,
            parameter_overrides,
        }
    }

    /// Builder helper to insert a parameter override.
    pub fn with_param(mut self, name: impl Into<String>, val: f64) -> Self {
        self.parameter_overrides.insert(name.into(), val);
        self
    }
}

/// Static and live runtime metadata for a registered cluster worker node.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkerNodeInfo {
    /// Unique worker node identifier string.
    pub id: String,
    /// Network address or hostname of worker daemon.
    pub address: String,
    /// Hardware core capacity of worker node.
    pub core_count: usize,
    /// Physical RAM allocated to worker daemon in megabytes.
    pub memory_mb: usize,
    /// Current node operational status.
    pub status: NodeStatus,
    /// Timestamp of most recent received heartbeat in seconds.
    pub last_heartbeat_s: f64,
    /// Cumulative count of completed batch jobs.
    pub jobs_completed: usize,
    /// Active throughput measured in jobs per second.
    pub active_throughput_jobs_sec: f64,
}

impl WorkerNodeInfo {
    /// Creates a new worker node info record in Idle status.
    pub fn new(
        id: impl Into<String>,
        address: impl Into<String>,
        core_count: usize,
        memory_mb: usize,
    ) -> Self {
        Self {
            id: id.into(),
            address: address.into(),
            core_count,
            memory_mb,
            status: NodeStatus::Idle,
            last_heartbeat_s: 0.0,
            jobs_completed: 0,
            active_throughput_jobs_sec: 0.0,
        }
    }
}

/// Remote Procedure Call (RPC) messages for distributed cluster orchestration.
#[derive(Debug, Clone, PartialEq)]
pub enum RpcMessage {
    /// Register a newly joined worker node with the central coordinator.
    RegisterNode {
        /// Metadata describing the worker node.
        node: WorkerNodeInfo,
    },
    /// Periodic heartbeat telemetry update emitted by active workers.
    Heartbeat {
        /// Unique node identifier.
        node_id: String,
        /// Current worker local clock timestamp in seconds.
        timestamp_s: f64,
        /// Current worker CPU utilization percentage (0.0 to 100.0).
        cpu_usage_pct: f64,
        /// Current worker resident memory consumption in megabytes.
        memory_usage_mb: f64,
    },
    /// Work dispatch instruction sending a chunk of parameter samples to a worker.
    DispatchBatch {
        /// Unique job identifier string.
        job_id: String,
        /// Batch sequence number within the job.
        batch_id: u32,
        /// Collection of parameter space samples to simulate.
        samples: Vec<SweepSample>,
        /// Category of simulation solver to execute.
        sim_type: SimulationType,
    },
    /// Execution completion report returned by worker upon batch finish.
    BatchResult {
        /// Unique job identifier string matching dispatch.
        job_id: String,
        /// Batch sequence number.
        batch_id: u32,
        /// Worker node identifier that executed the batch.
        node_id: String,
        /// Measured compute wall time in milliseconds.
        execution_time_ms: f64,
        /// Output scalar values harvested for each sample in the batch.
        sample_values: Vec<f64>,
        /// Count of failed evaluations within the batch.
        errors: u32,
    },
    /// Worker graceful teardown / deregistration notice.
    DeregisterNode {
        /// Unique node identifier to deregister.
        node_id: String,
    },
}

/// Errors occurring during RPC message serialization and deserialization.
#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum ProtocolError {
    /// Stream does not begin with expected 8-byte magic header.
    #[error("invalid magic header bytes")]
    InvalidMagic,
    /// Wire protocol version is unsupported.
    #[error("unsupported protocol version: {0}")]
    UnsupportedVersion(u16),
    /// Reached unexpected end of buffer while decoding payload.
    #[error("unexpected end of buffer during deserialization")]
    UnexpectedEof,
    /// Invalid discriminant value for an enum type.
    #[error("unknown message discriminant: {0}")]
    InvalidDiscriminant(u8),
    /// String bytes failed UTF-8 validation.
    #[error("invalid utf-8 string payload")]
    InvalidUtf8,
    /// Value parsing error.
    #[error("invalid protocol data: {0}")]
    InvalidData(String),
}

/// Magic byte sequence identifying Phonon Cluster RPC wire frames: `b"PHCLUST\x01"`.
pub const PROTOCOL_MAGIC: [u8; 8] = *b"PHCLUST\x01";

/// Current cluster RPC protocol version.
pub const PROTOCOL_VERSION: u16 = 1;

// Internal zero-unsafe serialization primitives

fn encode_u8(buf: &mut Vec<u8>, val: u8) {
    buf.push(val);
}

fn encode_u16(buf: &mut Vec<u8>, val: u16) {
    buf.extend_from_slice(&val.to_le_bytes());
}

fn encode_u32(buf: &mut Vec<u8>, val: u32) {
    buf.extend_from_slice(&val.to_le_bytes());
}

fn encode_u64(buf: &mut Vec<u8>, val: u64) {
    buf.extend_from_slice(&val.to_le_bytes());
}

fn encode_f64(buf: &mut Vec<u8>, val: f64) {
    buf.extend_from_slice(&val.to_le_bytes());
}

fn encode_string(buf: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    encode_u32(buf, bytes.len() as u32);
    buf.extend_from_slice(bytes);
}

fn encode_node_info(buf: &mut Vec<u8>, node: &WorkerNodeInfo) {
    encode_string(buf, &node.id);
    encode_string(buf, &node.address);
    encode_u64(buf, node.core_count as u64);
    encode_u64(buf, node.memory_mb as u64);
    encode_u8(buf, node.status.to_discriminant());
    encode_f64(buf, node.last_heartbeat_s);
    encode_u64(buf, node.jobs_completed as u64);
    encode_f64(buf, node.active_throughput_jobs_sec);
}

fn encode_sweep_sample(buf: &mut Vec<u8>, sample: &SweepSample) {
    encode_u64(buf, sample.sample_id);
    encode_u32(buf, sample.parameter_overrides.len() as u32);
    for (key, val) in &sample.parameter_overrides {
        encode_string(buf, key);
        encode_f64(buf, *val);
    }
}

// Internal zero-unsafe deserialization primitives

struct ByteReader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> ByteReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], ProtocolError> {
        if self.offset + len > self.data.len() {
            return Err(ProtocolError::UnexpectedEof);
        }
        let slice = &self.data[self.offset..self.offset + len];
        self.offset += len;
        Ok(slice)
    }

    fn read_u8(&mut self) -> Result<u8, ProtocolError> {
        let slice = self.read_bytes(1)?;
        Ok(slice[0])
    }

    fn read_u16(&mut self) -> Result<u16, ProtocolError> {
        let slice = self.read_bytes(2)?;
        Ok(u16::from_le_bytes([slice[0], slice[1]]))
    }

    fn read_u32(&mut self) -> Result<u32, ProtocolError> {
        let slice = self.read_bytes(4)?;
        Ok(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
    }

    fn read_u64(&mut self) -> Result<u64, ProtocolError> {
        let slice = self.read_bytes(8)?;
        Ok(u64::from_le_bytes([
            slice[0], slice[1], slice[2], slice[3], slice[4], slice[5], slice[6], slice[7],
        ]))
    }

    fn read_f64(&mut self) -> Result<f64, ProtocolError> {
        let slice = self.read_bytes(8)?;
        Ok(f64::from_le_bytes([
            slice[0], slice[1], slice[2], slice[3], slice[4], slice[5], slice[6], slice[7],
        ]))
    }

    fn read_string(&mut self) -> Result<String, ProtocolError> {
        let len = self.read_u32()? as usize;
        let bytes = self.read_bytes(len)?;
        std::str::from_utf8(bytes)
            .map(|s| s.to_string())
            .map_err(|_| ProtocolError::InvalidUtf8)
    }

    fn read_node_info(&mut self) -> Result<WorkerNodeInfo, ProtocolError> {
        let id = self.read_string()?;
        let address = self.read_string()?;
        let core_count = self.read_u64()? as usize;
        let memory_mb = self.read_u64()? as usize;
        let status = NodeStatus::from_discriminant(self.read_u8()?)?;
        let last_heartbeat_s = self.read_f64()?;
        let jobs_completed = self.read_u64()? as usize;
        let active_throughput_jobs_sec = self.read_f64()?;
        Ok(WorkerNodeInfo {
            id,
            address,
            core_count,
            memory_mb,
            status,
            last_heartbeat_s,
            jobs_completed,
            active_throughput_jobs_sec,
        })
    }

    fn read_sweep_sample(&mut self) -> Result<SweepSample, ProtocolError> {
        let sample_id = self.read_u64()?;
        let map_len = self.read_u32()? as usize;
        let mut parameter_overrides = HashMap::with_capacity(map_len);
        for _ in 0..map_len {
            let key = self.read_string()?;
            let val = self.read_f64()?;
            parameter_overrides.insert(key, val);
        }
        Ok(SweepSample {
            sample_id,
            parameter_overrides,
        })
    }
}

impl RpcMessage {
    /// Serializes an RPC message into a compact binary byte vector with header validation.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(128);
        buf.extend_from_slice(&PROTOCOL_MAGIC);
        encode_u16(&mut buf, PROTOCOL_VERSION);

        match self {
            Self::RegisterNode { node } => {
                encode_u8(&mut buf, 0);
                encode_node_info(&mut buf, node);
            }
            Self::Heartbeat {
                node_id,
                timestamp_s,
                cpu_usage_pct,
                memory_usage_mb,
            } => {
                encode_u8(&mut buf, 1);
                encode_string(&mut buf, node_id);
                encode_f64(&mut buf, *timestamp_s);
                encode_f64(&mut buf, *cpu_usage_pct);
                encode_f64(&mut buf, *memory_usage_mb);
            }
            Self::DispatchBatch {
                job_id,
                batch_id,
                samples,
                sim_type,
            } => {
                encode_u8(&mut buf, 2);
                encode_string(&mut buf, job_id);
                encode_u32(&mut buf, *batch_id);
                encode_u32(&mut buf, samples.len() as u32);
                for sample in samples {
                    encode_sweep_sample(&mut buf, sample);
                }
                encode_u8(&mut buf, sim_type.to_discriminant());
            }
            Self::BatchResult {
                job_id,
                batch_id,
                node_id,
                execution_time_ms,
                sample_values,
                errors,
            } => {
                encode_u8(&mut buf, 3);
                encode_string(&mut buf, job_id);
                encode_u32(&mut buf, *batch_id);
                encode_string(&mut buf, node_id);
                encode_f64(&mut buf, *execution_time_ms);
                encode_u32(&mut buf, sample_values.len() as u32);
                for v in sample_values {
                    encode_f64(&mut buf, *v);
                }
                encode_u32(&mut buf, *errors);
            }
            Self::DeregisterNode { node_id } => {
                encode_u8(&mut buf, 4);
                encode_string(&mut buf, node_id);
            }
        }

        buf
    }

    /// Deserializes an RPC message from a byte slice, validating magic header and version.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let mut reader = ByteReader::new(bytes);
        let magic = reader.read_bytes(8)?;
        if magic != PROTOCOL_MAGIC {
            return Err(ProtocolError::InvalidMagic);
        }
        let version = reader.read_u16()?;
        if version != PROTOCOL_VERSION {
            return Err(ProtocolError::UnsupportedVersion(version));
        }

        let tag = reader.read_u8()?;
        match tag {
            0 => {
                let node = reader.read_node_info()?;
                Ok(Self::RegisterNode { node })
            }
            1 => {
                let node_id = reader.read_string()?;
                let timestamp_s = reader.read_f64()?;
                let cpu_usage_pct = reader.read_f64()?;
                let memory_usage_mb = reader.read_f64()?;
                Ok(Self::Heartbeat {
                    node_id,
                    timestamp_s,
                    cpu_usage_pct,
                    memory_usage_mb,
                })
            }
            2 => {
                let job_id = reader.read_string()?;
                let batch_id = reader.read_u32()?;
                let sample_count = reader.read_u32()? as usize;
                let mut samples = Vec::with_capacity(sample_count);
                for _ in 0..sample_count {
                    samples.push(reader.read_sweep_sample()?);
                }
                let sim_type = SimulationType::from_discriminant(reader.read_u8()?)?;
                Ok(Self::DispatchBatch {
                    job_id,
                    batch_id,
                    samples,
                    sim_type,
                })
            }
            3 => {
                let job_id = reader.read_string()?;
                let batch_id = reader.read_u32()?;
                let node_id = reader.read_string()?;
                let execution_time_ms = reader.read_f64()?;
                let val_count = reader.read_u32()? as usize;
                let mut sample_values = Vec::with_capacity(val_count);
                for _ in 0..val_count {
                    sample_values.push(reader.read_f64()?);
                }
                let errors = reader.read_u32()?;
                Ok(Self::BatchResult {
                    job_id,
                    batch_id,
                    node_id,
                    execution_time_ms,
                    sample_values,
                    errors,
                })
            }
            4 => {
                let node_id = reader.read_string()?;
                Ok(Self::DeregisterNode { node_id })
            }
            other => Err(ProtocolError::InvalidDiscriminant(other)),
        }
    }
}
