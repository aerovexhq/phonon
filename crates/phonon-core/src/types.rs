//! Core circuit primitives, strongly-typed identifiers, and engineering unit formatting.

use std::fmt;

/// Unique identifier for an electrical circuit node.
/// Node 0 (`NodeId::GROUND`) is reserved as the global circuit ground reference ($0\text{ V}$).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeId(pub u32);

impl NodeId {
    /// Global circuit reference node (Ground, $0.0\text{ V}$).
    pub const GROUND: Self = Self(0);

    /// Creates a new NodeId with the specified integer index.
    #[inline(always)]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns true if this node is the reference ground node.
    #[inline(always)]
    pub const fn is_ground(&self) -> bool {
        self.0 == 0
    }

    /// Returns the integer index of the node.
    #[inline(always)]
    pub const fn index(&self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_ground() {
            write!(f, "GND (0)")
        } else {
            write!(f, "Node({})", self.0)
        }
    }
}

/// Unique identifier for an auxiliary MNA branch current variable.
/// Assigned to voltage sources, inductors, and current-constrained branches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BranchId(pub u32);

impl BranchId {
    #[inline(always)]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    #[inline(always)]
    pub const fn index(&self) -> usize {
        self.0 as usize
    }
}

/// Unique identifier for an integrated photonic optical waveguide port.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpticalPortId(pub u32);

impl OpticalPortId {
    #[inline(always)]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    #[inline(always)]
    pub const fn index(&self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for OpticalPortId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OptPort({})", self.0)
    }
}

/// Optical signal state representing wavelength, optical power, and phase.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OpticalSignal {
    /// Vacuum wavelength $\lambda_0$ in meters ($m$) (e.g. $1.55\,\mu\text{m} = 1.55 \times 10^{-6}\text{ m}$).
    pub wavelength_m: f64,
    /// Optical power in Watts ($W$).
    pub power_watts: f64,
    /// Optical phase in radians ($rad$).
    pub phase_rad: f64,
}

impl OpticalSignal {
    pub fn new(wavelength_m: f64, power_watts: f64, phase_rad: f64) -> Self {
        Self {
            wavelength_m,
            power_watts,
            phase_rad,
        }
    }

    /// Optical frequency $\nu = c / \lambda_0$ in Hertz ($Hz$).
    pub fn frequency_hz(&self) -> f64 {
        crate::SPEED_OF_LIGHT / self.wavelength_m.max(1e-12)
    }

    /// Complex electric field envelope $E = \sqrt{P} e^{i \phi}$ in $\sqrt{W}$.
    pub fn complex_field(&self) -> (f64, f64) {
        let amp = self.power_watts.max(0.0).sqrt();
        (amp * self.phase_rad.cos(), amp * self.phase_rad.sin())
    }

    /// Optical power in dBm: $P_{\text{dBm}} = 10 \log_{10}(P / 1\text{ mW})$.
    pub fn power_dbm(&self) -> f64 {
        if self.power_watts <= 1e-18 {
            -150.0
        } else {
            10.0 * (self.power_watts / 1e-3).log10()
        }
    }
}

impl fmt::Display for BranchId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Branch({})", self.0)
    }
}

/// Unique identifier for an instantiated circuit component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComponentId(pub u32);

impl ComponentId {
    #[inline(always)]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    #[inline(always)]
    pub const fn index(&self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for ComponentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Comp({})", self.0)
    }
}

/// Helper function to format electrical values with engineering SI prefixes.
/// E.g. `format_si(0.0012, "A")` -> `"1.200 mA"`.
pub fn format_si(val: f64, unit: &str) -> String {
    if val == 0.0 {
        return format!("0.000 {}", unit);
    }
    let abs_val = val.abs();
    let sign = if val < 0.0 { "-" } else { "" };

    if abs_val >= 1e12 {
        format!("{}{:.3} T{}", sign, abs_val / 1e12, unit)
    } else if abs_val >= 1e9 {
        format!("{}{:.3} G{}", sign, abs_val / 1e9, unit)
    } else if abs_val >= 1e6 {
        format!("{}{:.3} M{}", sign, abs_val / 1e6, unit)
    } else if abs_val >= 1e3 {
        format!("{}{:.3} k{}", sign, abs_val / 1e3, unit)
    } else if abs_val >= 1.0 {
        format!("{}{:.3} {}", sign, abs_val, unit)
    } else if abs_val >= 1e-3 {
        format!("{}{:.3} m{}", sign, abs_val * 1e3, unit)
    } else if abs_val >= 1e-6 {
        format!("{}{:.3} u{}", sign, abs_val * 1e6, unit)
    } else if abs_val >= 1e-9 {
        format!("{}{:.3} n{}", sign, abs_val * 1e9, unit)
    } else if abs_val >= 1e-12 {
        format!("{}{:.3} p{}", sign, abs_val * 1e12, unit)
    } else {
        format!("{}{:.3} f{}", sign, abs_val * 1e15, unit)
    }
}

