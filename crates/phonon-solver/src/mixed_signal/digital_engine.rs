#![deny(unsafe_code)]

//! Discrete event-driven digital simulation kernel.
//!
//! Provides multi-state discrete logic modeling (Low, High, HighZ, Unknown),
//! combinational and sequential logic primitives (AND, OR, NAND, NOR, XOR, XNOR, NOT, DFF, BUFFER),
//! and high-throughput min-heap event scheduling.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// Multi-valued discrete logic state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogicState {
    Low,
    High,
    HighZ,
    Unknown,
}

impl LogicState {
    /// Returns true if state is High.
    #[inline]
    pub fn is_high(&self) -> bool {
        *self == LogicState::High
    }

    /// Returns true if state is Low.
    #[inline]
    pub fn is_low(&self) -> bool {
        *self == LogicState::Low
    }

    /// Returns the inverted logic state.
    #[inline]
    pub fn invert(&self) -> Self {
        match self {
            LogicState::Low => LogicState::High,
            LogicState::High => LogicState::Low,
            LogicState::HighZ | LogicState::Unknown => LogicState::Unknown,
        }
    }

    /// Converts logic state to an equivalent DC voltage level.
    #[inline]
    pub fn to_voltage(&self, v_ol: f64, v_oh: f64) -> f64 {
        match self {
            LogicState::Low => v_ol,
            LogicState::High => v_oh,
            LogicState::HighZ | LogicState::Unknown => (v_ol + v_oh) * 0.5,
        }
    }
}

/// Primitive digital logic gate types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GateKind {
    And,
    Or,
    Nand,
    Nor,
    Xor,
    Xnor,
    Not,
    Dff,
    Buffer,
}

/// Discrete digital event scheduled at a given simulation timestamp.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DigitalEvent {
    pub time_s: f64,
    pub node_id: usize,
    pub new_state: LogicState,
}

impl Eq for DigitalEvent {}

impl Ord for DigitalEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering so BinaryHeap acts as a min-heap by timestamp
        other
            .time_s
            .total_cmp(&self.time_s)
            .then_with(|| other.node_id.cmp(&self.node_id))
            .then_with(|| (other.new_state as u8).cmp(&(self.new_state as u8)))
    }
}

impl PartialOrd for DigitalEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Digital logic gate instance interconnecting input and output nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct DigitalLogicGate {
    pub id: usize,
    pub kind: GateKind,
    pub inputs: Vec<usize>,
    pub output: usize,
    pub delay_s: f64,
}

impl DigitalLogicGate {
    /// Creates a new digital logic gate.
    pub fn new(id: usize, kind: GateKind, inputs: Vec<usize>, output: usize, delay_s: f64) -> Self {
        Self {
            id,
            kind,
            inputs,
            output,
            delay_s,
        }
    }

