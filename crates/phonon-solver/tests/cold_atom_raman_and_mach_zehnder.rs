//! Integration Test Suite:
//! Cold Atom Raman Transitions, Bragg Diffraction & Mach-Zehnder Matter-Wave Interferometry.

use phonon_core::constants::{H_BAR, PLANCK_CONSTANT, SPEED_OF_LIGHT};
use phonon_models::quantum::{
    AtomicSpecies, MachZehnderInterferometer, TwoPhotonTransition, TwoPhotonTransitionType,
    BOHR_RADIUS_METERS, EARTH_ROTATION_RATE_RAD_S, RB87_D2_LINEWIDTH_RAD_S,
    RB87_D2_WAVELENGTH_METERS, RB87_MASS_KG, RB87_S_WAVE_SCATTERING_LENGTH_M,
    SR88_CLOCK_LINEWIDTH_RAD_S, SR88_CLOCK_WAVELENGTH_METERS, SR88_MASS_KG, STANDARD_GRAVITY_M_S2,
};
use phonon_solver::quantum::Gpe1DPropagator;

#[test]
fn test_rubidium87_and_strontium88_physical_properties_and_recoils() {
    let rb = AtomicSpecies::Rubidium87;
    let sr = AtomicSpecies::Strontium88;

    // 1. Atomic rest masses
    assert!((rb.mass_kg() - RB87_MASS_KG).abs() < 1e-30);
    assert!((sr.mass_kg() - SR88_MASS_KG).abs() < 1e-30);
    assert!((1.44e-25..=1.45e-25).contains(&rb.mass_kg()));
    assert!((1.45e-25..=1.47e-25).contains(&sr.mass_kg()));

    // 2. Optical transitions
    assert_eq!(
        rb.primary_transition_wavelength(),
        RB87_D2_WAVELENGTH_METERS
    );
    assert_eq!(
        sr.primary_transition_wavelength(),
        SR88_CLOCK_WAVELENGTH_METERS
    );

    let freq_rb = rb.primary_transition_frequency();
    let expected_freq_rb = SPEED_OF_LIGHT / RB87_D2_WAVELENGTH_METERS;
    assert!((freq_rb - expected_freq_rb).abs() < 1.0);
    assert!((freq_rb - 384.23e12).abs() < 0.1e12); // ~384.23 THz

    // Linewidths
    assert_eq!(rb.natural_linewidth_rad_s(), RB87_D2_LINEWIDTH_RAD_S);
    assert_eq!(sr.natural_linewidth_rad_s(), SR88_CLOCK_LINEWIDTH_RAD_S);

    // 3. Recoil dynamics on Rb87 D2 line
    let p_rec = rb.recoil_momentum(RB87_D2_WAVELENGTH_METERS);
    let expected_p = PLANCK_CONSTANT / RB87_D2_WAVELENGTH_METERS;
    assert!((p_rec - expected_p).abs() < 1e-35);

    let v_rec = rb.recoil_velocity(RB87_D2_WAVELENGTH_METERS);
    // Rb87 recoil velocity is 5.8845 mm/s
    assert!((v_rec - 5.8845e-3).abs() < 1e-4);

    let e_rec = rb.recoil_energy(RB87_D2_WAVELENGTH_METERS);
    let expected_e = (p_rec * p_rec) / (2.0 * RB87_MASS_KG);
    assert!((e_rec - expected_e).abs() < 1e-35);

    let nu_rec = rb.recoil_frequency_hz(RB87_D2_WAVELENGTH_METERS);
    // Recoil frequency is ~3.77 kHz
    assert!((nu_rec - 3771.0).abs() < 10.0);

    // 4. Contact interaction strengths
    assert_eq!(rb.scattering_length_m(), RB87_S_WAVE_SCATTERING_LENGTH_M);
    assert!((rb.scattering_length_m() - 100.0 * BOHR_RADIUS_METERS).abs() < 1e-15);

    let g_3d = rb.interaction_strength_3d();
    let expected_g3d =
        (4.0 * std::f64::consts::PI * H_BAR * H_BAR * rb.scattering_length_m()) / rb.mass_kg();
    assert!((g_3d - expected_g3d).abs() < 1e-55);
    assert!(g_3d > 0.0); // Repulsive interaction stabilizes condensate

    let omega_perp = 2.0 * std::f64::consts::PI * 150.0; // 150 Hz radial trap
    let g_1d = rb.interaction_strength_1d(omega_perp);
    let expected_g1d = 2.0 * H_BAR * omega_perp * rb.scattering_length_m();
    assert!((g_1d - expected_g1d).abs() < 1e-45);
}

