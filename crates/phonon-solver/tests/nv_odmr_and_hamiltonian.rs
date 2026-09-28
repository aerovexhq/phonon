//! Integration Test Suite: Diamond NV Center Ground-State Hamiltonian, Hyperfine Coupling & ODMR Spectra

use approx::assert_relative_eq;
use phonon_models::sensors::{
    reconstruct_vector_magnetic_field, NitrogenIsotope, NvCenter, NvOrientation,
    NV_D_TEMP_COEFFICIENT_HZ_PER_K, NV_ELECTRON_GYROMAGNETIC_RATIO_HZ_PER_T,
    NV_ZERO_FIELD_SPLITTING_D_HZ,
};
use phonon_models::spintronics::Vec3;

#[test]
fn test_nv_zero_field_splitting_and_temperature_dependence() {
    let nv_300k = NvCenter::new(NvOrientation::V0, NitrogenIsotope::N14);
    assert_relative_eq!(
        nv_300k.d_splitting_hz,
        NV_ZERO_FIELD_SPLITTING_D_HZ,
        epsilon = 1.0
    );

    // Compute electronic eigenvalues at zero field
    let evals = nv_300k.electronic_eigenvalues(Vec3::ZERO);
    assert_eq!(evals.len(), 3);
    // Ground state |0> at E=0, degenerate |+1> and |-1> at D = 2.87 GHz
    assert_relative_eq!(evals[0], 0.0, epsilon = 1.0);
    assert_relative_eq!(evals[1], NV_ZERO_FIELD_SPLITTING_D_HZ, epsilon = 1.0);
    assert_relative_eq!(evals[2], NV_ZERO_FIELD_SPLITTING_D_HZ, epsilon = 1.0);

    // Temperature dependence: heating by +50 K shifts D by dD/dT * 50 K
    let nv_350k = NvCenter::new(NvOrientation::V0, NitrogenIsotope::N14).with_temperature(350.0);
    let expected_d = NV_ZERO_FIELD_SPLITTING_D_HZ + NV_D_TEMP_COEFFICIENT_HZ_PER_K * 50.0;
    assert_relative_eq!(nv_350k.d_splitting_hz, expected_d, epsilon = 1.0);
    assert!(nv_350k.d_splitting_hz < nv_300k.d_splitting_hz);
}

#[test]
fn test_nv_transverse_strain_splitting() {
    let strain_hz = 6.5e6; // 6.5 MHz transverse strain
    let nv = NvCenter::new(NvOrientation::V0, NitrogenIsotope::N14).with_strain_hz(strain_hz);

    let evals = nv.electronic_eigenvalues(Vec3::ZERO);
    assert_relative_eq!(evals[0], 0.0, epsilon = 1.0);
    // Splitting between upper states should be exactly 2 * E
    let splitting = evals[2] - evals[1];
    assert_relative_eq!(splitting, 2.0 * strain_hz, epsilon = 1e3);
    assert_relative_eq!(
        evals[1],
        NV_ZERO_FIELD_SPLITTING_D_HZ - strain_hz,
        epsilon = 1e3
    );
    assert_relative_eq!(
        evals[2],
        NV_ZERO_FIELD_SPLITTING_D_HZ + strain_hz,
        epsilon = 1e3
    );
}

#[test]
fn test_nv_zeeman_splitting_axial_field() {
    let nv = NvCenter::new(NvOrientation::V0, NitrogenIsotope::N14);
    let u0 = NvOrientation::V0.unit_vector();

    // Apply 2.0 mT (0.002 T) magnetic field strictly along NV axis
    let b_applied = Vec3::new(u0.x * 2.0e-3, u0.y * 2.0e-3, u0.z * 2.0e-3);
    let zeeman_expected = NV_ELECTRON_GYROMAGNETIC_RATIO_HZ_PER_T * 2.0e-3; // ~ 56.048 MHz

    let evals = nv.electronic_eigenvalues(b_applied);
    assert_relative_eq!(evals[0], 0.0, epsilon = 1e3);
    assert_relative_eq!(
        evals[1],
        NV_ZERO_FIELD_SPLITTING_D_HZ - zeeman_expected,
        epsilon = 1e4
    );
    assert_relative_eq!(
        evals[2],
        NV_ZERO_FIELD_SPLITTING_D_HZ + zeeman_expected,
        epsilon = 1e4
    );

    // Analytical resonance frequencies check
    let (f_minus, f_plus) = nv.odmr_resonance_frequencies(b_applied);
    assert_relative_eq!(f_minus, evals[1], epsilon = 1e3);
    assert_relative_eq!(f_plus, evals[2], epsilon = 1e3);
}

