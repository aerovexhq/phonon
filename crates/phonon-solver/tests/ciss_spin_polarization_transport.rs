//! Integration tests for Chiral-Induced Spin Selectivity (CISS) & Helical Quantum Transport.
//!
//! Validates:
//! 1. Helical 3D geometry and spin-orbit vector orientation.
//! 2. Chiral inversion symmetry: $P_s(\text{LeftHanded}) = -P_s(\text{RightHanded})$.
//! 3. High-efficiency room-temperature spin polarization ($P_s > 60\%$) in non-magnetic chiral molecules.
//! 4. Strict spin degeneracy ($P_s = 0.0$) in achiral linear chains and zero SOC ($\lambda_{SO} = 0$).
//! 5. Finite-temperature Landauer-Büttiker thermal current quadrature at 300K.

use phonon_models::spintronics::ciss::{ChiralHelixGeometry, Chirality, CissHamiltonian};

#[test]
fn test_helix_geometry_and_spin_orbit_vector() {
    let right_geom = ChiralHelixGeometry::new(
        1.0, // R = 1.0 nm
        3.4, // P = 3.4 nm
        2.0 * std::f64::consts::PI / 10.0,
        12,
        Chirality::RightHanded,
    );

    let left_geom = ChiralHelixGeometry::new(
        1.0,
        3.4,
        2.0 * std::f64::consts::PI / 10.0,
        12,
        Chirality::LeftHanded,
    );

    let achiral_geom = ChiralHelixGeometry::new(
        0.0,
        3.4,
        2.0 * std::f64::consts::PI / 10.0,
        12,
        Chirality::Achiral,
    );

    // Site 0 positions:
    let r0_right = right_geom.site_position(0);
    assert!((r0_right.x - 1.0).abs() < 1e-12);
    assert!(r0_right.y.abs() < 1e-12);
    assert!(r0_right.z.abs() < 1e-12);

    // Spin-orbit vectors at site 0:
    let omega_right = right_geom.spin_orbit_vector(0);
    let omega_left = left_geom.spin_orbit_vector(0);
    let omega_achiral = achiral_geom.spin_orbit_vector(0);

    // Axial z-component must be positive for right-handed, negative for left-handed, zero for achiral:
    assert!(
        omega_right.z > 0.0,
        "Right-handed helix must have Omega_z > 0"
    );
    assert!(
        omega_left.z < 0.0,
        "Left-handed helix must have Omega_z < 0"
    );
    assert!(
        (omega_right.z + omega_left.z).abs() < 1e-12,
        "Omega_z must invert sign under chiral inversion"
    );
    assert!(
        omega_achiral.z.abs() < 1e-12,
        "Achiral chain must have Omega_z == 0"
    );
}

#[test]
fn test_spin_polarization_chiral_inversion_symmetry() {
    let geom_right = ChiralHelixGeometry::new(
        1.0,
        3.4,
        2.0 * std::f64::consts::PI / 10.0,
        12,
        Chirality::RightHanded,
    );
    let geom_left = ChiralHelixGeometry::new(
        1.0,
        3.4,
        2.0 * std::f64::consts::PI / 10.0,
        12,
        Chirality::LeftHanded,
    );

    let h_right = CissHamiltonian::new(geom_right, 0.0, 1.2, 0.28, 0.20)
        .with_dephasing(0.02)
        .with_chiral_potential(0.15);

    let h_left = CissHamiltonian::new(geom_left, 0.0, 1.2, 0.28, 0.20)
        .with_dephasing(0.02)
        .with_chiral_potential(0.15);

    let energy = 0.0; // Probe at mid-gap / Fermi level
    let trans_right = h_right.calculate_transmission(energy);
    let trans_left = h_left.calculate_transmission(energy);

    // Right-handed should transmit spin-up preferentially over spin-down:
    assert!(
        trans_right.t_up > trans_right.t_down,
        "Right-handed CISS must favor T_up > T_down"
    );
    assert!(
        trans_left.t_down > trans_left.t_up,
        "Left-handed CISS must favor T_down > T_up"
    );

    // Chiral inversion symmetry: P_s(Left) == -P_s(Right)
    let p_right = trans_right.spin_polarization;
    let p_left = trans_left.spin_polarization;
    assert!(
        (p_right + p_left).abs() < 1e-4,
        "Spin polarization must be odd under chiral inversion: P_R = {}, P_L = {}",
        p_right,
        p_left
    );
}