/// Physical state of a non-volatile memristive device.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MemristorState {
    /// Instantaneous small-signal or chord conductance in Siemens ($S$).
    pub conductance_s: f64,
    /// Normalized internal state variable $w \in [0, 1]$ (e.g. filament radius or length fraction).
    pub internal_state_w: f64,
}

impl MemristorState {
    pub const fn new(conductance_s: f64, internal_state_w: f64) -> Self {
        Self {
            conductance_s,
            internal_state_w,
        }
    }

    /// Instantaneous resistance in Ohms ($\Omega$).
    #[inline(always)]
    pub fn resistance_ohms(&self) -> f64 {
        1.0 / self.conductance_s.max(1e-15)
    }
}

/// Spiking event emitted by a biological or neuromorphic integrate-and-fire neuron.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NeuronSpike {
    /// Simulation timestamp when the action potential crossed threshold in seconds ($s$).
    pub time_s: f64,
    /// Identifier or index of the firing neuron.
    pub neuron_id: u32,
    /// Peak spike potential amplitude in Volts ($V$).
    pub amplitude_v: f64,
}

impl NeuronSpike {
    pub const fn new(time_s: f64, neuron_id: u32, amplitude_v: f64) -> Self {
        Self {
            time_s,
            neuron_id,
            amplitude_v,
        }
    }
}

/// Single-event radiation strike event description (heavy-ion or proton track).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RadiationEvent {
    /// Unique event identifier.
    pub event_id: u64,
    /// Effective Linear Energy Transfer (LET) in $\text{MeV}\cdot\text{cm}^2/\text{mg}$.
    pub let_mev_cm2_mg: f64,
    /// Time of particle impact in seconds ($s$).
    pub strike_time_s: f64,
    /// Target sensitive electrical node where charge is deposited.
    pub sensitive_node: NodeId,
    /// Total deposited charge in Coulombs ($C$).
    pub total_charge_c: f64,
}

impl RadiationEvent {
    pub const fn new(
        event_id: u64,
        let_mev_cm2_mg: f64,
        strike_time_s: f64,
        sensitive_node: NodeId,
        total_charge_c: f64,
    ) -> Self {
        Self {
            event_id,
            let_mev_cm2_mg,
            strike_time_s,
            sensitive_node,
            total_charge_c,
        }
    }
}

/// Triple-Modular Redundancy (TMR) majority voting resolution.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TmrOutput {
    /// Majority voted output voltage in Volts ($V$).
    pub voted_output: f64,
    /// Index of disagreeing channel $(0, 1, 2)$, if a single-event upset was masked.
    pub disagreeing_channel: Option<usize>,
}

impl TmrOutput {
    pub const fn new(voted_output: f64, disagreeing_channel: Option<usize>) -> Self {
        Self {
            voted_output,
            disagreeing_channel,
        }
    }

    /// Returns true if a disagreeing channel was detected and masked by majority vote.
    #[inline(always)]
    pub fn fault_detected(&self) -> bool {
        self.disagreeing_channel.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_properties() {
        assert!(NodeId::GROUND.is_ground());
        assert_eq!(NodeId::GROUND.index(), 0);

        let n1 = NodeId::new(42);
        assert!(!n1.is_ground());
        assert_eq!(n1.index(), 42);
    }

    #[test]
    fn test_format_si() {
        assert_eq!(format_si(1000.0, "V"), "1.000 kV");
        assert_eq!(format_si(0.005, "A"), "5.000 mA");
        assert_eq!(format_si(2.2e-6, "F"), "2.200 uF");
        assert_eq!(format_si(-1.5e-9, "s"), "-1.500 ns");
    }
}
