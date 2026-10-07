#![deny(unsafe_code)]

//! Hierarchical component category taxonomy for Phonon Studio CAD palette.

use super::components::ComponentKind;

/// Categorized taxonomy for grouping schematic components into structured drawers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentCategory {
    Passives,
    Sources,
    Discretes,
    Transistors,
    LogicGates,
    IntegratedCircuits,
    AnalogICs,
    Indicators,
    Sensors,
    TopologicalMetamaterials,
    PortHamiltonian,
}

impl ComponentCategory {
    /// Returns the human-readable display name of the category.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Passives => "Passive Elements & Controls",
            Self::Sources => "Sources & Clocks",
            Self::Discretes => "Discrete Semiconductors",
            Self::Transistors => "Transistors & Advanced FETs",
            Self::LogicGates => "Digital Logic Gates (Primitives)",
            Self::IntegratedCircuits => "Digital & Arithmetic ICs",
            Self::AnalogICs => "Analog & Mixed-Signal ICs",
            Self::Indicators => "Outputs & Indicators",
            Self::Sensors => "Sensors & Transducers",
            Self::TopologicalMetamaterials => "Topological Metamaterials",
            Self::PortHamiltonian => "Port-Hamiltonian Articulatory Acoustics",
        }
    }

    /// Returns a comprehensive description of the category.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Passives => "Linear, non-linear, and electromechanical passive components including resistors, potentiometers, capacitors, inductors, transformers, and switches.",
            Self::Sources => "Independent DC supplies, sinusoidal AC generators, constant current sources, pulse generators, clock generators, and supply rails.",
            Self::Discretes => "Two-terminal semiconductor diodes, zener references, optoelectronic LEDs, and Schottky barriers.",
            Self::Transistors => "Planar MOSFETs, 3D FinFETs, GAA nanosheets, and bipolar junction transistors (BJT).",
            Self::LogicGates => "Fundamental CMOS logic gates including Buffer, Inverter (NOT), AND, OR, NAND, NOR, XOR, and XNOR primitives.",
            Self::IntegratedCircuits => "High-level digital combinational and sequential building blocks including Half Adders, Full Adders, Multiplexers, Demultiplexers, Flip-Flops, Latches, and Counters.",
            Self::AnalogICs => "Operational amplifiers, voltage comparators, 555 precision timers, and monolithic linear voltage regulators.",
            Self::Indicators => "Visual logic probes, 7-segment display readouts, and acoustic buzzers for circuit observation.",
            Self::Sensors => "Piezoresistive strain gauges, tactile force matrices, and 9-DOF inertial measurement units.",
            Self::TopologicalMetamaterials => "Surface acoustic wave (SAW) transducers, Majorana braiding junctions, parafermionic cavities, and skyrmion routers.",
            Self::PortHamiltonian => "Biomechanical lungs subglottal drive, Hirano 3-layer vocal fold self-oscillation, Riccati Webster-horn acoustic tract, and spherical lip radiation impedance.",
        }
    }

    /// Returns all 11 canonical component categories in structured order.
    pub fn all_categories() -> &'static [ComponentCategory] {
        &[
            Self::Passives,
            Self::Sources,
            Self::Discretes,
            Self::Transistors,
            Self::LogicGates,
            Self::IntegratedCircuits,
            Self::AnalogICs,
            Self::Indicators,
            Self::Sensors,
            Self::TopologicalMetamaterials,
            Self::PortHamiltonian,
        ]
    }

    /// Returns the static slice of components belonging to this category.
    pub const fn component_slice(&self) -> &'static [ComponentKind] {
        match self {
            Self::Passives => &[
                ComponentKind::Resistor,
                ComponentKind::Potentiometer,
                ComponentKind::Capacitor,
                ComponentKind::Inductor,
                ComponentKind::Ground,
                ComponentKind::Transformer,
                ComponentKind::SwitchSpst,
                ComponentKind::PushButton,
            ],
            Self::Sources => &[
                ComponentKind::VoltageSource,
                ComponentKind::AcVoltageSource,
                ComponentKind::CurrentSource,
                ComponentKind::PulseGenerator,
                ComponentKind::ClockSource,
                ComponentKind::VddRail,
            ],
            Self::Discretes => &[
                ComponentKind::Diode,
                ComponentKind::ZenerDiode,
                ComponentKind::Led,
                ComponentKind::SchottkyDiode,
            ],
            Self::Transistors => &[
                ComponentKind::Nmos,
                ComponentKind::Pmos,
                ComponentKind::FinFet,
                ComponentKind::GaaNanosheet,
                ComponentKind::BjtNpn,
                ComponentKind::BjtPnp,
            ],
            Self::LogicGates => &[
                ComponentKind::BufferGate,
                ComponentKind::Inverter,
                ComponentKind::AndGate,
                ComponentKind::OrGate,
                ComponentKind::NandGate,
                ComponentKind::NorGate,
                ComponentKind::XorGate,
                ComponentKind::XnorGate,
            ],
            Self::IntegratedCircuits => &[
                ComponentKind::HalfAdder,
                ComponentKind::FullAdder,
                ComponentKind::Mux2to1,
                ComponentKind::Mux4to1,
                ComponentKind::Demux1to2,
                ComponentKind::DFlipFlop,
                ComponentKind::SrLatch,
                ComponentKind::Counter4Bit,
            ],
            Self::AnalogICs => &[
                ComponentKind::OpAmp,
                ComponentKind::Comparator,
                ComponentKind::Timer555,
                ComponentKind::VoltageRegulator,
            ],
            Self::Indicators => &[
                ComponentKind::LogicProbe,
                ComponentKind::SevenSegment,
                ComponentKind::Buzzer,
            ],
            Self::Sensors => &[
                ComponentKind::StrainGauge,
                ComponentKind::TactileMatrix,
                ComponentKind::Imu9Dof,
            ],
            Self::TopologicalMetamaterials => &[
                ComponentKind::SawIdt,
                ComponentKind::MajoranaJunction,
                ComponentKind::ParafermionicCavity,
                ComponentKind::SkyrmionRouter,
            ],
            Self::PortHamiltonian => &[
                ComponentKind::PhLungs,
                ComponentKind::PhVocalFolds,
                ComponentKind::PhVocalTract,
                ComponentKind::PhLipRadiation,
            ],
        }
    }

    /// Returns the list of components belonging to this category.
    pub fn components(&self) -> Vec<ComponentKind> {
        self.component_slice().to_vec()
    }
}
