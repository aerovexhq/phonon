//! Abstract Syntax Tree (AST) representing SPICE netlist components, models, and directives.

use std::collections::HashMap;

/// An individual electrical component parsed from a SPICE netlist.
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentAst {
    Resistor {
        name: String,
        pos: String,
        neg: String,
        resistance: f64,
    },
    Capacitor {
        name: String,
        pos: String,
        neg: String,
        capacitance: f64,
        initial_voltage: Option<f64>,
    },
    Inductor {
        name: String,
        pos: String,
        neg: String,
        inductance: f64,
        initial_current: Option<f64>,
    },
    VoltageSource {
        name: String,
        pos: String,
        neg: String,
        dc_value: f64,
    },
    CurrentSource {
        name: String,
        pos: String,
        neg: String,
        dc_value: f64,
    },
    Vcvs {
        name: String,
        out_pos: String,
        out_neg: String,
        ctrl_pos: String,
        ctrl_neg: String,
        gain: f64,
    },
    Vccs {
        name: String,
        out_pos: String,
        out_neg: String,
        ctrl_pos: String,
        ctrl_neg: String,
        transconductance: f64,
    },
    Diode {
        name: String,
        pos: String,
        neg: String,
        model_name: String,
    },
    Mosfet {
        name: String,
        drain: String,
        gate: String,
        source: String,
        bulk: String,
        model_name: String,
        w: Option<f64>,
        l: Option<f64>,
    },
    Bjt {
        name: String,
        collector: String,
        base: String,
        emitter: String,
        model_name: String,
    },
    SubcircuitInstance {
        name: String,
        pins: Vec<String>,
        subcircuit_name: String,
    },
    TransmissionLine {
        name: String,
        in_pos: String,
        in_neg: String,
        out_pos: String,
        out_neg: String,
        z0: f64,
        td: f64,
    },
    A2dBridge {
        name: String,
        in_pos: String,
        in_neg: String,
        out_dig: String,
        vth_low: f64,
        vth_high: f64,
        r_in: Option<f64>,
    },
    D2aBridge {
        name: String,
        in_dig: String,
        out_pos: String,
        out_neg: String,
        v_low: f64,
        v_high: f64,
        rise_time: f64,
        fall_time: f64,
        r_out: f64,
    },
}

impl ComponentAst {
    pub fn name(&self) -> &str {
        match self {
            ComponentAst::Resistor { name, .. }
            | ComponentAst::Capacitor { name, .. }
            | ComponentAst::Inductor { name, .. }
            | ComponentAst::VoltageSource { name, .. }
            | ComponentAst::CurrentSource { name, .. }
            | ComponentAst::Vcvs { name, .. }
            | ComponentAst::Vccs { name, .. }
            | ComponentAst::Diode { name, .. }
            | ComponentAst::Mosfet { name, .. }
            | ComponentAst::Bjt { name, .. }
            | ComponentAst::SubcircuitInstance { name, .. }
            | ComponentAst::TransmissionLine { name, .. }
            | ComponentAst::A2dBridge { name, .. }
            | ComponentAst::D2aBridge { name, .. } => name,
        }
    }
}

/// A semiconductor device `.MODEL` definition card.
#[derive(Debug, Clone, PartialEq)]
pub struct ModelCard {
    pub name: String,
    /// Model type: D, NMOS, PMOS, NPN, PNP.
    pub model_type: String,
    /// Key-value parameters: e.g. "IS" -> 1e-14, "BF" -> 100.0, "VTH0" -> 0.7.
    pub params: HashMap<String, f64>,
}

/// Subcircuit definition block (`.SUBCKT name pin1 pin2 ...`).
#[derive(Debug, Clone, PartialEq)]
pub struct SubcircuitDef {
    pub name: String,
    pub pins: Vec<String>,
    pub components: Vec<ComponentAst>,
}

/// Simulation control directives.
#[derive(Debug, Clone, PartialEq)]
pub enum Directive {
    /// `.OP`: Operating point analysis
    Op,
    /// `.DC source_name start stop step`
    Dc {
        source_name: String,
        start: f64,
        stop: f64,
        step: f64,
    },
    /// `.TRAN tstep tstop`
    Tran { tstep: f64, tstop: f64 },
    /// `.TEMP temperature_celsius`
    Temp { temp_c: f64 },
}

/// Fully parsed SPICE netlist structure.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ParsedNetlist {
    pub title: String,
    pub components: Vec<ComponentAst>,
    pub models: HashMap<String, ModelCard>,
    pub subcircuits: HashMap<String, SubcircuitDef>,
    pub directives: Vec<Directive>,
}
