//! Integration tests for 2D hexagonal superconducting nanowire arrays,
//! tri-junction connectivity, orientation-dependent topological gap, and MZM hybridization.

use phonon_models::hexagonal_majorana::{
    HexagonalSuperconductingArray, InPlaneMagneticField, MajoranaMaterialParams, MajoranaZeroMode,
    NanowireSegment, TopologicalQubitRegister, TriJunction,
};
use std::f64::consts::PI;

#[test]
fn test_hexagonal_array_geometry_and_connectivity() {
    let arm_length_nm = 1200.0;
    let field_b = 1.0; // 1.0 Tesla
    let field_ang = PI / 6.0; // 30 degrees

    let cell =
        HexagonalSuperconductingArray::create_honeycomb_cell(arm_length_nm, field_b, field_ang);

    assert_eq!(
        cell.junctions.len(),
        6,
        "Honeycomb cell must have 6 tri-junction vertices"
    );
    assert_eq!(
        cell.segments.len(),
        12,
        "Honeycomb cell must have 6 perimeter + 6 external arms"
    );

    // Verify 120-degree tri-junction connectivity for each vertex
    for junc in &cell.junctions {
        assert_eq!(junc.connected_segments.len(), 3);
        let arm0 = &cell.segments[junc.connected_segments[0]];
        let arm1 = &cell.segments[junc.connected_segments[1]];
        let diff_angle = (arm0.orientation_angle_rad - arm1.orientation_angle_rad).abs();
        assert!(
            diff_angle > 0.1,
            "Connected arms must point in distinct spatial directions"
        );
    }

    // Verify transmission through open junction
    let open_junc = TriJunction::new(0, (0.0, 0.0), [0, 1, 2], 0.0);
    assert!((open_junc.junction_transmission(0.25) - 0.5).abs() < 1e-4);

    // Verify pinched-off junction
    let pinched_junc = TriJunction::new(0, (0.0, 0.0), [0, 1, 2], 5.0);
    assert!(pinched_junc.junction_transmission(0.25) < 1e-6);
}

#[test]
fn test_orientation_dependent_topological_gap_and_localization() {
    let materials = MajoranaMaterialParams::inas_al();
    assert!(
        materials.spin_orbit_energy_mev() > 0.1,
        "Spin-orbit energy must be physical (> 0.1 meV)"
    );
    assert!(
        materials.spin_orbit_length_nm() > 50.0,
        "Spin-orbit length must be physical (> 50 nm)"
    );

    // Field aligned along x-axis (0 deg)
    let field = InPlaneMagneticField::new(1.2, 0.0);

    // Wire along 0 deg (parallel to field) -> maximum topological gap
    let mut wire_par = NanowireSegment::new(0, 0, 1, 1500.0, 0.0);
    wire_par.set_gates(0.0, 0.0, 0.0); // mu = 0.0
    assert!(wire_par.is_topological(0.5, &materials, &field));
    let gap_par = wire_par.topological_gap_mev(0.5, &materials, &field);
    assert!(
        gap_par > 0.01,
        "Parallel wire must have significant topological gap"
    );

    // Localization length
    let xi_par = wire_par.majorana_coherence_length_nm(0.5, &materials, &field);
    assert!(
        xi_par > 50.0 && xi_par < 500.0,
        "xi_M must be in 50-500 nm range"
    );

    // Wire along 90 deg (perpendicular to field) -> zero parallel Zeeman -> gap vanishes
    let mut wire_perp = NanowireSegment::new(1, 0, 2, 1500.0, PI / 2.0);
    wire_perp.set_gates(0.0, 0.0, 0.0);
    assert!(!wire_perp.is_topological(0.5, &materials, &field));
    assert_eq!(wire_perp.topological_gap_mev(0.5, &materials, &field), 0.0);

    // Gate pinch-off: setting mu = 5.0 meV turns topological wire trivial
    wire_par.set_gates(5.0, 5.0, 5.0);
    assert!(!wire_par.is_topological(0.5, &materials, &field));
}

#[test]
fn test_majorana_hybridization_and_quantum_capacitance_readout() {
    let mzm = MajoranaZeroMode::new(0, 0, 0.0, 120.0); // xi_M = 120 nm

    // For short separation (d = 100 nm < xi_M), hybridization is strong
    let e_hyb_short = mzm.hybridization_energy_mev(100.0, 0.25, 40.0);
    // For long separation (d = 1200 nm >> xi_M), hybridization is exponentially suppressed
    let e_hyb_long = mzm.hybridization_energy_mev(1200.0, 0.25, 40.0);

    assert!(
        e_hyb_short > 1e-3,
        "Short distance must have measurable hybridization"
    );
    assert!(
        e_hyb_long < 1e-4,
        "Long distance must exponentially suppress hybridization"
    );
    assert!(
        e_hyb_short > 100.0 * e_hyb_long,
        "Hybridization must show massive exponential decay"
    );

    // Parity readout via dispersive quantum capacitance
    let qubit = TopologicalQubitRegister::new_zero(0.20);
    let c_q_even = qubit.dispersive_quantum_capacitance_f(e_hyb_short, 0.020, true);
    let c_q_odd = qubit.dispersive_quantum_capacitance_f(e_hyb_short, 0.020, false);

    assert!(
        c_q_even > 0.0,
        "Even parity must yield positive quantum capacitance"
    );
    assert!(
        c_q_odd < 0.0,
        "Odd parity must yield negative quantum capacitance"
    );
    assert_eq!(
        c_q_even, -c_q_odd,
        "Quantum capacitance must be anti-symmetric across parities"
    );
}