#[test]
fn test_two_photon_raman_and_bragg_transitions_and_rabi_evolution() {
    let rabi_freq = 2.0 * std::f64::consts::PI * 20.0e3; // 20 kHz Rabi frequency
    let mut raman = TwoPhotonTransition::standard_rb87_raman(rabi_freq);

    assert_eq!(raman.transition_type, TwoPhotonTransitionType::Raman);
    assert_eq!(raman.diffraction_order, 1);

    // Effective wavevector: counter-propagating gives k_eff = 2 * k_L
    let k_l = 2.0 * std::f64::consts::PI / RB87_D2_WAVELENGTH_METERS;
    let expected_k_eff = 2.0 * k_l;
    assert!((raman.effective_wavevector_mag() - expected_k_eff).abs() < 1e-6);

    // Momentum kick: Delta p = hbar * k_eff
    let delta_p = raman.momentum_kick();
    assert!((delta_p - H_BAR * expected_k_eff).abs() < 1e-35);

    // Recoil velocity kick: 2 * v_recoil = 11.77 mm/s
    let delta_v = raman.recoil_velocity_kick();
    assert!((delta_v - 2.0 * 5.8845e-3).abs() < 1e-4);

    // Pulse durations
    let t_pi_2 = raman.half_pi_pulse_duration();
    let t_pi = raman.pi_pulse_duration();
    assert!((t_pi - 2.0 * t_pi_2).abs() < 1e-12);
    assert!((t_pi_2 - (std::f64::consts::PI / (2.0 * rabi_freq))).abs() < 1e-12);

    // Resonant transition probabilities
    let p_beam_splitter = raman.transition_probability(t_pi_2);
    assert!((p_beam_splitter - 0.5).abs() < 1e-6);

    let p_mirror = raman.transition_probability(t_pi);
    assert!((p_mirror - 1.0).abs() < 1e-6);

    let p_zero = raman.transition_probability(0.0);
    assert_eq!(p_zero, 0.0);

    let p_two_pi = raman.transition_probability(2.0 * t_pi);
    assert!(p_two_pi.abs() < 1e-6);

    // Detuned transition
    raman.detuning = rabi_freq * 1.5;
    let p_detuned = raman.transition_probability(t_pi);
    assert!(p_detuned < 0.5); // Population transfer suppressed

    // Cayley-Klein Unitary Evolution Matrix
    raman.detuning = 0.0;
    let u_pi_2 = raman.evolution_matrix_2x2(t_pi_2, 0.0);
    // For theta = pi/2: cos(pi/4) = 1/sqrt(2), sin(pi/4) = 1/sqrt(2)
    let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
    assert!((u_pi_2[0][0].re - inv_sqrt2).abs() < 1e-6);
    assert!(u_pi_2[0][0].im.abs() < 1e-6);
    assert!(u_pi_2[0][1].re.abs() < 1e-6);
    assert!((u_pi_2[0][1].im + inv_sqrt2).abs() < 1e-6);

    // Unitarity check: U^\dagger U = I
    let u00 = u_pi_2[0][0];
    let u01 = u_pi_2[0][1];
    let u10 = u_pi_2[1][0];
    let u11 = u_pi_2[1][1];
    let norm_row0 = u00.norm_sq() + u01.norm_sq();
    let norm_row1 = u10.norm_sq() + u11.norm_sq();
    assert!((norm_row0 - 1.0).abs() < 1e-12);
    assert!((norm_row1 - 1.0).abs() < 1e-12);

    // Bragg higher-order diffraction
    let bragg = TwoPhotonTransition::standard_bragg(
        AtomicSpecies::Rubidium87,
        RB87_D2_WAVELENGTH_METERS,
        rabi_freq,
        4,
    );
    assert_eq!(bragg.transition_type, TwoPhotonTransitionType::Bragg);
    assert_eq!(bragg.diffraction_order, 4);
    assert!((bragg.effective_wavevector_mag() - 4.0 * expected_k_eff).abs() < 1e-6);
    assert!((bragg.recoil_velocity_kick() - 4.0 * delta_v).abs() < 1e-4);
}

