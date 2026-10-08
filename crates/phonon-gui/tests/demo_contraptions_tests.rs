#![deny(unsafe_code)]

//! Analytical verification and ERC validation test suite for Phonon Contraptions:
//! - 4-Bit Arithmetic Logic Unit (ALU) & Processor Slice
//! - Integrated RF Microwave-Acoustic Heterodyne Transceiver Front-End
//! - Topological Quantum Acoustic Metamaterial Super-Contraption

use egui::Pos2;
use phonon_gui::schematic::{
    circuit_compiler::compile_schematic,
    erc::{ErcEngine, ErcSeverity},
    ComponentKind, SchematicComponent, SchematicWire, WireSegment,
};

#[test]
fn test_4bit_alu_contraption_erc_and_compilation() {
    let mut components = Vec::new();
    let mut wires = Vec::new();
    let mut cid = 1usize;
    let mut wid = 1usize;

    // 1. Clock and Pulse Inputs
    let clk = SchematicComponent::new(cid, ComponentKind::ClockSource, Pos2::new(100.0, 180.0), 1); cid += 1;
    let va = SchematicComponent::new(cid, ComponentKind::PulseGenerator, Pos2::new(100.0, 300.0), 1); cid += 1;
    let vb = SchematicComponent::new(cid, ComponentKind::PulseGenerator, Pos2::new(100.0, 420.0), 2); cid += 1;
    let v_op = SchematicComponent::new(cid, ComponentKind::PulseGenerator, Pos2::new(100.0, 540.0), 3); cid += 1;
    let gnd = SchematicComponent::new(cid, ComponentKind::Ground, Pos2::new(100.0, 680.0), 1); cid += 1;

    // 2. Ripple Carry Full Adders (4 Bits)
    let fa0 = SchematicComponent::new(cid, ComponentKind::FullAdder, Pos2::new(320.0, 260.0), 1); cid += 1;
    let fa1 = SchematicComponent::new(cid, ComponentKind::FullAdder, Pos2::new(320.0, 380.0), 2); cid += 1;
    let fa2 = SchematicComponent::new(cid, ComponentKind::FullAdder, Pos2::new(320.0, 500.0), 3); cid += 1;
    let fa3 = SchematicComponent::new(cid, ComponentKind::FullAdder, Pos2::new(320.0, 620.0), 4); cid += 1;

    // 3. Bitwise Logic XOR Gates (4 Bits)
    let xor0 = SchematicComponent::new(cid, ComponentKind::XorGate, Pos2::new(460.0, 200.0), 1); cid += 1;
    let xor1 = SchematicComponent::new(cid, ComponentKind::XorGate, Pos2::new(460.0, 320.0), 2); cid += 1;
    let xor2 = SchematicComponent::new(cid, ComponentKind::XorGate, Pos2::new(460.0, 440.0), 3); cid += 1;
    let xor3 = SchematicComponent::new(cid, ComponentKind::XorGate, Pos2::new(460.0, 560.0), 4); cid += 1;

    // 4. Multiplexers (4 Bits: ADD vs XOR)
    let mux0 = SchematicComponent::new(cid, ComponentKind::Mux2to1, Pos2::new(580.0, 230.0), 1); cid += 1;
    let mux1 = SchematicComponent::new(cid, ComponentKind::Mux2to1, Pos2::new(580.0, 350.0), 2); cid += 1;
    let mux2 = SchematicComponent::new(cid, ComponentKind::Mux2to1, Pos2::new(580.0, 470.0), 3); cid += 1;
    let mux3 = SchematicComponent::new(cid, ComponentKind::Mux2to1, Pos2::new(580.0, 590.0), 4); cid += 1;

    // 5. Output Registers (D-Flip-Flops)
    let dff0 = SchematicComponent::new(cid, ComponentKind::DFlipFlop, Pos2::new(720.0, 230.0), 1); cid += 1;
    let dff1 = SchematicComponent::new(cid, ComponentKind::DFlipFlop, Pos2::new(720.0, 350.0), 2); cid += 1;
    let dff2 = SchematicComponent::new(cid, ComponentKind::DFlipFlop, Pos2::new(720.0, 470.0), 3); cid += 1;
    let dff3 = SchematicComponent::new(cid, ComponentKind::DFlipFlop, Pos2::new(720.0, 590.0), 4); cid += 1;

    // 6. Logic Probes (Q0..Q3, QN0..QN3, COUT)
    let prb_q0 = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(830.0, 215.0), 1); cid += 1;
    let prb_qn0 = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(830.0, 245.0), 2); cid += 1;
    let prb_q1 = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(830.0, 335.0), 3); cid += 1;
    let prb_qn1 = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(830.0, 365.0), 4); cid += 1;
    let prb_q2 = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(830.0, 455.0), 5); cid += 1;
    let prb_qn2 = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(830.0, 485.0), 6); cid += 1;
    let prb_q3 = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(830.0, 575.0), 7); cid += 1;
    let prb_qn3 = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(830.0, 605.0), 8); cid += 1;
    let prb_cout = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(830.0, 660.0), 9);

    // Ground Bus at X=60
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(100.0, 220.0), Pos2::new(60.0, 220.0)),
        WireSegment::new(Pos2::new(60.0, 220.0), Pos2::new(60.0, 660.0)),
        WireSegment::new(Pos2::new(60.0, 660.0), Pos2::new(100.0, 660.0)),
    ])); wid += 1;
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(100.0, 340.0), Pos2::new(60.0, 340.0)),
    ])); wid += 1;
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(100.0, 460.0), Pos2::new(60.0, 460.0)),
    ])); wid += 1;
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(100.0, 580.0), Pos2::new(60.0, 580.0)),
    ])); wid += 1;
    // Ground FA0 CIN (280, 280) to ground bus
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(60.0, 280.0), Pos2::new(280.0, 280.0)),
    ])); wid += 1;

    // Clock distribution line at X=660 to DFF CLKs (685, y)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(100.0, 140.0), Pos2::new(660.0, 140.0)),
        WireSegment::new(Pos2::new(660.0, 140.0), Pos2::new(660.0, 605.0)),
    ])); wid += 1;
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(660.0, 245.0), Pos2::new(685.0, 245.0)),
    ])); wid += 1;
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(660.0, 365.0), Pos2::new(685.0, 365.0)),
    ])); wid += 1;
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(660.0, 485.0), Pos2::new(685.0, 485.0)),
    ])); wid += 1;
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(660.0, 605.0), Pos2::new(685.0, 605.0)),
    ])); wid += 1;

    // Operation select line V_OP(+) (100, 500) to MUX SELs (580, y)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(100.0, 500.0), Pos2::new(160.0, 500.0)),
        WireSegment::new(Pos2::new(160.0, 500.0), Pos2::new(160.0, 640.0)),
        WireSegment::new(Pos2::new(160.0, 640.0), Pos2::new(580.0, 640.0)),
        WireSegment::new(Pos2::new(580.0, 640.0), Pos2::new(580.0, 620.0)),
    ])); wid += 1;
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(580.0, 620.0), Pos2::new(580.0, 260.0)),
    ])); wid += 1;

    // Input A distribution: VA(+) (100, 260) to FAs and XORs
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(100.0, 260.0), Pos2::new(200.0, 260.0)),
        WireSegment::new(Pos2::new(200.0, 260.0), Pos2::new(200.0, 600.0)),
    ])); wid += 1;
    // FA A inputs at (280, y)
    for y in [240.0, 360.0, 480.0, 600.0] {
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(200.0, y), Pos2::new(280.0, y)),
        ])); wid += 1;
    }
    // XOR A inputs at (430, y)
    for (y_src, y_dst) in [(240.0, 185.0), (360.0, 305.0), (480.0, 425.0), (600.0, 545.0)] {
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(280.0, y_src), Pos2::new(280.0, y_dst)),
            WireSegment::new(Pos2::new(280.0, y_dst), Pos2::new(430.0, y_dst)),
        ])); wid += 1;
    }

    // Input B distribution: VB(+) (100, 380) to FAs and XORs
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(100.0, 380.0), Pos2::new(240.0, 380.0)),
        WireSegment::new(Pos2::new(240.0, 380.0), Pos2::new(240.0, 620.0)),
    ])); wid += 1;
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(240.0, 380.0), Pos2::new(240.0, 260.0)),
    ])); wid += 1;
    // FA B inputs at (280, y)
    for y in [260.0, 380.0, 500.0, 620.0] {
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(240.0, y), Pos2::new(280.0, y)),
        ])); wid += 1;
    }
    // XOR B inputs at (430, y)
    for (y_src, y_dst) in [(260.0, 215.0), (380.0, 335.0), (500.0, 455.0), (620.0, 575.0)] {
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(240.0, y_src), Pos2::new(240.0, y_dst)),
            WireSegment::new(Pos2::new(240.0, y_dst), Pos2::new(430.0, y_dst)),
        ])); wid += 1;
    }

    // Ripple Carry propagation: COUT to CIN
    // FA0 COUT (360, 275) -> FA1 CIN (280, 400)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(360.0, 275.0), Pos2::new(390.0, 275.0)),
        WireSegment::new(Pos2::new(390.0, 275.0), Pos2::new(390.0, 330.0)),
        WireSegment::new(Pos2::new(390.0, 330.0), Pos2::new(260.0, 330.0)),
        WireSegment::new(Pos2::new(260.0, 330.0), Pos2::new(260.0, 400.0)),
        WireSegment::new(Pos2::new(260.0, 400.0), Pos2::new(280.0, 400.0)),
    ])); wid += 1;
    // FA1 COUT (360, 395) -> FA2 CIN (280, 520)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(360.0, 395.0), Pos2::new(390.0, 395.0)),
        WireSegment::new(Pos2::new(390.0, 395.0), Pos2::new(390.0, 450.0)),
        WireSegment::new(Pos2::new(390.0, 450.0), Pos2::new(260.0, 450.0)),
        WireSegment::new(Pos2::new(260.0, 450.0), Pos2::new(260.0, 520.0)),
        WireSegment::new(Pos2::new(260.0, 520.0), Pos2::new(280.0, 520.0)),
    ])); wid += 1;
    // FA2 COUT (360, 515) -> FA3 CIN (280, 640)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(360.0, 515.0), Pos2::new(390.0, 515.0)),
        WireSegment::new(Pos2::new(390.0, 515.0), Pos2::new(390.0, 570.0)),
        WireSegment::new(Pos2::new(390.0, 570.0), Pos2::new(260.0, 570.0)),
        WireSegment::new(Pos2::new(260.0, 570.0), Pos2::new(260.0, 640.0)),
        WireSegment::new(Pos2::new(260.0, 640.0), Pos2::new(280.0, 640.0)),
    ])); wid += 1;
    // FA3 COUT (360, 635) -> Carry Flag Probe (810, 660)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(360.0, 635.0), Pos2::new(400.0, 635.0)),
        WireSegment::new(Pos2::new(400.0, 635.0), Pos2::new(400.0, 660.0)),
        WireSegment::new(Pos2::new(400.0, 660.0), Pos2::new(810.0, 660.0)),
    ])); wid += 1;

    // FA SUM outputs to MUX D0 inputs (550, y)
    let fa_sums = [(245.0, 215.0), (365.0, 335.0), (485.0, 455.0), (605.0, 575.0)];
    for (y_fa, y_mux) in fa_sums {
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(360.0, y_fa), Pos2::new(510.0, y_fa)),
            WireSegment::new(Pos2::new(510.0, y_fa), Pos2::new(510.0, y_mux)),
            WireSegment::new(Pos2::new(510.0, y_mux), Pos2::new(550.0, y_mux)),
        ])); wid += 1;
    }

    // XOR outputs to MUX D1 inputs (550, y)
    let xor_outs = [(200.0, 245.0), (320.0, 365.0), (440.0, 485.0), (560.0, 605.0)];
    for (y_xor, y_mux) in xor_outs {
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(490.0, y_xor), Pos2::new(530.0, y_xor)),
            WireSegment::new(Pos2::new(530.0, y_xor), Pos2::new(530.0, y_mux)),
            WireSegment::new(Pos2::new(530.0, y_mux), Pos2::new(550.0, y_mux)),
        ])); wid += 1;
    }

    // MUX outputs (610, y) to DFF D inputs (685, y-15)
    for (y_mux, y_dff) in [(230.0, 215.0), (350.0, 335.0), (470.0, 455.0), (590.0, 575.0)] {
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(610.0, y_mux), Pos2::new(640.0, y_mux)),
            WireSegment::new(Pos2::new(640.0, y_mux), Pos2::new(640.0, y_dff)),
            WireSegment::new(Pos2::new(640.0, y_dff), Pos2::new(685.0, y_dff)),
        ])); wid += 1;
    }

    // DFF Q outputs (755, y) to Probes (810, y)
    for y in [215.0, 335.0, 455.0, 575.0] {
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(755.0, y), Pos2::new(810.0, y)),
        ])); wid += 1;
    }

    // DFF QN outputs (755, y) to Probes (810, y)
    for y in [245.0, 365.0, 485.0, 605.0] {
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(755.0, y), Pos2::new(810.0, y)),
        ])); wid += 1;
    }

    components.extend(vec![
        clk, va, vb, v_op, gnd,
        fa0, fa1, fa2, fa3,
        xor0, xor1, xor2, xor3,
        mux0, mux1, mux2, mux3,
        dff0, dff1, dff2, dff3,
        prb_q0, prb_qn0, prb_q1, prb_qn1, prb_q2, prb_qn2, prb_q3, prb_qn3, prb_cout,
    ]);

    let diags = ErcEngine::evaluate(&components, &wires);
    let errs: Vec<_> = diags.iter().filter(|d| d.severity == ErcSeverity::Error).collect();
    let warns: Vec<_> = diags.iter().filter(|d| d.severity == ErcSeverity::Warning).collect();
    if !diags.is_empty() {
        for d in &diags {
            eprintln!("[ERC {:?}] {}", d.severity, d.message);
        }
    }
    assert_eq!(errs.len(), 0, "4-Bit ALU must have 0 ERC errors");
    assert_eq!(warns.len(), 0, "4-Bit ALU must have 0 ERC warnings");

    let compiled = compile_schematic(&components, &wires);
    assert!(compiled.is_ok(), "4-Bit ALU schematic must compile cleanly");
}

