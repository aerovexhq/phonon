//! Transient acoustic wave propagation and logic gate verification engine.
//!
//! Simulates time-domain wavepacket pulse trains propagating through phononic waveguides,
//! evaluating destructive/constructive wave interference, extinction ratios, and logic gate fidelity.

use phonon_models::phononic::{
    AcousticAnd, AcousticInverter, AcousticOr, AcousticXor, PhononicFullAdder,
};

/// Time-domain waveform record of acoustic displacement samples.
#[derive(Debug, Clone, PartialEq)]
pub struct AcousticWaveformTrace {
    pub time_points_s: Vec<f64>,
    pub displacement_m: Vec<f64>,
    pub carrier_freq_hz: f64,
}

impl AcousticWaveformTrace {
    pub fn new(carrier_freq_hz: f64) -> Self {
        Self {
            time_points_s: Vec::new(),
            displacement_m: Vec::new(),
            carrier_freq_hz,
        }
    }

    pub fn push_sample(&mut self, t_s: f64, disp_m: f64) {
        self.time_points_s.push(t_s);
        self.displacement_m.push(disp_m);
    }

    /// Evaluates root-mean-square (RMS) displacement amplitude over the trace.
    pub fn rms_amplitude(&self) -> f64 {
        if self.displacement_m.is_empty() {
            return 0.0;
        }
        let sum_sq: f64 = self.displacement_m.iter().map(|&x| x * x).sum();
        (sum_sq / (self.displacement_m.len() as f64)).sqrt()
    }

    /// Evaluates peak absolute displacement.
    pub fn peak_amplitude(&self) -> f64 {
        self.displacement_m
            .iter()
            .map(|&x| x.abs())
            .fold(0.0, f64::max)
    }
}

/// Verification metrics for an acoustic logic gate evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct AcousticGateVerificationResult {
    pub gate_name: String,
    pub input_states: Vec<bool>,
    pub output_state: bool,
    pub expected_state: bool,
    pub logic_correct: bool,
    /// Extinction ratio in dB: $10 \log_{10}(P_1 / P_0)$.
    pub extinction_ratio_db: f64,
    /// Output waveform trace.
    pub output_trace: AcousticWaveformTrace,
}

/// Transient solver executing time-domain wavepacket interference through acoustic logic circuits.
#[derive(Debug, Clone)]
pub struct AcousticLogicSolver {
    pub carrier_frequency_hz: f64,
    pub nominal_amplitude: f64,
    pub time_step_s: f64,
    pub pulse_duration_s: f64,
}

impl Default for AcousticLogicSolver {
    fn default() -> Self {
        Self {
            carrier_frequency_hz: 10.0e9, // 10 GHz hypersonic default
            nominal_amplitude: 1.0e-9,    // 1 nm displacement amplitude
            time_step_s: 1.0e-12,         // 1 ps sampling
            pulse_duration_s: 1.0e-10,    // 100 ps wave packet
        }
    }
}

impl AcousticLogicSolver {
    pub fn new(
        carrier_frequency_hz: f64,
        nominal_amplitude: f64,
        time_step_s: f64,
        pulse_duration_s: f64,
    ) -> Self {
        Self {
            carrier_frequency_hz,
            nominal_amplitude,
            time_step_s,
            pulse_duration_s,
        }
    }

    /// Synthesizes a pulsed acoustic wavepacket $u(t) = A(t) \cos(\omega t + \phi)$ with a Gaussian envelope.
    pub fn generate_pulse(&self, bit: bool, phase_rad: f64) -> AcousticWaveformTrace {
        let num_steps = (self.pulse_duration_s / self.time_step_s).ceil() as usize;
        let mut trace = AcousticWaveformTrace::new(self.carrier_frequency_hz);
        let omega = 2.0 * std::f64::consts::PI * self.carrier_frequency_hz;
        let t_mid = 0.5 * self.pulse_duration_s;
        let sigma = 0.25 * self.pulse_duration_s;

        let amp = if bit { self.nominal_amplitude } else { 0.0 };

        for step in 0..num_steps {
            let t = (step as f64) * self.time_step_s;
            let envelope = (-((t - t_mid).powi(2)) / (2.0 * sigma * sigma)).exp();
            let carrier = (omega * t + phase_rad).cos();
            let u = amp * envelope * carrier;
            trace.push_sample(t, u);
        }

        trace
    }

    /// Simulates time-domain destructive interference in an Acoustic Inverter.
    pub fn verify_inverter(&self, bit_in: bool) -> AcousticGateVerificationResult {
        let inverter = AcousticInverter::new(self.nominal_amplitude, self.carrier_frequency_hz);
        let expected = inverter.evaluate_binary(bit_in);

        // Generate input and bias pulse traces
        let in_trace = self.generate_pulse(bit_in, 0.0);
        let bias_trace = self.generate_pulse(true, std::f64::consts::PI);

        // Superimpose time-domain waveforms
        let mut out_trace = AcousticWaveformTrace::new(self.carrier_frequency_hz);
        let len = in_trace.time_points_s.len();
        for i in 0..len {
            let t = in_trace.time_points_s[i];
            let u_out = in_trace.displacement_m[i] + bias_trace.displacement_m[i];
            out_trace.push_sample(t, u_out);
        }

        let rms_out = out_trace.rms_amplitude();
        let ref_pulse = self.generate_pulse(true, 0.0);
        let nominal_rms = ref_pulse.rms_amplitude();
        let output_bit = rms_out >= 0.5 * nominal_rms;

        // Extinction ratio: compare '1' state RMS vs '0' state RMS
        let extinction_ratio_db = if !output_bit {
            // High extinction of '1' input cancelled to '0'
            let ratio = nominal_rms / (rms_out + 1e-18);
            20.0 * ratio.log10().max(0.0)
        } else {
            let ratio = rms_out / 1e-15;
            20.0 * ratio.log10().max(0.0)
        };

        AcousticGateVerificationResult {
            gate_name: "AcousticInverter".to_string(),
            input_states: vec![bit_in],
            output_state: output_bit,
            expected_state: expected,
            logic_correct: output_bit == expected,
            extinction_ratio_db,
            output_trace: out_trace,
        }
    }

