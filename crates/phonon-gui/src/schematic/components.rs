#![deny(unsafe_code)]

//! Visual schematic components, pin geometries, symbol rendering, and hit testing.

use super::canvas::SchematicCanvas;
use super::categories::ComponentCategory;
use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, StrokeKind, Vec2};

/// The electrical or physical device type of a visual schematic component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentKind {
    // Passive Elements
    Resistor,
    Capacitor,
    Inductor,
    Ground,
    Transformer,

    // Sources & Generators
    VoltageSource,
    AcVoltageSource,
    CurrentSource,
    PulseGenerator,

    // Discrete Semiconductors
    Diode,
    ZenerDiode,
    Led,
    SchottkyDiode,

    // Transistors & Cryo-CMOS
    Nmos,
    Pmos,
    FinFet,
    GaaNanosheet,
    BjtNpn,
    BjtPnp,

    // Integrated Circuits & Logic
    OpAmp,
    Inverter,
    NandGate,
    NorGate,
    Mux2to1,

    // Sensors & Transducers
    StrainGauge,
    TactileMatrix,
    Imu9Dof,

    // Topological & Quantum Metamaterials
    SawIdt,
    MajoranaJunction,
    ParafermionicCavity,
    SkyrmionRouter,
}

impl ComponentKind {
    /// Returns the hierarchical category this component belongs to.
    pub fn category(&self) -> ComponentCategory {
        match self {
            Self::Resistor | Self::Capacitor | Self::Inductor | Self::Ground | Self::Transformer => {
                ComponentCategory::Passives
            }
            Self::VoltageSource
            | Self::AcVoltageSource
            | Self::CurrentSource
            | Self::PulseGenerator => ComponentCategory::Sources,
            Self::Diode | Self::ZenerDiode | Self::Led | Self::SchottkyDiode => {
                ComponentCategory::Discretes
            }
            Self::Nmos
            | Self::Pmos
            | Self::FinFet
            | Self::GaaNanosheet
            | Self::BjtNpn
            | Self::BjtPnp => ComponentCategory::Transistors,
            Self::OpAmp | Self::Inverter | Self::NandGate | Self::NorGate | Self::Mux2to1 => {
                ComponentCategory::IntegratedCircuits
            }
            Self::StrainGauge | Self::TactileMatrix | Self::Imu9Dof => ComponentCategory::Sensors,
            Self::SawIdt
            | Self::MajoranaJunction
            | Self::ParafermionicCavity
            | Self::SkyrmionRouter => ComponentCategory::TopologicalMetamaterials,
        }
    }

