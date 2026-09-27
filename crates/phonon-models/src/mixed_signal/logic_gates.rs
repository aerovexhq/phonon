//! Behavioral digital logic primitives and sequential elements for mixed-signal co-simulation:
//! logic gates (`And`, `Nand`, `Or`, `Nor`, `Xor`, `Inverter`, `Buffer`), `DFlipFlop`, `PeriodicClock`,
//! and the `DigitalNetwork` graph.

use phonon_core::{DigitalEvent, DigitalNodeId, EventQueue, LogicLevel};
use std::collections::HashMap;

/// Standard boolean logic gate types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicGateType {
    And,
    Nand,
    Or,
    Nor,
    Xor,
    Xnor,
    Inverter,
    Buffer,
}

/// A behavioral multi-input logic gate.
#[derive(Debug, Clone)]
pub struct LogicGate {
    pub id: usize,
    pub gate_type: LogicGateType,
    pub inputs: Vec<DigitalNodeId>,
    pub output: DigitalNodeId,
    pub propagation_delay: f64,
}

impl LogicGate {
    /// Creates a new logic gate.
    pub fn new(
        id: usize,
        gate_type: LogicGateType,
        inputs: Vec<DigitalNodeId>,
        output: DigitalNodeId,
        propagation_delay: f64,
    ) -> Self {
        Self {
            id,
            gate_type,
            inputs,
            output,
            propagation_delay: propagation_delay.max(0.0),
        }
    }

    /// Evaluates gate output from its current inputs.
    pub fn evaluate(&self, get_val: impl Fn(DigitalNodeId) -> LogicLevel) -> LogicLevel {
        match self.gate_type {
            LogicGateType::Inverter => {
                let in_val = self
                    .inputs
                    .first()
                    .map(|&n| get_val(n))
                    .unwrap_or(LogicLevel::U);
                !in_val
            }
            LogicGateType::Buffer => self
                .inputs
                .first()
                .map(|&n| get_val(n))
                .unwrap_or(LogicLevel::U),
            LogicGateType::And => {
                if self.inputs.is_empty() {
                    return LogicLevel::U;
                }
                let mut acc = get_val(self.inputs[0]);
                for &inp in &self.inputs[1..] {
                    acc = acc & get_val(inp);
                }
                acc
            }
            LogicGateType::Nand => {
                if self.inputs.is_empty() {
                    return LogicLevel::U;
                }
                let mut acc = get_val(self.inputs[0]);
                for &inp in &self.inputs[1..] {
                    acc = acc & get_val(inp);
                }
                !acc
            }
            LogicGateType::Or => {
                if self.inputs.is_empty() {
                    return LogicLevel::U;
                }
                let mut acc = get_val(self.inputs[0]);
                for &inp in &self.inputs[1..] {
                    acc = acc | get_val(inp);
                }
                acc
            }
            LogicGateType::Nor => {
                if self.inputs.is_empty() {
                    return LogicLevel::U;
                }
                let mut acc = get_val(self.inputs[0]);
                for &inp in &self.inputs[1..] {
                    acc = acc | get_val(inp);
                }
                !acc
            }
            LogicGateType::Xor => {
                if self.inputs.is_empty() {
                    return LogicLevel::U;
                }
                let mut acc = get_val(self.inputs[0]);
                for &inp in &self.inputs[1..] {
                    acc = acc ^ get_val(inp);
                }
                acc
            }
            LogicGateType::Xnor => {
                if self.inputs.is_empty() {
                    return LogicLevel::U;
                }
                let mut acc = get_val(self.inputs[0]);
                for &inp in &self.inputs[1..] {
                    acc = acc ^ get_val(inp);
                }
                !acc
            }
        }
    }
}

