//! Integration tests for Molecular Spintronic Logic, Memory Arrays & Comparative Benchmarks.
//!
//! Validates:
//! 1. Non-destructive zero-magnetic-field readout with MR ratio > 100%.
//! 2. Sub-femtojoule (< 0.05 fJ) write switching energy and zero static leakage (0.0 W).
//! 3. Molecular logic primitives: Molecular Inverter (NOT) and Majority-3 gate.
//! 4. 2D Addressable molecular memory array synthesis with density > 10^13 bits/cm^2.
//! 5. Rayon parallel comparative benchmark across 10,000 cells vs MTJ and 3nm GAA CMOS.

use phonon_models::spintronics::molecular_spintronics::{
    MolecularInverter, MolecularMajority3, MolecularSpinState, MolecularSpintronicCell,
};
use phonon_solver::spintronics::molecular_benchmark::MolecularBenchmarkRunner;
use phonon_solver::spintronics::molecular_synthesis::{
    MolecularSpintronicSynthesizer, MolecularSynthesisTarget,
};

#[test]
fn test_molecular_cell_non_destructive_readout() {
    let mut cell = MolecularSpintronicCell {
        state: MolecularSpinState::State1,
        ..Default::default()
    };

    // 1. Parallel state (State 1):
    let g_p = cell.read_conductance();
    let bit_1 = cell.read_bit();
    assert!(bit_1, "Expected bit 1 for State1");

    // 2. Antiparallel state (State 0):
    cell.state = MolecularSpinState::State0;
    let g_ap = cell.read_conductance();
    let bit_0 = cell.read_bit();
    assert!(!bit_0, "Expected bit 0 for State0");

    // Parallel conductance must strictly exceed antiparallel conductance:
    assert!(
        g_p > g_ap,
        "Parallel conductance G_P ({:e}) must exceed G_AP ({:e})",
        g_p,
        g_ap
    );

    // Giant Magnetoresistance ratio must exceed 100%:
    let mr = cell.mr_ratio();
    assert!(
        mr > 1.0,
        "Magnetoresistance ratio must exceed 100% (1.0), got {:.2}%",
        mr * 100.0
    );

    // Non-destructive: reading does not alter stored state:
    assert_eq!(cell.state, MolecularSpinState::State0);
    assert!(!cell.read_bit());
}

#[test]
fn test_molecular_cell_sub_femtojoule_write_energy() {
    let mut cell = MolecularSpintronicCell::default();

    // Write bit 0 -> 1:
    let energy_0_to_1 = cell.write_bit(true);
    let energy_fj = energy_0_to_1 * 1.0e15;

    // Must be strictly sub-femtojoule (< 0.05 fJ = 50 aJ):
    assert!(
        energy_fj < 0.05,
        "Write switching energy must be sub-femtojoule (< 0.05 fJ), got {:.4} fJ",
        energy_fj
    );
    assert!(cell.read_bit());

    // Writing the same bit should dissipate zero switching energy:
    let redundant_energy = cell.write_bit(true);
    assert_eq!(redundant_energy, 0.0);

    // Write bit 1 -> 0:
    let energy_1_to_0 = cell.write_bit(false);
    assert!(energy_1_to_0 * 1.0e15 < 0.05);
    assert!(!cell.read_bit());

    // Standby static leakage must be identically zero:
    assert_eq!(cell.static_leakage_w(), 0.0);
}

#[test]
fn test_molecular_inverter_and_majority3_logic() {
    // 1. Molecular Inverter (NOT):
    let mut inv = MolecularInverter::new();
    let (out_f, e_f) = inv.evaluate(false);
    assert!(out_f, "NOT false must be true");
    assert!(e_f * 1.0e15 < 0.05);

    let (out_t, e_t) = inv.evaluate(true);
    assert!(!out_t, "NOT true must be false");
    assert!(e_t * 1.0e15 < 0.05);

    // 2. Molecular Majority-3 Gate:
    let mut maj3 = MolecularMajority3::new();
    for a in [false, true] {
        for b in [false, true] {
            for c in [false, true] {
                let (out, energy) = maj3.evaluate(a, b, c);
                let expected = (a && b) || (b && c) || (a && c);
                assert_eq!(
                    out, expected,
                    "Majority-3 truth table failed for ({}, {}, {})",
                    a, b, c
                );
                assert!(energy * 1.0e15 < 0.20);
            }
        }
    }
}

#[test]
fn test_memory_array_synthesis_and_addressing() {
    let synth = MolecularSpintronicSynthesizer::new();
    let target = MolecularSynthesisTarget {
        min_polarization: 0.60,
        min_retention_years: 10.0,
        target_temp_k: 77.0,
        max_write_energy_fj: 0.05,
    };

    let mut array = synth.synthesize_memory_array(8, 8, &target);
    assert_eq!(array.capacity_bits(), 64);

    // Write checkboard pattern:
    for r in 0..8 {
        for c in 0..8 {
            let bit = (r + c) % 2 == 0;
            array.write(r, c, bit);
        }
    }

    // Read back and verify all 64 bits:
    for r in 0..8 {
        for c in 0..8 {
            let expected_bit = (r + c) % 2 == 0;
            assert_eq!(
                array.read(r, c),
                expected_bit,
                "Bit mismatch at ({}, {})",
                r,
                c
            );
        }
    }

    // Verify ultra-high integration density (> 10^13 bits/cm^2):
    let density = array.integration_density_bits_cm2();
    assert!(
        density > 1.0e13,
        "Molecular memory density must exceed 10^13 bits/cm^2, got {:e}",
        density
    );

    // Zero static standby power:
    assert_eq!(array.static_leakage_w(), 0.0);
}

#[test]
fn test_rayon_comparative_benchmark_10k_cells() {
    let runner = MolecularBenchmarkRunner::new();
    let report = runner.run_parallel_benchmark(10_000);

    assert_eq!(report.batch_cells_tested, 10_000);
    assert!(report.throughput_ops_per_sec > 10_000.0);

    // Density comparison:
    // Molecular spintronics (2nm pitch) vs MTJ (50nm) and CMOS (145nm):
    assert!(
        report.density_gain_vs_mtj > 100.0,
        "Density gain vs MTJ must exceed 100x, got {:.1}x",
        report.density_gain_vs_mtj
    );
    assert!(
        report.density_gain_vs_cmos > 1_000.0,
        "Density gain vs CMOS must exceed 1000x, got {:.1}x",
        report.density_gain_vs_cmos
    );

    // Write energy comparison:
    assert!(
        report.write_energy_reduction_vs_mtj > 1_000.0,
        "Write energy reduction vs MTJ must exceed 1000x, got {:.1}x",
        report.write_energy_reduction_vs_mtj
    );
    assert!(
        report.write_energy_reduction_vs_cmos > 20.0,
        "Write energy reduction vs CMOS must exceed 20x, got {:.1}x",
        report.write_energy_reduction_vs_cmos
    );

    // Static power elimination:
    assert_eq!(report.static_power_elimination_pct, 100.0);
    assert!(report.total_static_power_saved_w > 0.0);
}
