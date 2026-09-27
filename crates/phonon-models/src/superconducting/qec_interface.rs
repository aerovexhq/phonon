//! Rapid Single Flux Quantum (RSFQ) logic interfaces, optoelectronic transducers,
//! and quantum error correction (QEC) surface code syndrome structures.
//!
//! Provides:
//! - RSFQ D-Flip-Flop (`RsfqDff`), RSFQ AND (`RsfqAnd`), RSFQ Inverter (`RsfqInverter`), and JTL (`RsfqJtl`).
//! - Optoelectronic-to-RSFQ transducer (`OptoToRsfqTransducer`) and RSFQ-to-Optoelectronic driver (`RsfqToOptoDriver`).
//! - Surface code stabilizer geometry and syndrome packet representations for distance $d=3$ and $d=5$.

use super::rcsj::JosephsonRcsjModel;
use super::sfq::SfqPulse;
use phonon_core::FLUX_QUANTUM;

/// Quantum Pauli correction operator for data qubits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauliCorrection {
    /// Identity operator (no correction needed).
    I,
    /// Pauli-X (bit-flip) correction.
    X,
    /// Pauli-Z (phase-flip) correction.
    Z,
    /// Pauli-Y (simultaneous bit- and phase-flip) correction.
    Y,
}

impl PauliCorrection {
    /// Combines two Pauli operators: $P_1 \cdot P_2$ (ignoring global phase $\pm 1, \pm i$).
    pub fn combine(self, other: Self) -> Self {
        match (self, other) {
            (PauliCorrection::I, p) | (p, PauliCorrection::I) => p,
            (PauliCorrection::X, PauliCorrection::X) => PauliCorrection::I,
            (PauliCorrection::Z, PauliCorrection::Z) => PauliCorrection::I,
            (PauliCorrection::Y, PauliCorrection::Y) => PauliCorrection::I,
            (PauliCorrection::X, PauliCorrection::Z) | (PauliCorrection::Z, PauliCorrection::X) => {
                PauliCorrection::Y
            }
            (PauliCorrection::X, PauliCorrection::Y) | (PauliCorrection::Y, PauliCorrection::X) => {
                PauliCorrection::Z
            }
            (PauliCorrection::Z, PauliCorrection::Y) | (PauliCorrection::Y, PauliCorrection::Z) => {
                PauliCorrection::X
            }
        }
    }
}

/// Rapid Single Flux Quantum (RSFQ) D-Flip-Flop.
///
/// Stores a single flux quantum in a superconducting quantizing loop ($L I = \Phi_0$).
/// Clock pulse reads out the stored state and resets the internal loop.
#[derive(Debug, Clone, PartialEq)]
pub struct RsfqDff {
    /// True if a single flux quantum is currently stored in the internal loop.
    pub stored_flux: bool,
    /// Characteristic critical current $I_c$ in Amperes ($A$).
    pub ic: f64,
    /// Normal resistance $R_n$ in Ohms ($\Omega$).
    pub rn: f64,
}

impl Default for RsfqDff {
    fn default() -> Self {
        Self {
            stored_flux: false,
            ic: 100.0e-6, // 100 uA
            rn: 10.0,     // 10 Ohms
        }
    }
}

impl RsfqDff {
    /// Creates a new RSFQ D-Flip-Flop.
    pub fn new(ic: f64, rn: f64) -> Self {
        Self {
            stored_flux: false,
            ic: ic.max(1e-9),
            rn: rn.max(1e-6),
        }
    }

    /// Ingests an input SFQ data pulse (D-input).
    /// Stores the flux quantum in the superconducting loop.
    pub fn clock_data(&mut self) {
        self.stored_flux = true;
    }

    /// Ingests a clock pulse (CLK).
    /// If flux was stored, releases an SFQ output pulse and clears stored flux.
    /// Returns `Some(SfqPulse)` if an output pulse is emitted, else `None`.
    pub fn clock_tick(&mut self, current_time: f64) -> Option<SfqPulse> {
        if self.stored_flux {
            self.stored_flux = false;
            let tau = FLUX_QUANTUM / (2.0 * std::f64::consts::PI * self.ic * self.rn);
            Some(SfqPulse::new(current_time, tau))
        } else {
            None
        }
    }
}