/// An edge-triggered D-type Flip-Flop with asynchronous active-high reset/set.
#[derive(Debug, Clone)]
pub struct DFlipFlop {
    pub id: usize,
    pub clk: DigitalNodeId,
    pub d: DigitalNodeId,
    pub q: DigitalNodeId,
    pub q_bar: Option<DigitalNodeId>,
    pub reset: Option<DigitalNodeId>,
    pub set: Option<DigitalNodeId>,
    pub propagation_delay: f64,

    // Internal state
    pub last_clk: LogicLevel,
    pub current_q: LogicLevel,
}

impl DFlipFlop {
    /// Creates a new `DFlipFlop`.
    pub fn new(
        id: usize,
        clk: DigitalNodeId,
        d: DigitalNodeId,
        q: DigitalNodeId,
        q_bar: Option<DigitalNodeId>,
        propagation_delay: f64,
    ) -> Self {
        Self {
            id,
            clk,
            d,
            q,
            q_bar,
            reset: None,
            set: None,
            propagation_delay: propagation_delay.max(0.0),
            last_clk: LogicLevel::Zero,
            current_q: LogicLevel::Zero,
        }
    }

    /// Sets asynchronous reset and set input nodes (active High).
    pub fn with_async_controls(
        mut self,
        reset: Option<DigitalNodeId>,
        set: Option<DigitalNodeId>,
    ) -> Self {
        self.reset = reset;
        self.set = set;
        self
    }

    /// Evaluates input changes and returns any scheduled output events `(scheduled_time, node, new_level)`.
    pub fn on_input_change(
        &mut self,
        current_time: f64,
        get_val: impl Fn(DigitalNodeId) -> LogicLevel,
    ) -> Vec<(f64, DigitalNodeId, LogicLevel)> {
        let mut events = Vec::new();
        let target_time = current_time + self.propagation_delay;

        // Check asynchronous reset (active High)
        if let Some(rst_node) = self.reset {
            if get_val(rst_node).is_high() {
                if self.current_q != LogicLevel::Zero {
                    self.current_q = LogicLevel::Zero;
                    events.push((target_time, self.q, LogicLevel::Zero));
                    if let Some(qb) = self.q_bar {
                        events.push((target_time, qb, LogicLevel::One));
                    }
                }
                return events;
            }
        }

        // Check asynchronous set (active High)
        if let Some(set_node) = self.set {
            if get_val(set_node).is_high() {
                if self.current_q != LogicLevel::One {
                    self.current_q = LogicLevel::One;
                    events.push((target_time, self.q, LogicLevel::One));
                    if let Some(qb) = self.q_bar {
                        events.push((target_time, qb, LogicLevel::Zero));
                    }
                }
                return events;
            }
        }

        // Edge detection on CLK: 0 -> 1 or L -> 1
        let clk_val = get_val(self.clk);
        let was_low = self.last_clk.is_low();
        let is_high = clk_val.is_high();
        let is_rising_edge = was_low && is_high;
        self.last_clk = clk_val;

        if is_rising_edge {
            let d_val = get_val(self.d);
            let next_q = match d_val {
                LogicLevel::One | LogicLevel::H => LogicLevel::One,
                LogicLevel::Zero | LogicLevel::L => LogicLevel::Zero,
                _ => LogicLevel::X,
            };

            if next_q != self.current_q {
                self.current_q = next_q;
                events.push((target_time, self.q, next_q));
                if let Some(qb) = self.q_bar {
                    events.push((target_time, qb, !next_q));
                }
            }
        }

        events
    }
}

/// Autonomous periodic digital clock generator.
#[derive(Debug, Clone)]
pub struct PeriodicClock {
    pub output: DigitalNodeId,
    pub period: f64,
    pub duty_cycle: f64,
    pub initial_delay: f64,
    pub high_level: LogicLevel,
    pub low_level: LogicLevel,
    // Internal scheduling state
    pub current_level: LogicLevel,
    pub next_toggle_time: f64,
}

