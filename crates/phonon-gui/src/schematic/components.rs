#![deny(unsafe_code)]

//! Visual schematic components, pin geometries, symbol rendering, and hit testing.

use super::canvas::SchematicCanvas;
use super::categories::ComponentCategory;
use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, StrokeKind, Vec2};

/// The electrical or physical device type of a visual schematic component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentKind {
    // Passive Elements & Controls
    Resistor,
    Potentiometer,
    Capacitor,
    Inductor,
    Ground,
    Transformer,
    SwitchSpst,
    PushButton,

    // Sources & Clocks
    VoltageSource,
    AcVoltageSource,
    CurrentSource,
    PulseGenerator,
    ClockSource,
    VddRail,

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

    // Digital Logic Gates (Primitives)
    BufferGate,
    Inverter,
    AndGate,
    OrGate,
    NandGate,
    NorGate,
    XorGate,
    XnorGate,

    // Digital & Arithmetic ICs (High-Level)
    HalfAdder,
    FullAdder,
    Mux2to1,
    Mux4to1,
    Demux1to2,
    DFlipFlop,
    SrLatch,
    Counter4Bit,

    // Analog & Mixed-Signal ICs
    OpAmp,
    Comparator,
    Timer555,
    VoltageRegulator,

    // Outputs & Indicators
    LogicProbe,
    SevenSegment,
    Buzzer,

    // Sensors & Transducers
    StrainGauge,
    TactileMatrix,
    Imu9Dof,

    // Topological & Quantum Metamaterials
    SawIdt,
    MajoranaJunction,
    ParafermionicCavity,
    SkyrmionRouter,

    // Port-Hamiltonian Articulatory Acoustics
    PhLungs,
    PhVocalFolds,
    PhVocalTract,
    PhLipRadiation,
}

impl ComponentKind {
    /// Returns the hierarchical category this component belongs to.
    pub fn category(&self) -> ComponentCategory {
        match self {
            Self::Resistor
            | Self::Potentiometer
            | Self::Capacitor
            | Self::Inductor
            | Self::Ground
            | Self::Transformer
            | Self::SwitchSpst
            | Self::PushButton => ComponentCategory::Passives,
            Self::VoltageSource
            | Self::AcVoltageSource
            | Self::CurrentSource
            | Self::PulseGenerator
            | Self::ClockSource
            | Self::VddRail => ComponentCategory::Sources,
            Self::Diode | Self::ZenerDiode | Self::Led | Self::SchottkyDiode => {
                ComponentCategory::Discretes
            }
            Self::Nmos
            | Self::Pmos
            | Self::FinFet
            | Self::GaaNanosheet
            | Self::BjtNpn
            | Self::BjtPnp => ComponentCategory::Transistors,
            Self::BufferGate
            | Self::Inverter
            | Self::AndGate
            | Self::OrGate
            | Self::NandGate
            | Self::NorGate
            | Self::XorGate
            | Self::XnorGate => ComponentCategory::LogicGates,
            Self::HalfAdder
            | Self::FullAdder
            | Self::Mux2to1
            | Self::Mux4to1
            | Self::Demux1to2
            | Self::DFlipFlop
            | Self::SrLatch
            | Self::Counter4Bit => ComponentCategory::IntegratedCircuits,
            Self::OpAmp | Self::Comparator | Self::Timer555 | Self::VoltageRegulator => {
                ComponentCategory::AnalogICs
            }
            Self::LogicProbe | Self::SevenSegment | Self::Buzzer => ComponentCategory::Indicators,
            Self::StrainGauge | Self::TactileMatrix | Self::Imu9Dof => ComponentCategory::Sensors,
            Self::SawIdt
            | Self::MajoranaJunction
            | Self::ParafermionicCavity
            | Self::SkyrmionRouter => ComponentCategory::TopologicalMetamaterials,
            Self::PhLungs
            | Self::PhVocalFolds
            | Self::PhVocalTract
            | Self::PhLipRadiation => ComponentCategory::PortHamiltonian,
        }
    }