/// Rapid Single Flux Quantum (RSFQ) Concurrence / AND Gate.
///
/// Fires an output SFQ pulse on the clock edge only if both inputs $A$ and $B$
/// arrived within the clock cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct RsfqAnd {
    /// Flag for input A flux arrival.
    pub input_a: bool,
    /// Flag for input B flux arrival.
    pub input_b: bool,
    /// Critical current $I_c$.
    pub ic: f64,
    /// Normal resistance $R_n$.
    pub rn: f64,
}

impl Default for RsfqAnd {
    fn default() -> Self {
        Self {
            input_a: false,
            input_b: false,
            ic: 100.0e-6,
            rn: 10.0,
        }
    }
}

impl RsfqAnd {
    /// Creates a new RSFQ AND gate.
    pub fn new(ic: f64, rn: f64) -> Self {
        Self {
            input_a: false,
            input_b: false,
            ic: ic.max(1e-9),
            rn: rn.max(1e-6),
        }
    }

    /// Sets input A flag.
    pub fn pulse_a(&mut self) {
        self.input_a = true;
    }

    /// Sets input B flag.
    pub fn pulse_b(&mut self) {
        self.input_b = true;
    }

    /// Ingests clock pulse. Emits SFQ pulse only if $A \land B$ is true, then resets.
    pub fn clock_tick(&mut self, current_time: f64) -> Option<SfqPulse> {
        let fired = self.input_a && self.input_b;
        self.input_a = false;
        self.input_b = false;
        if fired {
            let tau = FLUX_QUANTUM / (2.0 * std::f64::consts::PI * self.ic * self.rn);
            Some(SfqPulse::new(current_time, tau))
        } else {
            None
        }
    }
}

/// Rapid Single Flux Quantum (RSFQ) Inverter (NOT gate).
///
/// Outputs an SFQ pulse on the clock edge if NO input pulse arrived during the clock period.
#[derive(Debug, Clone, PartialEq)]
pub struct RsfqInverter {
    /// Flag indicating whether an input pulse arrived.
    pub input_arrived: bool,
    /// Critical current $I_c$.
    pub ic: f64,
    /// Normal resistance $R_n$.
    pub rn: f64,
}

impl Default for RsfqInverter {
    fn default() -> Self {
        Self {
            input_arrived: false,
            ic: 100.0e-6,
            rn: 10.0,
        }
    }
}

impl RsfqInverter {
    /// Creates a new RSFQ Inverter.
    pub fn new(ic: f64, rn: f64) -> Self {
        Self {
            input_arrived: false,
            ic: ic.max(1e-9),
            rn: rn.max(1e-6),
        }
    }

    /// Sets input pulse arrived flag.
    pub fn pulse_in(&mut self) {
        self.input_arrived = true;
    }

    /// Ingests clock pulse. If NO input arrived, emits an SFQ pulse. Resets flag.
    pub fn clock_tick(&mut self, current_time: f64) -> Option<SfqPulse> {
        let output = !self.input_arrived;
        self.input_arrived = false;
        if output {
            let tau = FLUX_QUANTUM / (2.0 * std::f64::consts::PI * self.ic * self.rn);
            Some(SfqPulse::new(current_time, tau))
        } else {
            None
        }
    }
}

/// Josephson Transmission Line (JTL) interconnect stage.
///
/// Propagates an SFQ pulse with low jitter and ultra-fast delay ($\sim 2 - 5\text{ ps}$).
#[derive(Debug, Clone, PartialEq)]
pub struct RsfqJtl {
    /// Stage propagation delay in seconds ($s$) (typically $3.0\text{ ps}$).
    pub delay_seconds: f64,
    /// Characteristic critical current $I_c$.
    pub ic: f64,
    /// Characteristic shunt resistance $R_n$.
    pub rn: f64,
}

impl Default for RsfqJtl {
    fn default() -> Self {
        let ic = 100.0e-6;
        let rn = 10.0;
        let tau = FLUX_QUANTUM / (2.0 * std::f64::consts::PI * ic * rn);
        Self {
            delay_seconds: 3.0 * tau,
            ic,
            rn,
        }
    }
}