#[test]
fn test_rf_transceiver_contraption_erc_and_compilation() {
    let mut components = Vec::new();
    let mut wires = Vec::new();
    let mut cid = 1usize;
    let mut wid = 1usize;

    // 1. RF Antenna Source (2.4 GHz) & Matching Network
    let v_rf = SchematicComponent::new(cid, ComponentKind::AcVoltageSource, Pos2::new(100.0, 240.0), 1); cid += 1;
    let gnd = SchematicComponent::new(cid, ComponentKind::Ground, Pos2::new(100.0, 480.0), 1); cid += 1;
    let c_match = SchematicComponent::new(cid, ComponentKind::Capacitor, Pos2::new(180.0, 200.0), 1); cid += 1;
    let l_match = SchematicComponent::new(cid, ComponentKind::Inductor, Pos2::new(220.0, 280.0), 1); cid += 1;

    // 2. DC Power Supply (+3.3V) & Bias Network
    let vcc = SchematicComponent::new(cid, ComponentKind::VoltageSource, Pos2::new(280.0, 120.0), 2); cid += 1;
    let rb1 = SchematicComponent::new(cid, ComponentKind::Resistor, Pos2::new(340.0, 140.0), 1); cid += 1;
    let rb2 = SchematicComponent::new(cid, ComponentKind::Resistor, Pos2::new(340.0, 320.0), 2); cid += 1;

    // 3. Discrete RF BJT Cascode LNA Stage
    let q1 = SchematicComponent::new(cid, ComponentKind::BjtNpn, Pos2::new(400.0, 240.0), 1); cid += 1;
    let rc = SchematicComponent::new(cid, ComponentKind::Resistor, Pos2::new(420.0, 140.0), 3); cid += 1;
    let re = SchematicComponent::new(cid, ComponentKind::Resistor, Pos2::new(420.0, 360.0), 4); cid += 1;
    let ce = SchematicComponent::new(cid, ComponentKind::Capacitor, Pos2::new(480.0, 360.0), 2); cid += 1;

    // 4. Dynamic Cauer Thermal Network
    let r_th1 = SchematicComponent::new(cid, ComponentKind::Resistor, Pos2::new(500.0, 100.0), 5); cid += 1;
    let c_th1 = SchematicComponent::new(cid, ComponentKind::Capacitor, Pos2::new(560.0, 100.0), 4); cid += 1;

    // 5. Heterodyne Mixer & Local Oscillator
    let cc1 = SchematicComponent::new(cid, ComponentKind::Capacitor, Pos2::new(500.0, 200.0), 3); cid += 1;
    let d_mix = SchematicComponent::new(cid, ComponentKind::SchottkyDiode, Pos2::new(560.0, 200.0), 1); cid += 1;
    let v_lo = SchematicComponent::new(cid, ComponentKind::AcVoltageSource, Pos2::new(560.0, 340.0), 3); cid += 1;

    // 6. SAW IF Filter & Termination
    let xsaw = SchematicComponent::new(cid, ComponentKind::SawIdt, Pos2::new(680.0, 220.0), 1); cid += 1;
    let r_load = SchematicComponent::new(cid, ComponentKind::Resistor, Pos2::new(780.0, 220.0), 6); cid += 1;
    let prb_if = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(850.0, 200.0), 1); cid += 1;
    let prb_th = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(620.0, 60.0), 2);

    // Ground Bus at Y=460 from X=100 to X=780
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(100.0, 280.0), Pos2::new(100.0, 460.0)),
        WireSegment::new(Pos2::new(100.0, 460.0), Pos2::new(780.0, 460.0)),
    ])); wid += 1;

    // Connect GND1 pin at (100, 460) to ground bus
    // Connect L_MATCH pin 2 (220, 320) to ground bus
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(220.0, 320.0), Pos2::new(220.0, 460.0)),
    ])); wid += 1;

    // Connect VCC(-) (280, 160) to ground bus
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(280.0, 160.0), Pos2::new(280.0, 460.0)),
    ])); wid += 1;

    // Connect RB2 pin 2 (340, 360) to ground bus
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(340.0, 360.0), Pos2::new(340.0, 460.0)),
    ])); wid += 1;

    // Connect RE pin 2 (420, 400) to ground bus
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(420.0, 400.0), Pos2::new(420.0, 460.0)),
    ])); wid += 1;

    // Connect CE pin 2 (480, 400) to ground bus
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(480.0, 400.0), Pos2::new(480.0, 460.0)),
    ])); wid += 1;

    // Connect V_LO(-) (560, 380) to ground bus
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(560.0, 380.0), Pos2::new(560.0, 460.0)),
    ])); wid += 1;

    // Connect XSAW IN- (650, 240) to ground bus
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(650.0, 240.0), Pos2::new(650.0, 460.0)),
    ])); wid += 1;

    // Connect XSAW OUT- (710, 240) to ground bus
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(710.0, 240.0), Pos2::new(710.0, 460.0)),
    ])); wid += 1;

    // Connect R_LOAD pin 2 (780, 260) to ground bus
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(780.0, 260.0), Pos2::new(780.0, 460.0)),
    ])); wid += 1;

    // Connect R_TH1 pin 2 (500, 140) and C_TH1 pin 2 (560, 140) to ground bus
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(500.0, 140.0), Pos2::new(560.0, 140.0)),
        WireSegment::new(Pos2::new(560.0, 140.0), Pos2::new(560.0, 300.0)),
    ])); wid += 1;

    // VCC (+3.3V) Rail at Y=80 from X=280 to X=420
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(280.0, 80.0), Pos2::new(420.0, 80.0)),
    ])); wid += 1;
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(340.0, 100.0), Pos2::new(340.0, 80.0)),
    ])); wid += 1;
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(420.0, 100.0), Pos2::new(420.0, 80.0)),
    ])); wid += 1;

    // RF Input Matching: V_RF(+) (100, 200) -> C_MATCH pin 1 (180, 160)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(100.0, 200.0), Pos2::new(180.0, 160.0)),
    ])); wid += 1;
    // C_MATCH pin 2 (180, 240) -> L_MATCH pin 1 (220, 240) -> Base Net at (340, 240)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(180.0, 240.0), Pos2::new(380.0, 240.0)),
    ])); wid += 1;
    // Base bias divider connections: RB1 pin 2 (340, 180) and RB2 pin 1 (340, 280) to (340, 240)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(340.0, 180.0), Pos2::new(340.0, 280.0)),
    ])); wid += 1;

    // Collector Net: Q1 Collector (420, 200) -> RC pin 2 (420, 180)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(420.0, 200.0), Pos2::new(420.0, 180.0)),
    ])); wid += 1;
    // Q1 Collector (420, 200) -> CC1 pin 1 (500, 160)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(420.0, 200.0), Pos2::new(500.0, 160.0)),
    ])); wid += 1;
    // Thermal coupling: Q1 Collector (420, 200) -> (460, 200) -> (460, 60) -> R_TH1 pin 1 (500, 60) -> C_TH1 pin 1 (560, 60) -> PRB_TH (600, 60)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(420.0, 200.0), Pos2::new(460.0, 200.0)),
        WireSegment::new(Pos2::new(460.0, 200.0), Pos2::new(460.0, 60.0)),
        WireSegment::new(Pos2::new(460.0, 60.0), Pos2::new(600.0, 60.0)),
    ])); wid += 1;

    // Emitter Net: Q1 Emitter (420, 280) -> RE pin 1 (420, 320) -> CE pin 1 (480, 320)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(420.0, 280.0), Pos2::new(420.0, 320.0)),
        WireSegment::new(Pos2::new(420.0, 320.0), Pos2::new(480.0, 320.0)),
    ])); wid += 1;

    // Mixer input: CC1 pin 2 (500, 240) -> D_MIX Anode (560, 160)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(500.0, 240.0), Pos2::new(560.0, 160.0)),
    ])); wid += 1;
    // Mixer LO injection & IF Output: D_MIX Cathode (560, 240) -> V_LO(+) (560, 300) and XSAW IN+ (650, 200)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(560.0, 240.0), Pos2::new(560.0, 300.0)),
        WireSegment::new(Pos2::new(560.0, 240.0), Pos2::new(650.0, 200.0)),
    ])); wid += 1;

    // IF Filter Output: XSAW OUT+ (710, 200) -> R_LOAD pin 1 (780, 180) -> PRB_IF (830, 200)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(710.0, 200.0), Pos2::new(780.0, 180.0)),
        WireSegment::new(Pos2::new(780.0, 180.0), Pos2::new(830.0, 200.0)),
    ]));

    components.extend(vec![
        v_rf, gnd, c_match, l_match, vcc, rb1, rb2, q1, rc, re, ce,
        r_th1, c_th1, cc1, d_mix, v_lo, xsaw, r_load, prb_if, prb_th,
    ]);

    let diags = ErcEngine::evaluate(&components, &wires);
    let errs: Vec<_> = diags.iter().filter(|d| d.severity == ErcSeverity::Error).collect();
    let warns: Vec<_> = diags.iter().filter(|d| d.severity == ErcSeverity::Warning).collect();
    if !diags.is_empty() {
        for d in &diags {
            eprintln!("[ERC {:?}] {}", d.severity, d.message);
        }
    }
    assert_eq!(errs.len(), 0, "RF Transceiver must have 0 ERC errors");
    assert_eq!(warns.len(), 0, "RF Transceiver must have 0 ERC warnings");

    let compiled = compile_schematic(&components, &wires);
    assert!(compiled.is_ok(), "RF Transceiver schematic must compile cleanly");
}