    /// Evaluates the gate output given current node states, previous states, and the triggering node.
    pub fn evaluate(
        &self,
        node_states: &[LogicState],
        prev_node_states: &[LogicState],
        triggered_node: usize,
    ) -> Option<LogicState> {
        match self.kind {
            GateKind::And => {
                if self.inputs.is_empty() {
                    return Some(LogicState::Low);
                }
                let mut all_high = true;
                for &inp in &self.inputs {
                    match node_states.get(inp).copied().unwrap_or(LogicState::Unknown) {
                        LogicState::Low => return Some(LogicState::Low),
                        LogicState::High => {}
                        _ => all_high = false,
                    }
                }
                if all_high {
                    Some(LogicState::High)
                } else {
                    Some(LogicState::Unknown)
                }
            }
            GateKind::Or => {
                if self.inputs.is_empty() {
                    return Some(LogicState::Low);
                }
                let mut all_low = true;
                for &inp in &self.inputs {
                    match node_states.get(inp).copied().unwrap_or(LogicState::Unknown) {
                        LogicState::High => return Some(LogicState::High),
                        LogicState::Low => {}
                        _ => all_low = false,
                    }
                }
                if all_low {
                    Some(LogicState::Low)
                } else {
                    Some(LogicState::Unknown)
                }
            }
            GateKind::Nand => {
                if self.inputs.is_empty() {
                    return Some(LogicState::High);
                }
                let mut all_high = true;
                for &inp in &self.inputs {
                    match node_states.get(inp).copied().unwrap_or(LogicState::Unknown) {
                        LogicState::Low => return Some(LogicState::High),
                        LogicState::High => {}
                        _ => all_high = false,
                    }
                }
                if all_high {
                    Some(LogicState::Low)
                } else {
                    Some(LogicState::Unknown)
                }
            }
            GateKind::Nor => {
                if self.inputs.is_empty() {
                    return Some(LogicState::High);
                }
                let mut all_low = true;
                for &inp in &self.inputs {
                    match node_states.get(inp).copied().unwrap_or(LogicState::Unknown) {
                        LogicState::High => return Some(LogicState::Low),
                        LogicState::Low => {}
                        _ => all_low = false,
                    }
                }
                if all_low {
                    Some(LogicState::High)
                } else {
                    Some(LogicState::Unknown)
                }
            }
            GateKind::Xor => {
                if self.inputs.is_empty() {
                    return Some(LogicState::Low);
                }
                let mut high_count = 0usize;
                for &inp in &self.inputs {
                    match node_states.get(inp).copied().unwrap_or(LogicState::Unknown) {
                        LogicState::High => high_count += 1,
                        LogicState::Low => {}
                        _ => return Some(LogicState::Unknown),
                    }
                }
                if high_count % 2 == 1 {
                    Some(LogicState::High)
                } else {
                    Some(LogicState::Low)
                }
            }
            GateKind::Xnor => {
                if self.inputs.is_empty() {
                    return Some(LogicState::High);
                }
                let mut high_count = 0usize;
                for &inp in &self.inputs {
                    match node_states.get(inp).copied().unwrap_or(LogicState::Unknown) {
                        LogicState::High => high_count += 1,
                        LogicState::Low => {}
                        _ => return Some(LogicState::Unknown),
                    }
                }
                if high_count % 2 == 1 {
                    Some(LogicState::Low)
                } else {
                    Some(LogicState::High)
                }
            }
            GateKind::Not => {
                if let Some(&inp) = self.inputs.first() {
                    let st = node_states.get(inp).copied().unwrap_or(LogicState::Unknown);
                    Some(st.invert())
                } else {
                    Some(LogicState::Unknown)
                }
            }
            GateKind::Buffer => {
                if let Some(&inp) = self.inputs.first() {
                    Some(node_states.get(inp).copied().unwrap_or(LogicState::Unknown))
                } else {
                    Some(LogicState::Unknown)
                }
            }
            GateKind::Dff => {
                // Inputs: [clk, d] or [clk, d, rst]
                if self.inputs.len() < 2 {
                    return None;
                }
                let clk_node = self.inputs[0];
                let d_node = self.inputs[1];

                // Optional asynchronous reset
                if self.inputs.len() >= 3 {
                    let rst_node = self.inputs[2];
                    if let Some(&LogicState::High) = node_states.get(rst_node) {
                        return Some(LogicState::Low);
                    }
                }

                // Rising clock edge sensitivity
                if triggered_node == clk_node {
                    let prev_clk = prev_node_states
                        .get(clk_node)
                        .copied()
                        .unwrap_or(LogicState::Unknown);
                    let curr_clk = node_states
                        .get(clk_node)
                        .copied()
                        .unwrap_or(LogicState::Unknown);

                    let is_posedge = (prev_clk == LogicState::Low || prev_clk == LogicState::Unknown)
                        && curr_clk == LogicState::High;

                    if is_posedge {
                        return Some(node_states.get(d_node).copied().unwrap_or(LogicState::Unknown));
                    }
                }
                None
            }
        }
    }
}

/// High-performance discrete event-driven digital simulation engine.
#[derive(Debug, Clone)]
pub struct DigitalEngine {
    event_queue: BinaryHeap<DigitalEvent>,
    node_states: Vec<LogicState>,
    prev_node_states: Vec<LogicState>,
    gates: Vec<DigitalLogicGate>,
    fanout: Vec<Vec<usize>>,
    current_time_s: f64,
    total_events_processed: usize,
}

