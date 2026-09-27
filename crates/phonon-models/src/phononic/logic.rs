//! Non-linear acoustic wave interference logic gates and Phononic Full Adder.
//!
//! Executes digital computation purely through mechanical acoustic wavepacket interference
//! without electron/hole charge transport.

/// Coherent acoustic wavepacket state:
/// $u(t) = A \cos(2\pi f_0 t + \phi)$.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcousticWave {
    /// Displacement amplitude $A$ (arbitrary units or nanometers).
    pub amplitude: f64,
    /// Carrier phase $\phi$ in radians.
    pub phase_rad: f64,
    /// Hypersonic carrier frequency $f_0$ in Hz.
    pub frequency_hz: f64,
}

impl AcousticWave {
    pub const NULL: Self = Self {
        amplitude: 0.0,
        phase_rad: 0.0,
        frequency_hz: 10.0e9, // 10 GHz hypersonic default
    };

    pub fn new(amplitude: f64, phase_rad: f64, frequency_hz: f64) -> Self {
        Self {
            amplitude: amplitude.max(0.0),
            phase_rad,
            frequency_hz,
        }
    }

    /// Creates a binary '0' wave (amplitude = 0.0).
    pub fn binary_0(frequency_hz: f64) -> Self {
        Self {
            amplitude: 0.0,
            phase_rad: 0.0,
            frequency_hz,
        }
    }

    /// Creates a binary '1' wave with nominal amplitude $A_0$ and zero phase.
    pub fn binary_1(amplitude: f64, frequency_hz: f64) -> Self {
        Self {
            amplitude,
            phase_rad: 0.0,
            frequency_hz,
        }
    }

    /// Complex in-phase ($I$) and quadrature ($Q$) components:
    /// $\tilde{A} = I + i Q = A \cos\phi + i A \sin\phi$.
    #[inline]
    pub fn iq(&self) -> (f64, f64) {
        (
            self.amplitude * self.phase_rad.cos(),
            self.amplitude * self.phase_rad.sin(),
        )
    }

    /// Acoustic wave intensity $I \propto A^2$.
    #[inline]
    pub fn intensity(&self) -> f64 {
        self.amplitude * self.amplitude
    }

    /// Coherent wave interference / superposition:
    /// $\mathbf{u}_{tot} = \mathbf{u}_1 + \mathbf{u}_2$.
    pub fn superimpose(&self, other: &Self) -> Self {
        let (i1, q1) = self.iq();
        let (i2, q2) = other.iq();

        let i_tot = i1 + i2;
        let q_tot = q1 + q2;

        let amp = (i_tot * i_tot + q_tot * q_tot).sqrt();
        let phase = q_tot.atan2(i_tot);

        Self {
            amplitude: amp,
            phase_rad: phase,
            frequency_hz: self.frequency_hz,
        }
    }

    /// Converts acoustic wave intensity into a digital binary state {0, 1}
    /// against an intensity threshold $I_{th}$.
    pub fn to_binary(&self, threshold_intensity: f64) -> bool {
        self.intensity() >= threshold_intensity
    }
}

/// Acoustic Inverter (NOT gate) operating via destructive wave interference.
#[derive(Debug, Clone, PartialEq)]
pub struct AcousticInverter {
    pub nominal_amplitude: f64,
    pub carrier_frequency_hz: f64,
    pub bias_wave: AcousticWave,
}

impl AcousticInverter {
    pub fn new(nominal_amplitude: f64, carrier_frequency_hz: f64) -> Self {
        // Bias beam has phase pi for destructive cancellation with input '1'
        let bias_wave = AcousticWave::new(
            nominal_amplitude,
            std::f64::consts::PI,
            carrier_frequency_hz,
        );
        Self {
            nominal_amplitude,
            carrier_frequency_hz,
            bias_wave,
        }
    }

    /// Evaluates the inverter output wave for a given input wave.
    pub fn evaluate(&self, input: &AcousticWave) -> AcousticWave {
        self.bias_wave.superimpose(input)
    }

