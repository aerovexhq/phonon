#![deny(unsafe_code)]

use phonon_solver::chiral_majorana_braiding::{
    ChiralCliffordGateKind, ChiralMajoranaBraidingNetwork, ChiralMajoranaBraidingParams,
    ChiralMajoranaParams, ChiralMajoranaProcessor, ChiralSurfaceDecoder,
    ChiralSurfaceDecoderParams, ChiralTransmonParityReadout, ChiralTransmonReadoutParams,
};

#[test]
fn test_majorana_braiding_network_initialization_and_modes() {
    let params = ChiralMajoranaBraidingParams {
        qubit_count: 2,
        ..Default::default()
    };
    let network = ChiralMajoranaBraidingNetwork::new(params);
    assert_eq!(network.modes.len(), 8); // 4 MZMs per qubit * 2 qubits

    for mode in &network.modes {
        assert!(mode.id >= 1 && mode.id <= 8);
        assert!(mode.amplitude > 0.99);
    }
}

#[test]
fn test_artin_non_abelian_braid_relations() {
    let network = ChiralMajoranaBraidingNetwork::new(ChiralMajoranaBraidingParams::default());
    let (passed, residual) = network.verify_artin_braid_relation();
    assert!(passed, "Artin braid relation must hold in topological regime");
    assert!(residual < 1e-10);
}

#[test]
fn test_clifford_gate_compilation_and_high_fidelity() {
    let network = ChiralMajoranaBraidingNetwork::new(ChiralMajoranaBraidingParams::default());

    let gates = [
        ChiralCliffordGateKind::Hadamard,
        ChiralCliffordGateKind::PhaseS,
        ChiralCliffordGateKind::PauliX,
        ChiralCliffordGateKind::PauliZ,
        ChiralCliffordGateKind::Cnot,
    ];

    for gate in gates {
        let compiled = network.compile_clifford_gate(gate);
        assert!(!compiled.braid_word.is_empty(), "Braid word must not be empty");
        assert!(
            compiled.process_fidelity >= 0.999,
            "Gate {:?} fidelity {} must exceed 0.999",
            gate,
            compiled.process_fidelity
        );
        assert!(
            compiled.diabatic_error < 1e-4,
            "Diabatic error {} must be strictly suppressed below 1e-4",
            compiled.diabatic_error
        );
    }
}

#[test]
fn test_surface_code_syndrome_extraction_and_mwpm_decoding() {
    let params = ChiralSurfaceDecoderParams {
        code_distance: 3,
        physical_error_rate: 0.005,
        syndrome_rounds: 3,
    };
    let mut decoder = ChiralSurfaceDecoder::new(params);
    let result = decoder.decode_and_correct();

    assert!(
        result.logical_error_rate < 0.005,
        "Logical error rate {} must be suppressed below physical error rate 0.005",
        result.logical_error_rate
    );

    // Verify all detected defects have been resolved
    for defect in &decoder.defects {
        assert!(defect.is_resolved, "Every defect must be resolved by MWPM");
    }
}

#[test]
fn test_dispersive_transmon_parity_readout_snr_and_fidelity() {
    let params = ChiralTransmonReadoutParams::default();
    let readout = ChiralTransmonParityReadout::new(params);

    let snr_db = readout.readout_snr_db();
    assert!(
        snr_db >= 18.0,
        "Readout SNR {} dB must exceed 18.0 dB",
        snr_db
    );

    let fidelity = readout.parity_readout_fidelity();
    assert!(
        fidelity >= 0.998,
        "QND readout fidelity {} must exceed 0.998",
        fidelity
    );

    let spectrum = readout.generate_transmission_spectrum(61);
    assert_eq!(spectrum.len(), 61);
    let peak_even = spectrum
        .iter()
        .map(|pt| pt.s21_even_db)
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(peak_even > -1.0, "Cavity resonance peak must be sharp");
}

#[test]
fn test_master_processor_10_point_physics_audit() {
    let params = ChiralMajoranaParams::default();
    let processor = ChiralMajoranaProcessor::new(params);
    let audit = processor.audit_coprocessor();

    assert!(audit.braid_unitarity_passed);
    assert!(audit.artin_relations_passed);
    assert!(audit.clifford_gate_fidelity_passed);
    assert!(audit.diabatic_suppression_passed);
    assert!(audit.syndrome_extraction_passed);
    assert!(audit.mwpm_decoding_passed);
    assert!(audit.logical_error_suppression_passed);
    assert!(audit.cavity_doublet_splitting_passed);
    assert!(audit.parity_readout_snr_passed);
    assert!(audit.qnd_readout_fidelity_passed);
    assert_eq!(audit.total_pass_score, 10);
    assert!(audit.all_passed);
}
