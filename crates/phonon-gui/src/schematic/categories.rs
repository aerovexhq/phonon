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
    IntegratedCircuits,
    Sensors,
    TopologicalMetamaterials,
}

impl ComponentCategory {
    /// Returns the human-readable display name of the category.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Passives => "Passive Elements",
            Self::Sources => "Sources & Generators",
            Self::Discretes => "Discrete Semiconductors",
            Self::Transistors => "Transistors & Advanced FETs",
            Self::IntegratedCircuits => "Integrated Circuits & Logic",
            Self::Sensors => "Sensors & Transducers",
            Self::TopologicalMetamaterials => "Topological Metamaterials",
        }
    }

    /// Returns a comprehensive description of the category.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Passives => "Linear and coupled passive electrical components including resistors, capacitors, inductors, and transformers.",
            Self::Sources => "Independent and programmable DC, AC, pulse, and current excitation sources.",
            Self::Discretes => "Two-terminal semiconductor diodes, zener references, optoelectronic LEDs, and Schottky barriers.",
            Self::Transistors => "Planar MOSFETs, 3D FinFETs, GAA nanosheets, and bipolar junction transistors (BJT).",
            Self::IntegratedCircuits => "Operational amplifiers, CMOS logic gates, and analog/digital multiplexers.",
            Self::Sensors => "Piezoresistive strain gauges, tactile force matrices, and 9-DOF inertial measurement units.",
            Self::TopologicalMetamaterials => "Surface acoustic wave (SAW) transducers, Majorana braiding junctions, parafermionic cavities, and skyrmion routers.",
        }
    }

    /// Returns all 7 canonical component categories in structured order.
    pub fn all_categories() -> &'static [ComponentCategory] {
        &[
            Self::Passives,
            Self::Sources,
            Self::Discretes,
            Self::Transistors,
            Self::IntegratedCircuits,
            Self::Sensors,
            Self::TopologicalMetamaterials,
        ]
    }

    /// Returns the static slice of components belonging to this category.
    pub const fn component_slice(&self) -> &'static [ComponentKind] {
        match self {
            Self::Passives => &[
                ComponentKind::Resistor,
                ComponentKind::Capacitor,
                ComponentKind::Inductor,
                ComponentKind::Ground,
                ComponentKind::Transformer,
            ],
            Self::Sources => &[
                ComponentKind::VoltageSource,
                ComponentKind::AcVoltageSource,
                ComponentKind::CurrentSource,
                ComponentKind::PulseGenerator,
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
            Self::IntegratedCircuits => &[
                ComponentKind::OpAmp,
                ComponentKind::Inverter,
                ComponentKind::NandGate,
                ComponentKind::NorGate,
                ComponentKind::Mux2to1,
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
        }
    }

    /// Returns the list of components belonging to this category.
    pub fn components(&self) -> Vec<ComponentKind> {
        self.component_slice().to_vec()
    }
}