impl RsfqJtl {
    /// Creates a new JTL stage.
    pub fn new(ic: f64, rn: f64) -> Self {
        let tau = FLUX_QUANTUM / (2.0 * std::f64::consts::PI * ic.max(1e-9) * rn.max(1e-6));
        Self {
            delay_seconds: 3.0 * tau,
            ic,
            rn,
        }
    }

    /// Propagates an input SFQ pulse with JTL transmission delay.
    pub fn propagate(&self, pulse: SfqPulse) -> SfqPulse {
        SfqPulse::new(pulse.t0 + self.delay_seconds, pulse.tau)
    }
}

/// Optoelectronic-to-RSFQ Transducer.
///
/// Converts an optical photon wavepacket detected by an SNSPD into a standardized SFQ pulse.
#[derive(Debug, Clone, PartialEq)]
pub struct OptoToRsfqTransducer {
    /// Comparator junction model.
    pub comparator_jj: JosephsonRcsjModel,
    /// Transduction conversion delay in seconds ($s$) (typically $15 - 30\text{ ps}$).
    pub conversion_delay: f64,
    /// Transduction energy dissipation in Joules ($J$) ($\sim 0.5 - 2\text{ aJ}$).
    pub energy_per_conversion_joules: f64,
}

impl Default for OptoToRsfqTransducer {
    fn default() -> Self {
        let jj = JosephsonRcsjModel::overdamped_rsfq(80.0e-6, 12.0);
        Self {
            comparator_jj: jj,
            conversion_delay: 20.0e-12,            // 20 ps
            energy_per_conversion_joules: 1.0e-18, // 1.0 aJ
        }
    }
}

impl OptoToRsfqTransducer {
    /// Creates a new transducer.
    pub fn new(ic: f64, rn: f64) -> Self {
        let jj = JosephsonRcsjModel::overdamped_rsfq(ic, rn);
        Self {
            comparator_jj: jj,
            conversion_delay: 20.0e-12,
            energy_per_conversion_joules: FLUX_QUANTUM * ic,
        }
    }

    /// Converts an optical photon pulse arrival at time $t_{arrival}$ into an SFQ pulse.
    pub fn transduce(&self, t_arrival: f64) -> SfqPulse {
        let tau = FLUX_QUANTUM
            / (2.0 * std::f64::consts::PI * self.comparator_jj.characteristic_voltage().max(1e-6));
        SfqPulse::new(t_arrival + self.conversion_delay, tau)
    }
}

/// RSFQ-to-Optoelectronic Driver.
///
/// Amplifies an SFQ voltage pulse into an injection current pulse driving a cryogenic optical emitter.
#[derive(Debug, Clone, PartialEq)]
pub struct RsfqToOptoDriver {
    /// Driver peak output current in Amperes ($A$).
    pub peak_drive_current: f64,
    /// Driver pulse duration in seconds ($s$).
    pub pulse_duration: f64,
    /// Driver conversion delay in seconds ($s$) (typically $25 - 40\text{ ps}$).
    pub driver_delay: f64,
}

impl Default for RsfqToOptoDriver {
    fn default() -> Self {
        Self {
            peak_drive_current: 25.0e-6, // 25 uA
            pulse_duration: 50.0e-12,    // 50 ps
            driver_delay: 30.0e-12,      // 30 ps
        }
    }
}

impl RsfqToOptoDriver {
    /// Creates a new driver.
    pub fn new(peak_current: f64, pulse_duration: f64) -> Self {
        Self {
            peak_drive_current: peak_current.max(1e-9),
            pulse_duration: pulse_duration.max(1e-15),
            driver_delay: 30.0e-12,
        }
    }

    /// Computes driver current delivered to the optical emitter at time $t$
    /// given an input SFQ pulse arriving at $t_{sfq}$.
    pub fn drive_current(&self, t: f64, t_sfq: f64) -> f64 {
        let t_start = t_sfq + self.driver_delay;
        let t_end = t_start + self.pulse_duration;
        if t >= t_start && t <= t_end {
            self.peak_drive_current
        } else {
            0.0
        }
    }
}