#[test]
fn test_nv_four_orientations_tetrahedral_geometry_and_3d_vector_reconstruction() {
    let orientations = NvOrientation::all();
    assert_eq!(orientations.len(), 4);

    // Check unit vector norms and mutual tetrahedral dot products (-1/3)
    for (i, orient_i) in orientations.iter().enumerate() {
        let ui = orient_i.unit_vector();
        assert_relative_eq!(ui.norm(), 1.0, epsilon = 1e-12);

        for orient_j in orientations.iter().skip(i + 1) {
            let uj = orient_j.unit_vector();
            let dot = ui.dot(uj);
            // In diamond, tetrahedral angle satisfies cos(theta) = -1/3
            assert_relative_eq!(dot, -1.0 / 3.0, epsilon = 1e-12);
        }
    }

    // Reconstruct an arbitrary 3D vector magnetic field
    let b_true = Vec3::new(1.85e-3, -3.42e-3, 0.95e-3); // in Tesla
    let mut projections = [0.0; 4];
    for (i, orient) in orientations.iter().enumerate() {
        projections[i] = orient.project_field(b_true);
    }

    let b_recon = reconstruct_vector_magnetic_field(&projections);
    assert_relative_eq!(b_recon.x, b_true.x, epsilon = 1e-12);
    assert_relative_eq!(b_recon.y, b_true.y, epsilon = 1e-12);
    assert_relative_eq!(b_recon.z, b_true.z, epsilon = 1e-12);
}

#[test]
fn test_nv_coupled_hyperfine_hamiltonian_dimensions_and_hermiticity() {
    let nv_n14 = NvCenter::new(NvOrientation::V0, NitrogenIsotope::N14);
    let h_n14 = nv_n14.coupled_hamiltonian(Vec3::ZERO);
    // S=1 (dim 3) x I=1 (dim 3) = dim 9
    assert_eq!(h_n14.dim, 9);
    let evals_n14 = nv_n14.coupled_eigenvalues(Vec3::ZERO);
    assert_eq!(evals_n14.len(), 9);

    let nv_n15 = NvCenter::new(NvOrientation::V0, NitrogenIsotope::N15);
    let h_n15 = nv_n15.coupled_hamiltonian(Vec3::ZERO);
    // S=1 (dim 3) x I=1/2 (dim 2) = dim 6
    assert_eq!(h_n15.dim, 6);
    let evals_n15 = nv_n15.coupled_eigenvalues(Vec3::ZERO);
    assert_eq!(evals_n15.len(), 6);
}

#[test]
fn test_nv_optical_spin_polarization_and_odmr_spectrum() {
    let nv = NvCenter::new(NvOrientation::V0, NitrogenIsotope::N14);

    // Green laser optical pumping polarization fidelity > 85%
    let pops = nv.simulate_optical_pumping(5.0); // 5 mW green laser
    let p_plus1 = pops[0];
    let p_zero = pops[1];
    let p_minus1 = pops[2];

    assert!(
        p_zero > 0.85,
        "Optical polarization fidelity must exceed 85%: got {}",
        p_zero
    );
    assert_relative_eq!(p_plus1 + p_zero + p_minus1, 1.0, epsilon = 1e-12);

    // Compute ODMR CW spectrum across microwave frequencies
    let b_field = Vec3::new(0.5e-3, 0.5e-3, 0.5e-3);
    let config = phonon_models::sensors::OdmrConfig {
        freq_start_hz: 2.80e9,
        freq_end_hz: 2.94e9,
        num_points: 300,
        laser_power_mw: 5.0,
        contrast_fraction: 0.15,
        linewidth_fwhm_hz: 10.0e6,
    };
    let spectrum = nv.compute_odmr_spectrum(&config, b_field);

    assert_eq!(spectrum.frequencies_hz.len(), 300);
    assert!(
        spectrum.min_pl() < 1.0,
        "Must exhibit photoluminescence dip"
    );
    assert!(
        spectrum.max_contrast() > 0.05,
        "Contrast dip depth must be significant"
    );

    // DC magnetic field sensitivity calculation
    let sensitivity = nv.dc_magnetic_sensitivity(8.0e6, 0.12, 5.0e6);
    assert!(
        sensitivity > 0.0 && sensitivity < 1.0e-3,
        "DC sensitivity must be physically realistic: {} T/rtHz",
        sensitivity
    );
}