impl DigitalEngine {
    /// Creates a new digital engine with the specified initial number of nodes.
    pub fn new(num_nodes: usize) -> Self {
        Self {
            event_queue: BinaryHeap::new(),
            node_states: vec![LogicState::Low; num_nodes],
            prev_node_states: vec![LogicState::Low; num_nodes],
            gates: Vec::new(),
            fanout: vec![Vec::new(); num_nodes],
            current_time_s: 0.0,
            total_events_processed: 0,
        }
    }

    /// Ensures internal buffers can accommodate at least `count` nodes.
    pub fn ensure_nodes(&mut self, count: usize) {
        if count > self.node_states.len() {
            self.node_states.resize(count, LogicState::Low);
            self.prev_node_states.resize(count, LogicState::Low);
            self.fanout.resize(count, Vec::new());
        }
    }

    /// Adds a logic gate to the engine and configures fanout connectivity.
    pub fn add_gate(&mut self, gate: DigitalLogicGate) {
        let max_node = gate
            .inputs
            .iter()
            .copied()
            .max()
            .unwrap_or(0)
            .max(gate.output);
        self.ensure_nodes(max_node + 1);

        let gate_idx = self.gates.len();
        for &inp in &gate.inputs {
            self.fanout[inp].push(gate_idx);
        }
        self.gates.push(gate);
    }

    /// Schedules a future digital event in the priority queue.
    pub fn schedule_event(&mut self, event: DigitalEvent) {
        self.ensure_nodes(event.node_id + 1);
        self.event_queue.push(event);
    }

    /// Returns the current logic state of a given node.
    pub fn get_state(&self, node_id: usize) -> LogicState {
        self.node_states
            .get(node_id)
            .copied()
            .unwrap_or(LogicState::Unknown)
    }

    /// Forcibly sets the logic state of a node without scheduling downstream propagation.
    pub fn set_state(&mut self, node_id: usize, state: LogicState) {
        self.ensure_nodes(node_id + 1);
        self.prev_node_states[node_id] = self.node_states[node_id];
        self.node_states[node_id] = state;
    }

    /// Returns the current simulation timestamp in seconds.
    pub fn current_time(&self) -> f64 {
        self.current_time_s
    }

    /// Returns the number of instantiated logic gates.
    pub fn gate_count(&self) -> usize {
        self.gates.len()
    }

    /// Returns the number of allocated circuit nodes.
    pub fn node_count(&self) -> usize {
        self.node_states.len()
    }

    /// Returns the total number of events processed across the simulation lifetime.
    pub fn total_events(&self) -> usize {
        self.total_events_processed
    }

    /// Returns the number of events pending in the priority queue.
    pub fn pending_event_count(&self) -> usize {
        self.event_queue.len()
    }

    /// Advances the simulation time up to `target_time_s`, processing all pending events.
    pub fn step_until(&mut self, target_time_s: f64) -> Vec<DigitalEvent> {
        let mut processed_events = Vec::new();

        while let Some(event) = self.event_queue.peek().copied() {
            if event.time_s > target_time_s {
                break;
            }

            self.event_queue.pop();
            self.current_time_s = event.time_s;

            // Only process state transitions that result in a net change
            let current = self.get_state(event.node_id);
            if current != event.new_state {
                self.prev_node_states[event.node_id] = current;
                self.node_states[event.node_id] = event.new_state;
                self.total_events_processed += 1;
                processed_events.push(event);

                // Fan-out propagation to dependent gates
                let gate_indices = self.fanout[event.node_id].clone();
                for g_idx in gate_indices {
                    let gate = &self.gates[g_idx];
                    if let Some(new_output) = gate.evaluate(
                        &self.node_states,
                        &self.prev_node_states,
                        event.node_id,
                    ) {
                        let cur_out = self.get_state(gate.output);
                        if new_output != cur_out {
                            let next_event = DigitalEvent {
                                time_s: event.time_s + gate.delay_s,
                                node_id: gate.output,
                                new_state: new_output,
                            };
                            self.event_queue.push(next_event);
                        }
                    }
                }
            }
        }

        self.current_time_s = target_time_s;
        processed_events
    }
}