#[test]
fn test_high_efficiency_room_temperature_spin_polarization() {
    let geom = ChiralHelixGeometry::new(
        1.0,
        3.4,
        2.0 * std::f64::consts::PI / 10.0,
        12,
        Chirality::RightHanded,
    );

    let h = CissHamiltonian::new(geom, 0.0, 1.2, 0.28, 0.20)
        .with_dephasing(0.02)
        .with_chiral_potential(0.15);

    let trans = h.calculate_transmission(0.0);

    // Must exhibit high-efficiency spin polarization exceeding 60%:
    assert!(
        trans.spin_polarization > 0.60,
        "CISS spin polarization must exceed 60%, got {:.2}%",
        trans.spin_polarization * 100.0
    );

    // Total transmission must be non-zero and physically bounded <= 1.0:
    assert!(trans.t_total > 0.0);
    assert!(trans.t_up <= 1.0);
    assert!(trans.t_down <= 1.0);
}

#[test]
fn test_achiral_and_zero_soc_spin_degeneracy() {
    // 1. Achiral geometry:
    let geom_achiral = ChiralHelixGeometry::new(
        0.0,
        3.4,
        2.0 * std::f64::consts::PI / 10.0,
        12,
        Chirality::Achiral,
    );
    let h_achiral = CissHamiltonian::new(geom_achiral, 0.0, 1.2, 0.28, 0.20)
        .with_dephasing(0.02)
        .with_chiral_potential(0.15);

    let trans_achiral = h_achiral.calculate_transmission(0.0);
    assert!(
        trans_achiral.spin_polarization.abs() < 1e-6,
        "Achiral chain must exhibit exactly zero spin polarization, got {}",
        trans_achiral.spin_polarization
    );
    assert!(
        (trans_achiral.t_up - trans_achiral.t_down).abs() < 1e-6,
        "T_up and T_down must be degenerate in achiral chain"
    );

    // 2. Chiral geometry with zero SOC (lambda_SO = 0.0):
    let geom_chiral = ChiralHelixGeometry::new(
        1.0,
        3.4,
        2.0 * std::f64::consts::PI / 10.0,
        12,
        Chirality::RightHanded,
    );
    let h_no_soc = CissHamiltonian::new(geom_chiral, 0.0, 1.2, 0.0, 0.20)
        .with_dephasing(0.02)
        .with_chiral_potential(0.15);

    let trans_no_soc = h_no_soc.calculate_transmission(0.0);
    assert!(
        trans_no_soc.spin_polarization.abs() < 1e-6,
        "Zero SOC must exhibit zero spin polarization, got {}",
        trans_no_soc.spin_polarization
    );
}

#[test]
fn test_thermal_current_integration_at_300k() {
    let geom = ChiralHelixGeometry::new(
        1.0,
        3.4,
        2.0 * std::f64::consts::PI / 10.0,
        12,
        Chirality::RightHanded,
    );

    let h = CissHamiltonian::new(geom, 0.0, 1.2, 0.28, 0.20)
        .with_dephasing(0.02)
        .with_chiral_potential(0.15);

    // Evaluate thermal currents at 300K under 20 mV bias
    let (i_up, i_down, p_th) = h.calculate_thermal_currents(0.020, 300.0, 50);

    assert!(i_up > 0.0, "Thermal current I_up must be positive");
    assert!(i_down > 0.0, "Thermal current I_down must be positive");
    assert!(
        i_up > i_down,
        "Thermal current I_up must exceed I_down for right-handed helix"
    );
    assert!(
        p_th > 0.60,
        "Thermal spin polarization at 300K must exceed 60%, got {:.2}%",
        p_th * 100.0
    );
}
