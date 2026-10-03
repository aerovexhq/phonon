#![deny(unsafe_code)]

//! Comprehensive test suite for Phase 337: Fractional Quantum Hall Anyon Braiding & Interferometry.
//!
//! Verifies:
//! - Yang-Baxter topological consistency for Fibonacci and Moore-Read generators.
//! - Unitarity of braiding generators rho(sigma_i) * rho(sigma_i)^dagger = I.
//! - Topological gate synthesis fidelity >= 0.99 for Clifford+T target gates.
//! - Fabry-Perot and Mach-Zehnder Aharonov-Bohm oscillation periodicity Delta B = h / (e* A).
//! - Fractional shot noise Fano factor F = e*/e = 0.25 (nu=5/2) and 0.3333 (nu=1/3).
//! - Bulk anyon parity effect (even vs odd quasiparticles altering interference contrast).

use phonon_solver::fqh_braiding::{
    AnyonModelKind, FillingFraction, FqhEdgeInterferometer, InterferometerType,
    TargetGate, TopologicalGateSynthesizer, FLUX_QUANTUM_PHI0,
};

#[test]
fn test_yang_baxter_consistency_for_fibonacci_and_moore_read() {
    // 1. Moore-Read Pfaffian (nu = 5/2 Ising anyons)
    let moore_read = AnyonModelKind::MooreReadPfaffian;
    let (mr_yb_ok, mr_err) = moore_read.verify_yang_baxter();
    assert!(
        mr_yb_ok,
        "Moore-Read Yang-Baxter consistency failed: error = {}",
        mr_err
    );
    assert!(
        mr_err < 1e-10,
        "Moore-Read Yang-Baxter error must be < 1e-10, got {}",
        mr_err
    );

    // 2. Fibonacci anyons (golden ratio quantum dimension)
    let fibonacci = AnyonModelKind::Fibonacci;
    let (fib_yb_ok, fib_err) = fibonacci.verify_yang_baxter();
    assert!(
        fib_yb_ok,
        "Fibonacci Yang-Baxter consistency failed: error = {}",
        fib_err
    );
    assert!(
        fib_err < 1e-10,
        "Fibonacci Yang-Baxter error must be < 1e-10, got {}",
        fib_err
    );
}

#[test]
fn test_unitarity_of_braiding_generators() {
    // 1. Moore-Read Pfaffian
    let (mr_unitary, mr_u_err) = AnyonModelKind::MooreReadPfaffian.verify_unitarity();
    assert!(
        mr_unitary,
        "Moore-Read generators must be strictly unitary: error = {}",
        mr_u_err
    );
    assert!(
        mr_u_err < 1e-10,
        "Moore-Read unitarity error must be < 1e-10, got {}",
        mr_u_err
    );

    // 2. Fibonacci
    let (fib_unitary, fib_u_err) = AnyonModelKind::Fibonacci.verify_unitarity();
    assert!(
        fib_unitary,
        "Fibonacci generators must be strictly unitary: error = {}",
        fib_u_err
    );
    assert!(
        fib_u_err < 1e-10,
        "Fibonacci unitarity error must be < 1e-10, got {}",
        fib_u_err
    );
}

#[test]
fn test_topological_gate_synthesis_fidelity_clifford_plus_t() {
    // Test Clifford + T single-qubit gates on Moore-Read Pfaffian
    let clifford_gates = [
        TargetGate::Hadamard,
        TargetGate::PhaseS,
        TargetGate::PauliX,
        TargetGate::PauliZ,
        TargetGate::TGate,
    ];

    for &gate in &clifford_gates {
        let result = TopologicalGateSynthesizer::compile(gate, AnyonModelKind::MooreReadPfaffian);
        assert!(
            result.fidelity >= 0.99,
            "Gate {:?} on Moore-Read must have fidelity >= 0.99: got {}",
            gate,
            result.fidelity
        );
        assert!(
            !result.braid_sequence.generators.is_empty(),
            "Braid sequence for {:?} must contain generators",
            gate
        );
    }

    // Test universal synthesis on Fibonacci model
    let fib_gates = [
        TargetGate::Hadamard,
        TargetGate::PhaseS,
        TargetGate::PauliZ,
        TargetGate::TGate,
    ];

    for &gate in &fib_gates {
        let result = TopologicalGateSynthesizer::compile(gate, AnyonModelKind::Fibonacci);
        assert!(
            result.fidelity >= 0.99,
            "Gate {:?} on Fibonacci must have fidelity >= 0.99: got {}",
            gate,
            result.fidelity
        );
    }

    // Test two-qubit entangling gates (CNOT, CZ)
    let two_qubit_gates = [TargetGate::Cnot, TargetGate::Cz];
    for &gate in &two_qubit_gates {
        let result = TopologicalGateSynthesizer::compile(gate, AnyonModelKind::MooreReadPfaffian);
        assert!(
            result.fidelity >= 0.99,
            "Two-qubit gate {:?} must have fidelity >= 0.99: got {}",
            gate,
            result.fidelity
        );
        assert_eq!(result.num_strands, 8);
    }
}