    /// Returns the static variant identifier string.
    pub fn code_name(&self) -> &'static str {
        match self {
            Self::Resistor => "Resistor",
            Self::Capacitor => "Capacitor",
            Self::Inductor => "Inductor",
            Self::Ground => "Ground",
            Self::Transformer => "Transformer",
            Self::VoltageSource => "VoltageSource",
            Self::AcVoltageSource => "AcVoltageSource",
            Self::CurrentSource => "CurrentSource",
            Self::PulseGenerator => "PulseGenerator",
            Self::Diode => "Diode",
            Self::ZenerDiode => "ZenerDiode",
            Self::Led => "Led",
            Self::SchottkyDiode => "SchottkyDiode",
            Self::Nmos => "Nmos",
            Self::Pmos => "Pmos",
            Self::FinFet => "FinFet",
            Self::GaaNanosheet => "GaaNanosheet",
            Self::BjtNpn => "BjtNpn",
            Self::BjtPnp => "BjtPnp",
            Self::OpAmp => "OpAmp",
            Self::Inverter => "Inverter",
            Self::NandGate => "NandGate",
            Self::NorGate => "NorGate",
            Self::Mux2to1 => "Mux2to1",
            Self::StrainGauge => "StrainGauge",
            Self::TactileMatrix => "TactileMatrix",
            Self::Imu9Dof => "Imu9Dof",
            Self::SawIdt => "SawIdt",
            Self::MajoranaJunction => "MajoranaJunction",
            Self::ParafermionicCavity => "ParafermionicCavity",
            Self::SkyrmionRouter => "SkyrmionRouter",
        }
    }

    /// All 31 categorized component kinds in static array.
    pub const ALL: &'static [Self] = &[
        Self::Resistor,
        Self::Capacitor,
        Self::Inductor,
        Self::Ground,
        Self::Transformer,
        Self::VoltageSource,
        Self::AcVoltageSource,
        Self::CurrentSource,
        Self::PulseGenerator,
        Self::Diode,
        Self::ZenerDiode,
        Self::Led,
        Self::SchottkyDiode,
        Self::Nmos,
        Self::Pmos,
        Self::FinFet,
        Self::GaaNanosheet,
        Self::BjtNpn,
        Self::BjtPnp,
        Self::OpAmp,
        Self::Inverter,
        Self::NandGate,
        Self::NorGate,
        Self::Mux2to1,
        Self::StrainGauge,
        Self::TactileMatrix,
        Self::Imu9Dof,
        Self::SawIdt,
        Self::MajoranaJunction,
        Self::ParafermionicCavity,
        Self::SkyrmionRouter,
    ];

    /// Alias for ALL variants.
    pub const ALL_VARIANTS: &'static [Self] = Self::ALL;

    /// Precomputed lowercase search index string for zero-allocation query filtering.
    pub fn search_index(&self) -> &'static str {
        match self {
            Self::Resistor => "resistor resistor passive elements linear passive two-terminal electrical resistance element r",
            Self::Capacitor => "capacitor capacitor passive elements electrostatic charge and energy storage dielectric device c",
            Self::Inductor => "inductor inductor passive elements magnetic flux storage coil element l",
            Self::Ground => "ground ground passive elements reference potential net 0v reference ground datum gnd 0",
            Self::Transformer => "transformer transformer passive elements coupled mutual inductance dual-winding transformer tx",
            Self::VoltageSource => "dc voltage source voltagesource sources & generators constant dc electromotive force voltage supply v",
            Self::AcVoltageSource => "ac voltage source acvoltagesource sources & generators sinusoidal ac alternating electromotive voltage supply vac",
            Self::CurrentSource => "current source currentsource sources & generators constant electrical current supply i",
            Self::PulseGenerator => "pulse generator pulsegenerator sources & generators repetitive pulse and clock signal generator vpulse",
            Self::Diode => "diode diode discrete semiconductors standard semiconductor p-n junction rectifier diode d",
            Self::ZenerDiode => "zener diode zenerdiode discrete semiconductors precision reverse breakdown voltage reference zener diode dz",
            Self::Led => "led led discrete semiconductors optoelectronic light emitting semiconductor diode led",
            Self::SchottkyDiode => "schottky diode schottkydiode discrete semiconductors low forward drop metal-semiconductor barrier diode ds",
            Self::Nmos => "nmos transistor nmos transistors & advanced fets n-channel enhancement metal-oxide-semiconductor field-effect transistor m",
            Self::Pmos => "pmos transistor pmos transistors & advanced fets p-channel enhancement metal-oxide-semiconductor field-effect transistor m",
            Self::FinFet => "finfet (tri-gate) finfet transistors & advanced fets 3d multi-gate finfet device for nanoscale subthreshold control xfin",
            Self::GaaNanosheet => "gaa nanosheet fet gaananosheet transistors & advanced fets gate-all-around multi-channel nanosheet field-effect transistor xgaa",
            Self::BjtNpn => "npn bjt bjtnpn transistors & advanced fets npn bipolar junction amplifying and switching transistor q",
            Self::BjtPnp => "pnp bjt bjtpnp transistors & advanced fets pnp bipolar junction amplifying and switching transistor q",
            Self::OpAmp => "operational amplifier opamp integrated circuits & logic high-gain differential voltage operational amplifier xop",
            Self::Inverter => "inverter (not gate) inverter integrated circuits & logic cmos logic inverter not gate xinv",
            Self::NandGate => "nand gate nandgate integrated circuits & logic dual-input universal cmos nand logic gate xnand",
            Self::NorGate => "nor gate norgate integrated circuits & logic dual-input universal cmos nor logic gate xnor",
            Self::Mux2to1 => "2:1 multiplexer mux2to1 integrated circuits & logic 2-to-1 binary data multiplexer xmux",
            Self::StrainGauge => "piezo strain gauge straingauge sensors & transducers piezoresistive acoustic strain sensor bridge xsg",
            Self::TactileMatrix => "tactile force matrix tactilematrix sensors & transducers piezotronic tactile force pressure sensor array xtm",
            Self::Imu9Dof => "9-dof imu transducer imu9dof sensors & transducers nine-degree-of-freedom inertial measurement transducer ximu",
            Self::SawIdt => "saw idt filter sawidt topological metamaterials surface acoustic wave interdigital transducer acoustic filter xsaw",
            Self::MajoranaJunction => "majorana braiding junction majoranajunction topological metamaterials non-abelian topological quantum braiding junction xmj",
            Self::ParafermionicCavity => "parafermionic cavity parafermioniccavity topological metamaterials fractionalized topological quantum acoustic resonator cavity xpc",
            Self::SkyrmionRouter => "skyrmion router skyrmionrouter topological metamaterials chiral magnetic skyrmion topological acoustic wave router xsr",
        }
    }

    /// Returns human-readable display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Resistor => "Resistor",
            Self::Capacitor => "Capacitor",
            Self::Inductor => "Inductor",
            Self::Ground => "Ground",
            Self::Transformer => "Transformer",
            Self::VoltageSource => "DC Voltage Source",
            Self::AcVoltageSource => "AC Voltage Source",
            Self::CurrentSource => "Current Source",
            Self::PulseGenerator => "Pulse Generator",
            Self::Diode => "Diode",
            Self::ZenerDiode => "Zener Diode",
            Self::Led => "LED",
            Self::SchottkyDiode => "Schottky Diode",
            Self::Nmos => "NMOS Transistor",
            Self::Pmos => "PMOS Transistor",
            Self::FinFet => "FinFET (Tri-Gate)",
            Self::GaaNanosheet => "GAA Nanosheet FET",
            Self::BjtNpn => "NPN BJT",
            Self::BjtPnp => "PNP BJT",
            Self::OpAmp => "Operational Amplifier",
            Self::Inverter => "Inverter (NOT Gate)",
            Self::NandGate => "NAND Gate",
            Self::NorGate => "NOR Gate",
            Self::Mux2to1 => "2:1 Multiplexer",
            Self::StrainGauge => "Piezo Strain Gauge",
            Self::TactileMatrix => "Tactile Force Matrix",
            Self::Imu9Dof => "9-DOF IMU Transducer",
            Self::SawIdt => "SAW IDT Filter",
            Self::MajoranaJunction => "Majorana Braiding Junction",
            Self::ParafermionicCavity => "Parafermionic Cavity",
            Self::SkyrmionRouter => "Skyrmion Router",
        }
    }

    /// Returns descriptive summary for live search indexing.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Resistor => "Linear passive two-terminal electrical resistance element",
            Self::Capacitor => "Electrostatic charge and energy storage dielectric device",
            Self::Inductor => "Magnetic flux storage coil element",
            Self::Ground => "Reference potential net (0V reference ground datum)",
            Self::Transformer => "Coupled mutual inductance dual-winding transformer",
            Self::VoltageSource => "Constant DC electromotive force voltage supply",
            Self::AcVoltageSource => "Sinusoidal AC alternating electromotive voltage supply",
            Self::CurrentSource => "Constant electrical current supply",
            Self::PulseGenerator => "Repetitive pulse and clock signal generator",
            Self::Diode => "Standard semiconductor p-n junction rectifier diode",
            Self::ZenerDiode => "Precision reverse breakdown voltage reference zener diode",
            Self::Led => "Optoelectronic light emitting semiconductor diode",
            Self::SchottkyDiode => "Low forward drop metal-semiconductor barrier diode",
            Self::Nmos => "N-channel enhancement metal-oxide-semiconductor field-effect transistor",
            Self::Pmos => "P-channel enhancement metal-oxide-semiconductor field-effect transistor",
            Self::FinFet => "3D multi-gate FinFET device for nanoscale subthreshold control",
            Self::GaaNanosheet => "Gate-all-around multi-channel nanosheet field-effect transistor",
            Self::BjtNpn => "NPN bipolar junction amplifying and switching transistor",
            Self::BjtPnp => "PNP bipolar junction amplifying and switching transistor",
            Self::OpAmp => "High-gain differential voltage operational amplifier",
            Self::Inverter => "CMOS logic inverter NOT gate",
            Self::NandGate => "Dual-input universal CMOS NAND logic gate",
            Self::NorGate => "Dual-input universal CMOS NOR logic gate",
            Self::Mux2to1 => "2-to-1 binary data multiplexer",
            Self::StrainGauge => "Piezoresistive acoustic strain sensor bridge",
            Self::TactileMatrix => "Piezotronic tactile force pressure sensor array",
            Self::Imu9Dof => "Nine-degree-of-freedom inertial measurement transducer",
            Self::SawIdt => "Surface acoustic wave interdigital transducer acoustic filter",
            Self::MajoranaJunction => "Non-Abelian topological quantum braiding junction",
            Self::ParafermionicCavity => "Fractionalized topological quantum acoustic resonator cavity",
            Self::SkyrmionRouter => "Chiral magnetic skyrmion topological acoustic wave router",
        }
    }

    /// Returns standard SPICE / schematic reference designator prefix.
    pub fn prefix(&self) -> &'static str {
        match self {
            Self::Resistor => "R",
            Self::Capacitor => "C",
            Self::Inductor => "L",
            Self::Ground => "GND",
            Self::Transformer => "TX",
            Self::VoltageSource => "V",
            Self::AcVoltageSource => "VAC",
            Self::CurrentSource => "I",
            Self::PulseGenerator => "VPULSE",
            Self::Diode => "D",
            Self::ZenerDiode => "DZ",
            Self::Led => "LED",
            Self::SchottkyDiode => "DS",
            Self::Nmos => "M",
            Self::Pmos => "M",
            Self::FinFet => "XFIN",
            Self::GaaNanosheet => "XGAA",
            Self::BjtNpn => "Q",
            Self::BjtPnp => "Q",
            Self::OpAmp => "XOP",
            Self::Inverter => "XINV",
            Self::NandGate => "XNAND",
            Self::NorGate => "XNOR",
            Self::Mux2to1 => "XMUX",
            Self::StrainGauge => "XSG",
            Self::TactileMatrix => "XTM",
            Self::Imu9Dof => "XIMU",
            Self::SawIdt => "XSAW",
            Self::MajoranaJunction => "XMJ",
            Self::ParafermionicCavity => "XPC",
            Self::SkyrmionRouter => "XSR",
        }
    }

    /// Returns default schematic parameter value string.
    pub fn default_value(&self) -> &'static str {
        match self {
            Self::Resistor => "1k",
            Self::Capacitor => "100n",
            Self::Inductor => "10u",
            Self::Ground => "0",
            Self::Transformer => "1:1",
            Self::VoltageSource => "5.0",
            Self::AcVoltageSource => "SIN(0 1 1k)",
            Self::CurrentSource => "1m",
            Self::PulseGenerator => "PULSE(0 5 0 1n 1n 10u 20u)",
            Self::Diode => "1N4148",
            Self::ZenerDiode => "BZX84C5V1",
            Self::Led => "RED",
            Self::SchottkyDiode => "1N5819",
            Self::Nmos => "NMOS_MOD",
            Self::Pmos => "PMOS_MOD",
            Self::FinFet => "FINFET_3NM",
            Self::GaaNanosheet => "GAA_2NM",
            Self::BjtNpn => "2N2222",
            Self::BjtPnp => "2N3906",
            Self::OpAmp => "OPAMP_IDEAL",
            Self::Inverter => "INV_CMOS",
            Self::NandGate => "NAND2",
            Self::NorGate => "NOR2",
            Self::Mux2to1 => "MUX21",
            Self::StrainGauge => "STRAIN_350",
            Self::TactileMatrix => "TACTILE_8X8",
            Self::Imu9Dof => "IMU_6DOF_9DOF",
            Self::SawIdt => "SAW_1GHZ",
            Self::MajoranaJunction => "TOPOMAJ_1",
            Self::ParafermionicCavity => "PARAFERM_RES",
            Self::SkyrmionRouter => "SKYRMION_RT",
        }
    }

    /// Returns the local pin offsets relative to component origin (at rotation 0).
    pub fn pin_definitions(&self) -> Vec<(&'static str, Vec2)> {
        match self {
            Self::Resistor | Self::Capacitor | Self::Inductor => {
                vec![("1", Vec2::new(0.0, -40.0)), ("2", Vec2::new(0.0, 40.0))]
            }
            Self::Ground => vec![("GND", Vec2::new(0.0, -20.0))],
            Self::Transformer => vec![
                ("P+", Vec2::new(-30.0, -30.0)),
                ("P-", Vec2::new(-30.0, 30.0)),
                ("S+", Vec2::new(30.0, -30.0)),
                ("S-", Vec2::new(30.0, 30.0)),
            ],
            Self::VoltageSource
            | Self::AcVoltageSource
            | Self::CurrentSource
            | Self::PulseGenerator => {
                vec![("+", Vec2::new(0.0, -40.0)), ("-", Vec2::new(0.0, 40.0))]
            }
            Self::Diode | Self::ZenerDiode | Self::Led | Self::SchottkyDiode => {
                vec![("A", Vec2::new(0.0, -40.0)), ("K", Vec2::new(0.0, 40.0))]
            }
            Self::Nmos | Self::Pmos => vec![
                ("D", Vec2::new(20.0, -40.0)),
                ("G", Vec2::new(-20.0, 0.0)),
                ("S", Vec2::new(20.0, 40.0)),
            ],
            Self::FinFet | Self::GaaNanosheet => vec![
                ("D", Vec2::new(20.0, -40.0)),
                ("G", Vec2::new(-20.0, 0.0)),
                ("S", Vec2::new(20.0, 40.0)),
                ("B", Vec2::new(20.0, 0.0)),
            ],
            Self::BjtNpn | Self::BjtPnp => vec![
                ("C", Vec2::new(20.0, -40.0)),
                ("B", Vec2::new(-20.0, 0.0)),
                ("E", Vec2::new(20.0, 40.0)),
            ],
            Self::OpAmp => vec![
                ("IN+", Vec2::new(-40.0, -15.0)),
                ("IN-", Vec2::new(-40.0, 15.0)),
                ("OUT", Vec2::new(40.0, 0.0)),
                ("V+", Vec2::new(0.0, -30.0)),
                ("V-", Vec2::new(0.0, 30.0)),
            ],
            Self::Inverter => vec![
                ("IN", Vec2::new(-30.0, 0.0)),
                ("OUT", Vec2::new(30.0, 0.0)),
            ],
            Self::NandGate | Self::NorGate => vec![
                ("A", Vec2::new(-30.0, -15.0)),
                ("B", Vec2::new(-30.0, 15.0)),
                ("OUT", Vec2::new(30.0, 0.0)),
            ],
            Self::Mux2to1 => vec![
                ("D0", Vec2::new(-30.0, -15.0)),
                ("D1", Vec2::new(-30.0, 15.0)),
                ("SEL", Vec2::new(0.0, 30.0)),
                ("OUT", Vec2::new(30.0, 0.0)),
            ],
            Self::StrainGauge => vec![
                ("P+", Vec2::new(0.0, -30.0)),
                ("P-", Vec2::new(0.0, 30.0)),
            ],
            Self::TactileMatrix => vec![
                ("R+", Vec2::new(-30.0, 0.0)),
                ("R-", Vec2::new(30.0, 0.0)),
                ("C+", Vec2::new(0.0, -30.0)),
                ("C-", Vec2::new(0.0, 30.0)),
            ],
            Self::Imu9Dof => vec![
                ("VDD", Vec2::new(-30.0, -20.0)),
                ("GND", Vec2::new(-30.0, 20.0)),
                ("SCL", Vec2::new(30.0, -20.0)),
                ("SDA", Vec2::new(30.0, 20.0)),
            ],
            Self::SawIdt => vec![
                ("IN+", Vec2::new(-30.0, -20.0)),
                ("IN-", Vec2::new(-30.0, 20.0)),
                ("OUT+", Vec2::new(30.0, -20.0)),
                ("OUT-", Vec2::new(30.0, 20.0)),
            ],
            Self::MajoranaJunction => vec![
                ("J1", Vec2::new(-30.0, -20.0)),
                ("J2", Vec2::new(-30.0, 20.0)),
                ("J3", Vec2::new(30.0, 0.0)),
            ],
            Self::ParafermionicCavity => vec![
                ("PORT1", Vec2::new(-30.0, 0.0)),
                ("PORT2", Vec2::new(30.0, 0.0)),
            ],
            Self::SkyrmionRouter => vec![
                ("IN", Vec2::new(-30.0, 0.0)),
                ("CH0", Vec2::new(30.0, -20.0)),
                ("CH1", Vec2::new(30.0, 20.0)),
                ("GATE", Vec2::new(0.0, -30.0)),
            ],
        }
    }

    /// Renders geometric 2D strokes representing this component symbol.
    pub fn draw_symbol(
        &self,
        painter: &Painter,
        stroke: Stroke,
        to_screen: &dyn Fn(f32, f32) -> Pos2,
        zoom: f32,
    ) {
        match self {
            Self::Resistor => {
                let pts = [
                    to_screen(0.0, -40.0),
                    to_screen(0.0, -20.0),
                    to_screen(-8.0, -15.0),
                    to_screen(8.0, -5.0),
                    to_screen(-8.0, 5.0),
                    to_screen(8.0, 15.0),
                    to_screen(0.0, 20.0),
                    to_screen(0.0, 40.0),
                ];
                for i in 0..pts.len() - 1 {
                    painter.line_segment([pts[i], pts[i + 1]], stroke);
                }
            }
            Self::Capacitor => {
                painter.line_segment([to_screen(0.0, -40.0), to_screen(0.0, -6.0)], stroke);
                painter.line_segment([to_screen(-14.0, -6.0), to_screen(14.0, -6.0)], stroke);
                painter.line_segment([to_screen(-14.0, 6.0), to_screen(14.0, 6.0)], stroke);
                painter.line_segment([to_screen(0.0, 6.0), to_screen(0.0, 40.0)], stroke);
            }
            Self::Inductor => {
                painter.line_segment([to_screen(0.0, -40.0), to_screen(0.0, -20.0)], stroke);
                let arcs = [(-10.0, 10.0), (0.0, 10.0), (10.0, 10.0)];
                for &(cy, r) in &arcs {
                    let center = to_screen(0.0, cy);
                    painter.circle_stroke(center, r * zoom, stroke);
                }
                painter.line_segment([to_screen(0.0, 20.0), to_screen(0.0, 40.0)], stroke);
            }
            Self::Transformer => {
                // Primary coil (left)
                painter.line_segment([to_screen(-30.0, -30.0), to_screen(-15.0, -20.0)], stroke);
                painter.circle_stroke(to_screen(-15.0, -10.0), 8.0 * zoom, stroke);
                painter.circle_stroke(to_screen(-15.0, 10.0), 8.0 * zoom, stroke);
                painter.line_segment([to_screen(-15.0, 20.0), to_screen(-30.0, 30.0)], stroke);
                // Magnetic core bars (middle)
                painter.line_segment([to_screen(-3.0, -25.0), to_screen(-3.0, 25.0)], stroke);
                painter.line_segment([to_screen(3.0, -25.0), to_screen(3.0, 25.0)], stroke);
                // Secondary coil (right)
                painter.line_segment([to_screen(30.0, -30.0), to_screen(15.0, -20.0)], stroke);
                painter.circle_stroke(to_screen(15.0, -10.0), 8.0 * zoom, stroke);
                painter.circle_stroke(to_screen(15.0, 10.0), 8.0 * zoom, stroke);
                painter.line_segment([to_screen(15.0, 20.0), to_screen(30.0, 30.0)], stroke);
            }
            Self::Ground => {
                painter.line_segment([to_screen(0.0, -20.0), to_screen(0.0, 0.0)], stroke);
                painter.line_segment([to_screen(-15.0, 0.0), to_screen(15.0, 0.0)], stroke);
                painter.line_segment([to_screen(-10.0, 6.0), to_screen(10.0, 6.0)], stroke);
                painter.line_segment([to_screen(-5.0, 12.0), to_screen(5.0, 12.0)], stroke);
            }
            Self::VoltageSource => {
                painter.line_segment([to_screen(0.0, -40.0), to_screen(0.0, -20.0)], stroke);
                let center = to_screen(0.0, 0.0);
                painter.circle_stroke(center, 20.0 * zoom, stroke);
                painter.line_segment([to_screen(-4.0, -10.0), to_screen(4.0, -10.0)], stroke);
                painter.line_segment([to_screen(0.0, -14.0), to_screen(0.0, -6.0)], stroke);
                painter.line_segment([to_screen(-4.0, 10.0), to_screen(4.0, 10.0)], stroke);
                painter.line_segment([to_screen(0.0, 20.0), to_screen(0.0, 40.0)], stroke);
            }
            Self::AcVoltageSource => {
                painter.line_segment([to_screen(0.0, -40.0), to_screen(0.0, -20.0)], stroke);
                let center = to_screen(0.0, 0.0);
                painter.circle_stroke(center, 20.0 * zoom, stroke);
                let pts = [
                    to_screen(-10.0, 0.0),
                    to_screen(-5.0, -6.0),
                    to_screen(0.0, 0.0),
                    to_screen(5.0, 6.0),
                    to_screen(10.0, 0.0),
                ];
                for i in 0..pts.len() - 1 {
                    painter.line_segment([pts[i], pts[i + 1]], stroke);
                }
                painter.line_segment([to_screen(0.0, 20.0), to_screen(0.0, 40.0)], stroke);
            }
            Self::CurrentSource => {
                painter.line_segment([to_screen(0.0, -40.0), to_screen(0.0, -20.0)], stroke);
                let center = to_screen(0.0, 0.0);
                painter.circle_stroke(center, 20.0 * zoom, stroke);
                painter.line_segment([to_screen(0.0, 10.0), to_screen(0.0, -10.0)], stroke);
                painter.line_segment([to_screen(-4.0, -4.0), to_screen(0.0, -10.0)], stroke);
                painter.line_segment([to_screen(4.0, -4.0), to_screen(0.0, -10.0)], stroke);
                painter.line_segment([to_screen(0.0, 20.0), to_screen(0.0, 40.0)], stroke);
            }
            Self::PulseGenerator => {
                painter.line_segment([to_screen(0.0, -40.0), to_screen(0.0, -20.0)], stroke);
                let center = to_screen(0.0, 0.0);
                painter.circle_stroke(center, 20.0 * zoom, stroke);
                let pts = [
                    to_screen(-10.0, 6.0),
                    to_screen(-5.0, 6.0),
                    to_screen(-5.0, -6.0),
                    to_screen(5.0, -6.0),
                    to_screen(5.0, 6.0),
                    to_screen(10.0, 6.0),
                ];
                for i in 0..pts.len() - 1 {
                    painter.line_segment([pts[i], pts[i + 1]], stroke);
                }
                painter.line_segment([to_screen(0.0, 20.0), to_screen(0.0, 40.0)], stroke);
            }
            Self::Diode => {
                painter.line_segment([to_screen(0.0, -40.0), to_screen(0.0, -10.0)], stroke);
                let tri = [
                    to_screen(-12.0, -10.0),
                    to_screen(12.0, -10.0),
                    to_screen(0.0, 10.0),
                ];
                painter.line_segment([tri[0], tri[1]], stroke);
                painter.line_segment([tri[1], tri[2]], stroke);
                painter.line_segment([tri[2], tri[0]], stroke);
                painter.line_segment([to_screen(-12.0, 10.0), to_screen(12.0, 10.0)], stroke);
                painter.line_segment([to_screen(0.0, 10.0), to_screen(0.0, 40.0)], stroke);
            }
            Self::ZenerDiode => {
                painter.line_segment([to_screen(0.0, -40.0), to_screen(0.0, -10.0)], stroke);
                let tri = [
                    to_screen(-12.0, -10.0),
                    to_screen(12.0, -10.0),
                    to_screen(0.0, 10.0),
                ];
                painter.line_segment([tri[0], tri[1]], stroke);
                painter.line_segment([tri[1], tri[2]], stroke);
                painter.line_segment([tri[2], tri[0]], stroke);
                painter.line_segment([to_screen(-12.0, 15.0), to_screen(-12.0, 10.0)], stroke);
                painter.line_segment([to_screen(-12.0, 10.0), to_screen(12.0, 10.0)], stroke);
                painter.line_segment([to_screen(12.0, 10.0), to_screen(12.0, 5.0)], stroke);
                painter.line_segment([to_screen(0.0, 10.0), to_screen(0.0, 40.0)], stroke);
            }
            Self::Led => {
                painter.line_segment([to_screen(0.0, -40.0), to_screen(0.0, -10.0)], stroke);
                let tri = [
                    to_screen(-12.0, -10.0),
                    to_screen(12.0, -10.0),
                    to_screen(0.0, 10.0),
                ];
                painter.line_segment([tri[0], tri[1]], stroke);
                painter.line_segment([tri[1], tri[2]], stroke);
                painter.line_segment([tri[2], tri[0]], stroke);
                painter.line_segment([to_screen(-12.0, 10.0), to_screen(12.0, 10.0)], stroke);
                painter.line_segment([to_screen(0.0, 10.0), to_screen(0.0, 40.0)], stroke);
                painter.line_segment([to_screen(14.0, -4.0), to_screen(22.0, -12.0)], stroke);
                painter.line_segment([to_screen(18.0, 2.0), to_screen(26.0, -6.0)], stroke);
            }
            Self::SchottkyDiode => {
                painter.line_segment([to_screen(0.0, -40.0), to_screen(0.0, -10.0)], stroke);
                let tri = [
                    to_screen(-12.0, -10.0),
                    to_screen(12.0, -10.0),
                    to_screen(0.0, 10.0),
                ];
                painter.line_segment([tri[0], tri[1]], stroke);
                painter.line_segment([tri[1], tri[2]], stroke);
                painter.line_segment([tri[2], tri[0]], stroke);
                painter.line_segment([to_screen(-16.0, 6.0), to_screen(-12.0, 6.0)], stroke);
                painter.line_segment([to_screen(-12.0, 6.0), to_screen(-12.0, 10.0)], stroke);
                painter.line_segment([to_screen(-12.0, 10.0), to_screen(12.0, 10.0)], stroke);
                painter.line_segment([to_screen(12.0, 10.0), to_screen(12.0, 14.0)], stroke);
                painter.line_segment([to_screen(12.0, 14.0), to_screen(16.0, 14.0)], stroke);
                painter.line_segment([to_screen(0.0, 10.0), to_screen(0.0, 40.0)], stroke);
            }
            Self::Nmos | Self::Pmos => {
                painter.line_segment([to_screen(-20.0, 0.0), to_screen(-8.0, 0.0)], stroke);
                painter.line_segment([to_screen(-8.0, -18.0), to_screen(-8.0, 18.0)], stroke);
                painter.line_segment([to_screen(0.0, -15.0), to_screen(0.0, 15.0)], stroke);
                painter.line_segment([to_screen(0.0, -12.0), to_screen(20.0, -12.0)], stroke);
                painter.line_segment([to_screen(20.0, -12.0), to_screen(20.0, -40.0)], stroke);
                painter.line_segment([to_screen(0.0, 12.0), to_screen(20.0, 12.0)], stroke);
                painter.line_segment([to_screen(20.0, 12.0), to_screen(20.0, 40.0)], stroke);
            }
            Self::FinFet => {
                painter.line_segment([to_screen(-20.0, 0.0), to_screen(-8.0, 0.0)], stroke);
                painter.line_segment([to_screen(-8.0, -22.0), to_screen(-8.0, 22.0)], stroke);
                painter.line_segment([to_screen(-2.0, -12.0), to_screen(6.0, -12.0)], stroke);
                painter.line_segment([to_screen(-2.0, 0.0), to_screen(6.0, 0.0)], stroke);
                painter.line_segment([to_screen(-2.0, 12.0), to_screen(6.0, 12.0)], stroke);
                painter.line_segment([to_screen(6.0, -12.0), to_screen(20.0, -40.0)], stroke);
                painter.line_segment([to_screen(6.0, 12.0), to_screen(20.0, 40.0)], stroke);
                painter.line_segment([to_screen(6.0, 0.0), to_screen(20.0, 0.0)], stroke);
            }
            Self::GaaNanosheet => {
                painter.line_segment([to_screen(-20.0, 0.0), to_screen(-10.0, 0.0)], stroke);
                painter.rect_stroke(
                    Rect::from_min_max(to_screen(-10.0, -18.0), to_screen(-4.0, 18.0)),
                    0.0,
                    stroke,
                    StrokeKind::Middle,
                );
                painter.line_segment([to_screen(0.0, -12.0), to_screen(12.0, -12.0)], stroke);
                painter.line_segment([to_screen(0.0, 0.0), to_screen(12.0, 0.0)], stroke);
                painter.line_segment([to_screen(0.0, 12.0), to_screen(12.0, 12.0)], stroke);
                painter.line_segment([to_screen(12.0, -12.0), to_screen(20.0, -40.0)], stroke);
                painter.line_segment([to_screen(12.0, 12.0), to_screen(20.0, 40.0)], stroke);
                painter.line_segment([to_screen(12.0, 0.0), to_screen(20.0, 0.0)], stroke);
            }
            Self::BjtNpn | Self::BjtPnp => {
                painter.line_segment([to_screen(-20.0, 0.0), to_screen(0.0, 0.0)], stroke);
                painter.line_segment([to_screen(0.0, -15.0), to_screen(0.0, 15.0)], stroke);
                painter.line_segment([to_screen(0.0, -8.0), to_screen(20.0, -40.0)], stroke);
                painter.line_segment([to_screen(0.0, 8.0), to_screen(20.0, 40.0)], stroke);
            }
            Self::OpAmp => {
                let tri = [
                    to_screen(-25.0, -30.0),
                    to_screen(25.0, 0.0),
                    to_screen(-25.0, 30.0),
                ];
                painter.line_segment([tri[0], tri[1]], stroke);
                painter.line_segment([tri[1], tri[2]], stroke);
                painter.line_segment([tri[2], tri[0]], stroke);
                painter.line_segment([to_screen(-40.0, -15.0), to_screen(-25.0, -15.0)], stroke);
                painter.line_segment([to_screen(-40.0, 15.0), to_screen(-25.0, 15.0)], stroke);
                painter.line_segment([to_screen(25.0, 0.0), to_screen(40.0, 0.0)], stroke);
                painter.line_segment([to_screen(0.0, -30.0), to_screen(0.0, -15.0)], stroke);
                painter.line_segment([to_screen(0.0, 30.0), to_screen(0.0, 15.0)], stroke);
                painter.line_segment([to_screen(-20.0, -15.0), to_screen(-14.0, -15.0)], stroke);
                painter.line_segment([to_screen(-17.0, -18.0), to_screen(-17.0, -12.0)], stroke);
                painter.line_segment([to_screen(-20.0, 15.0), to_screen(-14.0, 15.0)], stroke);
            }
            Self::Inverter => {
                let tri = [
                    to_screen(-18.0, -20.0),
                    to_screen(12.0, 0.0),
                    to_screen(-18.0, 20.0),
                ];
                painter.line_segment([tri[0], tri[1]], stroke);
                painter.line_segment([tri[1], tri[2]], stroke);
                painter.line_segment([tri[2], tri[0]], stroke);
                painter.line_segment([to_screen(-30.0, 0.0), to_screen(-18.0, 0.0)], stroke);
                painter.circle_stroke(to_screen(17.0, 0.0), 5.0 * zoom, stroke);
                painter.line_segment([to_screen(22.0, 0.0), to_screen(30.0, 0.0)], stroke);
            }
            Self::NandGate => {
                painter.line_segment([to_screen(-30.0, -15.0), to_screen(-15.0, -15.0)], stroke);
                painter.line_segment([to_screen(-30.0, 15.0), to_screen(-15.0, 15.0)], stroke);
                painter.line_segment([to_screen(-15.0, -22.0), to_screen(-15.0, 22.0)], stroke);
                painter.line_segment([to_screen(-15.0, -22.0), to_screen(0.0, -22.0)], stroke);
                painter.line_segment([to_screen(-15.0, 22.0), to_screen(0.0, 22.0)], stroke);
                let arc_pts = [
                    to_screen(0.0, -22.0),
                    to_screen(10.0, -15.0),
                    to_screen(14.0, 0.0),
                    to_screen(10.0, 15.0),
                    to_screen(0.0, 22.0),
                ];
                for i in 0..arc_pts.len() - 1 {
                    painter.line_segment([arc_pts[i], arc_pts[i + 1]], stroke);
                }
                painter.circle_stroke(to_screen(18.0, 0.0), 4.0 * zoom, stroke);
                painter.line_segment([to_screen(22.0, 0.0), to_screen(30.0, 0.0)], stroke);
            }
            Self::NorGate => {
                painter.line_segment([to_screen(-30.0, -15.0), to_screen(-12.0, -15.0)], stroke);
                painter.line_segment([to_screen(-30.0, 15.0), to_screen(-12.0, 15.0)], stroke);
                let back_pts = [
                    to_screen(-18.0, -22.0),
                    to_screen(-12.0, 0.0),
                    to_screen(-18.0, 22.0),
                ];
                painter.line_segment([back_pts[0], back_pts[1]], stroke);
                painter.line_segment([back_pts[1], back_pts[2]], stroke);
                let front_top = [
                    to_screen(-18.0, -22.0),
                    to_screen(2.0, -16.0),
                    to_screen(14.0, 0.0),
                ];
                let front_bot = [
                    to_screen(-18.0, 22.0),
                    to_screen(2.0, 16.0),
                    to_screen(14.0, 0.0),
                ];
                painter.line_segment([front_top[0], front_top[1]], stroke);
                painter.line_segment([front_top[1], front_top[2]], stroke);
                painter.line_segment([front_bot[0], front_bot[1]], stroke);
                painter.line_segment([front_bot[1], front_bot[2]], stroke);
                painter.circle_stroke(to_screen(18.0, 0.0), 4.0 * zoom, stroke);
                painter.line_segment([to_screen(22.0, 0.0), to_screen(30.0, 0.0)], stroke);
            }
            Self::Mux2to1 => {
                let trap = [
                    to_screen(-15.0, -25.0),
                    to_screen(15.0, -15.0),
                    to_screen(15.0, 15.0),
                    to_screen(-15.0, 25.0),
                ];
                for i in 0..4 {
                    painter.line_segment([trap[i], trap[(i + 1) % 4]], stroke);
                }
                painter.line_segment([to_screen(-30.0, -15.0), to_screen(-15.0, -15.0)], stroke);
                painter.line_segment([to_screen(-30.0, 15.0), to_screen(-15.0, 15.0)], stroke);
                painter.line_segment([to_screen(0.0, 30.0), to_screen(0.0, 20.0)], stroke);
                painter.line_segment([to_screen(15.0, 0.0), to_screen(30.0, 0.0)], stroke);
            }
            Self::StrainGauge => {
                let pts = [
                    to_screen(0.0, -20.0),
                    to_screen(15.0, 0.0),
                    to_screen(0.0, 20.0),
                    to_screen(-15.0, 0.0),
                ];
                for i in 0..4 {
                    painter.line_segment([pts[i], pts[(i + 1) % 4]], stroke);
                }
                painter.line_segment([to_screen(-10.0, 10.0), to_screen(10.0, -10.0)], stroke);
                painter.line_segment([to_screen(0.0, -30.0), to_screen(0.0, -20.0)], stroke);
                painter.line_segment([to_screen(0.0, 20.0), to_screen(0.0, 30.0)], stroke);
            }
            Self::TactileMatrix => {
                painter.rect_stroke(
                    Rect::from_center_size(to_screen(0.0, 0.0), Vec2::new(36.0 * zoom, 36.0 * zoom)),
                    2.0,
                    stroke,
                    StrokeKind::Middle,
                );
                painter.line_segment([to_screen(-18.0, -6.0), to_screen(18.0, -6.0)], stroke);
                painter.line_segment([to_screen(-18.0, 6.0), to_screen(18.0, 6.0)], stroke);
                painter.line_segment([to_screen(-6.0, -18.0), to_screen(-6.0, 18.0)], stroke);
                painter.line_segment([to_screen(6.0, -18.0), to_screen(6.0, 18.0)], stroke);
                painter.line_segment([to_screen(-30.0, 0.0), to_screen(-18.0, 0.0)], stroke);
                painter.line_segment([to_screen(18.0, 0.0), to_screen(30.0, 0.0)], stroke);
                painter.line_segment([to_screen(0.0, -30.0), to_screen(0.0, -18.0)], stroke);
                painter.line_segment([to_screen(0.0, 18.0), to_screen(0.0, 30.0)], stroke);
            }
            Self::Imu9Dof => {
                painter.rect_stroke(
                    Rect::from_center_size(to_screen(0.0, 0.0), Vec2::new(36.0 * zoom, 36.0 * zoom)),
                    2.0,
                    stroke,
                    StrokeKind::Middle,
                );
                painter.line_segment([to_screen(-6.0, 6.0), to_screen(8.0, 6.0)], stroke);
                painter.line_segment([to_screen(-6.0, 6.0), to_screen(-6.0, -8.0)], stroke);
                painter.line_segment([to_screen(-30.0, -20.0), to_screen(-18.0, -20.0)], stroke);
                painter.line_segment([to_screen(-30.0, 20.0), to_screen(-18.0, 20.0)], stroke);
                painter.line_segment([to_screen(18.0, -20.0), to_screen(30.0, -20.0)], stroke);
                painter.line_segment([to_screen(18.0, 20.0), to_screen(30.0, 20.0)], stroke);
            }
            Self::SawIdt => {
                painter.line_segment([to_screen(-20.0, -15.0), to_screen(20.0, -15.0)], stroke);
                painter.line_segment([to_screen(-20.0, 15.0), to_screen(20.0, 15.0)], stroke);
                for x in [-12.0, -4.0, 4.0, 12.0] {
                    painter.line_segment([to_screen(x, -15.0), to_screen(x, 5.0)], stroke);
                }
                for x in [-8.0, 0.0, 8.0, 16.0] {
                    painter.line_segment([to_screen(x, 15.0), to_screen(x, -5.0)], stroke);
                }
                painter.line_segment([to_screen(-30.0, -20.0), to_screen(-20.0, -15.0)], stroke);
                painter.line_segment([to_screen(-30.0, 20.0), to_screen(-20.0, 15.0)], stroke);
                painter.line_segment([to_screen(20.0, -15.0), to_screen(30.0, -20.0)], stroke);
                painter.line_segment([to_screen(20.0, 15.0), to_screen(30.0, 20.0)], stroke);
            }
            Self::MajoranaJunction => {
                painter.line_segment([to_screen(-30.0, -20.0), to_screen(0.0, 0.0)], stroke);
                painter.line_segment([to_screen(-30.0, 20.0), to_screen(0.0, 0.0)], stroke);
                painter.line_segment([to_screen(0.0, 0.0), to_screen(30.0, 0.0)], stroke);
                painter.circle_stroke(to_screen(0.0, 0.0), 8.0 * zoom, stroke);
            }
            Self::ParafermionicCavity => {
                painter.circle_stroke(to_screen(0.0, 0.0), 16.0 * zoom, stroke);
                painter.circle_stroke(to_screen(0.0, 0.0), 8.0 * zoom, stroke);
                painter.line_segment([to_screen(-30.0, 0.0), to_screen(-16.0, 0.0)], stroke);
                painter.line_segment([to_screen(16.0, 0.0), to_screen(30.0, 0.0)], stroke);
            }
            Self::SkyrmionRouter => {
                painter.circle_stroke(to_screen(0.0, 0.0), 14.0 * zoom, stroke);
                painter.line_segment([to_screen(-30.0, 0.0), to_screen(-14.0, 0.0)], stroke);
                painter.line_segment([to_screen(10.0, -10.0), to_screen(30.0, -20.0)], stroke);
                painter.line_segment([to_screen(10.0, 10.0), to_screen(30.0, 20.0)], stroke);
                painter.line_segment([to_screen(0.0, -30.0), to_screen(0.0, -14.0)], stroke);
            }
        }
    }
}

