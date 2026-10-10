#![deny(unsafe_code)]

//! Real-Time Remote Cursor Presence, Selection Halos and Edit Conflict Detection.
//!
//! Tracks ephemeral peer state including canvas cursor coordinates, active CAD tool,
//! selected component halos, and detects concurrent edit conflicts on shared items.

use std::collections::HashMap;

/// Ephemeral collaborative presence for an individual mesh peer.
#[derive(Debug, Clone, PartialEq)]
pub struct PeerPresence {
    pub peer_id: String,
    pub display_name: String,
    pub cursor_x: f64,
    pub cursor_y: f64,
    pub viewport_pan_x: f64,
    pub viewport_pan_y: f64,
    pub viewport_zoom: f64,
    pub active_tool: String,
    pub selected_component_ids: Vec<String>,
    pub selected_wire_ids: Vec<String>,
    pub color_rgb: [u8; 3],
    pub last_active_ns: u64,
}

impl PeerPresence {
    pub fn new(peer_id: &str, display_name: &str, color: [u8; 3]) -> Self {
        Self {
            peer_id: peer_id.to_string(),
            display_name: display_name.to_string(),
            cursor_x: 0.0,
            cursor_y: 0.0,
            viewport_pan_x: 0.0,
            viewport_pan_y: 0.0,
            viewport_zoom: 1.0,
            active_tool: "Select".to_string(),
            selected_component_ids: Vec::new(),
            selected_wire_ids: Vec::new(),
            color_rgb: color,
            last_active_ns: 1_000_000_000,
        }
    }
}

/// A detected concurrent edit conflict between two or more peers.
#[derive(Debug, Clone, PartialEq)]
pub struct PresenceConflict {
    pub entity_id: String,
    pub entity_kind: &'static str,
    pub competing_peer_ids: Vec<String>,
    pub timestamp_ns: u64,
}

/// Master presence and remote cursor tracking coordinator.
#[derive(Debug, Clone, PartialEq)]
pub struct PresenceManager {
    pub local_presence: PeerPresence,
    pub remote_presences: HashMap<String, PeerPresence>,
    pub detected_conflicts: Vec<PresenceConflict>,
}

impl PresenceManager {
    pub fn new(local_peer_id: &str, local_display_name: &str) -> Self {
        Self {
            local_presence: PeerPresence::new(
                local_peer_id,
                local_display_name,
                [59, 130, 246], // Azure Blue for local peer
            ),
            remote_presences: HashMap::new(),
            detected_conflicts: Vec::new(),
        }
    }

    /// Pre-seeds baseline remote presence for default demo peers.
    pub fn default_session() -> Self {
        let mut mgr = Self::new("peer-alice-01", "Alice (Analog Lead)");

        let mut bob = PeerPresence::new("peer-bob-02", "Bob (RF Engineer)", [249, 115, 22]); // Coral Orange
        bob.cursor_x = 110.0;
        bob.cursor_y = 105.0;
        bob.active_tool = "Wire".to_string();
        bob.selected_component_ids = vec!["cmp-r1".to_string()];
        mgr.update_remote_presence(bob);

        let mut charlie = PeerPresence::new("peer-charlie-03", "Charlie (Layout Lead)", [16, 185, 129]); // Emerald Green
        charlie.cursor_x = 225.0;
        charlie.cursor_y = 115.0;
        charlie.active_tool = "Select".to_string();
        charlie.selected_component_ids = vec!["cmp-c1".to_string()];
        mgr.update_remote_presence(charlie);

        mgr.detect_conflicts();
        mgr
    }

    pub fn update_local_cursor(&mut self, x: f64, y: f64, tool: &str) {
        self.local_presence.cursor_x = x;
        self.local_presence.cursor_y = y;
        self.local_presence.active_tool = tool.to_string();
        self.detect_conflicts();
    }

    pub fn update_local_selection(&mut self, comps: Vec<String>, wires: Vec<String>) {
        self.local_presence.selected_component_ids = comps;
        self.local_presence.selected_wire_ids = wires;
        self.detect_conflicts();
    }

    pub fn update_remote_presence(&mut self, presence: PeerPresence) {
        self.remote_presences.insert(presence.peer_id.clone(), presence);
        self.detect_conflicts();
    }

    pub fn remove_remote_presence(&mut self, peer_id: &str) {
        self.remote_presences.remove(peer_id);
        self.detect_conflicts();
    }

    /// Scans for shared selections or simultaneous interactions on identical entities.
    pub fn detect_conflicts(&mut self) {
        self.detected_conflicts.clear();

        // 1. Check components
        let mut comp_map: HashMap<String, Vec<String>> = HashMap::new();
        for comp_id in &self.local_presence.selected_component_ids {
            comp_map.entry(comp_id.clone()).or_default().push(self.local_presence.peer_id.clone());
        }
        for (peer_id, remote) in &self.remote_presences {
            for comp_id in &remote.selected_component_ids {
                comp_map.entry(comp_id.clone()).or_default().push(peer_id.clone());
            }
        }
        for (comp_id, peers) in comp_map {
            if peers.len() > 1 {
                self.detected_conflicts.push(PresenceConflict {
                    entity_id: comp_id,
                    entity_kind: "Component",
                    competing_peer_ids: peers,
                    timestamp_ns: 1_000_000_000,
                });
            }
        }

        // 2. Check wires
        let mut wire_map: HashMap<String, Vec<String>> = HashMap::new();
        for wire_id in &self.local_presence.selected_wire_ids {
            wire_map.entry(wire_id.clone()).or_default().push(self.local_presence.peer_id.clone());
        }
        for (peer_id, remote) in &self.remote_presences {
            for wire_id in &remote.selected_wire_ids {
                wire_map.entry(wire_id.clone()).or_default().push(peer_id.clone());
            }
        }
        for (wire_id, peers) in wire_map {
            if peers.len() > 1 {
                self.detected_conflicts.push(PresenceConflict {
                    entity_id: wire_id,
                    entity_kind: "Wire",
                    competing_peer_ids: peers,
                    timestamp_ns: 1_000_000_000,
                });
            }
        }
    }

    /// Returns the color of the peer who has selected a given component ID, if any.
    pub fn component_selection_halo(&self, comp_id: &str) -> Option<[u8; 3]> {
        if self.local_presence.selected_component_ids.iter().any(|c| c == comp_id) {
            return Some(self.local_presence.color_rgb);
        }
        for remote in self.remote_presences.values() {
            if remote.selected_component_ids.iter().any(|c| c == comp_id) {
                return Some(remote.color_rgb);
            }
        }
        None
    }
}
