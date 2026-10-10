#![deny(unsafe_code)]

//! End-to-End Cryptographic Message Authentication and Anti-Replay Engine.
//!
//! Provides zero server-side storage message integrity and sender authentication
//! using keyed 128-bit MAC signatures, monotonic sequence counters, and replay filters.

use std::collections::HashMap;

/// Errors occurring during cryptographic verification of collaboration envelopes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CryptoAuthError {
    InvalidMac,
    TamperedPayload,
    ReplayAttackDetected {
        expected_greater_than: u64,
        received: u64,
    },
    UntrustedSender(String),
    RoomMismatch {
        expected: String,
        received: String,
    },
}

impl std::fmt::Display for CryptoAuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidMac => write!(f, "Cryptographic MAC authentication failure"),
            Self::TamperedPayload => write!(f, "Payload integrity check failed: payload was modified in transit"),
            Self::ReplayAttackDetected { expected_greater_than, received } => {
                write!(f, "Replay attack detected: sequence {} <= threshold {}", received, expected_greater_than)
            }
            Self::UntrustedSender(s) => write!(f, "Untrusted sender identity: {}", s),
            Self::RoomMismatch { expected, received } => {
                write!(f, "Room identifier mismatch: expected {}, received {}", expected, received)
            }
        }
    }
}

impl std::error::Error for CryptoAuthError {}

/// Cryptographically signed envelope carrying CRDT deltas over untrusted WebRTC channels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedCrdtEnvelope {
    pub sequence_number: u64,
    pub sender_id: String,
    pub room_id: String,
    pub timestamp_ns: u64,
    pub payload_bytes: Vec<u8>,
    pub auth_mac: [u8; 16],
}

impl SignedCrdtEnvelope {
    pub fn new(
        sequence: u64,
        sender_id: &str,
        room_id: &str,
        timestamp_ns: u64,
        payload: Vec<u8>,
        psk: &[u8; 32],
    ) -> Self {
        let mut envelope = Self {
            sequence_number: sequence,
            sender_id: sender_id.to_string(),
            room_id: room_id.to_string(),
            timestamp_ns,
            payload_bytes: payload,
            auth_mac: [0u8; 16],
        };
        envelope.auth_mac = envelope.calculate_mac(psk);
        envelope
    }

    /// Calculates a 128-bit keyed message authentication code over envelope fields.
    pub fn calculate_mac(&self, psk: &[u8; 32]) -> [u8; 16] {
        // Multi-round keyed sponge-style mixing in pure safe Rust
        let mut h0: u64 = 0x243F6A8885A308D3 ^ u64::from_le_bytes(psk[0..8].try_into().unwrap());
        let mut h1: u64 = 0x13198A2E03707344 ^ u64::from_le_bytes(psk[8..16].try_into().unwrap());
        let mut h2: u64 = 0xA4093822299F31D0 ^ u64::from_le_bytes(psk[16..24].try_into().unwrap());
        let mut h3: u64 = 0x082EFA98EC4E6C89 ^ u64::from_le_bytes(psk[24..32].try_into().unwrap());

        // Ingest sequence and timestamp
        h0 = h0.wrapping_add(self.sequence_number).rotate_left(13);
        h1 = h1.wrapping_add(self.timestamp_ns).rotate_left(17);

        // Ingest sender and room strings
        for b in self.sender_id.as_bytes() {
            h2 = (h2 ^ (*b as u64)).wrapping_mul(0x100000001B3);
        }
        for b in self.room_id.as_bytes() {
            h3 = (h3 ^ (*b as u64)).wrapping_mul(0x100000001B3);
        }

        // Ingest payload bytes
        for (i, b) in self.payload_bytes.iter().enumerate() {
            match i % 4 {
                0 => h0 = (h0 ^ (*b as u64)).rotate_left(7).wrapping_mul(0xBF58476D1CE4E5B9),
                1 => h1 = (h1 ^ (*b as u64)).rotate_left(11).wrapping_mul(0x94D049BB133111EB),
                2 => h2 = (h2 ^ (*b as u64)).rotate_left(19).wrapping_mul(0xBF58476D1CE4E5B9),
                _ => h3 = (h3 ^ (*b as u64)).rotate_left(23).wrapping_mul(0x94D049BB133111EB),
            }
        }

        // Final round diffusion
        let out0 = h0 ^ h2;
        let out1 = h1 ^ h3;
        let mut mac = [0u8; 16];
        mac[0..8].copy_from_slice(&out0.to_le_bytes());
        mac[8..16].copy_from_slice(&out1.to_le_bytes());
        mac
    }
}

/// Cryptographic authenticator and anti-replay manager.
#[derive(Debug, Clone, PartialEq)]
pub struct CryptoAuthEngine {
    pub room_id: String,
    pub room_psk: [u8; 32],
    pub local_sequence: u64,
    pub peer_highest_sequence: HashMap<String, u64>,
    pub authenticated_messages_count: u64,
    pub rejected_tampered_count: u64,
}

impl CryptoAuthEngine {
    pub fn new(room_id: &str, psk: [u8; 32]) -> Self {
        Self {
            room_id: room_id.to_string(),
            room_psk: psk,
            local_sequence: 0,
            peer_highest_sequence: HashMap::new(),
            authenticated_messages_count: 0,
            rejected_tampered_count: 0,
        }
    }

    /// Pre-seeds a default cryptographic room key for demo sessions.
    pub fn default_session(room_id: &str) -> Self {
        let mut psk = [0u8; 32];
        for (i, b) in b"PHONON-ZERO-CLOUD-CRDT-PSK-2026!".iter().enumerate().take(32) {
            psk[i] = *b;
        }
        Self::new(room_id, psk)
    }

    /// Signs an outgoing delta payload into a tamper-proof envelope.
    pub fn sign_delta(
        &mut self,
        sender_id: &str,
        timestamp_ns: u64,
        payload: Vec<u8>,
    ) -> SignedCrdtEnvelope {
        self.local_sequence += 1;
        SignedCrdtEnvelope::new(
            self.local_sequence,
            sender_id,
            &self.room_id,
            timestamp_ns,
            payload,
            &self.room_psk,
        )
    }

    /// Verifies incoming envelope MAC signature and monotonic sequence number.
    pub fn verify_envelope(&mut self, env: &SignedCrdtEnvelope) -> Result<(), CryptoAuthError> {
        // 1. Verify room matching
        if env.room_id != self.room_id {
            return Err(CryptoAuthError::RoomMismatch {
                expected: self.room_id.clone(),
                received: env.room_id.clone(),
            });
        }

        // 2. Anti-replay verification
        let highest = self.peer_highest_sequence.entry(env.sender_id.clone()).or_insert(0);
        if env.sequence_number <= *highest {
            self.rejected_tampered_count += 1;
            return Err(CryptoAuthError::ReplayAttackDetected {
                expected_greater_than: *highest,
                received: env.sequence_number,
            });
        }

        // 3. Cryptographic MAC check (constant-time comparison)
        let expected_mac = env.calculate_mac(&self.room_psk);
        let mut diff: u8 = 0;
        for i in 0..16 {
            diff |= env.auth_mac[i] ^ expected_mac[i];
        }

        if diff != 0 {
            self.rejected_tampered_count += 1;
            return Err(CryptoAuthError::TamperedPayload);
        }

        // 4. Update highest seen sequence
        *highest = env.sequence_number;
        self.authenticated_messages_count += 1;
        Ok(())
    }
}
