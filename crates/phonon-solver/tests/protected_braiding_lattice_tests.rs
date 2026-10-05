#![deny(unsafe_code)]

//! Integration and verification tests for Quantum Acoustic Protected Braiding Lattice Solver
//! & Surface Code Stabilizer Engine.

use phonon_solver::protected_braiding_lattice::{
    BraidStep, MajoranaBraidingParams, NonAbelianBraidGenerator, ParityReadout, SurfaceCodeGrid,
    TargetGate,
};

#[test]
fn test_artin_non_abelian_braid_relation() {
    let params = MajoranaBraidingParams::default();
    let gen = NonAbelianBraidGenerator::new(params);

    // Verify Yang-Baxter / Artin braid relation: B_1 * B_2 * B_1 == B_2 * B_1 * B_2
    let (is_verified, residual) = gen.verify_artin_relation();
    assert!(
        is_verified,
        "Artin braid relation B1*B2*B1 == B2*B1*B2 failed verification: residual = {}",
        residual
    );
    assert!(
        residual < 1e-12,
        "Artin braid relation residual too large: {}",
        residual
    );

    // Also verify on SO(N) Majorana basis transformations
    let r1 = NonAbelianBraidGenerator::majorana_so_matrix(BraidStep::B1, 4);
    let r2 = NonAbelianBraidGenerator::majorana_so_matrix(BraidStep::B2, 4);

    let mul = |a: &Vec<Vec<f64>>, b: &Vec<Vec<f64>>| {
        let n = a.len();
        let mut res = vec![vec![0.0; n]; n];
        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    res[i][j] += a[i][k] * b[k][j];
                }
            }
        }
        res
    };

    let r1_r2_r1 = mul(&mul(&r1, &r2), &r1);
    let r2_r1_r2 = mul(&mul(&r2, &r1), &r2);

    for i in 0..4 {
        for j in 0..4 {
            let diff = (r1_r2_r1[i][j] - r2_r1_r2[i][j]).abs();
            assert!(
                diff < 1e-12,
                "Artin relation mismatch in SO(4) representation at ({}, {}): diff = {}",
                i, j, diff
            );
        }
    }
}

#[test]
fn test_single_qubit_gate_compilation_fidelity() {
    let params = MajoranaBraidingParams::default();
    let gen = NonAbelianBraidGenerator::new(params);

    // Test Hadamard compilation
    let h_res = gen.compile_gate(TargetGate::Hadamard);
    assert_eq!(h_res.target_gate, TargetGate::Hadamard);
    assert_eq!(
        h_res.braid_word,
        vec![BraidStep::B1, BraidStep::B2, BraidStep::B1]
    );
    assert!(
        h_res.ideal_fidelity >= 0.9999,
        "Hadamard ideal fidelity too low: {}",
        h_res.ideal_fidelity
    );
    assert!(
        h_res.net_gate_fidelity >= 0.999,
        "Hadamard net gate fidelity too low: {}",
        h_res.net_gate_fidelity
    );

    // Test Phase S compilation
    let s_res = gen.compile_gate(TargetGate::PhaseS);
    assert_eq!(s_res.target_gate, TargetGate::PhaseS);
    assert_eq!(s_res.braid_word, vec![BraidStep::B3]);
    assert!(
        s_res.ideal_fidelity >= 0.9999,
        "Phase S ideal fidelity too low: {}",
        s_res.ideal_fidelity
    );
    assert!(
        s_res.net_gate_fidelity >= 0.999,
        "Phase S net gate fidelity too low: {}",
        s_res.net_gate_fidelity
    );

    // Test Pauli X compilation
    let x_res = gen.compile_gate(TargetGate::PauliX);
    assert_eq!(x_res.target_gate, TargetGate::PauliX);
    assert_eq!(x_res.braid_word, vec![BraidStep::B2, BraidStep::B2]);
    assert!(
        x_res.ideal_fidelity >= 0.9999,
        "Pauli X ideal fidelity too low: {}",
        x_res.ideal_fidelity
    );
    assert!(
        x_res.net_gate_fidelity >= 0.999,
        "Pauli X net gate fidelity too low: {}",
        x_res.net_gate_fidelity
    );

    // Test Pauli Z compilation
    let z_res = gen.compile_gate(TargetGate::PauliZ);
    assert_eq!(z_res.target_gate, TargetGate::PauliZ);
    assert_eq!(z_res.braid_word, vec![BraidStep::B1, BraidStep::B1]);
    assert!(
        z_res.ideal_fidelity >= 0.9999,
        "Pauli Z ideal fidelity too low: {}",
        z_res.ideal_fidelity
    );
    assert!(
        z_res.net_gate_fidelity >= 0.999,
        "Pauli Z net gate fidelity too low: {}",
        z_res.net_gate_fidelity
    );

    // Test CNOT compilation
    let cnot_res = gen.compile_gate(TargetGate::Cnot);
    assert_eq!(cnot_res.target_gate, TargetGate::Cnot);
    assert!(
        cnot_res.ideal_fidelity >= 0.9999,
        "CNOT ideal fidelity too low: {}",
        cnot_res.ideal_fidelity
    );
    assert!(
        cnot_res.net_gate_fidelity >= 0.999,
        "CNOT net gate fidelity too low: {}",
        cnot_res.net_gate_fidelity
    );
}

