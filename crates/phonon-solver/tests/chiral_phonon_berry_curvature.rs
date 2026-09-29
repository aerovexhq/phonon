use phonon_models::chiral_phonon::{
    ChiralPhononMaterialParams, HoneycombChiralLattice, Wavevector2D,
};
use std::f64::consts::PI;

#[test]
fn test_acoustic_spin_phonon_splitting() {
    let params = ChiralPhononMaterialParams::fe2mo3o8_standard();
    let lattice = HoneycombChiralLattice::new(params);

    // Near the Dirac point K = (4*pi / 3*a, 0)
    let a = params.lattice_constant_m;
    let kx = 4.0 * PI / (3.0 * a);
    let k = Wavevector2D::new(kx, 0.0);

    let mode = lattice.evaluate_mode(&k);

    // Chiral splitting should equal h_sp
    let expected_splitting = params.spin_phonon_coupling_rad_s;
    let diff = (mode.chiral_splitting_rad_s - expected_splitting).abs();
    assert!(
        diff / expected_splitting < 0.05,
        "Expected splitting near h_sp: got {}, expected {}",
        mode.chiral_splitting_rad_s,
        expected_splitting
    );

    // Verify non-zero angular momentum
    assert!(
        mode.angular_momentum_hbar.abs() > 0.0,
        "Chiral mode must carry non-zero angular momentum"
    );
}

#[test]
fn test_acoustic_berry_curvature_and_chern_number() {
    let params = ChiralPhononMaterialParams::fe2mo3o8_standard();
    let lattice = HoneycombChiralLattice::new(params);

    // Evaluate Berry curvature at various wavevectors
    let k1 = Wavevector2D::new(1e8, 1e8);
    let mode1 = lattice.evaluate_mode(&k1);
    assert!(
        mode1.berry_curvature_m2 > 0.0,
        "Acoustic Berry curvature must be positive for right-handed coupling"
    );

    // Verify topological acoustic Chern number
    let chern = lattice.compute_chern_number(16);
    assert_eq!(
        chern, 1,
        "Topological chiral honeycomb lattice must exhibit Chern number C = +1"
    );
}

#[test]
fn test_gyroscopic_phononic_crystal_scaling() {
    let gyro_params = ChiralPhononMaterialParams::gyroscopic_metamaterial(10.0, 50.0);
    let lattice = HoneycombChiralLattice::new(gyro_params);

    let k = Wavevector2D::new(10.0, 10.0);
    let mode = lattice.evaluate_mode(&k);

    assert!(
        mode.chiral_splitting_rad_s > 0.0,
        "Gyroscopic coupling must induce circular polarization splitting"
    );
    assert!(
        mode.group_velocity_m_s.0.abs() > 0.0 || mode.group_velocity_m_s.1.abs() > 0.0,
        "Acoustic wavepacket must possess non-zero group velocity"
    );
}
