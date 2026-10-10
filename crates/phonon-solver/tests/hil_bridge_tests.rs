#![deny(unsafe_code)]

//! Comprehensive Test Suite for Hardware-in-the-Loop (HIL) Protocol Bridge & Logic Analyzer.

use phonon_solver::hil_bridge::{
    audit_hil_bridge, CanDecoder, HilBridgeEngine, HilMode,
    HilSynchronizer, I2cDecoder, Ieee488DefiniteBlock,
    LogicAnalyzerCapture, LogicTriggerCondition,
    ScpiCommandKind, ScpiParseError, ScpiProtocolBridge, SpiDecoder,
    TriggerEdge, TriggerMode, UartDecoder, VoltageKind,
};
use std::time::Instant;

#[test]
fn test_scpi_command_parsing() {
    let bridge = ScpiProtocolBridge::new_virtual();

    assert_eq!(bridge.parse_command("*IDN?").unwrap(), ScpiCommandKind::IdnQuery);
    assert_eq!(bridge.parse_command("*RST").unwrap(), ScpiCommandKind::Reset);
    assert_eq!(bridge.parse_command("*CLS").unwrap(), ScpiCommandKind::ClearStatus);
    assert_eq!(bridge.parse_command("*OPC?").unwrap(), ScpiCommandKind::OperationCompleteQuery);
    assert_eq!(bridge.parse_command("*STB?").unwrap(), ScpiCommandKind::StatusByteQuery);

    assert_eq!(
        bridge.parse_command(":MEAS:VOLT:DC? CHAN1").unwrap(),
        ScpiCommandKind::MeasureVoltage {
            channel: 1,
            kind: VoltageKind::Dc,
        }
    );

    assert_eq!(
        bridge.parse_command(":MEAS:VOLT:PKPK? CHAN2").unwrap(),
        ScpiCommandKind::MeasureVoltage {
            channel: 2,
            kind: VoltageKind::PeakToPeak,
        }
    );

    assert_eq!(
        bridge.parse_command(":MEAS:FREQ? CHAN3").unwrap(),
        ScpiCommandKind::MeasureFrequency { channel: 3 }
    );

    assert_eq!(
        bridge.parse_command(":CHAN1:SCAL 2.5").unwrap(),
        ScpiCommandKind::SetChannelScale {
            channel: 1,
            volts_per_div: 2.5,
            offset: 0.0,
        }
    );

    assert_eq!(
        bridge.parse_command(":TIM:SCAL 50e-6").unwrap(),
        ScpiCommandKind::SetTimebase {
            seconds_per_div: 50.0e-6,
            delay: 0.0,
        }
    );

    assert_eq!(
        bridge.parse_command(":TRIG:LEV 1.2").unwrap(),
        ScpiCommandKind::SetTrigger {
            channel: 1,
            mode: TriggerMode::Edge,
            level: 1.2,
        }
    );
}

#[test]
fn test_scpi_execution() {
    let mut bridge = ScpiProtocolBridge::new_virtual();

    let idn = bridge.execute("*IDN?");
    assert!(idn.contains("PHONON-STUDIO") || idn.contains("VIRTUAL-HIL-BRIDGE"));

    let opc = bridge.execute("*OPC?");
    assert_eq!(opc, "1");

    let stb = bridge.execute("*STB?");
    assert_eq!(stb, "16");

    let v_dc = bridge.execute(":MEAS:VOLT:DC? CHAN1");
    assert!(v_dc.ends_with("V"));

    let rst = bridge.execute("*RST");
    assert_eq!(rst, "OK:RESET");
}

#[test]
fn test_ieee488_definite_block_roundtrip() {
    let original_samples = vec![0.0, 1.25, -2.5, 3.0, -1.5];
    let y_inc = 0.0002; // 200 uV per LSB -> max 6.55 V
    let y_origin = 0.0;

    let encoded = Ieee488DefiniteBlock::encode_i16_block(&original_samples, y_inc, y_origin);
    assert!(!encoded.is_empty());
    assert_eq!(encoded[0], b'#');

    let decoded = Ieee488DefiniteBlock::decode_i16_block(&encoded, y_inc, y_origin).expect("Decoding block failed");
    assert_eq!(decoded.len(), original_samples.len());

    for (orig, dec) in original_samples.iter().zip(decoded.iter()) {
        assert!((orig - dec).abs() < 2.0 * y_inc, "Sample divergence: orig={} dec={}", orig, dec);
    }
}

