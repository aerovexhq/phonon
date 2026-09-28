//! Integration tests for Rotated Surface Codes, Color Codes, and QEC Decoders.

use phonon_models::topological::{PauliOp, RotatedSurfaceCode, TriangularColorCode};
use phonon_solver::topological::{BeliefPropagationDecoder, MwpmDecoder};

#[test]
fn test_rotated_surface_code_geometry_and_stabilizers() {
    // Distance d=3 code
    let code_d3 = RotatedSurfaceCode::new(3);
    assert_eq!(code_d3.num_data_qubits, 9);
    assert_eq!(code_d3.num_stabilizers, 8); // d^2 - 1 = 8
    assert_eq!(code_d3.logical_x_indices.len(), 3);
    assert_eq!(code_d3.logical_z_indices.len(), 3);

    // Distance d=5 code
    let code_d5 = RotatedSurfaceCode::new(5);
    assert_eq!(code_d5.num_data_qubits, 25);
    assert_eq!(code_d5.num_stabilizers, 24); // 5^2 - 1 = 24
    assert_eq!(code_d5.logical_x_indices.len(), 5);
    assert_eq!(code_d5.logical_z_indices.len(), 5);

    // Logical X and Z operators must anticommute
    let mut x_errors = vec![PauliOp::I; 9];
    for &idx in &code_d3.logical_x_indices {
        x_errors[idx] = PauliOp::X;
    }
    let (x_fail, z_fail) = code_d3.causes_logical_error(&x_errors);
    assert!(!x_fail, "X_L should commute with Z_L");
    assert!(z_fail, "X_L should anticommute with Z_L");
}

#[test]
fn test_triangular_color_code_properties() {
    let color_code = TriangularColorCode::distance_3();
    assert_eq!(color_code.distance, 3);
    assert_eq!(color_code.num_data_qubits, 7);
    assert_eq!(color_code.num_plaquettes, 3);

    // In color codes, a single error produces non-trivial X and Z syndromes
    let mut errors = vec![PauliOp::I; 7];
    errors[0] = PauliOp::X;
    let (x_syn, z_syn) = color_code.measure_syndromes(&errors);
    assert_eq!(x_syn, vec![0, 0, 0]);
    assert_eq!(z_syn, vec![1, 0, 0], "Plaquette 0 contains qubit 0");
}

#[test]
fn test_mwpm_decoder_single_qubit_corrections() {
    let code = RotatedSurfaceCode::new(3);
    let decoder = MwpmDecoder::new(3);

    // Test all single-qubit Pauli X and Z errors
    for q in 0..code.num_data_qubits {
        // 1. Bit-flip error Pauli X
        let mut errors = vec![PauliOp::I; code.num_data_qubits];
        errors[q] = PauliOp::X;
        let syn = code.measure_syndrome(&errors);

        let corr = decoder.decode_rotated_surface(&code, &syn);
        let mut net = Vec::new();
        for i in 0..code.num_data_qubits {
            net.push(errors[i].multiply(corr[i]));
        }
        let (x_fail, z_fail) = code.causes_logical_error(&net);
        assert!(
            !x_fail && !z_fail,
            "MWPM failed to correct single Pauli X on qubit {}",
            q
        );

        // 2. Phase-flip error Pauli Z
        let mut errors_z = vec![PauliOp::I; code.num_data_qubits];
        errors_z[q] = PauliOp::Z;
        let syn_z = code.measure_syndrome(&errors_z);

        let corr_z = decoder.decode_rotated_surface(&code, &syn_z);
        let mut net_z = Vec::new();
        for i in 0..code.num_data_qubits {
            net_z.push(errors_z[i].multiply(corr_z[i]));
        }
        let (x_fail_z, z_fail_z) = code.causes_logical_error(&net_z);
        assert!(
            !x_fail_z && !z_fail_z,
            "MWPM failed to correct single Pauli Z on qubit {}",
            q
        );
    }
}

#[test]
fn test_belief_propagation_decoder_convergence() {
    let code = RotatedSurfaceCode::new(3);
    let decoder = BeliefPropagationDecoder::new(30, 0.65);

    // Single bit-flip error on center qubit 4
    let mut errors = vec![PauliOp::I; code.num_data_qubits];
    errors[4] = PauliOp::X;
    let syn = code.measure_syndrome(&errors);

    let (corr, converged) = decoder.decode(&code, &syn, 0.05);
    assert!(
        converged,
        "Belief propagation must converge on sparse syndrome"
    );

    let mut net = Vec::new();
    for i in 0..code.num_data_qubits {
        net.push(errors[i].multiply(corr[i]));
    }
    let (x_fail, z_fail) = code.causes_logical_error(&net);
    assert!(!x_fail && !z_fail, "BP decoder resulted in logical failure");
}
