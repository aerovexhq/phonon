#![deny(unsafe_code)]

//! Continuous-time analog and boundary synchronizer.
//!
//! Provides Analog-to-Digital Converter (BoundaryAdc) threshold detection with hysteresis,
//! Digital-to-Analog Converter (BoundaryDac) continuous-time dynamic smoothing,
//! and the adaptive MixedSignalSynchronizer lockstep barrier.

use crate::mixed_signal::digital_engine::{DigitalEngine, DigitalEvent, LogicState};

/// Analog-to-Digital Converter boundary bridge translating continuous voltages
/// into discrete digital logic transitions with configurable threshold hysteresis.
#[derive(Debug, Clone, PartialEq)]
pub struct BoundaryAdc {
    pub analog_node: usize,
    pub digital_node: usize,
    pub v_il: f64,
    pub v_ih: f64,
    pub hysteresis: f64,
    pub current_state: LogicState,
    pub last_voltage: f64,
    pub last_time_s: f64,
}

impl BoundaryAdc {
    /// Creates a new ADC bridge with default CMOS standard thresholds (0.8V / 2.0V, 0.1V hysteresis).
    pub fn new(analog_node: usize, digital_node: usize) -> Self {
        Self {
            analog_node,
            digital_node,
            v_il: 0.8,
            v_ih: 2.0,
            hysteresis: 0.1,
            current_state: LogicState::Low,
            last_voltage: 0.0,
            last_time_s: 0.0,
        }
    }

    /// Customizes logic threshold levels and hysteresis voltage.
    pub fn with_thresholds(mut self, v_il: f64, v_ih: f64, hysteresis: f64) -> Self {
        self.v_il = v_il;
        self.v_ih = v_ih;
        self.hysteresis = hysteresis;
        self
    }

    /// Sets initial state of the ADC.
    pub fn with_initial_state(mut self, state: LogicState, voltage: f64, time_s: f64) -> Self {
        self.current_state = state;
        self.last_voltage = voltage;
        self.last_time_s = time_s;
        self
    }

    /// Evaluates continuous analog voltage at `time_s`, detecting threshold crossings
    /// and interpolating the exact crossing time t*.
    pub fn update(&mut self, voltage: f64, time_s: f64) -> Option<DigitalEvent> {
        let v_th_high = self.v_ih + self.hysteresis;
        let v_th_low = self.v_il - self.hysteresis;

        // Transition to High
        if self.current_state != LogicState::High && voltage >= v_th_high {
            let t_star = if (voltage - self.last_voltage).abs() > 1e-15 {
                let frac = ((v_th_high - self.last_voltage) / (voltage - self.last_voltage)).clamp(0.0, 1.0);
                self.last_time_s + frac * (time_s - self.last_time_s)
            } else {
                time_s
            };

            self.current_state = LogicState::High;
            self.last_voltage = voltage;
            self.last_time_s = time_s;

            return Some(DigitalEvent {
                time_s: t_star,
                node_id: self.digital_node,
                new_state: LogicState::High,
            });
        }

        // Transition to Low
        if self.current_state != LogicState::Low && voltage <= v_th_low {
            let t_star = if (voltage - self.last_voltage).abs() > 1e-15 {
                let frac = ((v_th_low - self.last_voltage) / (voltage - self.last_voltage)).clamp(0.0, 1.0);
                self.last_time_s + frac * (time_s - self.last_time_s)
            } else {
                time_s
            };

            self.current_state = LogicState::Low;
            self.last_voltage = voltage;
            self.last_time_s = time_s;

            return Some(DigitalEvent {
                time_s: t_star,
                node_id: self.digital_node,
                new_state: LogicState::Low,
            });
        }

        // No threshold crossing detected
        self.last_voltage = voltage;
        self.last_time_s = time_s;
        None
    }
}

/// Dynamic smoothing method for DAC voltage transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DacSmoothing {
    Linear,
    SmoothStep,
}

/// Digital-to-Analog Converter boundary bridge translating discrete digital states
/// into continuous-time equivalent driving voltages with rise/fall dynamics and source resistance.
#[derive(Debug, Clone, PartialEq)]
pub struct BoundaryDac {
    pub digital_node: usize,
    pub analog_node: usize,
    pub v_ol: f64,
    pub v_oh: f64,
    pub rise_time_s: f64,
    pub fall_time_s: f64,
    pub r_out: f64,
    pub smoothing: DacSmoothing,
    pub last_state: LogicState,
    pub transition_start_time: f64,
    pub transition_start_voltage: f64,
    pub target_voltage: f64,
}