#[test]
fn test_topological_quantum_processor_contraption_erc_and_compilation() {
    let mut components = Vec::new();
    let mut wires = Vec::new();
    let mut cid = 1usize;
    let mut wid = 1usize;

    // 1. Excitation Microwave-Acoustic Pulse Generator
    let v_pulse = SchematicComponent::new(cid, ComponentKind::PulseGenerator, Pos2::new(100.0, 240.0), 1); cid += 1;
    let gnd = SchematicComponent::new(cid, ComponentKind::Ground, Pos2::new(100.0, 440.0), 1); cid += 1;

    // 2. 3.5 GHz Piezoelectric SAW IDT Transducer
    let xsaw = SchematicComponent::new(cid, ComponentKind::SawIdt, Pos2::new(230.0, 220.0), 1); cid += 1;

    // 3. Fractionalized Parafermionic Cavity
    let xpc = SchematicComponent::new(cid, ComponentKind::ParafermionicCavity, Pos2::new(360.0, 200.0), 1); cid += 1;

    // 4. Synthetic Gauge Flux Bias Supply (+1.5V)
    let vg = SchematicComponent::new(cid, ComponentKind::VoltageSource, Pos2::new(450.0, 80.0), 2); cid += 1;

    // 5. Chiral Skyrmion Acoustic Router
    let xsr = SchematicComponent::new(cid, ComponentKind::SkyrmionRouter, Pos2::new(490.0, 200.0), 1); cid += 1;

    // 6. Non-Abelian Majorana Braiding Junction 1
    let xmj1 = SchematicComponent::new(cid, ComponentKind::MajoranaJunction, Pos2::new(630.0, 180.0), 1); cid += 1;

    // 7. Non-Abelian Majorana Braiding Junction 2
    let xmj2 = SchematicComponent::new(cid, ComponentKind::MajoranaJunction, Pos2::new(750.0, 180.0), 2); cid += 1;

    // 8. Superconducting Nanowire Single-Phonon Detector (SNSPD)
    let r_snspd = SchematicComponent::new(cid, ComponentKind::Resistor, Pos2::new(860.0, 220.0), 1); cid += 1;
    let d_snspd = SchematicComponent::new(cid, ComponentKind::ZenerDiode, Pos2::new(860.0, 340.0), 1); cid += 1;

    // 9. Readout Probes
    let prb_click = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(940.0, 180.0), 1); cid += 1;
    let prb_braid = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(720.0, 120.0), 2);

    // Ground Bus at Y=420 from X=100 to X=860
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(100.0, 280.0), Pos2::new(100.0, 420.0)),
        WireSegment::new(Pos2::new(100.0, 420.0), Pos2::new(860.0, 420.0)),
    ])); wid += 1;

    // SAW IN- (200, 240) to ground bus
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(200.0, 240.0), Pos2::new(200.0, 420.0)),
    ])); wid += 1;
    // SAW OUT- (260, 240) to ground bus
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(260.0, 240.0), Pos2::new(260.0, 420.0)),
    ])); wid += 1;
    // VG(-) (450, 120) to ground bus
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(450.0, 120.0), Pos2::new(450.0, 420.0)),
    ])); wid += 1;
    // D_SNSPD pin 2 (860, 380) to ground bus
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(860.0, 380.0), Pos2::new(860.0, 420.0)),
    ])); wid += 1;

    // Microwave pulse to SAW IN+: V_PULSE(+) (100, 200) -> XSAW IN+ (200, 200)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(100.0, 200.0), Pos2::new(200.0, 200.0)),
    ])); wid += 1;

    // SAW OUT+ (260, 200) to Parafermionic Cavity PORT1 (330, 200)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(260.0, 200.0), Pos2::new(330.0, 200.0)),
    ])); wid += 1;

    // Parafermionic Cavity PORT2 (390, 200) to Skyrmion Router IN (460, 200)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(390.0, 200.0), Pos2::new(460.0, 200.0)),
    ])); wid += 1;

    // Synthetic flux bias: VG(+) (450, 40) -> (490, 40) -> XSR GATE (490, 170)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(450.0, 40.0), Pos2::new(490.0, 40.0)),
        WireSegment::new(Pos2::new(490.0, 40.0), Pos2::new(490.0, 170.0)),
    ])); wid += 1;

    // Skyrmion Router CH0 (520, 180) to XMJ1 J1 (600, 160)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(520.0, 180.0), Pos2::new(600.0, 160.0)),
    ])); wid += 1;
    // Skyrmion Router CH1 (520, 220) to XMJ1 J2 (600, 200)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(520.0, 220.0), Pos2::new(600.0, 200.0)),
    ])); wid += 1;

    // XMJ1 J3 (660, 180) to XMJ2 J1 (720, 160) and Braid Probe (700, 120)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(660.0, 180.0), Pos2::new(720.0, 160.0)),
        WireSegment::new(Pos2::new(660.0, 180.0), Pos2::new(700.0, 120.0)),
    ])); wid += 1;

    // XMJ2 J2 (720, 200) looped back to XMJ1 J2 (600, 200) for crossbar braiding topology
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(600.0, 200.0), Pos2::new(720.0, 200.0)),
    ])); wid += 1;

    // XMJ2 J3 (780, 180) to R_SNSPD pin 1 (860, 180) and PRB_CLICK (920, 180)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(780.0, 180.0), Pos2::new(860.0, 180.0)),
        WireSegment::new(Pos2::new(860.0, 180.0), Pos2::new(920.0, 180.0)),
    ])); wid += 1;

    // R_SNSPD pin 2 (860, 260) to D_SNSPD pin 1 (860, 300)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(860.0, 260.0), Pos2::new(860.0, 300.0)),
    ]));

    components.extend(vec![
        v_pulse, gnd, xsaw, xpc, vg, xsr, xmj1, xmj2, r_snspd, d_snspd, prb_click, prb_braid,
    ]);

    let diags = ErcEngine::evaluate(&components, &wires);
    let errs: Vec<_> = diags.iter().filter(|d| d.severity == ErcSeverity::Error).collect();
    let warns: Vec<_> = diags.iter().filter(|d| d.severity == ErcSeverity::Warning).collect();
    if !diags.is_empty() {
        for d in &diags {
            eprintln!("[ERC {:?}] {}", d.severity, d.message);
        }
    }
    assert_eq!(errs.len(), 0, "Topological Quantum Processor must have 0 ERC errors");
    assert_eq!(warns.len(), 0, "Topological Quantum Processor must have 0 ERC warnings");

    let compiled = compile_schematic(&components, &wires);
    assert!(compiled.is_ok(), "Topological Quantum Processor schematic must compile cleanly");
}