    /// Evaluates digital truth value.
    pub fn evaluate_binary(&self, bit_in: bool) -> bool {
        let in_wave = if bit_in {
            AcousticWave::binary_1(self.nominal_amplitude, self.carrier_frequency_hz)
        } else {
            AcousticWave::binary_0(self.carrier_frequency_hz)
        };
        let out_wave = self.evaluate(&in_wave);
        // Detection threshold set at 25% of nominal intensity
        let threshold = 0.25 * self.nominal_amplitude * self.nominal_amplitude;
        out_wave.to_binary(threshold)
    }
}

/// Acoustic OR gate combining acoustic energy from two input waveguides.
#[derive(Debug, Clone, PartialEq)]
pub struct AcousticOr {
    pub nominal_amplitude: f64,
    pub carrier_frequency_hz: f64,
}

impl AcousticOr {
    pub fn new(nominal_amplitude: f64, carrier_frequency_hz: f64) -> Self {
        Self {
            nominal_amplitude,
            carrier_frequency_hz,
        }
    }

    pub fn evaluate(&self, in_a: &AcousticWave, in_b: &AcousticWave) -> AcousticWave {
        // In-phase coupling
        in_a.superimpose(in_b)
    }

    pub fn evaluate_binary(&self, a: bool, b: bool) -> bool {
        let wa = if a {
            AcousticWave::binary_1(self.nominal_amplitude, self.carrier_frequency_hz)
        } else {
            AcousticWave::binary_0(self.carrier_frequency_hz)
        };
        let wb = if b {
            AcousticWave::binary_1(self.nominal_amplitude, self.carrier_frequency_hz)
        } else {
            AcousticWave::binary_0(self.carrier_frequency_hz)
        };
        let out = self.evaluate(&wa, &wb);
        let threshold = 0.25 * self.nominal_amplitude * self.nominal_amplitude;
        out.to_binary(threshold)
    }
}

/// Acoustic AND gate operating via constructive interference intensity thresholding.
/// $A=1, B=1 \implies I = (2 A_0)^2 = 4 A_0^2 > I_{th} = 2.5 A_0^2$.
#[derive(Debug, Clone, PartialEq)]
pub struct AcousticAnd {
    pub nominal_amplitude: f64,
    pub carrier_frequency_hz: f64,
}

impl AcousticAnd {
    pub fn new(nominal_amplitude: f64, carrier_frequency_hz: f64) -> Self {
        Self {
            nominal_amplitude,
            carrier_frequency_hz,
        }
    }

    pub fn evaluate(&self, in_a: &AcousticWave, in_b: &AcousticWave) -> AcousticWave {
        in_a.superimpose(in_b)
    }

    pub fn evaluate_binary(&self, a: bool, b: bool) -> bool {
        let wa = if a {
            AcousticWave::binary_1(self.nominal_amplitude, self.carrier_frequency_hz)
        } else {
            AcousticWave::binary_0(self.carrier_frequency_hz)
        };
        let wb = if b {
            AcousticWave::binary_1(self.nominal_amplitude, self.carrier_frequency_hz)
        } else {
            AcousticWave::binary_0(self.carrier_frequency_hz)
        };
        let out = self.evaluate(&wa, &wb);
        // Intensity threshold requires both in-phase waves to be present:
        // I(1, 0) = A_0^2, I(1, 1) = 4*A_0^2 -> Threshold = 2.5 * A_0^2
        let threshold = 2.5 * self.nominal_amplitude * self.nominal_amplitude;
        out.to_binary(threshold)
    }
}

/// Acoustic XOR gate operating via anti-phase interference.
#[derive(Debug, Clone, PartialEq)]
pub struct AcousticXor {
    pub nominal_amplitude: f64,
    pub carrier_frequency_hz: f64,
}

impl AcousticXor {
    pub fn new(nominal_amplitude: f64, carrier_frequency_hz: f64) -> Self {
        Self {
            nominal_amplitude,
            carrier_frequency_hz,
        }
    }

    pub fn evaluate(&self, in_a: &AcousticWave, in_b: &AcousticWave) -> AcousticWave {
        // Arm B is inverted (phase shifted by pi)
        let inverted_b = AcousticWave::new(
            in_b.amplitude,
            in_b.phase_rad + std::f64::consts::PI,
            in_b.frequency_hz,
        );
        in_a.superimpose(&inverted_b)
    }

