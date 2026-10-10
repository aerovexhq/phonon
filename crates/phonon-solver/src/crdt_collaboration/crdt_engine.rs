#![deny(unsafe_code)]

//! Conflict-Free Replicated Data Types (CRDTs) and Delta Synchronization Engine.
//!
//! Provides mathematically proven commutative, associative, and idempotent
//! state-based and operation-based delta synchronization for schematic CAD components,
//! inter-pin wires, netlist labels, and multi-conductor buses.

use std::cmp::Ordering;
use std::collections::HashMap;

/// Lamport logical timestamp with deterministic peer ID tie-breaking for strict total ordering.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LamportTimestamp {
    pub counter: u64,
    pub peer_id: String,
}

impl LamportTimestamp {
    pub fn new(counter: u64, peer_id: &str) -> Self {
        Self {
            counter,
            peer_id: peer_id.to_string(),
        }
    }
}

impl PartialOrd for LamportTimestamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LamportTimestamp {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.counter.cmp(&other.counter) {
            Ordering::Equal => self.peer_id.cmp(&other.peer_id),
            other_ord => other_ord,
        }
    }
}

/// Vector clock tracking causal dependencies across all mesh participants.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VectorClock {
    pub clocks: HashMap<String, u64>,
}

impl VectorClock {
    pub fn new() -> Self {
        Self {
            clocks: HashMap::new(),
        }
    }

    pub fn get(&self, peer_id: &str) -> u64 {
        self.clocks.get(peer_id).copied().unwrap_or(0)
    }

    pub fn increment(&mut self, peer_id: &str) -> u64 {
        let val = self.clocks.entry(peer_id.to_string()).or_insert(0);
        *val += 1;
        *val
    }

    pub fn merge(&mut self, other: &VectorClock) {
        for (peer, &val) in &other.clocks {
            let current = self.clocks.entry(peer.clone()).or_insert(0);
            if val > *current {
                *current = val;
            }
        }
    }

    /// Returns true if self causally dominates other (self >= other on all keys, and strictly > on at least one).
    pub fn dominates(&self, other: &VectorClock) -> bool {
        let mut strictly_greater = false;
        for (peer, &other_val) in &other.clocks {
            let self_val = self.get(peer);
            if self_val < other_val {
                return false;
            }
            if self_val > other_val {
                strictly_greater = true;
            }
        }
        for (peer, &self_val) in &self.clocks {
            if !other.clocks.contains_key(peer) && self_val > 0 {
                strictly_greater = true;
            }
        }
        strictly_greater
    }

    /// Returns true if two vector clocks are concurrent (neither dominates the other).
    pub fn is_concurrent(&self, other: &VectorClock) -> bool {
        !self.dominates(other) && !other.dominates(self) && self != other
    }
}

/// Replicated schematic component state in the CRDT document.
#[derive(Debug, Clone, PartialEq)]
pub struct CrdtComponent {
    pub id: String,
    pub designator: String,
    pub component_type: String,
    pub value: String,
    pub footprint: String,
    pub pos_x: f64,
    pub pos_y: f64,
    pub rotation_deg: f64,
    pub mirrored: bool,
    pub lamport: LamportTimestamp,
    pub tombstoned: bool,
}

impl CrdtComponent {
    pub fn new(
        id: &str,
        designator: &str,
        comp_type: &str,
        val: &str,
        footprint: &str,
        x: f64,
        y: f64,
        lamport: LamportTimestamp,
    ) -> Self {
        Self {
            id: id.to_string(),
            designator: designator.to_string(),
            component_type: comp_type.to_string(),
            value: val.to_string(),
            footprint: footprint.to_string(),
            pos_x: x,
            pos_y: y,
            rotation_deg: 0.0,
            mirrored: false,
            lamport,
            tombstoned: false,
        }
    }
}

/// Replicated schematic wire segment connecting two coordinates or pins.
#[derive(Debug, Clone, PartialEq)]
pub struct CrdtWire {
    pub id: String,
    pub start_x: f64,
    pub start_y: f64,
    pub end_x: f64,
    pub end_y: f64,
    pub net_name: String,
    pub lamport: LamportTimestamp,
    pub tombstoned: bool,
}