impl PeriodicClock {
    /// Creates a new periodic clock with given frequency and 50% duty cycle.
    pub fn new(output: DigitalNodeId, frequency_hz: f64) -> Self {
        let period = if frequency_hz > 0.0 {
            1.0 / frequency_hz
        } else {
            1e-3
        };
        Self {
            output,
            period,
            duty_cycle: 0.5,
            initial_delay: 0.0,
            high_level: LogicLevel::One,
            low_level: LogicLevel::Zero,
            current_level: LogicLevel::Zero,
            next_toggle_time: 0.0,
        }
    }

    /// Sets duty cycle (0.0 to 1.0) and initial phase delay (seconds).
    pub fn with_phase_duty(mut self, initial_delay: f64, duty_cycle: f64) -> Self {
        self.initial_delay = initial_delay.max(0.0);
        self.duty_cycle = duty_cycle.clamp(0.001, 0.999);
        self
    }

    /// Schedules initial clock transition into the event queue.
    pub fn init(&mut self, event_queue: &mut EventQueue, t_start: f64) {
        self.current_level = self.low_level;
        self.next_toggle_time = t_start + self.initial_delay;
        event_queue.schedule(self.next_toggle_time, self.output, self.high_level);
    }

    /// Advances the clock and schedules the subsequent edge into the event queue.
    pub fn on_event_fired(&mut self, event_queue: &mut EventQueue, current_time: f64) {
        let high_duration = self.period * self.duty_cycle;
        let low_duration = self.period * (1.0 - self.duty_cycle);

        if self.current_level == self.low_level {
            // It just went high
            self.current_level = self.high_level;
            self.next_toggle_time = current_time + high_duration;
            event_queue.schedule(self.next_toggle_time, self.output, self.low_level);
        } else {
            // It just went low
            self.current_level = self.low_level;
            self.next_toggle_time = current_time + low_duration;
            event_queue.schedule(self.next_toggle_time, self.output, self.high_level);
        }
    }
}

/// Autonomous Successive Approximation Register (SAR) digital sequencer.
#[derive(Debug, Clone)]
pub struct SarController {
    pub id: usize,
    /// Conversion clock input.
    pub clk: DigitalNodeId,
    /// Comparator output input.
    pub comp_in: DigitalNodeId,
    /// Digital bit outputs from MSB down to LSB (index 0 is LSB, index N-1 is MSB).
    pub bit_outputs: Vec<DigitalNodeId>,
    /// End of conversion indicator output.
    pub eoc: Option<DigitalNodeId>,
    /// Propagation delay (seconds).
    pub propagation_delay: f64,
    /// Internal sequencer step: 0..=N.
    pub current_step: usize,
    /// Latched bits.
    pub bit_levels: Vec<LogicLevel>,
    pub last_clk: LogicLevel,
}

impl SarController {
    /// Creates a new SAR controller.
    pub fn new(
        id: usize,
        clk: DigitalNodeId,
        comp_in: DigitalNodeId,
        bit_outputs: Vec<DigitalNodeId>,
        eoc: Option<DigitalNodeId>,
        propagation_delay: f64,
    ) -> Self {
        let n = bit_outputs.len();
        Self {
            id,
            clk,
            comp_in,
            bit_outputs,
            eoc,
            propagation_delay: propagation_delay.max(0.0),
            current_step: 0,
            bit_levels: vec![LogicLevel::Zero; n],
            last_clk: LogicLevel::Zero,
        }
    }