#[test]
fn test_ieee488_block_error_handling() {
    let y_inc = 0.001;
    let y_origin = 0.0;

    // Missing # prefix
    let invalid_prefix = b"123456";
    assert!(matches!(
        Ieee488DefiniteBlock::decode_i16_block(invalid_prefix, y_inc, y_origin),
        Err(ScpiParseError::CorruptedBinaryBlock(_))
    ));

    // Truncated header
    let truncated = b"#";
    assert!(matches!(
        Ieee488DefiniteBlock::decode_i16_block(truncated, y_inc, y_origin),
        Err(ScpiParseError::CorruptedBinaryBlock(_))
    ));
}

#[test]
fn test_logic_analyzer_triggers() {
    let mut capture = LogicAnalyzerCapture::new(1.0e6, 4);

    // Channel 0 low for 10 samples, high for 10 samples
    for _ in 0..10 {
        capture.push_sample(0x0000);
    }
    for _ in 0..10 {
        capture.push_sample(0x0001);
    }

    let rising_cond = LogicTriggerCondition {
        channel: 0,
        edge: TriggerEdge::Rising,
        pattern_mask: 0x0001,
        pattern_val: 0x0001,
        holdoff_samples: 0,
    };

    let trig_idx = capture.find_trigger(&rising_cond);
    assert_eq!(trig_idx, Some(10));

    let falling_cond = LogicTriggerCondition {
        channel: 0,
        edge: TriggerEdge::Falling,
        pattern_mask: 0x0001,
        pattern_val: 0x0000,
        holdoff_samples: 0,
    };
    assert_eq!(capture.find_trigger(&falling_cond), None);
}

#[test]
fn test_spi_protocol_decoder() {
    let mut capture = LogicAnalyzerCapture::new(1.0e6, 4);
    let sck_mask = 1u16 << 2; // Ch 2
    let cs_mask = 1u16 << 3;  // Ch 3
    let mosi_mask = 1u16 << 0;// Ch 0
    let miso_mask = 1u16 << 1;// Ch 1

    // Idle: CS high
    for _ in 0..5 {
        capture.push_sample(cs_mask);
    }

    // CS Assert (Low)
    let mosi_byte = 0xAAu8; // 10101010
    let miso_byte = 0x55u8; // 01010101

    for bit_idx in (0..8).rev() {
        let mosi_bit = (mosi_byte >> bit_idx) & 1;
        let miso_bit = (miso_byte >> bit_idx) & 1;
        let d_val = (if mosi_bit == 1 { mosi_mask } else { 0 }) | (if miso_bit == 1 { miso_mask } else { 0 });

        // Clock Low
        capture.push_sample(d_val);
        // Clock High (Sampling edge)
        capture.push_sample(d_val | sck_mask);
    }

    // CS Deassert (High)
    for _ in 0..5 {
        capture.push_sample(cs_mask);
    }

    let decoder = SpiDecoder::default();
    let packets = decoder.decode(&capture);
    assert_eq!(packets.len(), 1);
    assert_eq!(packets[0].mosi_bytes, vec![0xAA]);
    assert_eq!(packets[0].miso_bytes, vec![0x55]);
}

#[test]
fn test_i2c_protocol_decoder() {
    let mut capture = LogicAnalyzerCapture::new(1.0e6, 8);
    let sda_mask = 1u16 << 4; // Ch 4
    let scl_mask = 1u16 << 5; // Ch 5

    // Idle: SCL=1, SDA=1
    for _ in 0..5 {
        capture.push_sample(scl_mask | sda_mask);
    }

    // START condition: SDA goes 1 -> 0 while SCL=1
    capture.push_sample(scl_mask);

    // Stream bits: Address 0x48 (7 bits: 1001000) + R/W=0 + ACK=0 (9 bits total)
    let bits = [true, false, false, true, false, false, false, false, false];
    for &b in &bits {
        let d = if b { sda_mask } else { 0 };
        // SCL Low: change data
        capture.push_sample(d);
        // SCL High: sample data
        capture.push_sample(d | scl_mask);
    }

    // STOP condition: SDA goes 0 -> 1 while SCL=1
    capture.push_sample(scl_mask);
    capture.push_sample(scl_mask | sda_mask);

    let decoder = I2cDecoder::default();
    let packets = decoder.decode(&capture);
    assert_eq!(packets.len(), 1);
    assert_eq!(packets[0].address, 0x48);
    assert!(!packets[0].is_read);
    assert!(packets[0].ack);
}