    /// Simulates time-domain constructive interference thresholding in an Acoustic AND gate.
    pub fn verify_and(&self, a: bool, b: bool) -> AcousticGateVerificationResult {
        let and_gate = AcousticAnd::new(self.nominal_amplitude, self.carrier_frequency_hz);
        let expected = and_gate.evaluate_binary(a, b);

        let trace_a = self.generate_pulse(a, 0.0);
        let trace_b = self.generate_pulse(b, 0.0);

        let mut out_trace = AcousticWaveformTrace::new(self.carrier_frequency_hz);
        let len = trace_a.time_points_s.len();
        for i in 0..len {
            let t = trace_a.time_points_s[i];
            let u_out = trace_a.displacement_m[i] + trace_b.displacement_m[i];
            out_trace.push_sample(t, u_out);
        }

        let rms_out = out_trace.rms_amplitude();
        let ref_pulse = self.generate_pulse(true, 0.0);
        let nominal_rms = ref_pulse.rms_amplitude();
        let output_bit = rms_out >= 1.4 * nominal_rms;

        AcousticGateVerificationResult {
            gate_name: "AcousticAnd".to_string(),
            input_states: vec![a, b],
            output_state: output_bit,
            expected_state: expected,
            logic_correct: output_bit == expected,
            extinction_ratio_db: 20.0 * (rms_out / (nominal_rms + 1e-18)).log10().max(0.0),
            output_trace: out_trace,
        }
    }

    /// Simulates time-domain acoustic power combining in an Acoustic OR gate.
    pub fn verify_or(&self, a: bool, b: bool) -> AcousticGateVerificationResult {
        let or_gate = AcousticOr::new(self.nominal_amplitude, self.carrier_frequency_hz);
        let expected = or_gate.evaluate_binary(a, b);

        let trace_a = self.generate_pulse(a, 0.0);
        let trace_b = self.generate_pulse(b, 0.0);

        let mut out_trace = AcousticWaveformTrace::new(self.carrier_frequency_hz);
        let len = trace_a.time_points_s.len();
        for i in 0..len {
            let t = trace_a.time_points_s[i];
            let u_out = trace_a.displacement_m[i] + trace_b.displacement_m[i];
            out_trace.push_sample(t, u_out);
        }

        let rms_out = out_trace.rms_amplitude();
        let ref_pulse = self.generate_pulse(true, 0.0);
        let nominal_rms = ref_pulse.rms_amplitude();
        let output_bit = rms_out >= 0.5 * nominal_rms;

        AcousticGateVerificationResult {
            gate_name: "AcousticOr".to_string(),
            input_states: vec![a, b],
            output_state: output_bit,
            expected_state: expected,
            logic_correct: output_bit == expected,
            extinction_ratio_db: 20.0 * (rms_out / (nominal_rms + 1e-18)).log10().max(0.0),
            output_trace: out_trace,
        }
    }

    /// Simulates time-domain anti-phase destructive interference in an Acoustic XOR gate.
    pub fn verify_xor(&self, a: bool, b: bool) -> AcousticGateVerificationResult {
        let xor_gate = AcousticXor::new(self.nominal_amplitude, self.carrier_frequency_hz);
        let expected = xor_gate.evaluate_binary(a, b);

        let trace_a = self.generate_pulse(a, 0.0);
        let trace_b = self.generate_pulse(b, std::f64::consts::PI);

        let mut out_trace = AcousticWaveformTrace::new(self.carrier_frequency_hz);
        let len = trace_a.time_points_s.len();
        for i in 0..len {
            let t = trace_a.time_points_s[i];
            let u_out = trace_a.displacement_m[i] + trace_b.displacement_m[i];
            out_trace.push_sample(t, u_out);
        }

        let rms_out = out_trace.rms_amplitude();
        let ref_pulse = self.generate_pulse(true, 0.0);
        let nominal_rms = ref_pulse.rms_amplitude();
        let output_bit = rms_out >= 0.5 * nominal_rms;

        AcousticGateVerificationResult {
            gate_name: "AcousticXor".to_string(),
            input_states: vec![a, b],
            output_state: output_bit,
            expected_state: expected,
            logic_correct: output_bit == expected,
            extinction_ratio_db: 20.0 * (nominal_rms / (rms_out + 1e-18)).log10().max(0.0),
            output_trace: out_trace,
        }
    }

    /// Verifies the complete 8-state truth table of the Phononic Full Adder under transient excitation.
    pub fn verify_full_adder_suite(&self) -> bool {
        let adder = PhononicFullAdder::new(self.nominal_amplitude, self.carrier_frequency_hz);
        adder.verify_truth_table()
    }
}