impl BoundaryDac {
    /// Creates a new DAC bridge with standard 3.3V CMOS parameters (1ns edge rate, 50 Ohm output impedance).
    pub fn new(digital_node: usize, analog_node: usize) -> Self {
        Self {
            digital_node,
            analog_node,
            v_ol: 0.0,
            v_oh: 3.3,
            rise_time_s: 1.0e-9,
            fall_time_s: 1.0e-9,
            r_out: 50.0,
            smoothing: DacSmoothing::Linear,
            last_state: LogicState::Low,
            transition_start_time: 0.0,
            transition_start_voltage: 0.0,
            target_voltage: 0.0,
        }
    }

    /// Sets custom output low and high voltage levels.
    pub fn with_levels(mut self, v_ol: f64, v_oh: f64) -> Self {
        self.v_ol = v_ol;
        self.v_oh = v_oh;
        self
    }

    /// Sets custom rise and fall transition times in seconds.
    pub fn with_timing(mut self, rise_time_s: f64, fall_time_s: f64) -> Self {
        self.rise_time_s = rise_time_s;
        self.fall_time_s = fall_time_s;
        self
    }

    /// Sets output impedance in Ohms.
    pub fn with_impedance(mut self, r_out: f64) -> Self {
        self.r_out = r_out;
        self
    }

    /// Sets voltage smoothing profile.
    pub fn with_smoothing(mut self, smoothing: DacSmoothing) -> Self {
        self.smoothing = smoothing;
        self
    }

    /// Informs the DAC of a digital logic transition event.
    pub fn on_digital_event(&mut self, event: &DigitalEvent) {
        if event.node_id == self.digital_node {
            let current_v = self.voltage_at(event.time_s);
            self.transition_start_time = event.time_s;
            self.transition_start_voltage = current_v;
            self.target_voltage = match event.new_state {
                LogicState::High => self.v_oh,
                LogicState::Low => self.v_ol,
                LogicState::HighZ | LogicState::Unknown => (self.v_ol + self.v_oh) * 0.5,
            };
            self.last_state = event.new_state;
        }
    }

    /// Computes the instantaneous continuous output voltage at simulation time `time_s`.
    pub fn voltage_at(&self, time_s: f64) -> f64 {
        if time_s <= self.transition_start_time {
            return self.transition_start_voltage;
        }

        let duration = if self.target_voltage >= self.transition_start_voltage {
            self.rise_time_s
        } else {
            self.fall_time_s
        };

        if duration <= 1e-18 {
            return self.target_voltage;
        }

        let dt = time_s - self.transition_start_time;
        if dt >= duration {
            return self.target_voltage;
        }

        let u = (dt / duration).clamp(0.0, 1.0);
        let factor = match self.smoothing {
            DacSmoothing::Linear => u,
            DacSmoothing::SmoothStep => u * u * (3.0 - 2.0 * u),
        };

        self.transition_start_voltage + factor * (self.target_voltage - self.transition_start_voltage)
    }
}

/// Report summarizing synchronization events across a lockstep barrier step.
#[derive(Debug, Clone, PartialEq)]
pub struct MixedSignalStepReport {
    pub time_s: f64,
    pub digital_events: Vec<DigitalEvent>,
    pub adc_events: Vec<DigitalEvent>,
    pub analog_voltages: Vec<f64>,
}

/// Adaptive lockstep co-simulation barrier synchronizing discrete event digital engines
/// with continuous analog network state.
#[derive(Debug, Clone)]
pub struct MixedSignalSynchronizer {
    pub digital_engine: DigitalEngine,
    pub adcs: Vec<BoundaryAdc>,
    pub dacs: Vec<BoundaryDac>,
    pub current_time_s: f64,
    pub analog_voltages: Vec<f64>,
    pub max_step_size_s: f64,
}

impl MixedSignalSynchronizer {
    /// Creates a new synchronizer with a digital engine and continuous analog node storage.
    pub fn new(digital_engine: DigitalEngine, num_analog_nodes: usize) -> Self {
        Self {
            digital_engine,
            adcs: Vec::new(),
            dacs: Vec::new(),
            current_time_s: 0.0,
            analog_voltages: vec![0.0; num_analog_nodes],
            max_step_size_s: 1.0e-9,
        }
    }

