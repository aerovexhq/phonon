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
    let mux0 = SchematicComponent::new(cid, ComponentKind::Mux2to1, Pos2::new(580.0, 220.0), 1); cid += 1;
    let mux1 = SchematicComponent::new(cid, ComponentKind::Mux2to1, Pos2::new(580.0, 340.0), 2); cid += 1;
    let mux2 = SchematicComponent::new(cid, ComponentKind::Mux2to1, Pos2::new(580.0, 460.0), 3); cid += 1;
    let mux3 = SchematicComponent::new(cid, ComponentKind::Mux2to1, Pos2::new(580.0, 580.0), 4); cid += 1;

    // 5. Output Registers (D-Flip-Flops)
    let dff0 = SchematicComponent::new(cid, ComponentKind::DFlipFlop, Pos2::new(720.0, 240.0), 1); cid += 1;
    let dff1 = SchematicComponent::new(cid, ComponentKind::DFlipFlop, Pos2::new(720.0, 360.0), 2); cid += 1;
    let dff2 = SchematicComponent::new(cid, ComponentKind::DFlipFlop, Pos2::new(720.0, 480.0), 3); cid += 1;
    let dff3 = SchematicComponent::new(cid, ComponentKind::DFlipFlop, Pos2::new(720.0, 600.0), 4); cid += 1;

    // 6. Logic Probes (Q0..Q3, QN0..QN3, COUT)
    let prb_q0 = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(840.0, 220.0), 1); cid += 1;
    let prb_qn0 = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(840.0, 260.0), 2); cid += 1;
    let prb_q1 = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(840.0, 340.0), 3); cid += 1;
    let prb_qn1 = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(840.0, 380.0), 4); cid += 1;
    let prb_q2 = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(840.0, 460.0), 5); cid += 1;
    let prb_qn2 = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(840.0, 500.0), 6); cid += 1;
    let prb_q3 = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(840.0, 580.0), 7); cid += 1;
    let prb_qn3 = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(840.0, 620.0), 8); cid += 1;
    let prb_cout = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(840.0, 660.0), 9);

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

    // Clock distribution line at X=640 to DFF CLKs (680, y)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(100.0, 140.0), Pos2::new(640.0, 140.0)),
        WireSegment::new(Pos2::new(640.0, 140.0), Pos2::new(640.0, 620.0)),
    ])); wid += 1;
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(640.0, 260.0), Pos2::new(680.0, 260.0)),
    ])); wid += 1;
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(640.0, 380.0), Pos2::new(680.0, 380.0)),
    ])); wid += 1;
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(640.0, 500.0), Pos2::new(680.0, 500.0)),
    ])); wid += 1;
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(640.0, 620.0), Pos2::new(680.0, 620.0)),
    ])); wid += 1;

    // Operation select line V_OP(+) (100, 500) to MUX SELs (580, y)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(100.0, 500.0), Pos2::new(160.0, 500.0)),
        WireSegment::new(Pos2::new(160.0, 500.0), Pos2::new(160.0, 660.0)),
        WireSegment::new(Pos2::new(160.0, 660.0), Pos2::new(580.0, 660.0)),
        WireSegment::new(Pos2::new(580.0, 660.0), Pos2::new(580.0, 260.0)),
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
    // XOR A inputs at (420, y)
    for (y_src, y_dst) in [(240.0, 180.0), (360.0, 300.0), (480.0, 420.0), (600.0, 540.0)] {
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(280.0, y_src), Pos2::new(280.0, y_dst)),
            WireSegment::new(Pos2::new(280.0, y_dst), Pos2::new(420.0, y_dst)),
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
    // XOR B inputs at (420, y)
    for (y_src, y_dst) in [(260.0, 220.0), (380.0, 340.0), (500.0, 460.0), (620.0, 580.0)] {
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(240.0, y_src), Pos2::new(240.0, y_dst)),
            WireSegment::new(Pos2::new(240.0, y_dst), Pos2::new(420.0, y_dst)),
        ])); wid += 1;
    }

    // Ripple Carry propagation: COUT to CIN
    // FA0 COUT (360, 280) -> FA1 CIN (280, 400)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(360.0, 280.0), Pos2::new(390.0, 280.0)),
        WireSegment::new(Pos2::new(390.0, 280.0), Pos2::new(390.0, 340.0)),
        WireSegment::new(Pos2::new(390.0, 340.0), Pos2::new(260.0, 340.0)),
        WireSegment::new(Pos2::new(260.0, 340.0), Pos2::new(260.0, 400.0)),
        WireSegment::new(Pos2::new(260.0, 400.0), Pos2::new(280.0, 400.0)),
    ])); wid += 1;
    // FA1 COUT (360, 400) -> FA2 CIN (280, 520)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(360.0, 400.0), Pos2::new(390.0, 400.0)),
        WireSegment::new(Pos2::new(390.0, 400.0), Pos2::new(390.0, 460.0)),
        WireSegment::new(Pos2::new(390.0, 460.0), Pos2::new(260.0, 460.0)),
        WireSegment::new(Pos2::new(260.0, 460.0), Pos2::new(260.0, 520.0)),
        WireSegment::new(Pos2::new(260.0, 520.0), Pos2::new(280.0, 520.0)),
    ])); wid += 1;
    // FA2 COUT (360, 520) -> FA3 CIN (280, 640)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(360.0, 520.0), Pos2::new(390.0, 520.0)),
        WireSegment::new(Pos2::new(390.0, 520.0), Pos2::new(390.0, 580.0)),
        WireSegment::new(Pos2::new(390.0, 580.0), Pos2::new(260.0, 580.0)),
        WireSegment::new(Pos2::new(260.0, 580.0), Pos2::new(260.0, 640.0)),
        WireSegment::new(Pos2::new(260.0, 640.0), Pos2::new(280.0, 640.0)),
    ])); wid += 1;
    // FA3 COUT (360, 640) -> Carry Flag Probe (820, 660)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(360.0, 640.0), Pos2::new(400.0, 640.0)),
        WireSegment::new(Pos2::new(400.0, 640.0), Pos2::new(400.0, 660.0)),
        WireSegment::new(Pos2::new(400.0, 660.0), Pos2::new(820.0, 660.0)),
    ])); wid += 1;

    // FA SUM outputs (360, y) to MUX D1 inputs (540, y)
    let fa_sums = [(240.0, 240.0), (360.0, 360.0), (480.0, 480.0), (600.0, 600.0)];
    for (y_fa, y_mux) in fa_sums {
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(360.0, y_fa), Pos2::new(540.0, y_mux)),
        ])); wid += 1;
    }

    // XOR outputs (500, y) to MUX D0 inputs (540, y)
    let xor_outs = [(200.0, 200.0), (320.0, 320.0), (440.0, 440.0), (560.0, 560.0)];
    for (y_xor, y_mux) in xor_outs {
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(500.0, y_xor), Pos2::new(540.0, y_mux)),
        ])); wid += 1;
    }

    // MUX outputs (620, y) to DFF D inputs (680, y)
    for y in [220.0, 340.0, 460.0, 580.0] {
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(620.0, y), Pos2::new(680.0, y)),
        ])); wid += 1;
    }

    // DFF Q outputs (760, y) to Probes (820, y)
    for y in [220.0, 340.0, 460.0, 580.0] {
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(760.0, y), Pos2::new(820.0, y)),
        ])); wid += 1;
    }

    // DFF QN outputs (760, y) to Probes (820, y)
    for y in [260.0, 380.0, 500.0, 620.0] {
        wires.push(SchematicWire::new(wid, vec![
            WireSegment::new(Pos2::new(760.0, y), Pos2::new(820.0, y)),
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
    let prb_if = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(840.0, 180.0), 1); cid += 1;
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

    // Connect XSAW IN- (640, 240) to ground bus
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(640.0, 240.0), Pos2::new(640.0, 460.0)),
    ])); wid += 1;

    // Connect XSAW OUT- (720, 240) to ground bus
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(720.0, 240.0), Pos2::new(720.0, 460.0)),
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
    // Mixer LO injection & IF Output: D_MIX Cathode (560, 240) -> V_LO(+) (560, 300) and XSAW IN+ (640, 200)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(560.0, 240.0), Pos2::new(560.0, 300.0)),
        WireSegment::new(Pos2::new(560.0, 240.0), Pos2::new(600.0, 240.0)),
        WireSegment::new(Pos2::new(600.0, 240.0), Pos2::new(600.0, 200.0)),
        WireSegment::new(Pos2::new(600.0, 200.0), Pos2::new(640.0, 200.0)),
    ])); wid += 1;

    // IF Filter Output: XSAW OUT+ (720, 200) -> R_LOAD pin 1 (780, 180) -> PRB_IF (820, 180)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(720.0, 200.0), Pos2::new(780.0, 200.0)),
        WireSegment::new(Pos2::new(780.0, 200.0), Pos2::new(780.0, 180.0)),
        WireSegment::new(Pos2::new(780.0, 180.0), Pos2::new(820.0, 180.0)),
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
    let xsaw = SchematicComponent::new(cid, ComponentKind::SawIdt, Pos2::new(240.0, 220.0), 1); cid += 1;

    // 3. Fractionalized Parafermionic Cavity
    let xpc = SchematicComponent::new(cid, ComponentKind::ParafermionicCavity, Pos2::new(360.0, 200.0), 1); cid += 1;

    // 4. Synthetic Gauge Flux Bias Supply (+1.5V)
    let vg = SchematicComponent::new(cid, ComponentKind::VoltageSource, Pos2::new(440.0, 80.0), 2); cid += 1;

    // 5. Chiral Skyrmion Acoustic Router
    let xsr = SchematicComponent::new(cid, ComponentKind::SkyrmionRouter, Pos2::new(480.0, 200.0), 1); cid += 1;

    // 6. Non-Abelian Majorana Braiding Junction 1
    let xmj1 = SchematicComponent::new(cid, ComponentKind::MajoranaJunction, Pos2::new(600.0, 200.0), 1); cid += 1;

    // 7. Non-Abelian Majorana Braiding Junction 2
    let xmj2 = SchematicComponent::new(cid, ComponentKind::MajoranaJunction, Pos2::new(720.0, 200.0), 2); cid += 1;

    // 8. Superconducting Nanowire Single-Phonon Detector (SNSPD)
    let r_snspd = SchematicComponent::new(cid, ComponentKind::Resistor, Pos2::new(840.0, 240.0), 1); cid += 1;
    let d_snspd = SchematicComponent::new(cid, ComponentKind::ZenerDiode, Pos2::new(840.0, 360.0), 1); cid += 1;

    // 9. Readout Probes
    let prb_click = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(920.0, 200.0), 1); cid += 1;
    let prb_braid = SchematicComponent::new(cid, ComponentKind::LogicProbe, Pos2::new(680.0, 140.0), 2);

    // Ground Bus at Y=420 from X=100 to X=860
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(100.0, 280.0), Pos2::new(100.0, 420.0)),
        WireSegment::new(Pos2::new(100.0, 420.0), Pos2::new(860.0, 420.0)),
    ])); wid += 1;

    // SAW IN- (200, 240) to ground bus
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(200.0, 240.0), Pos2::new(200.0, 420.0)),
    ])); wid += 1;
    // SAW OUT- (280, 240) to ground bus
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(280.0, 240.0), Pos2::new(280.0, 420.0)),
    ])); wid += 1;
    // VG(-) (440, 120) to ground bus
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(440.0, 120.0), Pos2::new(440.0, 420.0)),
    ])); wid += 1;
    // D_SNSPD pin 2 (840, 400) to ground bus
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(840.0, 400.0), Pos2::new(840.0, 420.0)),
    ])); wid += 1;

    // Microwave pulse to SAW IN+: V_PULSE(+) (100, 200) -> XSAW IN+ (200, 200)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(100.0, 200.0), Pos2::new(200.0, 200.0)),
    ])); wid += 1;

    // SAW OUT+ (280, 200) to Parafermionic Cavity PORT1 (320, 200)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(280.0, 200.0), Pos2::new(320.0, 200.0)),
    ])); wid += 1;

    // Parafermionic Cavity PORT2 (400, 200) to Skyrmion Router IN (440, 200)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(400.0, 200.0), Pos2::new(440.0, 200.0)),
    ])); wid += 1;

    // Synthetic flux bias: VG(+) (440, 40) -> (480, 40) -> XSR GATE (480, 160)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(440.0, 40.0), Pos2::new(480.0, 40.0)),
        WireSegment::new(Pos2::new(480.0, 40.0), Pos2::new(480.0, 160.0)),
    ])); wid += 1;

    // Skyrmion Router CH0 (520, 180) to XMJ1 J1 (560, 180)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(520.0, 180.0), Pos2::new(560.0, 180.0)),
    ])); wid += 1;
    // Skyrmion Router CH1 (520, 220) to XMJ1 J2 (560, 220)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(520.0, 220.0), Pos2::new(560.0, 220.0)),
    ])); wid += 1;

    // XMJ1 J3 (640, 200) to XMJ2 J1 (680, 180) and Braid Probe (660, 140)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(640.0, 200.0), Pos2::new(660.0, 200.0)),
        WireSegment::new(Pos2::new(660.0, 200.0), Pos2::new(660.0, 180.0)),
        WireSegment::new(Pos2::new(660.0, 180.0), Pos2::new(680.0, 180.0)),
    ])); wid += 1;
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(660.0, 180.0), Pos2::new(660.0, 140.0)),
    ])); wid += 1;

    // XMJ2 J2 (680, 220) looped back to XMJ1 J2 (560, 220) for crossbar braiding topology
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(560.0, 220.0), Pos2::new(680.0, 220.0)),
    ])); wid += 1;

    // XMJ2 J3 (760, 200) to R_SNSPD pin 1 (840, 200) and PRB_CLICK (900, 200)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(760.0, 200.0), Pos2::new(840.0, 200.0)),
        WireSegment::new(Pos2::new(840.0, 200.0), Pos2::new(900.0, 200.0)),
    ])); wid += 1;

    // R_SNSPD pin 2 (840, 280) to D_SNSPD pin 1 (840, 320)
    wires.push(SchematicWire::new(wid, vec![
        WireSegment::new(Pos2::new(840.0, 280.0), Pos2::new(840.0, 320.0)),
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