    /// Evaluates input changes and returns scheduled bit transition events.
    pub fn on_input_change(
        &mut self,
        current_time: f64,
        get_val: impl Fn(DigitalNodeId) -> LogicLevel,
    ) -> Vec<(f64, DigitalNodeId, LogicLevel)> {
        let mut events = Vec::new();
        let target_time = current_time + self.propagation_delay;

        let clk_val = get_val(self.clk);
        let was_low = self.last_clk.is_low();
        let is_high = clk_val.is_high();
        let is_rising_edge = was_low && is_high;
        self.last_clk = clk_val;

        if !is_rising_edge {
            return events;
        }

        let n = self.bit_outputs.len();
        if n == 0 {
            return events;
        }

        if self.current_step == 0 {
            // Start of conversion: reset all bits to Zero, set MSB to One
            for i in 0..n {
                self.bit_levels[i] = LogicLevel::Zero;
                events.push((target_time, self.bit_outputs[i], LogicLevel::Zero));
            }
            let msb_idx = n - 1;
            self.bit_levels[msb_idx] = LogicLevel::One;
            events.push((target_time, self.bit_outputs[msb_idx], LogicLevel::One));

            if let Some(eoc_node) = self.eoc {
                events.push((target_time, eoc_node, LogicLevel::Zero));
            }
            self.current_step = 1;
        } else if self.current_step <= n {
            // Evaluate comparator response for the bit tested in previous cycle
            let tested_idx = n - self.current_step;
            let comp_val = get_val(self.comp_in);

            if comp_val.is_low() {
                // Analog input is lower than DAC: reset tested bit to Zero
                self.bit_levels[tested_idx] = LogicLevel::Zero;
                events.push((target_time, self.bit_outputs[tested_idx], LogicLevel::Zero));
            } else {
                // Analog input >= DAC: keep tested bit at One
                self.bit_levels[tested_idx] = LogicLevel::One;
            }

            if self.current_step < n {
                // Test next lower bit
                let next_idx = tested_idx - 1;
                self.bit_levels[next_idx] = LogicLevel::One;
                events.push((target_time, self.bit_outputs[next_idx], LogicLevel::One));
                self.current_step += 1;
            } else {
                // Conversion complete for all N bits!
                if let Some(eoc_node) = self.eoc {
                    events.push((target_time, eoc_node, LogicLevel::One));
                }
                self.current_step = n + 1; // Hold latched conversion result
            }
        }

        events
    }

    /// Evaluates current code as an unsigned integer.
    pub fn current_code(&self) -> u32 {
        let mut code = 0u32;
        for (i, &lvl) in self.bit_levels.iter().enumerate() {
            if lvl.is_high() {
                code |= 1 << i;
            }
        }
        code
    }
}

/// Complete discrete digital network containing gates, registers, and interconnects.
#[derive(Debug, Clone, Default)]
pub struct DigitalNetwork {
    pub node_names: HashMap<String, DigitalNodeId>,
    pub node_levels: Vec<LogicLevel>,
    pub gates: Vec<LogicGate>,
    pub flip_flops: Vec<DFlipFlop>,
    pub clocks: Vec<PeriodicClock>,
    pub sar_controllers: Vec<SarController>,
    // Listeners for fast event dispatch
    node_to_gates: Vec<Vec<usize>>,
    node_to_ffs: Vec<Vec<usize>>,
    node_to_sar: Vec<Vec<usize>>,
}

impl DigitalNetwork {
    /// Creates an empty digital network.
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocates a new digital node with an optional name.
    pub fn add_node(&mut self, name: Option<&str>) -> DigitalNodeId {
        let id = DigitalNodeId::new(self.node_levels.len() as u32);
        self.node_levels.push(LogicLevel::U);
        self.node_to_gates.push(Vec::new());
        self.node_to_ffs.push(Vec::new());
        self.node_to_sar.push(Vec::new());
        if let Some(n) = name {
            self.node_names.insert(n.to_string(), id);
        }
        id
    }

    /// Finds a digital node ID by name.
    pub fn find_node(&self, name: &str) -> Option<DigitalNodeId> {
        self.node_names.get(name).copied()
    }

    /// Adds a logic gate to the network and registers input listeners.
    pub fn add_gate(&mut self, gate: LogicGate) {
        let gate_idx = self.gates.len();
        for &inp in &gate.inputs {
            let idx = inp.index();
            if idx >= self.node_to_gates.len() {
                self.node_to_gates.resize(idx + 1, Vec::new());
            }
            self.node_to_gates[idx].push(gate_idx);
        }
        self.gates.push(gate);
    }

