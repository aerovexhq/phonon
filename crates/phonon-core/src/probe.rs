#![deny(unsafe_code)]

//! Autonomous sub-millisecond presence discovery and health handshake for the Aerovex simulation kernel.
//!
//! This module inspects the standard POSIX shared memory ring buffer (`/dev/shm/aerovex_sim_state.bin`)
//! to detect running multi-world Aerovex simulation sessions without blocking execution threads.

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Default POSIX shared memory path utilized by Aerovex simulation sessions.
pub const DEFAULT_SHM_PATH: &str = "/dev/shm/aerovex_sim_state.bin";

/// 4-byte magic signature designating an Aerovex Shared Memory buffer ("AVSM").
pub const AVSM_MAGIC: [u8; 4] = *b"AVSM";

/// Maximum permissible age of the simulation heartbeat before declaring the session stale (1.5 seconds).
pub const MAX_HEARTBEAT_AGE_MS: f64 = 1500.0;

/// Health and discovery status returned by the presence probe.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ProbeStatus {
    /// Active simulation daemon verified with fresh heartbeat.
    Available {
        /// Number of parallel simulation worlds actively executing.
        active_worlds: u32,
        /// Binary layout format version of the shared memory buffer.
        version: u32,
        /// Latency taken to complete the presence probe in microseconds.
        latency_us: f64,
        /// Heartbeat age in milliseconds since last daemon update.
        heartbeat_age_ms: f64,
    },
    /// Shared memory buffer does not exist or daemon is not running.
    NotRunning,
    /// Shared memory exists but last update exceeds the maximum allowed age threshold.
    StaleHeartbeat {
        /// Elapsed time in milliseconds since the last heartbeat timestamp.
        age_ms: f64,
    },
    /// Shared memory exists but header magic or binary layout is invalid.
    InvalidFormat(String),
}

/// Lightweight, non-blocking presence probe for detecting active Aerovex simulation kernels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AerovexPresenceProbe;

impl AerovexPresenceProbe {
    /// Probes the default POSIX shared memory path (`/dev/shm/aerovex_sim_state.bin`).
    #[inline]
    pub fn probe() -> ProbeStatus {
        Self::probe_path(DEFAULT_SHM_PATH)
    }

    /// Probes an arbitrary filesystem path for an active Aerovex shared memory buffer.
    ///
    /// Executes non-blocking read-only verification:
    /// 1. Queries file metadata to ensure the file exists and is large enough for the AVSM header (64 bytes).
    /// 2. Reads the initial 64-byte header to validate the `AVSM` magic signature and layout version.
    /// 3. Computes the heartbeat age from the header timestamp or file modification time.
    /// 4. Completes within sub-millisecond execution latency (< 1.0 ms, typically < 20 us).
    pub fn probe_path<P: AsRef<Path>>(path: P) -> ProbeStatus {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = path;
            return ProbeStatus::NotRunning;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let start = Instant::now();
            let path_ref = path.as_ref();

        // 1. Verify existence and read metadata
        let metadata = match std::fs::metadata(path_ref) {
            Ok(m) => m,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    return ProbeStatus::NotRunning;
                }
                return ProbeStatus::InvalidFormat(format!("Failed to query file metadata: {}", e));
            }
        };

        if metadata.len() < 64 {
            return ProbeStatus::InvalidFormat(format!(
                "File size ({} bytes) is smaller than minimum AVSM header size (64 bytes)",
                metadata.len()
            ));
        }

        // 2. Open file read-only and read header
        let mut file = match File::open(path_ref) {
            Ok(f) => f,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    return ProbeStatus::NotRunning;
                }
                return ProbeStatus::InvalidFormat(format!("Failed to open SHM file: {}", e));
            }
        };

        let mut header_buf = [0u8; 64];
        if let Err(e) = file.read_exact(&mut header_buf) {
            return ProbeStatus::InvalidFormat(format!("Failed to read AVSM header: {}", e));
        }

        // 3. Validate magic signature
        if header_buf[0..4] != AVSM_MAGIC {
            return ProbeStatus::InvalidFormat(format!(
                "Invalid magic header: expected {:?}, found {:?}",
                AVSM_MAGIC,
                &header_buf[0..4]
            ));
        }

        let version = u32::from_le_bytes(header_buf[4..8].try_into().unwrap());
        let active_worlds = u32::from_le_bytes(header_buf[12..16].try_into().unwrap());

        // 4. Calculate heartbeat freshness
        let header_u64 = u64::from_le_bytes(header_buf[32..40].try_into().unwrap());
        let header_f64 = f64::from_le_bytes(header_buf[32..40].try_into().unwrap());

        let now_system = SystemTime::now();
        let now_epoch_ms = now_system
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as f64)
            .unwrap_or(0.0);
        let now_epoch_s = now_epoch_ms / 1000.0;

        let age_ms = if header_f64 > 1.0e9 && header_f64 < 1.0e11 {
            // Epoch seconds represented as 64-bit float
            (now_epoch_s - header_f64).max(0.0) * 1000.0
        } else if header_u64 > 1_000_000_000_000 {
            // Epoch milliseconds represented as 64-bit integer
            (now_epoch_ms - header_u64 as f64).max(0.0)
        } else if header_u64 > 1_000_000_000 && header_u64 < 100_000_000_000 {
            // Epoch seconds represented as 64-bit integer
            (now_epoch_s - header_u64 as f64).max(0.0) * 1000.0
        } else {
            // Fall back to filesystem modification time
            match metadata.modified() {
                Ok(mod_time) => match now_system.duration_since(mod_time) {
                    Ok(d) => d.as_secs_f64() * 1000.0,
                    Err(_) => 0.0,
                },
                Err(_) => 0.0,
            }
        };

        if age_ms > MAX_HEARTBEAT_AGE_MS {
            return ProbeStatus::StaleHeartbeat { age_ms };
        }

        let latency_us = start.elapsed().as_nanos() as f64 / 1000.0;
        ProbeStatus::Available {
            active_worlds,
            version,
            latency_us,
            heartbeat_age_ms: age_ms,
        }
        }
    }

    /// Returns `true` if an active Aerovex simulation session is reachable at the default SHM path.
    #[inline]
    pub fn is_available() -> bool {
        matches!(Self::probe(), ProbeStatus::Available { .. })
    }

    /// Returns `true` if an active Aerovex simulation session is reachable at the specified path.
    #[inline]
    pub fn is_available_at<P: AsRef<Path>>(path: P) -> bool {
        matches!(Self::probe_path(path), ProbeStatus::Available { .. })
    }
}