impl CrdtWire {
    pub fn new(
        id: &str,
        sx: f64,
        sy: f64,
        ex: f64,
        ey: f64,
        net: &str,
        lamport: LamportTimestamp,
    ) -> Self {
        Self {
            id: id.to_string(),
            start_x: sx,
            start_y: sy,
            end_x: ex,
            end_y: ey,
            net_name: net.to_string(),
            lamport,
            tombstoned: false,
        }
    }
}

/// Replicated multi-conductor bus structure.
#[derive(Debug, Clone, PartialEq)]
pub struct CrdtBus {
    pub id: String,
    pub bus_name: String,
    pub signals: Vec<String>,
    pub lamport: LamportTimestamp,
    pub tombstoned: bool,
}

impl CrdtBus {
    pub fn new(id: &str, bus_name: &str, signals: Vec<String>, lamport: LamportTimestamp) -> Self {
        Self {
            id: id.to_string(),
            bus_name: bus_name.to_string(),
            signals,
            lamport,
            tombstoned: false,
        }
    }
}

/// Atomic delta operation for broadcast over the P2P mesh network.
#[derive(Debug, Clone, PartialEq)]
pub enum CrdtDelta {
    UpsertComponent(CrdtComponent),
    DeleteComponent {
        id: String,
        lamport: LamportTimestamp,
    },
    UpsertWire(CrdtWire),
    DeleteWire {
        id: String,
        lamport: LamportTimestamp,
    },
    UpsertBus(CrdtBus),
    DeleteBus {
        id: String,
        lamport: LamportTimestamp,
    },
    Batch(Vec<CrdtDelta>),
}

impl CrdtDelta {
    /// Estimates serialized size in bytes for network transmission budgeting.
    pub fn estimated_serialized_bytes(&self) -> usize {
        match self {
            Self::UpsertComponent(c) => 64 + c.id.len() + c.designator.len() + c.value.len(),
            Self::DeleteComponent { id, .. } => 32 + id.len(),
            Self::UpsertWire(w) => 48 + w.id.len() + w.net_name.len(),
            Self::DeleteWire { id, .. } => 32 + id.len(),
            Self::UpsertBus(b) => 40 + b.id.len() + b.signals.iter().map(|s| s.len()).sum::<usize>(),
            Self::DeleteBus { id, .. } => 32 + id.len(),
            Self::Batch(items) => items.iter().map(|i| i.estimated_serialized_bytes()).sum(),
        }
    }
}

/// Master Conflict-Free Replicated Data Type (CRDT) document store.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CrdtEngine {
    pub components: HashMap<String, CrdtComponent>,
    pub wires: HashMap<String, CrdtWire>,
    pub buses: HashMap<String, CrdtBus>,
    pub vector_clock: VectorClock,
    pub local_peer_id: String,
    pub local_counter: u64,
    pub applied_deltas_count: u64,
}

impl CrdtEngine {
    pub fn new(local_peer_id: &str) -> Self {
        Self {
            components: HashMap::new(),
            wires: HashMap::new(),
            buses: HashMap::new(),
            vector_clock: VectorClock::new(),
            local_peer_id: local_peer_id.to_string(),
            local_counter: 0,
            applied_deltas_count: 0,
        }
    }

    /// Pre-seeds a default collaborative CAD schematic document.
    pub fn default_document(peer_id: &str) -> Self {
        let mut engine = Self::new(peer_id);

        let t1 = engine.next_lamport();
        let r1 = CrdtComponent::new("cmp-r1", "R1", "Resistor", "10k", "0805", 100.0, 100.0, t1.clone());
        engine.apply_delta(&CrdtDelta::UpsertComponent(r1));

        let t2 = engine.next_lamport();
        let c1 = CrdtComponent::new("cmp-c1", "C1", "Capacitor", "100nF", "0805", 220.0, 100.0, t2.clone());
        engine.apply_delta(&CrdtDelta::UpsertComponent(c1));

        let t3 = engine.next_lamport();
        let w1 = CrdtWire::new("wire-w1", 120.0, 100.0, 200.0, 100.0, "NET_DIVIDER", t3.clone());
        engine.apply_delta(&CrdtDelta::UpsertWire(w1));

        engine
    }

    pub fn next_lamport(&mut self) -> LamportTimestamp {
        self.local_counter += 1;
        self.vector_clock.increment(&self.local_peer_id);
        LamportTimestamp::new(self.local_counter, &self.local_peer_id)
    }