#[test]
fn test_uart_protocol_decoder() {
    let sample_rate = 1.0e6;
    let baud = 100_000.0;
    let samples_per_bit = (sample_rate / baud) as usize; // 10 samples/bit
    let mut capture = LogicAnalyzerCapture::new(sample_rate, 8);
    let ch_mask = 1u16 << 6; // Ch 6

    // Idle High
    for _ in 0..samples_per_bit {
        capture.push_sample(ch_mask);
    }

    // Start bit: 0
    for _ in 0..samples_per_bit {
        capture.push_sample(0);
    }

    // 8 data bits for 'A' (0x41 = 0b01000001, LSB first: 1, 0, 0, 0, 0, 0, 1, 0)
    let byte_val = 0x41u8;
    for b in 0..8 {
        let bit = (byte_val >> b) & 1;
        let sample = if bit == 1 { ch_mask } else { 0 };
        for _ in 0..samples_per_bit {
            capture.push_sample(sample);
        }
    }

    // Stop bit: 1
    for _ in 0..samples_per_bit {
        capture.push_sample(ch_mask);
    }

    let mut decoder = UartDecoder::default();
    decoder.baud_rate = baud;
    let frames = decoder.decode(&capture);
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0].byte_value, 0x41);
    assert!(!frames[0].framing_error);
}

#[test]
fn test_can_protocol_decoder() {
    let sample_rate = 1.0e6;
    let bit_rate = 100_000.0;
    let samples_per_bit = (sample_rate / bit_rate) as usize; // 10 samples/bit
    let mut capture = LogicAnalyzerCapture::new(sample_rate, 8);
    let ch_mask = 1u16 << 7; // Ch 7

    // Idle Recessive (1)
    for _ in 0..(samples_per_bit * 5) {
        capture.push_sample(ch_mask);
    }

    // SOF: Dominant (0)
    let mut can_bits = vec![0];

    // 11-bit ID: 0x7E0 = 0b11111100000
    let id = 0x7E0u32;
    for b in (0..11).rev() {
        can_bits.push(((id >> b) & 1) as u8);
    }

    // RTR = 0, IDE = 0, r0 = 0
    can_bits.push(0);
    can_bits.push(0);
    can_bits.push(0);

    // DLC = 1 (0b0001)
    can_bits.push(0);
    can_bits.push(0);
    can_bits.push(0);
    can_bits.push(1);

    // Payload byte 0x55 (0b01010101)
    let payload = 0x55u8;
    for b in (0..8).rev() {
        can_bits.push(((payload >> b) & 1) as u8);
    }

    for bit in can_bits {
        let val = if bit == 1 { ch_mask } else { 0 };
        for _ in 0..samples_per_bit {
            capture.push_sample(val);
        }
    }

    // EOF: Recessive (1)
    for _ in 0..(samples_per_bit * 10) {
        capture.push_sample(ch_mask);
    }

    let mut decoder = CanDecoder::default();
    decoder.nominal_bit_rate = bit_rate;
    let frames = decoder.decode(&capture);
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0].arbitration_id, 0x7E0);
    assert_eq!(frames[0].dlc, 1);
    assert_eq!(frames[0].payload, vec![0x55]);
}

#[test]
fn test_hil_synchronizer_interpolation_and_jitter() {
    let mut syn = HilSynchronizer::new_synthetic();
    assert_eq!(syn.mode, HilMode::HwToSpiceInjection);

    // Test continuous interpolation
    let v_interp = syn.interpolate_hw_voltage(50.0e-6);
    assert!((v_interp - 1.65).abs() < 1.5);

    // Step co-simulation
    for i in 0..100 {
        let t = (i as f64) * 1.0e-6;
        syn.step_co_simulation(t, 1.65, 0x0001);
    }

    assert!(syn.is_jitter_compliant());
    assert!(syn.metrics.rms_jitter_us < 1.0);
    assert_eq!(syn.metrics.co_sim_steps_completed, 100);
}

#[test]
fn test_hil_10_point_audit() {
    let report = audit_hil_bridge();
    for c in &report.criteria {
        println!("CRITERION: {} -> passed={} (meas={} thresh={})", c.name, c.passed, c.measured_value, c.target_threshold);
    }
    assert!(report.overall_pass);
    assert_eq!(report.passed_count, 10);
    assert_eq!(report.total_count, 10);
    assert!(report.is_all_pass());
    assert!(report.cold_boot_latency_us < 2000.0);
}

#[test]
fn test_hil_bridge_engine_fast_boot() {
    let start = Instant::now();
    let mut engine = HilBridgeEngine::new_fast();
    let elapsed_us = start.elapsed().as_micros() as f64;

    assert!(elapsed_us < 2000.0, "Cold boot exceeded 2.0 ms: {:.2} us", elapsed_us);
    assert!(engine.audit_report.is_all_pass());

    // SCPI execution
    let resp = engine.execute_scpi("*IDN?");
    assert!(resp.contains("PHONON") || resp.contains("VIRTUAL"));

    // Protocol decoders
    let spi = engine.decode_spi();
    let i2c = engine.decode_i2c();
    let uart = engine.decode_uart();
    let can = engine.decode_can();
    assert!(spi.len() <= 10);
    assert!(i2c.len() <= 10);
    assert!(uart.len() <= 10);
    assert!(can.len() <= 10);
}