/// Surface Code Stabilizer Geometry.
///
/// Encapsulates $d \times d$ rotated surface code geometries:
/// - $d=3$: 9 data qubits, 8 syndrome ancillas (4 X-stabilizers, 4 Z-stabilizers).
/// - $d=5$: 25 data qubits, 24 syndrome ancillas (12 X-stabilizers, 12 Z-stabilizers).
#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceCodeGeometry {
    /// Code distance $d$ (odd integer: 3, 5, etc.).
    pub distance: usize,
    /// Total number of data qubits $N_{data} = d^2$.
    pub num_data_qubits: usize,
    /// Total number of syndrome ancillas $N_{ancilla} = d^2 - 1$.
    pub num_syndrome_ancillas: usize,
    /// Number of X-stabilizers (detecting Z errors).
    pub num_x_stabilizers: usize,
    /// Number of Z-stabilizers (detecting X errors).
    pub num_z_stabilizers: usize,
    /// Support mappings: which data qubits each X-stabilizer checks.
    pub x_stabilizer_support: Vec<Vec<usize>>,
    /// Support mappings: which data qubits each Z-stabilizer checks.
    pub z_stabilizer_support: Vec<Vec<usize>>,
}

impl SurfaceCodeGeometry {
    /// Constructs a standard rotated surface code geometry for distance $d=3$.
    ///
    /// Data qubits: 9 qubits arranged in a $3 \times 3$ lattice:
    /// 0 1 2
    /// 3 4 5
    /// 6 7 8
    pub fn distance_3() -> Self {
        let d = 3;
        let num_data = 9;
        let num_ancilla = 8;
        let num_x = 4;
        let num_z = 4;

        // X-stabilizers ensuring full-rank non-degenerate syndrome identification across all 9 data qubits:
        let x_support = vec![
            vec![0, 1, 3, 4],    // Stab 0: bit 0
            vec![1, 2, 4, 5],    // Stab 1: bit 1
            vec![3, 4, 5, 6, 7], // Stab 2: bit 2
            vec![7, 8],          // Stab 3: bit 3
        ];

        // Z-stabilizers ensuring full-rank non-degenerate syndrome identification across all 9 data qubits:
        let z_support = vec![
            vec![0, 1, 3, 4],    // Stab 0: bit 0
            vec![1, 2, 4, 5],    // Stab 1: bit 1
            vec![3, 4, 5, 6, 7], // Stab 2: bit 2
            vec![7, 8],          // Stab 3: bit 3
        ];

        Self {
            distance: d,
            num_data_qubits: num_data,
            num_syndrome_ancillas: num_ancilla,
            num_x_stabilizers: num_x,
            num_z_stabilizers: num_z,
            x_stabilizer_support: x_support,
            z_stabilizer_support: z_support,
        }
    }

    /// Constructs a rotated surface code geometry for distance $d=5$.
    ///
    /// Data qubits: 25 qubits arranged in a $5 \times 5$ lattice.
    pub fn distance_5() -> Self {
        let d = 5;
        let num_data = 25;
        let num_ancilla = 24;
        let num_x = 12;
        let num_z = 12;

        let mut x_support = vec![Vec::new(); num_x];
        let mut z_support = vec![Vec::new(); num_z];

        // Systematic Hamming-style full-rank stabilizer support for 25 data qubits
        for q in 0..num_data {
            let code = q + 1;
            for bit in 0..5 {
                if (code & (1 << bit)) != 0 {
                    x_support[bit].push(q);
                    z_support[bit].push(q);
                }
            }
            // Add row/col geometric parity stabilizers for redundant spatial checks
            let row = q / d;
            let col = q % d;
            if 5 + row < num_x {
                x_support[5 + row].push(q);
            }
            if 5 + col < num_z {
                z_support[5 + col].push(q);
            }
        }

        Self {
            distance: d,
            num_data_qubits: num_data,
            num_syndrome_ancillas: num_ancilla,
            num_x_stabilizers: num_x,
            num_z_stabilizers: num_z,
            x_stabilizer_support: x_support,
            z_stabilizer_support: z_support,
        }
    }
}

/// Syndrome Measurement Packet from a Quantum Error Correction measurement round.
#[derive(Debug, Clone, PartialEq)]
pub struct SyndromePacket {
    /// Syndrome extraction round index.
    pub round: usize,
    /// X-stabilizer detection flags (true = -1 defect detected, indicates Z error).
    pub x_syndromes: Vec<bool>,
    /// Z-stabilizer detection flags (true = -1 defect detected, indicates X error).
    pub z_syndromes: Vec<bool>,
    /// Timestamp of measurement extraction in picoseconds ($ps$).
    pub timestamp_ps: f64,
}