/// A visual component placed on the schematic canvas.
#[derive(Debug, Clone, PartialEq)]
pub struct SchematicComponent {
    pub id: usize,
    pub name: String,
    pub kind: ComponentKind,
    pub pos: Pos2,
    /// Rotation in increments of 90 degrees (0 = 0 deg, 1 = 90 deg, 2 = 180 deg, 3 = 270 deg).
    pub rotation: u8,
    pub value_str: String,
    pub model_name: Option<String>,
    pub properties: Vec<(String, String)>,
}

impl SchematicComponent {
    pub fn new(id: usize, kind: ComponentKind, pos: Pos2, count: usize) -> Self {
        let name = format!("{}{}", kind.prefix(), count);
        let value_str = kind.default_value().to_string();
        Self {
            id,
            name,
            kind,
            pos,
            rotation: 0,
            value_str,
            model_name: None,
            properties: Vec::new(),
        }
    }

    /// Builder method attaching a custom key-value metadata property.
    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.push((key.into(), value.into()));
        self
    }

    /// Looks up a custom property value by key.
    pub fn get_property(&self, key: &str) -> Option<&str> {
        self.properties
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// Rotates the component clockwise by 90 degrees.
    pub fn rotate_clockwise(&mut self) {
        self.rotation = (self.rotation + 1) % 4;
    }

    /// Returns the current rotation angle in degrees (0, 90, 180, or 270).
    pub fn rotation_degrees(&self) -> u32 {
        (self.rotation % 4) as u32 * 90
    }

    /// Transforms a local vector according to the component's rotation.
    fn rotate_vec(&self, v: Vec2) -> Vec2 {
        match self.rotation % 4 {
            0 => v,
            1 => Vec2::new(-v.y, v.x),  // 90 deg CW
            2 => Vec2::new(-v.x, -v.y), // 180 deg
            3 => Vec2::new(v.y, -v.x),  // 270 deg
            _ => v,
        }
    }

    /// Returns the world position of pin at `pin_idx`.
    pub fn pin_world_pos(&self, pin_idx: usize) -> Option<Pos2> {
        let pins = self.kind.pin_definitions();
        let &(_, local_offset) = pins.get(pin_idx)?;
        let rotated = self.rotate_vec(local_offset);
        Some(self.pos + rotated)
    }

    /// Returns all pin names and their world coordinates.
    pub fn all_pins(&self) -> Vec<(&'static str, Pos2)> {
        let pins = self.kind.pin_definitions();
        pins.iter()
            .map(|&(name, offset)| (name, self.pos + self.rotate_vec(offset)))
            .collect()
    }

    /// Hit-test: checks if a world position lies within the component's bounding box.
    pub fn contains(&self, world_pos: Pos2) -> bool {
        let bbox = Rect::from_center_size(self.pos, Vec2::new(70.0, 70.0));
        bbox.contains(world_pos)
    }

    /// Renders the component schematic symbol onto the screen painter.
    pub fn render(
        &self,
        painter: &Painter,
        canvas: &SchematicCanvas,
        is_selected: bool,
        node_voltages: Option<&[(&str, f64)]>,
    ) {
        let stroke_color = if is_selected {
            Color32::from_rgb(255, 180, 50)
        } else {
            Color32::from_rgb(220, 230, 240)
        };
        let stroke = Stroke::new(2.0 * canvas.zoom.clamp(0.8, 2.0), stroke_color);

        // Helper to transform local component coords to screen
        let to_screen = |lx: f32, ly: f32| -> Pos2 {
            let rotated = self.rotate_vec(Vec2::new(lx, ly));
            canvas.world_to_screen(self.pos + rotated)
        };

        // Render geometric symbol strokes
        self.kind.draw_symbol(painter, stroke, &to_screen, canvas.zoom);

        // Draw pin snap dots
        let pin_color = Color32::from_rgb(80, 200, 255);
        for (_, p_world) in self.all_pins() {
            let p_screen = canvas.world_to_screen(p_world);
            painter.circle_filled(p_screen, 3.5 * canvas.zoom.clamp(0.8, 1.4), pin_color);
        }

        // Draw labels (Name and Value)
        let label_pos = canvas.world_to_screen(self.pos + Vec2::new(18.0, -10.0));
        let val_pos = canvas.world_to_screen(self.pos + Vec2::new(18.0, 6.0));
        let font_size = 12.0 * canvas.zoom.clamp(0.8, 1.8);

        painter.text(
            label_pos,
            Align2::LEFT_CENTER,
            &self.name,
            FontId::proportional(font_size),
            if is_selected {
                Color32::from_rgb(255, 200, 80)
            } else {
                Color32::from_rgb(180, 220, 255)
            },
        );

        if self.kind != ComponentKind::Ground {
            painter.text(
                val_pos,
                Align2::LEFT_CENTER,
                &self.value_str,
                FontId::proportional(font_size * 0.9),
                Color32::from_rgb(170, 185, 200),
            );
        }

        // Optional live voltage readout badges near pins
        if let Some(voltages) = node_voltages {
            for (pin_name, pin_pos) in self.all_pins() {
                if let Some(&(_, v)) = voltages.iter().find(|(name, _)| *name == pin_name) {
                    let badge_pos = canvas.world_to_screen(pin_pos + Vec2::new(8.0, -8.0));
                    painter.text(
                        badge_pos,
                        Align2::LEFT_CENTER,
                        format!("{:.2}V", v),
                        FontId::monospace(10.0 * canvas.zoom.clamp(0.8, 1.5)),
                        Color32::from_rgb(100, 255, 160),
                    );
                }
            }
        }
    }
}