#[test]
fn test_diabatic_transition_error_suppression() {
    let mut params = MajoranaBraidingParams::default();
    let gen_default = NonAbelianBraidGenerator::new(params.clone());

    let p_diabatic_default = gen_default.compute_diabatic_error();
    assert!(
        p_diabatic_default < 1e-4,
        "Diabatic transition error for default parameters must be < 1e-4, got {}",
        p_diabatic_default
    );

    // Verify exponential suppression when braid duration increases
    params.braid_duration_ns = 50.0;
    let gen_fast = NonAbelianBraidGenerator::new(params.clone());
    let p_fast = gen_fast.compute_diabatic_error();

    params.braid_duration_ns = 150.0;
    let gen_slow = NonAbelianBraidGenerator::new(params.clone());
    let p_slow = gen_slow.compute_diabatic_error();

    assert!(
        p_slow < p_fast,
        "Slower braiding must yield lower diabatic transition error: p_slow={}, p_fast={}",
        p_slow, p_fast
    );
    assert!(
        p_slow < 1e-5,
        "Diabatic error at 150 ns should be strongly suppressed: {}",
        p_slow
    );
}

#[test]
fn test_dispersive_parity_readout() {
    let params = MajoranaBraidingParams::default();
    let readout = ParityReadout::from_params(&params);

    // Test dispersive frequency splitting
    let splitting = readout.frequency_splitting_mhz();
    assert_eq!(
        splitting,
        2.0 * params.dispersive_shift_mhz,
        "Cavity frequency splitting must be 2 * chi"
    );
    assert_eq!(splitting, 8.0);

    // Test SNR >= 15.0 dB
    let snr_db = readout.snr_db();
    assert!(
        snr_db >= 15.0,
        "Dispersive readout SNR must be >= 15.0 dB, got {} dB",
        snr_db
    );

    // Test parity readout fidelity >= 0.995
    let fidelity = readout.readout_fidelity();
    assert!(
        fidelity >= 0.995,
        "Parity readout fidelity must be >= 0.995, got {}",
        fidelity
    );

    // Test spectrum transmission peaks
    let s21_even_peak = readout.transmission_s21(readout.frequency_shift_even_mhz(), true);
    let s21_even_off = readout.transmission_s21(readout.frequency_shift_odd_mhz(), true);
    assert!(
        (s21_even_peak - 1.0).abs() < 1e-6,
        "Even parity resonance peak should be unity transmission"
    );
    assert!(
        s21_even_off < 0.05,
        "Cross-parity transmission at opposite peak must be suppressed"
    );
}

#[test]
fn test_surface_code_stabilizer_syndromes_and_correction() {
    let grid = SurfaceCodeGrid::new();

    // Case 1: Clean state has zero defects
    let clean_res = grid.extract_syndromes_from_errors([false; 9], [false; 9], 0.0);
    assert!(clean_res.is_clean());
    assert_eq!(clean_res.defect_count(), 0);

    let clean_corr = grid.decode_and_correct(&clean_res);
    assert!(clean_corr.is_clean);
    assert!(!clean_corr.has_logical_error);

    // Case 2: Single Pauli Z error on qubit 1 (top edge)
    let mut z_err = [false; 9];
    z_err[1] = true;
    let syn_z1 = grid.extract_syndromes_from_errors([false; 9], z_err, 0.01);
    // Qubit 1 is in Star checks A_0 and A_1
    assert_eq!(syn_z1.star_defects, vec![0, 1]);
    let corr_z1 = grid.decode_and_correct(&syn_z1);
    assert!(corr_z1.is_clean);
    assert!(!corr_z1.has_logical_error);
    assert!(corr_z1.correction_z[1]);

    // Case 3: Single Pauli X error on qubit 3 (left edge)
    let mut x_err = [false; 9];
    x_err[3] = true;
    let syn_x3 = grid.extract_syndromes_from_errors(x_err, [false; 9], 0.01);
    // Qubit 3 is in Plaquette checks B_0 and B_2
    assert_eq!(syn_x3.plaquette_defects, vec![0, 2]);
    let corr_x3 = grid.decode_and_correct(&syn_x3);
    assert!(corr_x3.is_clean);
    assert!(!corr_x3.has_logical_error);
    assert!(corr_x3.correction_x[3]);

    // Case 4: Verify exponential suppression of logical errors: P_L << P_phys
    let p_phys = 0.01; // 1% physical error rate
    let p_logical = grid.evaluate_logical_error_rate(p_phys, 1000);
    assert!(
        p_logical < p_phys,
        "Logical error rate {} must be << physical error rate {}",
        p_logical, p_phys
    );
}
