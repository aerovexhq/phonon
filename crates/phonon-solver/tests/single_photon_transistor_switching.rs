#![deny(unsafe_code)]

//! Integration Tests for Sub-Diffraction Plasmonic Transistors & Maxwell-Bloch Dynamics.

use approx::assert_relative_eq;
use phonon_models::quantum_plasmonics::{
    NobleMetal, PlasmonicSlotWaveguide, QuantumEmitter, SinglePhotonTransistor,
};
use phonon_solver::quantum_plasmonics::{
    evaluate_plasmonic_directional_coupler, evaluate_transistor_logic, evaluate_waveguide_bend,
    PlasmonicMaxwellBlochSolver,
};

#[test]
fn test_subdiffraction_mode_volume_and_purcell_enhancement() {
    let metal = NobleMetal::Silver;
    let eps_d = 2.25;
    let slot_width = 5.0e-9; // 5 nm
    let slot_height = 40.0e-9; // 40 nm
    let length = 1.0e-6; // 1 um

    let waveguide = PlasmonicSlotWaveguide::new(metal, eps_d, slot_width, slot_height, length);
    let wavelength = 780.0e-9; // 780 nm

    let v_norm = waveguide.normalized_mode_volume(wavelength);
    assert!(
        v_norm < 1.0e-3,
        "Normalized mode volume must be sub-diffraction (< 1e-3), got {}",
        v_norm
    );

    let omega = (2.0 * std::f64::consts::PI * phonon_models::quantum_plasmonics::SPEED_OF_LIGHT)
        / wavelength;
    let e_zpf = waveguide.zero_point_electric_field(omega, wavelength);
    assert!(
        e_zpf > 1.0e5,
        "Zero-point electric field {} V/m is below expectation",
        e_zpf
    );

    let dipole = 25.0 * 3.336e-30; // 25 Debye
    let emitter = QuantumEmitter::new(wavelength, dipole, 1.0e7, 1.0e8);
    let transistor = SinglePhotonTransistor::new(waveguide, emitter, 30.0);

    let fp = transistor.purcell_factor();
    assert!(
        fp >= 100.0,
        "Purcell enhancement factor must exceed 100, got {}",
        fp
    );

    let beta = transistor.beta_factor();
    assert!(
        beta >= 0.90,
        "Coupling beta factor must exceed 90%, got {:.4}",
        beta
    );
}

#[test]
fn test_single_photon_transistor_switching_contrast_and_gain() {
    let metal = NobleMetal::Silver;
    let eps_d = 2.25;
    let waveguide = PlasmonicSlotWaveguide::new(metal, eps_d, 4.0e-9, 40.0e-9, 1.0e-6);
    let wavelength = 780.0e-9;
    let dipole = 30.0 * 3.336e-30; // 30 Debye
    let emitter = QuantumEmitter::new(wavelength, dipole, 1.0e7, 1.0e8);
    let transistor = SinglePhotonTransistor::new(waveguide, emitter, 35.0);

    let t0 = transistor.unpumped_transmission();
    let t1 = transistor.saturated_transmission();
    let contrast_db = transistor.switching_contrast_db();

    assert!(
        contrast_db > 20.0,
        "All-optical switching contrast must exceed 20 dB, got {:.2} dB",
        contrast_db
    );
    assert!(t0 < 0.01, "OFF transmission should be < 1%, got {}", t0);
    assert!(t1 >= 0.80, "ON transmission should be >= 80%, got {}", t1);

    // Digital logic evaluation
    let logic = evaluate_transistor_logic(&transistor, 1.0e10); // 10 Gbit/s probe stream
    assert_relative_eq!(logic.switching_contrast_db, contrast_db, epsilon = 1e-6);
    assert!(logic.optical_gain > 5.0, "Optical gain should exceed 5x");
    assert!(
        logic.switching_energy_joules > 1e-19 && logic.switching_energy_joules < 1e-18,
        "Single-photon switching energy should be sub-attojoule"
    );

    // Maxwell-Bloch dynamics simulation
    let mb_solver = PlasmonicMaxwellBlochSolver::new(transistor, 0.0);
    let (t_off, t_switched, final_bloch) =
        mb_solver.simulate_switching_transient(std::f64::consts::PI, 100);
    assert_relative_eq!(t_off, t0, epsilon = 1e-9);
    assert!(
        t_switched > t_off * 10.0,
        "Switched transmission should be >10x unpumped transmission"
    );
    assert!(final_bloch.excited_population() > 0.5);
}

#[test]
fn test_plasmonic_directional_coupler_and_bend_routing() {
    // Directional coupler
    let gap = 20.0e-9; // 20 nm
    let decay_len = 15.0e-9;
    let l_wg = 500.0e-9; // 500 nm

    let coupler = evaluate_plasmonic_directional_coupler(gap, l_wg, decay_len);
    assert_relative_eq!(
        coupler.through_power_fraction + coupler.cross_power_fraction,
        1.0,
        epsilon = 1e-6
    );
    assert!(coupler.crossover_length > 0.0);

    // Waveguide bend
    let bend_radius = 250.0e-9; // 250 nm sub-micron sharp bend
    let bend_angle = std::f64::consts::FRAC_PI_2; // 90 degree bend
    let n_eff = 2.5;
    let l_prop = 15.0e-6; // 15 um

    let bend = evaluate_waveguide_bend(bend_radius, bend_angle, n_eff, l_prop);
    assert!(
        bend.transmission >= 0.85,
        "Sub-micron bend transmission should exceed 85%, got {}",
        bend.transmission
    );
    assert!(bend.loss_db < 1.0, "Bend loss should be < 1 dB");
}