#[test]
fn test_aharonov_bohm_oscillation_periodicity() {
    // 1. Fabry-Perot interferometer
    let fp = FqhEdgeInterferometer {
        filling: FillingFraction::Nu5_2, // e* = 0.25 e
        interferometer_type: InterferometerType::FabryPerot,
        area_um2: 2.0,
        ..Default::default()
    };
    let expected_delta_b_fp = FLUX_QUANTUM_PHI0 / (0.25 * 2.0e-12);
    let delta_b_fp = fp.oscillation_period_delta_b();
    assert!(
        (delta_b_fp - expected_delta_b_fp).abs() < 1e-6,
        "Fabry-Perot Delta B must match h / (e* A): expected {}, got {}",
        expected_delta_b_fp,
        delta_b_fp
    );

    // Verify 2*pi phase accumulation over exactly one period
    let phase_0 = fp.aharonov_bohm_phase(0.0);
    let phase_1 = fp.aharonov_bohm_phase(delta_b_fp);
    let phase_diff = phase_1 - phase_0;
    assert!(
        (phase_diff - 2.0 * std::f64::consts::PI).abs() < 1e-9,
        "Phase difference over Delta B must equal 2*pi: got {}",
        phase_diff
    );

    // 2. Mach-Zehnder interferometer with nu = 1/3 (e* = 1/3 e)
    let mz = FqhEdgeInterferometer {
        filling: FillingFraction::Nu1_3,
        interferometer_type: InterferometerType::MachZehnder,
        area_um2: 3.0,
        ..Default::default()
    };
    let expected_delta_b_mz = FLUX_QUANTUM_PHI0 / ((1.0 / 3.0) * 3.0e-12);
    let delta_b_mz = mz.oscillation_period_delta_b();
    assert!(
        (delta_b_mz - expected_delta_b_mz).abs() < 1e-6,
        "Mach-Zehnder Delta B must match h / (e* A): expected {}, got {}",
        expected_delta_b_mz,
        delta_b_mz
    );
}

#[test]
fn test_fractional_shot_noise_and_fano_factor() {
    // 1. Moore-Read nu = 5/2: Fano factor F = e* / e = 0.25
    let mut ifm_5_2 = FqhEdgeInterferometer {
        filling: FillingFraction::Nu5_2,
        bias_current_na: 2.0,
        gate_voltage_v: -0.4,
        ..Default::default()
    };
    let fano_5_2 = ifm_5_2.fano_factor();
    assert!(
        (fano_5_2 - 0.25).abs() < 1e-6,
        "Fano factor for nu=5/2 must equal 0.25: got {}",
        fano_5_2
    );

    // 2. Laughlin nu = 1/3: Fano factor F = e* / e = 0.333333
    ifm_5_2.filling = FillingFraction::Nu1_3;
    let fano_1_3 = ifm_5_2.fano_factor();
    assert!(
        (fano_1_3 - (1.0 / 3.0)).abs() < 1e-6,
        "Fano factor for nu=1/3 must equal 1/3: got {}",
        fano_1_3
    );

    // 3. Jain nu = 2/5: Fano factor F = e* / e = 0.20
    ifm_5_2.filling = FillingFraction::Nu2_5;
    let fano_2_5 = ifm_5_2.fano_factor();
    assert!(
        (fano_2_5 - 0.20).abs() < 1e-6,
        "Fano factor for nu=2/5 must equal 0.20: got {}",
        fano_2_5
    );
}

#[test]
fn test_bulk_anyon_parity_effect() {
    // In non-Abelian Moore-Read Pfaffian (nu = 5/2):
    // Even bulk anyons -> full interference visibility V = 1.0
    // Odd bulk anyons -> complete suppression of interference contrast V = 0.0
    let mut ifm = FqhEdgeInterferometer {
        filling: FillingFraction::Nu5_2,
        area_um2: 2.0,
        temperature_mk: 10.0,
        ..Default::default()
    };

    // Even parity (n_bulk = 0, 2, 4): visibility = 1.0
    ifm.bulk_anyon_count = 0;
    assert_eq!(ifm.bulk_parity_visibility(), 1.0);
    ifm.bulk_anyon_count = 2;
    assert_eq!(ifm.bulk_parity_visibility(), 1.0);
    ifm.bulk_anyon_count = 4;
    assert_eq!(ifm.bulk_parity_visibility(), 1.0);

    // Conductance oscillation amplitude must be non-zero for even parity
    let delta_b = ifm.oscillation_period_delta_b();
    let g_max = ifm.conductance(0.0, -0.5);
    let g_min = ifm.conductance(0.5 * delta_b, -0.5);
    let contrast_even = (g_max - g_min).abs();
    assert!(
        contrast_even > 1e-8,
        "Even parity must yield non-zero interference contrast: got {}",
        contrast_even
    );

    // Odd parity (n_bulk = 1, 3, 5): visibility = 0.0 (extinction)
    ifm.bulk_anyon_count = 1;
    assert_eq!(ifm.bulk_parity_visibility(), 0.0);
    ifm.bulk_anyon_count = 3;
    assert_eq!(ifm.bulk_parity_visibility(), 0.0);

    let g_odd_1 = ifm.conductance(0.0, -0.5);
    let g_odd_2 = ifm.conductance(0.5 * delta_b, -0.5);
    let contrast_odd = (g_odd_1 - g_odd_2).abs();
    assert!(
        contrast_odd < 1e-14,
        "Odd parity must suppress interference oscillations: contrast = {}",
        contrast_odd
    );

    // For Abelian Laughlin nu = 1/3, contrast is preserved regardless of parity
    ifm.filling = FillingFraction::Nu1_3;
    ifm.bulk_anyon_count = 1;
    assert_eq!(
        ifm.bulk_parity_visibility(),
        1.0,
        "Abelian fluids must preserve visibility for odd bulk anyons"
    );
}