#[test]
fn test_mach_zehnder_gravitational_and_gradient_phase_accumulation() {
    let t_interrogation = 0.10; // 100 ms pulse separation time
    let mut mz = MachZehnderInterferometer::standard_rb87_gravimeter(t_interrogation);

    assert_eq!(mz.total_time(), 0.20);
    assert_eq!(mz.interrogation_time, 0.10);
    assert!((mz.contrast - 0.90).abs() < 1e-6);

    let _k_mag = mz.effective_k_magnitude();
    let t2 = t_interrogation * t_interrogation;

    // 1. Gravitational phase accumulation
    let phi_g = mz.gravitational_phase();
    // DeltaPhi_g = k_eff * (-g) * T^2 = 1.6106e7 * (-9.80665) * 0.01 = -1.5794e6 rad
    let expected_phi_g = mz.effective_k[2] * mz.gravity[2] * t2;
    assert!((phi_g - expected_phi_g).abs() < 1e-6);
    assert!((-1.7e6..=-1.5e6).contains(&phi_g));

    // 2. Vertical gravity gradient phase accumulation across baseline d = 1.0 m
    mz.gravity_gradient_zz = 3.086e-6; // Nominal Earth vertical gradient: 3086 Eötvös
    mz.baseline_distance = 1.0;
    let dphi_grad = mz.gravity_gradient_phase_diff();
    // DeltaPhi_grad = k_eff * T_zz * d * T^2 = 1.6106e7 * 3.086e-6 * 1.0 * 0.01 = 0.497 rad
    assert!((dphi_grad - 0.497).abs() < 0.01);

    // 3. Output port excited and ground state populations
    mz.laser_phase_shift = 0.0;
    let p_e = mz.excited_state_population();
    let p_g = mz.ground_state_population();
    assert!((p_e + p_g - 1.0).abs() < 1e-12);
    assert!((0.0..=1.0).contains(&p_e));
    assert!((0.0..=1.0).contains(&p_g));

    // When total phase is 0 mod 2pi, P_e = 0.5 * (1 - C) = 0.5 * (1 - 0.9) = 0.05
    mz.laser_phase_shift = -phi_g; // Cancel gravitational phase exactly
    let p_e_res = mz.excited_state_population();
    assert!((p_e_res - 0.05).abs() < 1e-6);

    // When total phase is pi mod 2pi, P_e = 0.5 * (1 + C) = 0.5 * (1 + 0.9) = 0.95
    mz.laser_phase_shift = -phi_g + std::f64::consts::PI;
    let p_e_pi = mz.excited_state_population();
    assert!((p_e_pi - 0.95).abs() < 1e-6);

    // 4. Dual-cloud gravity gradiometer populations
    mz.laser_phase_shift = -phi_g;
    let (p1, p2) = mz.dual_cloud_gradiometer_populations();
    assert!((p1 - 0.05).abs() < 1e-6);
    // Cloud 2 experiences phase shift dphi_grad ~ 0.497 rad
    let expected_p2 = 0.5 * (1.0 - 0.90 * dphi_grad.cos());
    assert!((p2 - expected_p2).abs() < 1e-6);

    // 5. Acceleration sensitivity
    let atom_count = 1_000_000.0;
    let cycle_time = 1.0;
    let sens_g = mz.acceleration_sensitivity(atom_count, cycle_time);
    // eta_g <= 1e-8 m/s^2 / sqrt(Hz)
    assert!(sens_g <= 1.0e-8);
    assert!(sens_g > 1.0e-10);

    let sens_grad = mz.gradiometer_sensitivity(atom_count, cycle_time);
    assert!(sens_grad < 2.0e-8);
}