    pub fn evaluate_binary(&self, a: bool, b: bool) -> bool {
        let wa = if a {
            AcousticWave::binary_1(self.nominal_amplitude, self.carrier_frequency_hz)
        } else {
            AcousticWave::binary_0(self.carrier_frequency_hz)
        };
        let wb = if b {
            AcousticWave::binary_1(self.nominal_amplitude, self.carrier_frequency_hz)
        } else {
            AcousticWave::binary_0(self.carrier_frequency_hz)
        };
        let out = self.evaluate(&wa, &wb);
        let threshold = 0.25 * self.nominal_amplitude * self.nominal_amplitude;
        out.to_binary(threshold)
    }
}

/// Phononic Full Adder synthesized entirely from interconnected acoustic wave logic gates.
#[derive(Debug, Clone, PartialEq)]
pub struct PhononicFullAdder {
    pub xor1: AcousticXor,
    pub xor2: AcousticXor,
    pub and1: AcousticAnd,
    pub and2: AcousticAnd,
    pub or1: AcousticOr,
    pub nominal_amplitude: f64,
    pub carrier_frequency_hz: f64,
}

impl PhononicFullAdder {
    pub fn new(nominal_amplitude: f64, carrier_frequency_hz: f64) -> Self {
        Self {
            xor1: AcousticXor::new(nominal_amplitude, carrier_frequency_hz),
            xor2: AcousticXor::new(nominal_amplitude, carrier_frequency_hz),
            and1: AcousticAnd::new(nominal_amplitude, carrier_frequency_hz),
            and2: AcousticAnd::new(nominal_amplitude, carrier_frequency_hz),
            or1: AcousticOr::new(nominal_amplitude, carrier_frequency_hz),
            nominal_amplitude,
            carrier_frequency_hz,
        }
    }

    /// Evaluates the 1-bit full adder: $(A, B, C_{in}) \to (\text{Sum}, C_{out})$.
    pub fn evaluate(&self, a: bool, b: bool, c_in: bool) -> (bool, bool) {
        // sum1 = A ^ B
        let sum1 = self.xor1.evaluate_binary(a, b);
        // Sum = sum1 ^ C_in
        let sum = self.xor2.evaluate_binary(sum1, c_in);

        // carry1 = A & B
        let carry1 = self.and1.evaluate_binary(a, b);
        // carry2 = sum1 & C_in
        let carry2 = self.and2.evaluate_binary(sum1, c_in);
        // C_out = carry1 | carry2
        let c_out = self.or1.evaluate_binary(carry1, carry2);

        (sum, c_out)
    }

    /// Verifies all 8 rows of the 1-bit full adder truth table.
    /// Returns true if all outputs are 100% correct.
    pub fn verify_truth_table(&self) -> bool {
        let test_cases = [
            // (A, B, Cin, Expected_Sum, Expected_Cout)
            (false, false, false, false, false),
            (false, false, true, true, false),
            (false, true, false, true, false),
            (false, true, true, false, true),
            (true, false, false, true, false),
            (true, false, true, false, true),
            (true, true, false, false, true),
            (true, true, true, true, true),
        ];

        for &(a, b, cin, exp_sum, exp_cout) in &test_cases {
            let (sum, cout) = self.evaluate(a, b, cin);
            if sum != exp_sum || cout != exp_cout {
                return false;
            }
        }
        true
    }

    /// Dynamic acoustic switching energy per operation (in Joules).
    /// Typically $\approx 25\text{ aJ}$ ($2.5 \times 10^{-17}\text{ J}$).
    #[inline]
    pub fn dynamic_energy_per_op_joules(&self) -> f64 {
        25.0e-18
    }

    /// Static standby power dissipation: identically zero ($0.0\text{ W}$)
    /// because no DC voltage is applied across semiconductor channels.
    #[inline]
    pub fn static_leakage_power_watts(&self) -> f64 {
        0.0
    }
}