    /// Returns the static variant identifier string.
    pub fn code_name(&self) -> &'static str {
        match self {
            Self::Resistor => "Resistor",
            Self::Potentiometer => "Potentiometer",
            Self::Capacitor => "Capacitor",
            Self::Inductor => "Inductor",
            Self::Ground => "Ground",
            Self::Transformer => "Transformer",
            Self::SwitchSpst => "SwitchSpst",
            Self::PushButton => "PushButton",
            Self::VoltageSource => "VoltageSource",
            Self::AcVoltageSource => "AcVoltageSource",
            Self::CurrentSource => "CurrentSource",
            Self::PulseGenerator => "PulseGenerator",
            Self::ClockSource => "ClockSource",
            Self::VddRail => "VddRail",
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
            Self::BufferGate => "BufferGate",
            Self::Inverter => "Inverter",
            Self::AndGate => "AndGate",
            Self::OrGate => "OrGate",
            Self::NandGate => "NandGate",
            Self::NorGate => "NorGate",
            Self::XorGate => "XorGate",
            Self::XnorGate => "XnorGate",
            Self::HalfAdder => "HalfAdder",
            Self::FullAdder => "FullAdder",
            Self::Mux2to1 => "Mux2to1",
            Self::Mux4to1 => "Mux4to1",
            Self::Demux1to2 => "Demux1to2",
            Self::DFlipFlop => "DFlipFlop",
            Self::SrLatch => "SrLatch",
            Self::Counter4Bit => "Counter4Bit",
            Self::OpAmp => "OpAmp",
            Self::Comparator => "Comparator",
            Self::Timer555 => "Timer555",
            Self::VoltageRegulator => "VoltageRegulator",
            Self::LogicProbe => "LogicProbe",
            Self::SevenSegment => "SevenSegment",
            Self::Buzzer => "Buzzer",
            Self::StrainGauge => "StrainGauge",
            Self::TactileMatrix => "TactileMatrix",
            Self::Imu9Dof => "Imu9Dof",
            Self::SawIdt => "SawIdt",
            Self::MajoranaJunction => "MajoranaJunction",
            Self::ParafermionicCavity => "ParafermionicCavity",
            Self::SkyrmionRouter => "SkyrmionRouter",
            Self::PhLungs => "PhLungs",
            Self::PhVocalFolds => "PhVocalFolds",
            Self::PhVocalTract => "PhVocalTract",
            Self::PhLipRadiation => "PhLipRadiation",
        }
    }

    /// All 58 categorized component kinds in static array.
    pub const ALL: &'static [Self] = &[
        Self::Resistor,
        Self::Potentiometer,
        Self::Capacitor,
        Self::Inductor,
        Self::Ground,
        Self::Transformer,
        Self::SwitchSpst,
        Self::PushButton,
        Self::VoltageSource,
        Self::AcVoltageSource,
        Self::CurrentSource,
        Self::PulseGenerator,
        Self::ClockSource,
        Self::VddRail,
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
        Self::BufferGate,
        Self::Inverter,
        Self::AndGate,
        Self::OrGate,
        Self::NandGate,
        Self::NorGate,
        Self::XorGate,
        Self::XnorGate,
        Self::HalfAdder,
        Self::FullAdder,
        Self::Mux2to1,
        Self::Mux4to1,
        Self::Demux1to2,
        Self::DFlipFlop,
        Self::SrLatch,
        Self::Counter4Bit,
        Self::OpAmp,
        Self::Comparator,
        Self::Timer555,
        Self::VoltageRegulator,
        Self::LogicProbe,
        Self::SevenSegment,
        Self::Buzzer,
        Self::StrainGauge,
        Self::TactileMatrix,
        Self::Imu9Dof,
        Self::SawIdt,
        Self::MajoranaJunction,
        Self::ParafermionicCavity,
        Self::SkyrmionRouter,
        Self::PhLungs,
        Self::PhVocalFolds,
        Self::PhVocalTract,
        Self::PhLipRadiation,
    ];

    /// Alias for ALL variants.
    pub const ALL_VARIANTS: &'static [Self] = Self::ALL;

    /// Precomputed lowercase search index string for zero-allocation query filtering.
    pub fn search_index(&self) -> &'static str {
        match self {
            Self::Resistor => "resistor resistor passive elements linear passive two-terminal electrical resistance element r",
            Self::Potentiometer => "potentiometer pot variable resistor wiper passive controls divider volume pot",
            Self::Capacitor => "capacitor capacitor passive elements electrostatic charge and energy storage dielectric device c",
            Self::Inductor => "inductor inductor passive elements magnetic flux storage coil element l",
            Self::Ground => "ground ground passive elements reference potential net 0v reference ground datum gnd 0",
            Self::Transformer => "transformer transformer passive elements coupled mutual inductance dual-winding transformer tx",
            Self::SwitchSpst => "switch spst single pole single throw toggle mechanical contact breaker sw",
            Self::PushButton => "pushbutton push button momentary tactile switch normally open contact pb",
            Self::VoltageSource => "dc voltage source voltagesource sources & generators constant dc electromotive force voltage supply v",
            Self::AcVoltageSource => "ac voltage source acvoltagesource sources & generators sinusoidal ac alternating electromotive voltage supply vac",
            Self::CurrentSource => "current source currentsource sources & generators constant electrical current supply i",
            Self::PulseGenerator => "pulse generator pulsegenerator sources & generators repetitive pulse and clock signal generator vpulse",
            Self::ClockSource => "clock source digital clock oscillator square wave pulse train vclk clk",
            Self::VddRail => "vdd rail +5v power supply rail dc positive rail vdd vcc",
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
            Self::BufferGate => "buffer gate ubuf digital logic gate non-inverting driver cmos logic",
            Self::Inverter => "inverter (not gate) inverter integrated circuits & logic cmos logic inverter not gate xinv uinv",
            Self::AndGate => "and gate uand digital logic gate dual-input cmos conjunction and logic",
            Self::OrGate => "or gate uor digital logic gate dual-input cmos disjunction or logic",
            Self::NandGate => "nand gate nandgate integrated circuits & logic dual-input universal cmos nand logic gate xnand unand",
            Self::NorGate => "nor gate norgate integrated circuits & logic dual-input universal cmos nor logic gate xnor unor",
            Self::XorGate => "xor gate exclusive or uxor digital logic gate dual-input cmos parity sum gate",
            Self::XnorGate => "xnor gate exclusive nor uxnor digital logic gate equivalence coincidence gate",
            Self::HalfAdder => "half adder uha arithmetic block 1-bit binary adder sum carry xor and ha",
            Self::FullAdder => "full adder ufa arithmetic block 1-bit binary adder carry-in sum carry-out fa",
            Self::Mux2to1 => "2:1 multiplexer mux2to1 integrated circuits & logic 2-to-1 binary data multiplexer xmux umux",
            Self::Mux4to1 => "4:1 multiplexer mux4to1 high-level digital 4-to-1 binary data selector umux4",
            Self::Demux1to2 => "1:2 demultiplexer demux1to2 data distributor 1-to-2 decoder udemux",
            Self::DFlipFlop => "d flip-flop dff sequential logic edge-triggered data register latch udff",
            Self::SrLatch => "sr latch set reset bistable multivibrator memory cell usrl",
            Self::Counter4Bit => "4-bit binary counter ripple counter digital divider ucnt",
            Self::OpAmp => "operational amplifier opamp integrated circuits & logic high-gain differential voltage operational amplifier xop uop",
            Self::Comparator => "voltage comparator ucmp analog mixed-signal high-speed threshold detector",
            Self::Timer555 => "555 timer ne555 lm555 precision astable monostable timer oscillator u555",
            Self::VoltageRegulator => "linear voltage regulator lm7805 7805 5v step-down power regulator ureg",
            Self::LogicProbe => "logic probe uprb digital state indicator high low probe tester",
            Self::SevenSegment => "7-segment display seven segment udisp numeric led display readout",
            Self::Buzzer => "piezo buzzer ubz audible alarm acoustic transducer speaker",
            Self::StrainGauge => "piezo strain gauge straingauge sensors & transducers piezoresistive acoustic strain sensor bridge xsg",
            Self::TactileMatrix => "tactile force matrix tactilematrix sensors & transducers piezotronic tactile force pressure sensor array xtm",
            Self::Imu9Dof => "9-dof imu transducer imu9dof sensors & transducers nine-degree-of-freedom inertial measurement transducer ximu",
            Self::SawIdt => "saw idt filter sawidt topological metamaterials surface acoustic wave interdigital transducer acoustic filter xsaw",
            Self::MajoranaJunction => "majorana braiding junction majoranajunction topological metamaterials non-abelian topological quantum braiding junction xmj",
            Self::ParafermionicCavity => "parafermionic cavity parafermioniccavity topological metamaterials fractionalized topological quantum acoustic resonator cavity xpc",
            Self::SkyrmionRouter => "skyrmion router skyrmionrouter topological metamaterials chiral magnetic skyrmion topological acoustic wave router xsr",
            Self::PhLungs => "lungs subglottal drive phlungs port-hamiltonian articulatory acoustics respiratory pulmonary pressure compliance xlung",
            Self::PhVocalFolds => "hirano vocal folds phvocalfolds port-hamiltonian articulatory acoustics 3-layer cover-body self-oscillation glottis xvf",
            Self::PhVocalTract => "webster acoustic horn tract phvocaltract port-hamiltonian articulatory acoustics riccati waveguide transmission line xvt",
            Self::PhLipRadiation => "lip radiation impedance phlipradiation port-hamiltonian articulatory acoustics mouth spherical wavefront termination xrad",
        }
    }

    /// Returns human-readable display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Resistor => "Resistor",
            Self::Potentiometer => "Potentiometer",
            Self::Capacitor => "Capacitor",
            Self::Inductor => "Inductor",
            Self::Ground => "Ground",
            Self::Transformer => "Transformer",
            Self::SwitchSpst => "SPST Switch",
            Self::PushButton => "Pushbutton",
            Self::VoltageSource => "DC Voltage Source",
            Self::AcVoltageSource => "AC Voltage Source",
            Self::CurrentSource => "Current Source",
            Self::PulseGenerator => "Pulse Generator",
            Self::ClockSource => "Clock Source",
            Self::VddRail => "VDD Rail (+5V)",
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
            Self::BufferGate => "Buffer Gate",
            Self::Inverter => "Inverter (NOT Gate)",
            Self::AndGate => "AND Gate",
            Self::OrGate => "OR Gate",
            Self::NandGate => "NAND Gate",
            Self::NorGate => "NOR Gate",
            Self::XorGate => "XOR Gate",
            Self::XnorGate => "XNOR Gate",
            Self::HalfAdder => "Half Adder",
            Self::FullAdder => "Full Adder",
            Self::Mux2to1 => "2:1 Multiplexer",
            Self::Mux4to1 => "4:1 Multiplexer",
            Self::Demux1to2 => "1:2 Demultiplexer",
            Self::DFlipFlop => "D Flip-Flop",
            Self::SrLatch => "SR Latch",
            Self::Counter4Bit => "4-Bit Counter",
            Self::OpAmp => "Operational Amplifier",
            Self::Comparator => "Voltage Comparator",
            Self::Timer555 => "555 Timer",
            Self::VoltageRegulator => "Voltage Regulator",
            Self::LogicProbe => "Logic Probe",
            Self::SevenSegment => "7-Segment Display",
            Self::Buzzer => "Piezo Buzzer",
            Self::StrainGauge => "Piezo Strain Gauge",
            Self::TactileMatrix => "Tactile Force Matrix",
            Self::Imu9Dof => "9-DOF IMU Transducer",
            Self::SawIdt => "SAW IDT Filter",
            Self::MajoranaJunction => "Majorana Braiding Junction",
            Self::ParafermionicCavity => "Parafermionic Cavity",
            Self::SkyrmionRouter => "Skyrmion Router",
            Self::PhLungs => "Lungs Subglottal Drive",
            Self::PhVocalFolds => "Hirano Vocal Folds",
            Self::PhVocalTract => "Webster Acoustic Horn Tract",
            Self::PhLipRadiation => "Lip Radiation Impedance",
        }
    }

    /// Returns descriptive summary for live search indexing.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Resistor => "Linear passive two-terminal electrical resistance element",
            Self::Potentiometer => "Three-terminal adjustable potentiometer voltage divider / variable resistor",
            Self::Capacitor => "Electrostatic charge and energy storage dielectric device",
            Self::Inductor => "Magnetic flux storage coil element",
            Self::Ground => "Reference potential net (0V reference ground datum)",
            Self::Transformer => "Coupled mutual inductance dual-winding transformer",
            Self::SwitchSpst => "Single-pole single-throw toggle mechanical switch",
            Self::PushButton => "Momentary tactile normally-open push-button switch",
            Self::VoltageSource => "Constant DC electromotive force voltage supply",
            Self::AcVoltageSource => "Sinusoidal AC alternating electromotive voltage supply",
            Self::CurrentSource => "Constant electrical current supply",
            Self::PulseGenerator => "Repetitive pulse and clock signal generator",
            Self::ClockSource => "Synchronous high-speed square-wave digital clock source",
            Self::VddRail => "Positive direct-current voltage supply distribution rail (+5V)",
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
            Self::BufferGate => "Non-inverting digital logic buffer and line driver",
            Self::Inverter => "CMOS logic inverter NOT gate",
            Self::AndGate => "Dual-input digital CMOS AND logic gate",
            Self::OrGate => "Dual-input digital CMOS OR logic gate",
            Self::NandGate => "Dual-input universal CMOS NAND logic gate",
            Self::NorGate => "Dual-input universal CMOS NOR logic gate",
            Self::XorGate => "Dual-input digital CMOS exclusive-OR (XOR) logic gate",
            Self::XnorGate => "Dual-input digital CMOS exclusive-NOR (XNOR) logic gate",
            Self::HalfAdder => "1-bit combinational binary half adder (Sum = A ^ B, Carry = A & B)",
            Self::FullAdder => "1-bit combinational binary full adder with carry input (Sum, Carry Out)",
            Self::Mux2to1 => "2-to-1 binary data multiplexer",
            Self::Mux4to1 => "4-to-1 binary data multiplexer with dual select lines",
            Self::Demux1to2 => "1-to-2 binary data demultiplexer and line distributor",
            Self::DFlipFlop => "Positive-edge triggered D flip-flop with complementary outputs (Q, Q_bar)",
            Self::SrLatch => "Bistable set-reset (SR) flip-flop latch memory element",
            Self::Counter4Bit => "Synchronous 4-bit binary up-counter with reset",
            Self::OpAmp => "High-gain differential voltage operational amplifier",
            Self::Comparator => "High-speed precision analog voltage comparator",
            Self::Timer555 => "Industry-standard 555 precision monolithic timer and multivibrator",
            Self::VoltageRegulator => "Monolithic 3-terminal linear positive voltage regulator (7805)",
            Self::LogicProbe => "Interactive visual logic level indicator probe (0 / 1 / Hi-Z)",
            Self::SevenSegment => "Common-cathode 7-segment LED visual alphanumeric display block",
            Self::Buzzer => "Piezoelectric acoustic transducer and audio annunciator",
            Self::StrainGauge => "Piezoresistive acoustic strain sensor bridge",
            Self::TactileMatrix => "Piezotronic tactile force pressure sensor array",
            Self::Imu9Dof => "Nine-degree-of-freedom inertial measurement transducer",
            Self::SawIdt => "Surface acoustic wave interdigital transducer acoustic filter",
            Self::MajoranaJunction => "Non-Abelian topological quantum braiding junction",
            Self::ParafermionicCavity => "Fractionalized topological quantum acoustic resonator cavity",
            Self::SkyrmionRouter => "Chiral magnetic skyrmion topological acoustic wave router",
            Self::PhLungs => "Aerodynamic subglottal pressure source with continuous lung compliance",
            Self::PhVocalFolds => "Nonlinear 3-layer cover-body mucosal traveling wave self-oscillating vocal folds",
            Self::PhVocalTract => "Continuous Riccati Webster transmission-line horn with visco-thermal losses",
            Self::PhLipRadiation => "Frequency-dependent radiation boundary impedance (+6 dB/octave high-pass)",
        }
    }

    /// Returns standard SPICE / schematic reference designator prefix.
    pub fn prefix(&self) -> &'static str {
        match self {
            Self::Resistor => "R",
            Self::Potentiometer => "POT",
            Self::Capacitor => "C",
            Self::Inductor => "L",
            Self::Ground => "GND",
            Self::Transformer => "TX",
            Self::SwitchSpst => "SW",
            Self::PushButton => "PB",
            Self::VoltageSource => "V",
            Self::AcVoltageSource => "VAC",
            Self::CurrentSource => "I",
            Self::PulseGenerator => "VPULSE",
            Self::ClockSource => "VCLK",
            Self::VddRail => "VDD",
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
            Self::BufferGate => "UBUF",
            Self::Inverter => "UINV",
            Self::AndGate => "UAND",
            Self::OrGate => "UOR",
            Self::NandGate => "UNAND",
            Self::NorGate => "UNOR",
            Self::XorGate => "UXOR",
            Self::XnorGate => "UXNOR",
            Self::HalfAdder => "UHA",
            Self::FullAdder => "UFA",
            Self::Mux2to1 => "UMUX",
            Self::Mux4to1 => "UMUX",
            Self::Demux1to2 => "UDMX",
            Self::DFlipFlop => "UDFF",
            Self::SrLatch => "USRL",
            Self::Counter4Bit => "UCNT",
            Self::OpAmp => "UOP",
            Self::Comparator => "UCMP",
            Self::Timer555 => "U555",
            Self::VoltageRegulator => "UREG",
            Self::LogicProbe => "UPRB",
            Self::SevenSegment => "UDISP",
            Self::Buzzer => "UBZ",
            Self::StrainGauge => "XSG",
            Self::TactileMatrix => "XTM",
            Self::Imu9Dof => "XIMU",
            Self::SawIdt => "XSAW",
            Self::MajoranaJunction => "XMJ",
            Self::ParafermionicCavity => "XPC",
            Self::SkyrmionRouter => "XSR",
            Self::PhLungs => "XLUNG",
            Self::PhVocalFolds => "XVF",
            Self::PhVocalTract => "XVT",
            Self::PhLipRadiation => "XRAD",
        }
    }

    /// Returns default schematic parameter value string.
    pub fn default_value(&self) -> &'static str {
        match self {
            Self::Resistor => "1k",
            Self::Potentiometer => "10k",
            Self::Capacitor => "100n",
            Self::Inductor => "10u",
            Self::Ground => "0",
            Self::Transformer => "1:1",
            Self::SwitchSpst => "OPEN",
            Self::PushButton => "NO",
            Self::VoltageSource => "5.0",
            Self::AcVoltageSource => "SIN(0 1 1k)",
            Self::CurrentSource => "1m",
            Self::PulseGenerator => "PULSE(0 5 0 1n 1n 10u 20u)",
            Self::ClockSource => "PULSE(0 5 0 10n 10n 500u 1m)",
            Self::VddRail => "5.0",
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
            Self::BufferGate => "BUF_CMOS",
            Self::Inverter => "INV_CMOS",
            Self::AndGate => "AND2",
            Self::OrGate => "OR2",
            Self::NandGate => "NAND2",
            Self::NorGate => "NOR2",
            Self::XorGate => "XOR2",
            Self::XnorGate => "XNOR2",
            Self::HalfAdder => "HALF_ADDER",
            Self::FullAdder => "FULL_ADDER",
            Self::Mux2to1 => "MUX21",
            Self::Mux4to1 => "MUX41",
            Self::Demux1to2 => "DEMUX12",
            Self::DFlipFlop => "DFF",
            Self::SrLatch => "SRLATCH",
            Self::Counter4Bit => "COUNTER4",
            Self::OpAmp => "OPAMP_IDEAL",
            Self::Comparator => "COMPARATOR",
            Self::Timer555 => "LM555",
            Self::VoltageRegulator => "LM7805",
            Self::LogicProbe => "LOGIC_PROBE",
            Self::SevenSegment => "7SEG_CC",
            Self::Buzzer => "BUZZER_5V",
            Self::StrainGauge => "STRAIN_350",
            Self::TactileMatrix => "TACTILE_8X8",
            Self::Imu9Dof => "IMU_6DOF_9DOF",
            Self::SawIdt => "SAW_1GHZ",
            Self::MajoranaJunction => "TOPOMAJ_1",
            Self::ParafermionicCavity => "PARAFERM_RES",
            Self::SkyrmionRouter => "SKYRMION_RT",
            Self::PhLungs => "PH_LUNGS",
            Self::PhVocalFolds => "PH_VOCAL_FOLDS",
            Self::PhVocalTract => "PH_VOCAL_TRACT",
            Self::PhLipRadiation => "PH_LIP_RADIATION",
        }
    }

    /// Returns the local pin offsets relative to component origin (at rotation 0).
    pub fn pin_definitions(&self) -> Vec<(&'static str, Vec2)> {
        match self {
            Self::Resistor | Self::Capacitor | Self::Inductor => {
                vec![("1", Vec2::new(0.0, -40.0)), ("2", Vec2::new(0.0, 40.0))]
            }
            Self::Potentiometer => vec![
                ("1", Vec2::new(0.0, -40.0)),
                ("WIPER", Vec2::new(20.0, 0.0)),
                ("2", Vec2::new(0.0, 40.0)),
            ],
            Self::Ground => vec![("GND", Vec2::new(0.0, -20.0))],
            Self::Transformer => vec![
                ("P+", Vec2::new(-30.0, -30.0)),
                ("P-", Vec2::new(-30.0, 30.0)),
                ("S+", Vec2::new(30.0, -30.0)),
                ("S-", Vec2::new(30.0, 30.0)),
            ],
            Self::SwitchSpst | Self::PushButton => vec![
                ("1", Vec2::new(-30.0, 0.0)),
                ("2", Vec2::new(30.0, 0.0)),
            ],
            Self::VoltageSource
            | Self::AcVoltageSource
            | Self::CurrentSource
            | Self::PulseGenerator => {
                vec![("+", Vec2::new(0.0, -40.0)), ("-", Vec2::new(0.0, 40.0))]
            }
            Self::ClockSource => {
                vec![("CLK", Vec2::new(0.0, -40.0)), ("GND", Vec2::new(0.0, 40.0))]
            }
            Self::VddRail => vec![("VDD", Vec2::new(0.0, 20.0))],
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
            Self::BufferGate | Self::Inverter => vec![
                ("IN", Vec2::new(-30.0, 0.0)),
                ("OUT", Vec2::new(30.0, 0.0)),
            ],
            Self::AndGate
            | Self::OrGate
            | Self::NandGate
            | Self::NorGate
            | Self::XorGate
            | Self::XnorGate => vec![
                ("A", Vec2::new(-30.0, -15.0)),
                ("B", Vec2::new(-30.0, 15.0)),
                ("OUT", Vec2::new(30.0, 0.0)),
            ],
            Self::HalfAdder => vec![
                ("A", Vec2::new(-40.0, -15.0)),
                ("B", Vec2::new(-40.0, 15.0)),
                ("SUM", Vec2::new(40.0, -15.0)),
                ("COUT", Vec2::new(40.0, 15.0)),
            ],
            Self::FullAdder => vec![
                ("A", Vec2::new(-40.0, -20.0)),
                ("B", Vec2::new(-40.0, 0.0)),
                ("CIN", Vec2::new(-40.0, 20.0)),
                ("SUM", Vec2::new(40.0, -15.0)),
                ("COUT", Vec2::new(40.0, 15.0)),
            ],
            Self::Mux2to1 => vec![
                ("D0", Vec2::new(-30.0, -15.0)),
                ("D1", Vec2::new(-30.0, 15.0)),
                ("SEL", Vec2::new(0.0, 30.0)),
                ("OUT", Vec2::new(30.0, 0.0)),
            ],
            Self::Mux4to1 => vec![
                ("D0", Vec2::new(-30.0, -25.0)),
                ("D1", Vec2::new(-30.0, -10.0)),
                ("D2", Vec2::new(-30.0, 10.0)),
                ("D3", Vec2::new(-30.0, 25.0)),
                ("S0", Vec2::new(-10.0, 35.0)),
                ("S1", Vec2::new(10.0, 35.0)),
                ("OUT", Vec2::new(30.0, 0.0)),
            ],
            Self::Demux1to2 => vec![
                ("IN", Vec2::new(-30.0, 0.0)),
                ("SEL", Vec2::new(0.0, 30.0)),
                ("Y0", Vec2::new(30.0, -15.0)),
                ("Y1", Vec2::new(30.0, 15.0)),
            ],
            Self::DFlipFlop => vec![
                ("D", Vec2::new(-35.0, -15.0)),
                ("CLK", Vec2::new(-35.0, 15.0)),
                ("Q", Vec2::new(35.0, -15.0)),
                ("QN", Vec2::new(35.0, 15.0)),
            ],
            Self::SrLatch => vec![
                ("S", Vec2::new(-35.0, -15.0)),
                ("R", Vec2::new(-35.0, 15.0)),
                ("Q", Vec2::new(35.0, -15.0)),
                ("QN", Vec2::new(35.0, 15.0)),
            ],
            Self::Counter4Bit => vec![
                ("CLK", Vec2::new(-40.0, -15.0)),
                ("RST", Vec2::new(-40.0, 15.0)),
                ("Q0", Vec2::new(40.0, -30.0)),
                ("Q1", Vec2::new(40.0, -10.0)),
                ("Q2", Vec2::new(40.0, 10.0)),
                ("Q3", Vec2::new(40.0, 30.0)),
            ],
            Self::OpAmp => vec![
                ("IN+", Vec2::new(-40.0, -15.0)),
                ("IN-", Vec2::new(-40.0, 15.0)),
                ("OUT", Vec2::new(40.0, 0.0)),
                ("V+", Vec2::new(0.0, -30.0)),
                ("V-", Vec2::new(0.0, 30.0)),
            ],
            Self::Comparator => vec![
                ("IN+", Vec2::new(-40.0, -15.0)),
                ("IN-", Vec2::new(-40.0, 15.0)),
                ("OUT", Vec2::new(40.0, 0.0)),
            ],
            Self::Timer555 => vec![
                ("GND", Vec2::new(-30.0, 30.0)),
                ("TRIG", Vec2::new(-30.0, 10.0)),
                ("OUT", Vec2::new(30.0, -10.0)),
                ("RESET", Vec2::new(-30.0, -30.0)),
                ("CTRL", Vec2::new(30.0, 30.0)),
                ("THRES", Vec2::new(30.0, 10.0)),
                ("DISCH", Vec2::new(30.0, -30.0)),
                ("VCC", Vec2::new(-30.0, -10.0)),
            ],
            Self::VoltageRegulator => vec![
                ("VIN", Vec2::new(-30.0, 0.0)),
                ("GND", Vec2::new(0.0, 30.0)),
                ("VOUT", Vec2::new(30.0, 0.0)),
            ],
            Self::LogicProbe => vec![("IN", Vec2::new(-20.0, 0.0))],
            Self::SevenSegment => vec![
                ("A", Vec2::new(-30.0, -30.0)),
                ("B", Vec2::new(-30.0, -10.0)),
                ("C", Vec2::new(-30.0, 10.0)),
                ("D", Vec2::new(-30.0, 30.0)),
                ("E", Vec2::new(30.0, -30.0)),
                ("F", Vec2::new(30.0, -10.0)),
                ("G", Vec2::new(30.0, 10.0)),
                ("COM", Vec2::new(0.0, 35.0)),
            ],
            Self::Buzzer => vec![("+", Vec2::new(0.0, -30.0)), ("-", Vec2::new(0.0, 30.0))],
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
            Self::PhLungs => vec![
                ("P_SUB", Vec2::new(0.0, -32.0)),
                ("REF", Vec2::new(0.0, 32.0)),
            ],
            Self::PhVocalFolds => vec![
                ("SUB", Vec2::new(-32.0, 0.0)),
                ("SUPRA", Vec2::new(32.0, 0.0)),
                ("CTRL", Vec2::new(0.0, -32.0)),
                ("REF", Vec2::new(0.0, 32.0)),
            ],
            Self::PhVocalTract => vec![
                ("IN", Vec2::new(-40.0, 0.0)),
                ("OUT", Vec2::new(40.0, 0.0)),
                ("WALL", Vec2::new(0.0, 32.0)),
                ("CTRL", Vec2::new(0.0, -32.0)),
            ],
            Self::PhLipRadiation => vec![
                ("IN", Vec2::new(-30.0, 0.0)),
                ("RAD", Vec2::new(30.0, 0.0)),
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
            Self::Potentiometer => {
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
                // Wiper terminal and arrow
                painter.line_segment([to_screen(20.0, 0.0), to_screen(4.0, 0.0)], stroke);
                painter.line_segment([to_screen(8.0, -4.0), to_screen(4.0, 0.0)], stroke);
                painter.line_segment([to_screen(8.0, 4.0), to_screen(4.0, 0.0)], stroke);
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
            Self::SwitchSpst => {
                painter.line_segment([to_screen(-30.0, 0.0), to_screen(-18.0, 0.0)], stroke);
                painter.circle_stroke(to_screen(-15.0, 0.0), 3.0 * zoom, stroke);
                painter.circle_stroke(to_screen(15.0, 0.0), 3.0 * zoom, stroke);
                painter.line_segment([to_screen(18.0, 0.0), to_screen(30.0, 0.0)], stroke);
                // Switch blade in open position
                painter.line_segment([to_screen(-12.0, 0.0), to_screen(12.0, -14.0)], stroke);
            }
            Self::PushButton => {
                painter.line_segment([to_screen(-30.0, 0.0), to_screen(-15.0, 0.0)], stroke);
                painter.circle_stroke(to_screen(-12.0, 0.0), 3.0 * zoom, stroke);
                painter.circle_stroke(to_screen(12.0, 0.0), 3.0 * zoom, stroke);
                painter.line_segment([to_screen(15.0, 0.0), to_screen(30.0, 0.0)], stroke);
                // Contact bar
                painter.line_segment([to_screen(-16.0, -10.0), to_screen(16.0, -10.0)], stroke);
                // Push stem
                painter.line_segment([to_screen(0.0, -10.0), to_screen(0.0, -22.0)], stroke);
                painter.line_segment([to_screen(-6.0, -22.0), to_screen(6.0, -22.0)], stroke);
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
            Self::ClockSource => {
                painter.line_segment([to_screen(0.0, -40.0), to_screen(0.0, -20.0)], stroke);
                let center = to_screen(0.0, 0.0);
                painter.circle_stroke(center, 20.0 * zoom, stroke);
                let pts = [
                    to_screen(-10.0, 5.0),
                    to_screen(-10.0, -5.0),
                    to_screen(0.0, -5.0),
                    to_screen(0.0, 5.0),
                    to_screen(10.0, 5.0),
                    to_screen(10.0, -5.0),
                ];
                for i in 0..pts.len() - 1 {
                    painter.line_segment([pts[i], pts[i + 1]], stroke);
                }
                painter.line_segment([to_screen(0.0, 20.0), to_screen(0.0, 40.0)], stroke);
            }
            Self::VddRail => {
                painter.line_segment([to_screen(0.0, 20.0), to_screen(0.0, 0.0)], stroke);
                painter.line_segment([to_screen(-12.0, 0.0), to_screen(12.0, 0.0)], stroke);
                painter.line_segment([to_screen(-8.0, 0.0), to_screen(0.0, -10.0)], stroke);
                painter.line_segment([to_screen(8.0, 0.0), to_screen(0.0, -10.0)], stroke);
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
            Self::BufferGate => {
                let tri = [
                    to_screen(-18.0, -20.0),
                    to_screen(18.0, 0.0),
                    to_screen(-18.0, 20.0),
                ];
                painter.line_segment([tri[0], tri[1]], stroke);
                painter.line_segment([tri[1], tri[2]], stroke);
                painter.line_segment([tri[2], tri[0]], stroke);
                painter.line_segment([to_screen(-30.0, 0.0), to_screen(-18.0, 0.0)], stroke);
                painter.line_segment([to_screen(18.0, 0.0), to_screen(30.0, 0.0)], stroke);
            }
            Self::AndGate => {
                painter.line_segment([to_screen(-30.0, -15.0), to_screen(-15.0, -15.0)], stroke);
                painter.line_segment([to_screen(-30.0, 15.0), to_screen(-15.0, 15.0)], stroke);
                painter.line_segment([to_screen(-15.0, -22.0), to_screen(-15.0, 22.0)], stroke);
                painter.line_segment([to_screen(-15.0, -22.0), to_screen(0.0, -22.0)], stroke);
                painter.line_segment([to_screen(-15.0, 22.0), to_screen(0.0, 22.0)], stroke);
                let arc_pts = [
                    to_screen(0.0, -22.0),
                    to_screen(10.0, -15.0),
                    to_screen(18.0, 0.0),
                    to_screen(10.0, 15.0),
                    to_screen(0.0, 22.0),
                ];
                for i in 0..arc_pts.len() - 1 {
                    painter.line_segment([arc_pts[i], arc_pts[i + 1]], stroke);
                }
                painter.line_segment([to_screen(18.0, 0.0), to_screen(30.0, 0.0)], stroke);
            }
            Self::OrGate => {
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
                    to_screen(18.0, 0.0),
                ];
                let front_bot = [
                    to_screen(-18.0, 22.0),
                    to_screen(2.0, 16.0),
                    to_screen(18.0, 0.0),
                ];
                painter.line_segment([front_top[0], front_top[1]], stroke);
                painter.line_segment([front_top[1], front_top[2]], stroke);
                painter.line_segment([front_bot[0], front_bot[1]], stroke);
                painter.line_segment([front_bot[1], front_bot[2]], stroke);
                painter.line_segment([to_screen(18.0, 0.0), to_screen(30.0, 0.0)], stroke);
            }
            Self::XorGate => {
                painter.line_segment([to_screen(-30.0, -15.0), to_screen(-20.0, -15.0)], stroke);
                painter.line_segment([to_screen(-30.0, 15.0), to_screen(-20.0, 15.0)], stroke);
                let outer_back = [
                    to_screen(-24.0, -22.0),
                    to_screen(-18.0, 0.0),
                    to_screen(-24.0, 22.0),
                ];
                painter.line_segment([outer_back[0], outer_back[1]], stroke);
                painter.line_segment([outer_back[1], outer_back[2]], stroke);
                let inner_back = [
                    to_screen(-18.0, -22.0),
                    to_screen(-12.0, 0.0),
                    to_screen(-18.0, 22.0),
                ];
                painter.line_segment([inner_back[0], inner_back[1]], stroke);
                painter.line_segment([inner_back[1], inner_back[2]], stroke);
                let front_top = [
                    to_screen(-18.0, -22.0),
                    to_screen(2.0, -16.0),
                    to_screen(18.0, 0.0),
                ];
                let front_bot = [
                    to_screen(-18.0, 22.0),
                    to_screen(2.0, 16.0),
                    to_screen(18.0, 0.0),
                ];
                painter.line_segment([front_top[0], front_top[1]], stroke);
                painter.line_segment([front_top[1], front_top[2]], stroke);
                painter.line_segment([front_bot[0], front_bot[1]], stroke);
                painter.line_segment([front_bot[1], front_bot[2]], stroke);
                painter.line_segment([to_screen(18.0, 0.0), to_screen(30.0, 0.0)], stroke);
            }
            Self::XnorGate => {
                painter.line_segment([to_screen(-30.0, -15.0), to_screen(-20.0, -15.0)], stroke);
                painter.line_segment([to_screen(-30.0, 15.0), to_screen(-20.0, 15.0)], stroke);
                let outer_back = [
                    to_screen(-24.0, -22.0),
                    to_screen(-18.0, 0.0),
                    to_screen(-24.0, 22.0),
                ];
                painter.line_segment([outer_back[0], outer_back[1]], stroke);
                painter.line_segment([outer_back[1], outer_back[2]], stroke);
                let inner_back = [
                    to_screen(-18.0, -22.0),
                    to_screen(-12.0, 0.0),
                    to_screen(-18.0, 22.0),
                ];
                painter.line_segment([inner_back[0], inner_back[1]], stroke);
                painter.line_segment([inner_back[1], inner_back[2]], stroke);
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
            Self::HalfAdder => {
                painter.rect_stroke(
                    Rect::from_center_size(to_screen(0.0, 0.0), Vec2::new(48.0 * zoom, 44.0 * zoom)),
                    2.0,
                    stroke,
                    StrokeKind::Middle,
                );
                painter.line_segment([to_screen(-40.0, -15.0), to_screen(-24.0, -15.0)], stroke);
                painter.line_segment([to_screen(-40.0, 15.0), to_screen(-24.0, 15.0)], stroke);
                painter.line_segment([to_screen(24.0, -15.0), to_screen(40.0, -15.0)], stroke);
                painter.line_segment([to_screen(24.0, 15.0), to_screen(40.0, 15.0)], stroke);
                // "HA" symbol inside
                painter.line_segment([to_screen(-10.0, -8.0), to_screen(-10.0, 8.0)], stroke);
                painter.line_segment([to_screen(-10.0, 0.0), to_screen(-2.0, 0.0)], stroke);
                painter.line_segment([to_screen(-2.0, -8.0), to_screen(-2.0, 8.0)], stroke);
                painter.line_segment([to_screen(2.0, 8.0), to_screen(6.0, -8.0)], stroke);
                painter.line_segment([to_screen(6.0, -8.0), to_screen(10.0, 8.0)], stroke);
                painter.line_segment([to_screen(3.5, 2.0), to_screen(8.5, 2.0)], stroke);
            }
            Self::FullAdder => {
                painter.rect_stroke(
                    Rect::from_center_size(to_screen(0.0, 0.0), Vec2::new(48.0 * zoom, 54.0 * zoom)),
                    2.0,
                    stroke,
                    StrokeKind::Middle,
                );
                painter.line_segment([to_screen(-40.0, -20.0), to_screen(-24.0, -20.0)], stroke);
                painter.line_segment([to_screen(-40.0, 0.0), to_screen(-24.0, 0.0)], stroke);
                painter.line_segment([to_screen(-40.0, 20.0), to_screen(-24.0, 20.0)], stroke);
                painter.line_segment([to_screen(24.0, -15.0), to_screen(40.0, -15.0)], stroke);
                painter.line_segment([to_screen(24.0, 15.0), to_screen(40.0, 15.0)], stroke);
                // "FA" glyph
                painter.line_segment([to_screen(-10.0, -8.0), to_screen(-10.0, 8.0)], stroke);
                painter.line_segment([to_screen(-10.0, -8.0), to_screen(-3.0, -8.0)], stroke);
                painter.line_segment([to_screen(-10.0, 0.0), to_screen(-4.0, 0.0)], stroke);
                painter.line_segment([to_screen(2.0, 8.0), to_screen(6.0, -8.0)], stroke);
                painter.line_segment([to_screen(6.0, -8.0), to_screen(10.0, 8.0)], stroke);
            }
            Self::Mux4to1 => {
                let trap = [
                    to_screen(-15.0, -32.0),
                    to_screen(15.0, -20.0),
                    to_screen(15.0, 20.0),
                    to_screen(-15.0, 32.0),
                ];
                for i in 0..4 {
                    painter.line_segment([trap[i], trap[(i + 1) % 4]], stroke);
                }
                for y in [-25.0, -10.0, 10.0, 25.0] {
                    painter.line_segment([to_screen(-30.0, y), to_screen(-15.0, y)], stroke);
                }
                painter.line_segment([to_screen(-10.0, 35.0), to_screen(-10.0, 26.0)], stroke);
                painter.line_segment([to_screen(10.0, 35.0), to_screen(10.0, 22.0)], stroke);
                painter.line_segment([to_screen(15.0, 0.0), to_screen(30.0, 0.0)], stroke);
            }
            Self::Demux1to2 => {
                let trap = [
                    to_screen(-15.0, -15.0),
                    to_screen(15.0, -25.0),
                    to_screen(15.0, 25.0),
                    to_screen(-15.0, 15.0),
                ];
                for i in 0..4 {
                    painter.line_segment([trap[i], trap[(i + 1) % 4]], stroke);
                }
                painter.line_segment([to_screen(-30.0, 0.0), to_screen(-15.0, 0.0)], stroke);
                painter.line_segment([to_screen(0.0, 30.0), to_screen(0.0, 20.0)], stroke);
                painter.line_segment([to_screen(15.0, -15.0), to_screen(30.0, -15.0)], stroke);
                painter.line_segment([to_screen(15.0, 15.0), to_screen(30.0, 15.0)], stroke);
            }
            Self::DFlipFlop => {
                painter.rect_stroke(
                    Rect::from_center_size(to_screen(0.0, 0.0), Vec2::new(44.0 * zoom, 44.0 * zoom)),
                    2.0,
                    stroke,
                    StrokeKind::Middle,
                );
                painter.line_segment([to_screen(-35.0, -15.0), to_screen(-22.0, -15.0)], stroke);
                painter.line_segment([to_screen(-35.0, 15.0), to_screen(-22.0, 15.0)], stroke);
                painter.line_segment([to_screen(22.0, -15.0), to_screen(35.0, -15.0)], stroke);
                painter.circle_stroke(to_screen(25.0, 15.0), 3.0 * zoom, stroke);
                painter.line_segment([to_screen(28.0, 15.0), to_screen(35.0, 15.0)], stroke);
                // Clock triangle marker
                painter.line_segment([to_screen(-22.0, 10.0), to_screen(-14.0, 15.0)], stroke);
                painter.line_segment([to_screen(-14.0, 15.0), to_screen(-22.0, 20.0)], stroke);
            }
            Self::SrLatch => {
                painter.rect_stroke(
                    Rect::from_center_size(to_screen(0.0, 0.0), Vec2::new(44.0 * zoom, 44.0 * zoom)),
                    2.0,
                    stroke,
                    StrokeKind::Middle,
                );
                painter.line_segment([to_screen(-35.0, -15.0), to_screen(-22.0, -15.0)], stroke);
                painter.line_segment([to_screen(-35.0, 15.0), to_screen(-22.0, 15.0)], stroke);
                painter.line_segment([to_screen(22.0, -15.0), to_screen(35.0, -15.0)], stroke);
                painter.circle_stroke(to_screen(25.0, 15.0), 3.0 * zoom, stroke);
                painter.line_segment([to_screen(28.0, 15.0), to_screen(35.0, 15.0)], stroke);
            }
            Self::Counter4Bit => {
                painter.rect_stroke(
                    Rect::from_center_size(to_screen(0.0, 0.0), Vec2::new(48.0 * zoom, 70.0 * zoom)),
                    2.0,
                    stroke,
                    StrokeKind::Middle,
                );
                painter.line_segment([to_screen(-40.0, -15.0), to_screen(-24.0, -15.0)], stroke);
                painter.line_segment([to_screen(-40.0, 15.0), to_screen(-24.0, 15.0)], stroke);
                for y in [-30.0, -10.0, 10.0, 30.0] {
                    painter.line_segment([to_screen(24.0, y), to_screen(40.0, y)], stroke);
                }
                // Clock marker
                painter.line_segment([to_screen(-24.0, -20.0), to_screen(-16.0, -15.0)], stroke);
                painter.line_segment([to_screen(-16.0, -15.0), to_screen(-24.0, -10.0)], stroke);
            }
            Self::Comparator => {
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
                // Plus & minus markers
                painter.line_segment([to_screen(-20.0, -15.0), to_screen(-14.0, -15.0)], stroke);
                painter.line_segment([to_screen(-17.0, -18.0), to_screen(-17.0, -12.0)], stroke);
                painter.line_segment([to_screen(-20.0, 15.0), to_screen(-14.0, 15.0)], stroke);
            }
            Self::Timer555 => {
                painter.rect_stroke(
                    Rect::from_center_size(to_screen(0.0, 0.0), Vec2::new(40.0 * zoom, 70.0 * zoom)),
                    2.0,
                    stroke,
                    StrokeKind::Middle,
                );
                // Left 4 pins
                for y in [-30.0, -10.0, 10.0, 30.0] {
                    painter.line_segment([to_screen(-30.0, y), to_screen(-20.0, y)], stroke);
                }
                // Right 4 pins
                for y in [-30.0, -10.0, 10.0, 30.0] {
                    painter.line_segment([to_screen(20.0, y), to_screen(30.0, y)], stroke);
                }
                // Notch at top
                painter.line_segment([to_screen(-6.0, -35.0), to_screen(6.0, -35.0)], stroke);
            }
            Self::VoltageRegulator => {
                painter.rect_stroke(
                    Rect::from_center_size(to_screen(0.0, 0.0), Vec2::new(40.0 * zoom, 34.0 * zoom)),
                    2.0,
                    stroke,
                    StrokeKind::Middle,
                );
                painter.line_segment([to_screen(-30.0, 0.0), to_screen(-20.0, 0.0)], stroke);
                painter.line_segment([to_screen(0.0, 17.0), to_screen(0.0, 30.0)], stroke);
                painter.line_segment([to_screen(20.0, 0.0), to_screen(30.0, 0.0)], stroke);
                // Center line
                painter.line_segment([to_screen(-10.0, -8.0), to_screen(10.0, -8.0)], stroke);
            }
            Self::LogicProbe => {
                painter.line_segment([to_screen(-20.0, 0.0), to_screen(-10.0, 0.0)], stroke);
                painter.circle_stroke(to_screen(5.0, 0.0), 12.0 * zoom, stroke);
                // "P" glyph
                painter.line_segment([to_screen(2.0, -6.0), to_screen(2.0, 6.0)], stroke);
                painter.line_segment([to_screen(2.0, -6.0), to_screen(8.0, -6.0)], stroke);
                painter.line_segment([to_screen(8.0, -6.0), to_screen(8.0, 0.0)], stroke);
                painter.line_segment([to_screen(2.0, 0.0), to_screen(8.0, 0.0)], stroke);
            }
            Self::SevenSegment => {
                painter.rect_stroke(
                    Rect::from_center_size(to_screen(0.0, 0.0), Vec2::new(40.0 * zoom, 64.0 * zoom)),
                    2.0,
                    stroke,
                    StrokeKind::Middle,
                );
                // 7-segment standard layout
                painter.line_segment([to_screen(-8.0, -18.0), to_screen(8.0, -18.0)], stroke);
                painter.line_segment([to_screen(8.0, -18.0), to_screen(8.0, -2.0)], stroke);
                painter.line_segment([to_screen(8.0, 2.0), to_screen(8.0, 18.0)], stroke);
                painter.line_segment([to_screen(-8.0, 18.0), to_screen(8.0, 18.0)], stroke);
                painter.line_segment([to_screen(-8.0, 2.0), to_screen(-8.0, 18.0)], stroke);
                painter.line_segment([to_screen(-8.0, -18.0), to_screen(-8.0, -2.0)], stroke);
                painter.line_segment([to_screen(-8.0, 0.0), to_screen(8.0, 0.0)], stroke);
                // pin leads
                for y in [-30.0, -10.0, 10.0, 30.0] {
                    painter.line_segment([to_screen(-30.0, y), to_screen(-20.0, y)], stroke);
                }
                for y in [-30.0, -10.0, 10.0] {
                    painter.line_segment([to_screen(20.0, y), to_screen(30.0, y)], stroke);
                }
                painter.line_segment([to_screen(0.0, 32.0), to_screen(0.0, 35.0)], stroke);
            }
            Self::Buzzer => {
                painter.line_segment([to_screen(0.0, -30.0), to_screen(0.0, -14.0)], stroke);
                painter.circle_stroke(to_screen(0.0, 0.0), 14.0 * zoom, stroke);
                painter.line_segment([to_screen(-8.0, -4.0), to_screen(8.0, -4.0)], stroke);
                painter.line_segment([to_screen(-8.0, 4.0), to_screen(8.0, 4.0)], stroke);
                painter.line_segment([to_screen(0.0, 14.0), to_screen(0.0, 30.0)], stroke);
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
            Self::PhLungs => {
                // Trachea main stem
                painter.line_segment([to_screen(0.0, -32.0), to_screen(0.0, -10.0)], stroke);
                // Bronchial bifurcation
                painter.line_segment([to_screen(0.0, -10.0), to_screen(-12.0, -2.0)], stroke);
                painter.line_segment([to_screen(0.0, -10.0), to_screen(12.0, -2.0)], stroke);
                // Left and right pulmonary compliance lobes
                painter.circle_stroke(to_screen(-12.0, 8.0), 10.0 * zoom, stroke);
                painter.circle_stroke(to_screen(12.0, 8.0), 10.0 * zoom, stroke);
                // Lower reference ground line
                painter.line_segment([to_screen(0.0, 18.0), to_screen(0.0, 32.0)], stroke);
            }
            Self::PhVocalFolds => {
                // Convergent-divergent vocal fold tissue boundaries
                painter.line_segment([to_screen(-20.0, -18.0), to_screen(-6.0, 0.0)], stroke);
                painter.line_segment([to_screen(-6.0, 0.0), to_screen(-18.0, 18.0)], stroke);
                painter.line_segment([to_screen(-18.0, 18.0), to_screen(-20.0, -18.0)], stroke);

                painter.line_segment([to_screen(20.0, -18.0), to_screen(6.0, 0.0)], stroke);
                painter.line_segment([to_screen(6.0, 0.0), to_screen(18.0, 18.0)], stroke);
                painter.line_segment([to_screen(18.0, 18.0), to_screen(20.0, -18.0)], stroke);

                // Mucosal wave glottal vibration indicator lines
                painter.line_segment([to_screen(-3.0, -8.0), to_screen(-3.0, 8.0)], stroke);
                painter.line_segment([to_screen(3.0, -8.0), to_screen(3.0, 8.0)], stroke);

                // Subglottal and supraglottal pins
                painter.line_segment([to_screen(-32.0, 0.0), to_screen(-20.0, 0.0)], stroke);
                painter.line_segment([to_screen(20.0, 0.0), to_screen(32.0, 0.0)], stroke);
                // Control and reference pins
                painter.line_segment([to_screen(0.0, -32.0), to_screen(0.0, -18.0)], stroke);
                painter.line_segment([to_screen(0.0, 18.0), to_screen(0.0, 32.0)], stroke);
            }
            Self::PhVocalTract => {
                // Flaring Webster horn profile
                painter.line_segment([to_screen(-24.0, -8.0), to_screen(-6.0, -10.0)], stroke);
                painter.line_segment([to_screen(-6.0, -10.0), to_screen(12.0, -15.0)], stroke);
                painter.line_segment([to_screen(12.0, -15.0), to_screen(24.0, -22.0)], stroke);

                painter.line_segment([to_screen(-24.0, 8.0), to_screen(-6.0, 10.0)], stroke);
                painter.line_segment([to_screen(-6.0, 10.0), to_screen(12.0, 15.0)], stroke);
                painter.line_segment([to_screen(12.0, 15.0), to_screen(24.0, 22.0)], stroke);

                // Entrance and exit boundaries
                painter.line_segment([to_screen(-24.0, -8.0), to_screen(-24.0, 8.0)], stroke);
                painter.line_segment([to_screen(24.0, -22.0), to_screen(24.0, 22.0)], stroke);

                // Internal acoustic waveguide slice lines
                painter.line_segment([to_screen(-6.0, -10.0), to_screen(-6.0, 10.0)], stroke);
                painter.line_segment([to_screen(12.0, -15.0), to_screen(12.0, 15.0)], stroke);

                // Terminals
                painter.line_segment([to_screen(-40.0, 0.0), to_screen(-24.0, 0.0)], stroke);
                painter.line_segment([to_screen(24.0, 0.0), to_screen(40.0, 0.0)], stroke);
                painter.line_segment([to_screen(0.0, -32.0), to_screen(0.0, -12.0)], stroke);
                painter.line_segment([to_screen(0.0, 12.0), to_screen(0.0, 32.0)], stroke);
            }
            Self::PhLipRadiation => {
                // Mouth orifice boundary
                painter.line_segment([to_screen(-30.0, 0.0), to_screen(-12.0, 0.0)], stroke);
                painter.line_segment([to_screen(-12.0, -18.0), to_screen(-12.0, 18.0)], stroke);

                // Radiating spherical wavefront arcs
                painter.line_segment([to_screen(-6.0, -8.0), to_screen(-2.0, 0.0)], stroke);
                painter.line_segment([to_screen(-2.0, 0.0), to_screen(-6.0, 8.0)], stroke);

                painter.line_segment([to_screen(0.0, -14.0), to_screen(8.0, 0.0)], stroke);
                painter.line_segment([to_screen(8.0, 0.0), to_screen(0.0, 14.0)], stroke);

                painter.line_segment([to_screen(6.0, -20.0), to_screen(18.0, 0.0)], stroke);
                painter.line_segment([to_screen(18.0, 0.0), to_screen(6.0, 20.0)], stroke);

                // Outward radiation terminal
                painter.line_segment([to_screen(18.0, 0.0), to_screen(30.0, 0.0)], stroke);
            }
        }
    }

    /// Returns default (name_offset, value_offset) in local world coordinates.
    pub fn default_label_offsets(&self) -> (Vec2, Vec2) {
        match self {
            Self::VoltageSource
            | Self::AcVoltageSource
            | Self::CurrentSource
            | Self::PulseGenerator
            | Self::ClockSource => {
                // Circle radius is 20.0 px. Offset to x=28.0 to guarantee zero collision with circle perimeter.
                (Vec2::new(28.0, -10.0), Vec2::new(28.0, 8.0))
            }
            Self::Transformer | Self::SawIdt | Self::ParafermionicCavity => {
                (Vec2::new(26.0, -12.0), Vec2::new(26.0, 8.0))
            }
            Self::OpAmp
            | Self::Comparator
            | Self::Timer555
            | Self::VoltageRegulator
            | Self::Inverter
            | Self::BufferGate
            | Self::AndGate
            | Self::OrGate
            | Self::NandGate
            | Self::NorGate
            | Self::XorGate
            | Self::XnorGate
            | Self::HalfAdder
            | Self::FullAdder
            | Self::Mux2to1
            | Self::Mux4to1
            | Self::Demux1to2
            | Self::DFlipFlop
            | Self::SrLatch
            | Self::Counter4Bit
            | Self::SevenSegment => {
                (Vec2::new(28.0, -12.0), Vec2::new(28.0, 8.0))
            }
            Self::PhLungs | Self::PhVocalFolds | Self::PhVocalTract | Self::PhLipRadiation => {
                (Vec2::new(26.0, -12.0), Vec2::new(26.0, 8.0))
            }
            Self::Ground => (Vec2::new(16.0, 4.0), Vec2::new(16.0, 18.0)),
            Self::VddRail => (Vec2::new(16.0, -8.0), Vec2::new(16.0, 8.0)),
            _ => (Vec2::new(20.0, -10.0), Vec2::new(20.0, 8.0)),
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
    /// Mirror state (false = normal, true = mirrored horizontally before rotation).
    pub mirrored: bool,
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
            mirrored: false,
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

    /// Builder method setting the parameter value string.
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value_str = value.into();
        self
    }

    /// Looks up a custom property value by key.
    pub fn get_property(&self, key: &str) -> Option<&str> {
        self.properties
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// Sets or updates a custom property key-value pair.
    pub fn set_property(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let key_str = key.into();
        let val_str = value.into();
        if let Some(prop) = self.properties.iter_mut().find(|(k, _)| k == &key_str) {
            prop.1 = val_str;
        } else {
            self.properties.push((key_str, val_str));
        }
    }

    /// Returns (name_offset, value_offset) in local world coordinates.
    pub fn label_offsets(&self) -> (Vec2, Vec2) {
        let (def_name, def_val) = self.kind.default_label_offsets();
        let name_x = self
            .get_property("label_offset_x")
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(def_name.x);
        let name_y = self
            .get_property("label_offset_y")
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(def_name.y);
        let val_x = self
            .get_property("value_offset_x")
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(def_val.x);
        let val_y = self
            .get_property("value_offset_y")
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(def_val.y);
        (Vec2::new(name_x, name_y), Vec2::new(val_x, val_y))
    }

    /// Sets custom label offset for the component name.
    pub fn set_label_offset(&mut self, offset: Vec2) {
        self.set_property("label_offset_x", format!("{:.1}", offset.x));
        self.set_property("label_offset_y", format!("{:.1}", offset.y));
    }

    /// Sets custom label offset for the component value.
    pub fn set_value_offset(&mut self, offset: Vec2) {
        self.set_property("value_offset_x", format!("{:.1}", offset.x));
        self.set_property("value_offset_y", format!("{:.1}", offset.y));
    }

    /// Rotates the component clockwise by 90 degrees.
    pub fn rotate_clockwise(&mut self) {
        self.rotation = (self.rotation + 1) % 4;
    }

    /// Toggles the horizontal mirror state of the component (flipping across the local vertical axis).
    pub fn mirror_horizontal(&mut self) {
        self.mirrored = !self.mirrored;
    }

    /// Returns the current rotation angle in degrees (0, 90, 180, or 270).
    pub fn rotation_degrees(&self) -> u32 {
        (self.rotation % 4) as u32 * 90
    }

    /// Transforms a local vector according to the component's mirror state and rotation.
    pub fn transform_vec(&self, v: Vec2) -> Vec2 {
        let mx = if self.mirrored { -v.x } else { v.x };
        let my = v.y;
        match self.rotation % 4 {
            0 => Vec2::new(mx, my),
            1 => Vec2::new(-my, mx),  // 90 deg CW
            2 => Vec2::new(-mx, -my), // 180 deg
            3 => Vec2::new(my, -mx),  // 270 deg
            _ => Vec2::new(mx, my),
        }
    }

    /// Transforms a local vector according to the component's orientation (alias for backwards compatibility).
    pub fn rotate_vec(&self, v: Vec2) -> Vec2 {
        self.transform_vec(v)
    }

    /// Returns the world position of pin at `pin_idx`.
    pub fn pin_world_pos(&self, pin_idx: usize) -> Option<Pos2> {
        let pins = self.kind.pin_definitions();
        let &(_, local_offset) = pins.get(pin_idx)?;
        let rotated = self.transform_vec(local_offset);
        Some(self.pos + rotated)
    }

    /// Returns all pin names and their world coordinates.
    pub fn all_pins(&self) -> Vec<(&'static str, Pos2)> {
        let pins = self.kind.pin_definitions();
        pins.iter()
            .map(|&(name, offset)| (name, self.pos + self.transform_vec(offset)))
            .collect()
    }

    /// Returns the departure normal of the pin at `pin_idx` in world coordinates.
    pub fn pin_normal(&self, pin_idx: usize) -> crate::schematic::wire::PinNormal {
        let pins = self.kind.pin_definitions();
        let &(_, local_offset) = match pins.get(pin_idx) {
            Some(p) => p,
            None => return crate::schematic::wire::PinNormal::North,
        };
        let local_norm = if local_offset.y.abs() >= local_offset.x.abs() {
            if local_offset.y <= 0.0 {
                Vec2::new(0.0, -1.0)
            } else {
                Vec2::new(0.0, 1.0)
            }
        } else {
            if local_offset.x <= 0.0 {
                Vec2::new(-1.0, 0.0)
            } else {
                Vec2::new(1.0, 0.0)
            }
        };
        let rotated_norm = self.transform_vec(local_norm);
        crate::schematic::wire::PinNormal::from_vec(rotated_norm)
    }

    /// Returns the axis-aligned bounding box of this component in world coordinates.
    /// Narrowed for voltage sources and two-pin passives by 1 grid unit (20.0 px) on both sides.
    pub fn bounding_box(&self) -> Rect {
        let (w, h) = match self.kind {
            ComponentKind::VoltageSource
            | ComponentKind::AcVoltageSource
            | ComponentKind::CurrentSource
            | ComponentKind::PulseGenerator
            | ComponentKind::ClockSource
            | ComponentKind::Resistor
            | ComponentKind::Potentiometer
            | ComponentKind::Capacitor
            | ComponentKind::Inductor
            | ComponentKind::Diode
            | ComponentKind::ZenerDiode
            | ComponentKind::Led
            | ComponentKind::SchottkyDiode
            | ComponentKind::Buzzer => {
                if self.rotation % 2 == 0 {
                    (30.0, 70.0)
                } else {
                    (70.0, 30.0)
                }
            }
            ComponentKind::Ground | ComponentKind::VddRail | ComponentKind::LogicProbe => (30.0, 40.0),
            ComponentKind::HalfAdder
            | ComponentKind::FullAdder
            | ComponentKind::Counter4Bit
            | ComponentKind::Timer555
            | ComponentKind::SevenSegment => (70.0, 70.0),
            _ => (60.0, 60.0),
        };
        Rect::from_center_size(self.pos, Vec2::new(w, h))
    }

    /// Checks if this component's bounding box intersects with the given rectangle in world coordinates.
    pub fn intersects_rect(&self, rect: &Rect) -> bool {
        rect.intersects(self.bounding_box())
    }

    /// Hit-test: checks if a world position lies within the component's bounding box.
    pub fn contains(&self, world_pos: Pos2) -> bool {
        self.bounding_box().contains(world_pos)
    }

    /// Renders the component schematic symbol onto the screen painter.
    pub fn render(
        &self,
        painter: &Painter,
        canvas: &SchematicCanvas,
        is_selected: bool,
        node_voltages: Option<&[(&str, f64)]>,
    ) {
        self.render_with_theme(
            painter,
            canvas,
            is_selected,
            node_voltages,
            &crate::theme::PhononTheme::default(),
        );
    }

    /// Renders the component onto the painter using customizable theme colors.
    pub fn render_with_theme(
        &self,
        painter: &Painter,
        canvas: &SchematicCanvas,
        is_selected: bool,
        node_voltages: Option<&[(&str, f64)]>,
        theme: &crate::theme::PhononTheme,
    ) {
        self.render_with_theme_and_override(
            painter,
            canvas,
            is_selected,
            node_voltages,
            theme,
            None,
        );
    }

    /// Renders the component onto the painter with customizable theme colors and optional pin color override.
    pub fn render_with_theme_and_override(
        &self,
        painter: &Painter,
        canvas: &SchematicCanvas,
        is_selected: bool,
        node_voltages: Option<&[(&str, f64)]>,
        theme: &crate::theme::PhononTheme,
        pin_color_override: Option<Color32>,
    ) {
        let stroke_color = if is_selected {
            theme.component_selected
        } else {
            theme.component_stroke
        };
        let stroke = Stroke::new(2.0 * canvas.zoom.clamp(0.8, 2.0), stroke_color);

        // Helper to transform local component coords to screen
        let to_screen = |lx: f32, ly: f32| -> Pos2 {
            let rotated = self.transform_vec(Vec2::new(lx, ly));
            canvas.world_to_screen(self.pos + rotated)
        };

        // Render geometric symbol strokes
        self.kind.draw_symbol(painter, stroke, &to_screen, canvas.zoom);

        // Draw pin snap dots
        let pin_color = pin_color_override.unwrap_or(theme.pin_normal);
        let pin_radius = if pin_color_override.is_some() {
            4.5 * canvas.zoom.clamp(0.8, 1.6)
        } else {
            3.5 * canvas.zoom.clamp(0.8, 1.4)
        };
        for (_, p_world) in self.all_pins() {
            let p_screen = canvas.world_to_screen(p_world);
            painter.circle_filled(p_screen, pin_radius, pin_color);
        }

        // Draw labels (Name and Value)
        let (name_offset, val_offset) = self.label_offsets();
        let label_pos = canvas.world_to_screen(self.pos + name_offset);
        let val_pos = canvas.world_to_screen(self.pos + val_offset);
        let font_size = 12.0 * canvas.zoom.clamp(0.8, 1.8);

        painter.text(
            label_pos,
            Align2::LEFT_CENTER,
            &self.name,
            FontId::proportional(font_size),
            if is_selected {
                theme.component_selected
            } else {
                theme.text_primary
            },
        );

        if self.kind != ComponentKind::Ground {
            painter.text(
                val_pos,
                Align2::LEFT_CENTER,
                &self.value_str,
                FontId::proportional(font_size * 0.9),
                theme.text_secondary,
            );
        }

        // Optional live voltage readout badges near pins
        if let Some(voltages) = node_voltages {
            for (pin_name, pin_pos) in self.all_pins() {
                if let Some(&(_, v)) = voltages.iter().find(|(name, _)| *name == pin_name) {
                    let badge_pos = canvas.world_to_screen(pin_pos + Vec2::new(16.0, -10.0));
                    let text = format!("{:.2}V", v);
                    let style = crate::widgets::pill_badge::PillBadgeStyle::voltage(theme.voltage_badge);
                    crate::widgets::pill_badge::render_pill_badge(
                        painter,
                        badge_pos,
                        &text,
                        &style,
                        canvas.zoom,
                    );
                }
            }
        }
    }
}