#[test]
fn test_sagnac_rotational_matter_wave_phase_shift() {
    let mut mz = MachZehnderInterferometer::standard_rb87_gravimeter(0.08); // 80 ms
                                                                            // Horizontal beam launch velocity along X: 15 m/s
    mz.initial_velocity = [15.0, 0.0, 0.0];
    // Vertical effective wavevector along Z
    let k_mag = 2.0 * (2.0 * std::f64::consts::PI / RB87_D2_WAVELENGTH_METERS);
    mz.effective_k = [0.0, 0.0, k_mag];
    // Platform rotation around Y axis
    mz.rotation_rate = [0.0, EARTH_ROTATION_RATE_RAD_S, 0.0];

    // Enclosed loop area vector A
    let area = mz.enclosed_area_vector();
    // Area should point along Y (since k is along Z, v is along X, k x v is along -Y)
    assert!(area[0].abs() < 1e-12);
    assert!(area[1].abs() > 0.0);
    assert!(area[2].abs() < 1e-12);

    let sagnac_phase = mz.sagnac_rotational_phase();
    assert!(sagnac_phase.abs() > 0.0);

    // Reversing rotation rate reverses Sagnac phase
    mz.rotation_rate[1] = -EARTH_ROTATION_RATE_RAD_S;
    let sagnac_neg = mz.sagnac_rotational_phase();
    assert!((sagnac_phase + sagnac_neg).abs() < 1e-12);

    // Doubling horizontal velocity doubles enclosed area and Sagnac phase
    mz.initial_velocity[0] = 30.0;
    let sagnac_double = mz.sagnac_rotational_phase();
    assert!((sagnac_double - 2.0 * sagnac_neg).abs() < 1e-10);
}

#[test]
fn test_split_step_fourier_gpe_wavepacket_propagation() {
    let grid_size = 256;
    let window_m = 40.0e-6; // 40 microns spatial window
    let mut gpe = Gpe1DPropagator::new(AtomicSpecies::Rubidium87, grid_size, window_m);

    assert_eq!(gpe.grid_points, 256);
    assert_eq!(gpe.x_coords.len(), 256);
    assert_eq!(gpe.k_coords.len(), 256);
    assert_eq!(gpe.wavefunction.len(), 256);

    // Initialize Gaussian centered at x = 0 with width sigma = 3 microns
    let sigma_0 = 3.0e-6;
    gpe.initialize_gaussian(0.0, sigma_0, 0.0);

    let initial_norm = gpe.total_norm();
    assert!((initial_norm - 1.0).abs() < 1e-6);

    let initial_x = gpe.expectation_position();
    assert!(initial_x.abs() < 1e-9);

    let initial_width = gpe.position_width();
    // Standard deviation of |psi|^2 for exp(-x^2 / 2sigma^2) is sigma / sqrt(2) ~ 2.12 um
    assert!((initial_width - sigma_0 / 2.0_f64.sqrt()).abs() < 0.1e-6);

    // Apply 50 microsecond propagation in free space with gravity
    gpe.gravity_acceleration = STANDARD_GRAVITY_M_S2;
    gpe.propagate(50.0e-6, 1.0e-6);

    // Norm strictly conserved
    let norm_after = gpe.total_norm();
    assert!((norm_after - 1.0).abs() < 1e-5);

    // Quantum dispersion causes wavepacket expansion
    let width_after = gpe.position_width();
    assert!(width_after >= initial_width);

    // Beam splitter application creates two momentum components
    let k_kick = 1.0e6; // 1e6 rad/m kick
    gpe.apply_beam_splitter(k_kick);
    assert!((gpe.total_norm() - 1.0).abs() < 1e-5);
}