    /// Attaches an ADC boundary bridge.
    pub fn add_adc(&mut self, adc: BoundaryAdc) {
        if adc.analog_node >= self.analog_voltages.len() {
            self.analog_voltages.resize(adc.analog_node + 1, 0.0);
        }
        self.digital_engine.ensure_nodes(adc.digital_node + 1);
        self.adcs.push(adc);
    }

    /// Attaches a DAC boundary bridge.
    pub fn add_dac(&mut self, dac: BoundaryDac) {
        if dac.analog_node >= self.analog_voltages.len() {
            self.analog_voltages.resize(dac.analog_node + 1, 0.0);
        }
        self.digital_engine.ensure_nodes(dac.digital_node + 1);
        self.dacs.push(dac);
    }

    /// Sets the voltage of an analog node.
    pub fn set_analog_voltage(&mut self, node: usize, voltage: f64) {
        if node >= self.analog_voltages.len() {
            self.analog_voltages.resize(node + 1, 0.0);
        }
        self.analog_voltages[node] = voltage;
    }

    /// Gets the voltage of an analog node.
    pub fn get_analog_voltage(&self, node: usize) -> f64 {
        self.analog_voltages.get(node).copied().unwrap_or(0.0)
    }

    /// Evaluates the driving voltage of a DAC at the current or specified time.
    pub fn get_dac_voltage(&self, dac_index: usize, time_s: f64) -> f64 {
        self.dacs
            .get(dac_index)
            .map(|dac| dac.voltage_at(time_s))
            .unwrap_or(0.0)
    }

    /// Advances the co-simulation across a lockstep barrier to `target_time_s`,
    /// integrating candidate analog voltages and digital events.
    pub fn sync_step(
        &mut self,
        target_time_s: f64,
        candidate_analog_voltages: &[f64],
    ) -> MixedSignalStepReport {
        // 1. Detect ADC crossings from candidate analog voltages
        let mut adc_events = Vec::new();
        for adc in &mut self.adcs {
            let v = candidate_analog_voltages
                .get(adc.analog_node)
                .copied()
                .unwrap_or(0.0);
            if let Some(event) = adc.update(v, target_time_s) {
                self.digital_engine.schedule_event(event);
                adc_events.push(event);
            }
        }

        // 2. Advance digital engine up to target_time_s
        let digital_events = self.digital_engine.step_until(target_time_s);

        // 3. Propagate digital events to attached DACs
        for event in &digital_events {
            for dac in &mut self.dacs {
                dac.on_digital_event(event);
            }
        }

        // 4. Update analog voltage cache and timestamp
        for (i, &v) in candidate_analog_voltages.iter().enumerate() {
            if i < self.analog_voltages.len() {
                self.analog_voltages[i] = v;
            } else {
                self.analog_voltages.push(v);
            }
        }
        self.current_time_s = target_time_s;

        MixedSignalStepReport {
            time_s: target_time_s,
            digital_events,
            adc_events,
            analog_voltages: self.analog_voltages.clone(),
        }
    }

    /// Executes closed-loop co-simulation over a duration with a supplied analog stepping function.
    pub fn run_cosim<F>(
        &mut self,
        duration_s: f64,
        step_size_s: f64,
        mut analog_stepper: F,
    ) -> Vec<MixedSignalStepReport>
    where
        F: FnMut(f64, f64, &[f64], &[f64]) -> Vec<f64>,
    {
        let mut reports = Vec::new();
        let mut t = self.current_time_s;

        while t < duration_s - 1e-15 {
            let next_t = (t + step_size_s).min(duration_s);

            // Compute current DAC voltages driving the analog domain
            let dac_voltages: Vec<f64> = self
                .dacs
                .iter()
                .map(|dac| dac.voltage_at(t))
                .collect();

            // Continuous analog advance
            let candidate_voltages =
                analog_stepper(t, next_t, &self.analog_voltages, &dac_voltages);

            // Synchronized barrier reconciliation
            let report = self.sync_step(next_t, &candidate_voltages);
            reports.push(report);

            t = next_t;
        }

        reports
    }
}