    /// Applies a delta following Last-Write-Wins (LWW) rules with Lamport ordering.
    pub fn apply_delta(&mut self, delta: &CrdtDelta) -> bool {
        self.applied_deltas_count += 1;
        match delta {
            CrdtDelta::UpsertComponent(comp) => {
                if let Some(existing) = self.components.get(&comp.id) {
                    if comp.lamport > existing.lamport {
                        self.components.insert(comp.id.clone(), comp.clone());
                        true
                    } else {
                        false
                    }
                } else {
                    self.components.insert(comp.id.clone(), comp.clone());
                    true
                }
            }
            CrdtDelta::DeleteComponent { id, lamport } => {
                if let Some(existing) = self.components.get_mut(id) {
                    if *lamport > existing.lamport {
                        existing.tombstoned = true;
                        existing.lamport = lamport.clone();
                        true
                    } else {
                        false
                    }
                } else {
                    // Create tombstone entry to prevent zombie resurrection from delayed packets
                    let mut tomb = CrdtComponent::new(id, "", "", "", "", 0.0, 0.0, lamport.clone());
                    tomb.tombstoned = true;
                    self.components.insert(id.clone(), tomb);
                    true
                }
            }
            CrdtDelta::UpsertWire(wire) => {
                if let Some(existing) = self.wires.get(&wire.id) {
                    if wire.lamport > existing.lamport {
                        self.wires.insert(wire.id.clone(), wire.clone());
                        true
                    } else {
                        false
                    }
                } else {
                    self.wires.insert(wire.id.clone(), wire.clone());
                    true
                }
            }
            CrdtDelta::DeleteWire { id, lamport } => {
                if let Some(existing) = self.wires.get_mut(id) {
                    if *lamport > existing.lamport {
                        existing.tombstoned = true;
                        existing.lamport = lamport.clone();
                        true
                    } else {
                        false
                    }
                } else {
                    let mut tomb = CrdtWire::new(id, 0.0, 0.0, 0.0, 0.0, "", lamport.clone());
                    tomb.tombstoned = true;
                    self.wires.insert(id.clone(), tomb);
                    true
                }
            }
            CrdtDelta::UpsertBus(bus) => {
                if let Some(existing) = self.buses.get(&bus.id) {
                    if bus.lamport > existing.lamport {
                        self.buses.insert(bus.id.clone(), bus.clone());
                        true
                    } else {
                        false
                    }
                } else {
                    self.buses.insert(bus.id.clone(), bus.clone());
                    true
                }
            }
            CrdtDelta::DeleteBus { id, lamport } => {
                if let Some(existing) = self.buses.get_mut(id) {
                    if *lamport > existing.lamport {
                        existing.tombstoned = true;
                        existing.lamport = lamport.clone();
                        true
                    } else {
                        false
                    }
                } else {
                    let mut tomb = CrdtBus::new(id, "", Vec::new(), lamport.clone());
                    tomb.tombstoned = true;
                    self.buses.insert(id.clone(), tomb);
                    true
                }
            }
            CrdtDelta::Batch(deltas) => {
                let mut any_applied = false;
                for d in deltas {
                    if self.apply_delta(d) {
                        any_applied = true;
                    }
                }
                any_applied
            }
        }
    }

    /// Merges an entire other CRDT engine state into self (CvRDT state-based merge).
    pub fn merge_state(&mut self, other: &CrdtEngine) {
        self.vector_clock.merge(&other.vector_clock);
        if other.local_counter > self.local_counter {
            self.local_counter = other.local_counter;
        }

        for comp in other.components.values() {
            self.apply_delta(&CrdtDelta::UpsertComponent(comp.clone()));
        }
        for wire in other.wires.values() {
            self.apply_delta(&CrdtDelta::UpsertWire(wire.clone()));
        }
        for bus in other.buses.values() {
            self.apply_delta(&CrdtDelta::UpsertBus(bus.clone()));
        }
    }

    /// Returns non-tombstoned active components count.
    pub fn active_component_count(&self) -> usize {
        self.components.values().filter(|c| !c.tombstoned).count()
    }

    /// Returns non-tombstoned active wires count.
    pub fn active_wire_count(&self) -> usize {
        self.wires.values().filter(|w| !w.tombstoned).count()
    }

    /// Returns non-tombstoned active buses count.
    pub fn active_bus_count(&self) -> usize {
        self.buses.values().filter(|b| !b.tombstoned).count()
    }
}
