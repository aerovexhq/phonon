//! Integration tests for the Cryogenic Hybrid Superconducting-Photonic Coprocessor
//! and Quantum Error Correction (QEC) Surface Code Decoder.
//!
//! Validates:
//! - Rapid Single Flux Quantum (RSFQ) logic primitives (DFF, AND, Inverter, JTL).
//! - Optoelectronic transducers (SNSPD-to-RSFQ, RSFQ-to-Optical).
//! - Cryogenic Surface Code Syndrome Decoding for distance $d=3$ (100% single-qubit fidelity).
//! - Distance $d=5$ decoding functionality.
//! - Sub-10 ns real-time decoding latency directly at $4\text{ K}$.
//! - Multi-threaded Rayon benchmark comparing the cryogenic coprocessor against 3nm CMOS.

use phonon_models::superconducting::{
    OptoToRsfqTransducer, PauliCorrection, RsfqAnd, RsfqDff, RsfqInverter, RsfqJtl,
    RsfqToOptoDriver,
};
use phonon_solver::superconducting::{CryoQecDecoder, HybridCoprocessorBenchmarkRunner};

#[test]
fn test_rsfq_primitives_and_transducers() {
    // 1. RSFQ D-Flip-Flop
    let mut dff = RsfqDff::new(120.0e-6, 8.0);
    assert!(dff.clock_tick(0.0).is_none());
    dff.clock_data();
    let pulse_out = dff.clock_tick(10.0e-12);
    assert!(pulse_out.is_some());
    assert!(pulse_out.unwrap().peak_voltage() > 0.0);

    // 2. RSFQ AND gate
    let mut and_gate = RsfqAnd::new(100.0e-6, 10.0);
    and_gate.pulse_a();
    assert!(and_gate.clock_tick(0.0).is_none());
    and_gate.pulse_a();
    and_gate.pulse_b();
    assert!(and_gate.clock_tick(10.0e-12).is_some());

    // 3. RSFQ Inverter
    let mut inv = RsfqInverter::new(100.0e-6, 10.0);
    assert!(inv.clock_tick(0.0).is_some()); // No input -> emits pulse
    inv.pulse_in();
    assert!(inv.clock_tick(10.0e-12).is_none()); // Input present -> inhibited

    // 4. JTL Delay stage
    let jtl = RsfqJtl::new(100.0e-6, 10.0);
    assert!(jtl.delay_seconds > 0.5e-12 && jtl.delay_seconds < 10.0e-12);

    // 5. Opto-to-RSFQ transducer
    let opto_transducer = OptoToRsfqTransducer::default();
    let sfq = opto_transducer.transduce(50.0e-12);
    assert!(sfq.t0 > 50.0e-12);

    // 6. RSFQ-to-Opto driver
    let driver = RsfqToOptoDriver::default();
    assert_eq!(driver.drive_current(0.0, 50.0e-12), 0.0);
    let i_drive = driver.drive_current(50.0e-12 + driver.driver_delay + 10.0e-12, 50.0e-12);
    assert!(i_drive > 20.0e-6);
}

#[test]
fn test_cryo_qec_surface_code_d3_100_percent_fidelity() {
    let decoder = CryoQecDecoder::distance_3_rsfq();

    // Verify 100% single-qubit error correction fidelity across all 9 data qubits and all 3 Pauli errors (X, Z, Y)
    let (tested, passed, fidelity) = decoder.verify_single_qubit_fidelity();
    assert_eq!(tested, 27);
    assert_eq!(passed, 27);
    assert!((fidelity - 100.0).abs() < 1e-6);

    // Verify real-time decoding latency is strictly sub-nanosecond (< 1000 ps = 1 ns)
    let pkt = decoder.generate_syndrome_for_error(2, PauliCorrection::Y);
    let res = decoder.decode_syndrome(&pkt);
    assert!(res.success);
    assert!(
        res.latency_ps < 100.0,
        "d=3 RSFQ decoding latency must be < 100 ps"
    );
    assert_eq!(res.corrections.len(), 1);
    assert_eq!(res.corrections[0].qubit_index, 2);
    assert_eq!(res.corrections[0].correction, PauliCorrection::Y);
}

#[test]
fn test_cryo_qec_surface_code_d5_decoding() {
    let decoder = CryoQecDecoder::distance_5_rsfq();
    assert_eq!(decoder.geometry.distance, 5);
    assert_eq!(decoder.geometry.num_data_qubits, 25);
    assert_eq!(decoder.geometry.num_syndrome_ancillas, 24);

    // Test Pauli X on qubit 12 (center qubit in 5x5 lattice)
    let (pkt, res, corrected) = decoder.test_single_error(12, PauliCorrection::X);
    assert!(pkt.has_defects());
    assert!(res.success);
    assert!(corrected, "Center qubit Pauli X error must be corrected");
    assert!(
        res.latency_ps < 1000.0,
        "d=5 decoding latency must be < 1 ns"
    );
}

#[test]
fn test_hybrid_coprocessor_rayon_parallel_benchmark() {
    // Run parallel benchmark with 5,000 QEC syndrome extraction rounds and 8 SOEN neurons
    let report = HybridCoprocessorBenchmarkRunner::run_benchmark(5000, 8);

    assert_eq!(report.qec_rounds_evaluated, 5000);
    assert_eq!(report.qec_corrections_successful, 5000);
    assert!((report.qec_fidelity_percent - 100.0).abs() < 1e-6);

    // Latency speedup: cryogenic decoding is > 1000x faster than room-temp coaxial round trip
    assert!(
        report.qec_latency_speedup > 1000.0,
        "Latency speedup was {:.1}x",
        report.qec_latency_speedup
    );

    // Wall-plug energy efficiency (including 1000x cryogenic Carnot factor): > 10x better than 3nm CMOS
    assert!(
        report.wall_plug_energy_advantage > 10.0,
        "Wall-plug energy advantage was {:.1}x",
        report.wall_plug_energy_advantage
    );

    // Microscopic energy efficiency directly at 4K: > 10,000x lower dissipation
    assert!(
        report.microscopic_energy_advantage > 10_000.0,
        "Microscopic energy advantage was {:.1}x",
        report.microscopic_energy_advantage
    );

    // Thermal load reduction: optical fibers eliminate > 1000x of the conductive heat load of coaxial cables
    assert!(
        report.thermal_load_reduction_factor > 1000.0,
        "Thermal load reduction was {:.1}x",
        report.thermal_load_reduction_factor
    );

    // Interconnect bandwidth density > 100 Tbps/cm^2
    assert!(report.cryo_bandwidth_density_tbps_cm2 >= 100.0);

    // Throughput should be high on multi-threaded Rayon
    assert!(report.qec_throughput_rounds_per_s > 1000.0);
}