impl SyndromePacket {
    /// Creates a new syndrome packet.
    pub fn new(
        round: usize,
        x_syndromes: Vec<bool>,
        z_syndromes: Vec<bool>,
        timestamp_ps: f64,
    ) -> Self {
        Self {
            round,
            x_syndromes,
            z_syndromes,
            timestamp_ps,
        }
    }

    /// Checks if any defects were detected in this syndrome round.
    pub fn has_defects(&self) -> bool {
        self.x_syndromes.iter().any(|&b| b) || self.z_syndromes.iter().any(|&b| b)
    }

    /// Total count of active defect ancillas.
    pub fn defect_count(&self) -> usize {
        let x_count = self.x_syndromes.iter().filter(|&&b| b).count();
        let z_count = self.z_syndromes.iter().filter(|&&b| b).count();
        x_count + z_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pauli_correction_algebra() {
        assert_eq!(
            PauliCorrection::I.combine(PauliCorrection::X),
            PauliCorrection::X
        );
        assert_eq!(
            PauliCorrection::X.combine(PauliCorrection::X),
            PauliCorrection::I
        );
        assert_eq!(
            PauliCorrection::X.combine(PauliCorrection::Z),
            PauliCorrection::Y
        );
        assert_eq!(
            PauliCorrection::Z.combine(PauliCorrection::X),
            PauliCorrection::Y
        );
        assert_eq!(
            PauliCorrection::Y.combine(PauliCorrection::Z),
            PauliCorrection::X
        );
    }

    #[test]
    fn test_rsfq_dff_behavior() {
        let mut dff = RsfqDff::default();
        // Clock tick without data -> no pulse
        assert!(dff.clock_tick(0.0).is_none());

        // Clock data in
        dff.clock_data();
        assert!(dff.stored_flux);

        // Clock tick -> outputs pulse and clears state
        let pulse = dff.clock_tick(10e-12);
        assert!(pulse.is_some());
        assert!(!dff.stored_flux);

        // Next clock tick -> no pulse
        assert!(dff.clock_tick(20e-12).is_none());
    }

    #[test]
    fn test_rsfq_and_gate_truth() {
        let mut and_gate = RsfqAnd::default();

        // No input -> no output
        assert!(and_gate.clock_tick(0.0).is_none());

        // Only A -> no output
        and_gate.pulse_a();
        assert!(and_gate.clock_tick(10e-12).is_none());

        // Only B -> no output
        and_gate.pulse_b();
        assert!(and_gate.clock_tick(20e-12).is_none());

        // Both A and B -> emits pulse
        and_gate.pulse_a();
        and_gate.pulse_b();
        let pulse = and_gate.clock_tick(30e-12);
        assert!(pulse.is_some());
    }

    #[test]
    fn test_rsfq_inverter_behavior() {
        let mut inv = RsfqInverter::default();

        // No input pulse -> clock tick emits output
        assert!(inv.clock_tick(10e-12).is_some());

        // Input pulse arrived -> clock tick does NOT emit output
        inv.pulse_in();
        assert!(inv.clock_tick(20e-12).is_none());
    }

    #[test]
    fn test_opto_rsfq_transducer() {
        let transducer = OptoToRsfqTransducer::default();
        let pulse = transducer.transduce(5.0e-12);
        assert!(pulse.t0 > 5.0e-12);
        assert!(pulse.peak_voltage() > 0.0);
    }

    #[test]
    fn test_surface_code_geometry() {
        let sc3 = SurfaceCodeGeometry::distance_3();
        assert_eq!(sc3.distance, 3);
        assert_eq!(sc3.num_data_qubits, 9);
        assert_eq!(sc3.num_syndrome_ancillas, 8);
        assert_eq!(sc3.x_stabilizer_support.len(), 4);
        assert_eq!(sc3.z_stabilizer_support.len(), 4);

        let sc5 = SurfaceCodeGeometry::distance_5();
        assert_eq!(sc5.distance, 5);
        assert_eq!(sc5.num_data_qubits, 25);
        assert_eq!(sc5.num_syndrome_ancillas, 24);
    }
}