    /// Adds a flip-flop and registers listeners for clk, d, reset, set.
    pub fn add_flip_flop(&mut self, ff: DFlipFlop) {
        let ff_idx = self.flip_flops.len();
        let mut listen_nodes = vec![ff.clk, ff.d];
        if let Some(r) = ff.reset {
            listen_nodes.push(r);
        }
        if let Some(s) = ff.set {
            listen_nodes.push(s);
        }

        for node in listen_nodes {
            let idx = node.index();
            if idx >= self.node_to_ffs.len() {
                self.node_to_ffs.resize(idx + 1, Vec::new());
            }
            self.node_to_ffs[idx].push(ff_idx);
        }
        self.flip_flops.push(ff);
    }

    /// Adds an autonomous periodic clock.
    pub fn add_clock(&mut self, clock: PeriodicClock) {
        self.clocks.push(clock);
    }

    /// Adds a SAR ADC controller sequencer.
    pub fn add_sar_controller(&mut self, sar: SarController) {
        let sar_idx = self.sar_controllers.len();
        let idx = sar.clk.index();
        if idx >= self.node_to_sar.len() {
            self.node_to_sar.resize(idx + 1, Vec::new());
        }
        self.node_to_sar[idx].push(sar_idx);
        self.sar_controllers.push(sar);
    }

    /// Current logic level of a node.
    #[inline]
    pub fn get_level(&self, node: DigitalNodeId) -> LogicLevel {
        self.node_levels
            .get(node.index())
            .copied()
            .unwrap_or(LogicLevel::U)
    }

    /// Sets the logic level of a node directly.
    #[inline]
    pub fn set_level(&mut self, node: DigitalNodeId, level: LogicLevel) {
        let idx = node.index();
        if idx >= self.node_levels.len() {
            self.node_levels.resize(idx + 1, LogicLevel::U);
        }
        self.node_levels[idx] = level;
    }

    /// Initializes all clocks into the event queue at `t_start`.
    pub fn init_clocks(&mut self, event_queue: &mut EventQueue, t_start: f64) {
        for clk in &mut self.clocks {
            clk.init(event_queue, t_start);
        }
    }

    /// Dispatches a single digital event and schedules subsequent fanout gate/flip-flop updates.
    pub fn dispatch_event(&mut self, event: DigitalEvent, event_queue: &mut EventQueue) {
        let node = event.node;
        let old_level = self.get_level(node);

        // Check if event came from a periodic clock
        for clk in &mut self.clocks {
            if clk.output == node && (event.time - clk.next_toggle_time).abs() < 1e-12 {
                clk.on_event_fired(event_queue, event.time);
            }
        }

        if old_level == event.level {
            return;
        }

        self.set_level(node, event.level);

        // Trigger connected gates
        let node_idx = node.index();
        if node_idx < self.node_to_gates.len() {
            let levels = &self.node_levels;
            for &gate_idx in &self.node_to_gates[node_idx] {
                let gate = &self.gates[gate_idx];
                let out_val =
                    gate.evaluate(|n| levels.get(n.index()).copied().unwrap_or(LogicLevel::U));
                if out_val
                    != levels
                        .get(gate.output.index())
                        .copied()
                        .unwrap_or(LogicLevel::U)
                {
                    event_queue.schedule(event.time + gate.propagation_delay, gate.output, out_val);
                }
            }
        }

        // Trigger connected flip-flops
        if node_idx < self.node_to_ffs.len() {
            let levels = &self.node_levels;
            for &ff_idx in &self.node_to_ffs[node_idx] {
                let ff = &mut self.flip_flops[ff_idx];
                let scheduled = ff.on_input_change(event.time, |n| {
                    levels.get(n.index()).copied().unwrap_or(LogicLevel::U)
                });
                for (t_target, out_node, level) in scheduled {
                    event_queue.schedule(t_target, out_node, level);
                }
            }
        }

        // Trigger connected SAR controllers
        if node_idx < self.node_to_sar.len() {
            let levels = &self.node_levels;
            for &sar_idx in &self.node_to_sar[node_idx] {
                let sar = &mut self.sar_controllers[sar_idx];
                let scheduled = sar.on_input_change(event.time, |n| {
                    levels.get(n.index()).copied().unwrap_or(LogicLevel::U)
                });
                for (t_target, out_node, level) in scheduled {
                    event_queue.schedule(t_target, out_node, level);
                }
            }
        }
    }

    /// Evaluates zero-delay delta cycles at `current_time` until no simultaneous events remain
    /// or `max_delta_cycles` is reached.
    pub fn process_delta_cycles(
        &mut self,
        event_queue: &mut EventQueue,
        current_time: f64,
        max_delta_cycles: usize,
    ) -> Result<usize, String> {
        let mut delta_count = 0;
        let epsilon = 1e-15;

        loop {
            // Check if next event is at current_time
            if let Some(next_t) = event_queue.next_event_time() {
                if next_t <= current_time + epsilon {
                    let event = event_queue.pop().unwrap();
                    self.dispatch_event(event, event_queue);
                    delta_count += 1;
                    if delta_count > max_delta_cycles {
                        return Err(format!(
                            "Delta cycle limit ({}) exceeded at t={:.6e}s: potential zero-delay logic oscillation detected",
                            max_delta_cycles, current_time
                        ));
                    }
                    continue;
                }
            }
            break;
        }

        Ok(delta_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logic_gate_evaluation() {
        let mut net = DigitalNetwork::new();
        let a = net.add_node(Some("A"));
        let b = net.add_node(Some("B"));
        let out = net.add_node(Some("OUT"));

        let gate = LogicGate::new(0, LogicGateType::And, vec![a, b], out, 1e-9);
        net.add_gate(gate);

        let mut queue = EventQueue::new();
        net.set_level(a, LogicLevel::One);
        net.set_level(b, LogicLevel::Zero);

        // When B transitions to One at t=1.0s
        net.dispatch_event(
            DigitalEvent {
                time: 1.0,
                node: b,
                level: LogicLevel::One,
                priority: 0,
            },
            &mut queue,
        );

        assert_eq!(queue.len(), 1);
        let ev = queue.pop().unwrap();
        assert_eq!(ev.node, out);
        assert_eq!(ev.level, LogicLevel::One);
        assert!((ev.time - 1.000000001).abs() < 1e-12);
    }

    #[test]
    fn test_d_flip_flop_clocking() {
        let mut net = DigitalNetwork::new();
        let d = net.add_node(Some("D"));
        let clk = net.add_node(Some("CLK"));
        let q = net.add_node(Some("Q"));
        let q_bar = net.add_node(Some("Q_BAR"));

        let ff = DFlipFlop::new(0, clk, d, q, Some(q_bar), 2e-9);
        net.add_flip_flop(ff);

        let mut queue = EventQueue::new();
        net.set_level(d, LogicLevel::One);
        net.set_level(clk, LogicLevel::Zero);

        // Rising clock edge at t=10ns
        net.dispatch_event(
            DigitalEvent {
                time: 10e-9,
                node: clk,
                level: LogicLevel::One,
                priority: 0,
            },
            &mut queue,
        );

        assert_eq!(queue.len(), 2);
        let ev1 = queue.pop().unwrap();
        let ev2 = queue.pop().unwrap();
        assert_eq!(ev1.time, 12e-9);
        assert_eq!(ev2.time, 12e-9);

        // Q should be 1, Q_BAR should be 0
        let q_event = if ev1.node == q { ev1 } else { ev2 };
        let qb_event = if ev1.node == q_bar { ev1 } else { ev2 };
        assert_eq!(q_event.level, LogicLevel::One);
        assert_eq!(qb_event.level, LogicLevel::Zero);
    }
}
