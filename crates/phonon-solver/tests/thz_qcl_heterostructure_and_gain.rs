//! Integration test suite for Phase 42:
//! Terahertz Quantum Cascade Lasers, Multiple Quantum Well Heterostructures,
//! 1D Schrödinger Solver, Resonant LO-Phonon Depopulation & Polaritonic Waveguides.

use phonon_models::quantum::{
    HeterostructureProfile, QclMaterialSystem, ResonantLoPhononDepopulation, Schrodinger1DSolver,
    ThzOpticalGainModel, ThzPolaritonicWaveguide, GAAS_LO_PHONON_ENERGY_EV,
};

#[test]
fn test_multiple_quantum_well_heterostructure_and_schrodinger_eigensolver() {
    // 1. Benchmark infinite square well analytical eigenstates:
    let l_well = 20.0e-9; // 20 nm
    let mat_gaas = QclMaterialSystem::GaAsAlGaAs;
    let well_profile = HeterostructureProfile::infinite_square_well(l_well, mat_gaas);
    let bound_states = Schrodinger1DSolver::solve_bound_states(&well_profile, 4);

    assert!(bound_states.len() >= 4);

    // Orthonormality verification: integral psi_i(z) * psi_j(z) dz = delta_ij
    let dz = well_profile.grid_step_m;
    for i in 0..bound_states.len() {
        for j in 0..bound_states.len() {
            let overlap: f64 = bound_states[i]
                .wavefunction
                .iter()
                .zip(&bound_states[j].wavefunction)
                .map(|(w_i, w_j)| w_i * w_j * dz)
                .sum();

            if i == j {
                assert!(
                    (overlap - 1.0).abs() < 1e-2,
                    "Wavefunction norm not unity for state {}: {}",
                    i,
                    overlap
                );
            } else {
                assert!(
                    overlap.abs() < 1e-2,
                    "Wavefunctions {} and {} not orthogonal: overlap = {}",
                    i,
                    j,
                    overlap
                );
            }
        }
    }

    // Dipole matrix element z12 and oscillator strength:
    let z12 = Schrodinger1DSolver::compute_dipole_matrix_element(
        &well_profile,
        &bound_states[0],
        &bound_states[1],
    );
    assert!(z12 > 1.0e-9 && z12 < 5.0e-9, "Dipole z12: {} m", z12);

    let delta_e_joules = bound_states[1].energy_joules - bound_states[0].energy_joules;
    let f12 = Schrodinger1DSolver::compute_oscillator_strength(mat_gaas, delta_e_joules, z12);
    assert!(
        f12 > 0.5 && f12 < 1.5,
        "Oscillator strength f12 should be order unity (Thomas-Reiche-Kuhn sum rule): {}",
        f12
    );

    // 2. InGaAs/InAlAs material system:
    let mat_inp = QclMaterialSystem::InGaAsInAlAs;
    let inp_well = HeterostructureProfile::infinite_square_well(15.0e-9, mat_inp);
    let inp_states = Schrodinger1DSolver::solve_bound_states(&inp_well, 3);
    assert!(inp_states.len() >= 3);
    assert!(inp_states[0].energy_ev > 0.0);
    assert!(inp_states[1].energy_ev > inp_states[0].energy_ev);
}

#[test]
fn test_resonant_lo_phonon_depopulation_scheme() {
    let active_stage = HeterostructureProfile::standard_gaas_thz_active_stage();
    let states = Schrodinger1DSolver::solve_bound_states(&active_stage, 4);

    assert!(states.len() >= 3);

    // Extraction spacing between state 1 (lower laser level) and state 0 (ground level):
    let delta_e21_ev = states[1].energy_ev - states[0].energy_ev;

    // Verify subband spacing closely matches GaAs LO phonon energy (36.25 meV):
    let delta_lo_diff = (delta_e21_ev - GAAS_LO_PHONON_ENERGY_EV).abs();
    assert!(
        delta_lo_diff < 0.008, // within 8 meV of resonant LO phonon
        "Delta E21 ({:.2} meV) deviates from LO phonon ({:.2} meV)",
        delta_e21_ev * 1e3,
        GAAS_LO_PHONON_ENERGY_EV * 1e3
    );

    let lo_depop = ResonantLoPhononDepopulation::standard_gaas(delta_e21_ev);

    // Fast LO-phonon emission extraction lifetime: tau_21 < 1.0 ps (sub-picosecond extraction):
    let tau_21 = lo_depop.extraction_lifetime_tau_21();
    assert!(
        tau_21 < 1.0e-12,
        "Resonant LO phonon depopulation lifetime should be sub-picosecond, got: {:.3e} s",
        tau_21
    );

    // Non-resonant upper state relaxation lifetime tau_32 >> tau_21:
    assert!(
        lo_depop.upper_to_lower_tau_32_s > 4.0 * tau_21,
        "tau_32 should be much longer than tau_21"
    );

    // Extraction efficiency sustaining steady-state population inversion (1 - tau_2 / tau_32 > 0):
    let inv_factor = lo_depop.inversion_sustainability_factor();
    assert!(
        inv_factor > 0.80,
        "Inversion sustainability factor should be > 0.80: got {}",
        inv_factor
    );

    // Steady-state population inversion Delta_n = n3 - n2 > 0:
    let j_inj_a_m2 = 300.0 * 1e4; // 300 A/cm^2 = 3e6 A/m^2
    let eta_inj = 0.85;
    let stage_len_m = 45e-9;
    let delta_n = lo_depop.steady_state_population_inversion(j_inj_a_m2, eta_inj, stage_len_m);
    assert!(
        delta_n > 1.0e20,
        "Population inversion Delta_n should be positive (> 1e20 m^-3): got {}",
        delta_n
    );
}

#[test]
fn test_terahertz_optical_gain_spectrum_across_band() {
    let center_freq = 3.2e12; // 3.2 THz (lambda ~ 93.7 um)
    let dipole_z32 = 3.5e-9; // 3.5 nm transition dipole
    let n_r = 3.60;
    let fwhm = 0.8e12; // 0.8 THz linewidth
    let stage_len = 45.0e-9;

    let gain_model = ThzOpticalGainModel::new(center_freq, dipole_z32, n_r, fwhm, stage_len);

    // Verify center wavelength corresponds to sub-millimeter band (30 to 600 um):
    let lambda_um = gain_model.center_wavelength_m() * 1e6;
    assert!(
        (30.0..=600.0).contains(&lambda_um),
        "Wavelength should be in 30 - 600 um, got: {} um",
        lambda_um
    );

    let delta_n = 1.0e20; // 1e14 cm^-3 inversion density
    let peak_gain_cm = gain_model.peak_gain_per_cm(delta_n);

    assert!(
        peak_gain_cm > 10.0 && peak_gain_cm < 100.0,
        "Peak gain out of physical range: {} cm^-1",
        peak_gain_cm
    );

    // Gain spectrum falloff across 0.5 - 10 THz band:
    let thz_frequencies = [0.5e12, 1.5e12, 3.2e12, 5.0e12, 8.0e12, 10.0e12];
    for &nu in &thz_frequencies {
        let g = gain_model.evaluate_gain_per_cm(nu, delta_n);
        assert!(g >= 0.0);
        if (nu - center_freq).abs() < 1e10 {
            assert!((g - peak_gain_cm).abs() < 1e-4);
        } else {
            assert!(g < peak_gain_cm);
        }
    }

    // Differential gain g_diff = dg / d(Delta_n):
    let g_diff = gain_model.differential_gain_m2();
    assert!(g_diff > 1e-21, "Differential gain: {} m^2", g_diff);
}

#[test]
fn test_metal_metal_and_semi_insulating_surface_plasmon_waveguides() {
    let mm = ThzPolaritonicWaveguide::metal_metal(10e-6, 100e-6, 2.5e-3);
    let sisp = ThzPolaritonicWaveguide::semi_insulating_surface_plasmon(10e-6, 150e-6, 2.5e-3);

    // 1. Metal-Metal (MM) waveguide:
    // Confinement factor Gamma ~ 0.85 - 0.95:
    let gamma_mm = mm.optical_confinement_factor();
    assert!(
        (0.85..=0.95).contains(&gamma_mm),
        "MM confinement factor should be 0.85 - 0.95, got {}",
        gamma_mm
    );

    // High facet reflectivity from strong impedance mismatch (R ~ 0.70 - 0.85):
    assert!(mm.front_reflectivity >= 0.75);

    // Drude loss alpha_w ~ 18 cm^-1:
    let alpha_w_mm_cm = mm.waveguide_loss_alpha_w_m * 0.01;
    assert!((15.0..=25.0).contains(&alpha_w_mm_cm));

    // Intracavity photon lifetime tau_ph ~ 5 - 20 ps:
    let tau_ph_mm = mm.photon_cavity_lifetime_s();
    assert!(
        tau_ph_mm > 3e-12 && tau_ph_mm < 30e-12,
        "Photon lifetime: {:.2e} s",
        tau_ph_mm
    );

    // 2. Semi-Insulating Surface-Plasmon (SI-SP) waveguide:
    // Moderate confinement factor Gamma ~ 0.35 - 0.45:
    let gamma_sisp = sisp.optical_confinement_factor();
    assert!(
        (0.35..=0.45).contains(&gamma_sisp),
        "SI-SP confinement factor should be 0.35 - 0.45, got {}",
        gamma_sisp
    );

    // Fresnel facet reflectivity (R ~ 0.32):
    assert!((sisp.front_reflectivity - 0.32).abs() < 0.02);

    // Total mirror loss is higher in SI-SP than MM:
    assert!(sisp.mirror_loss_alpha_m_per_cm() > mm.mirror_loss_alpha_m_per_cm());

    // 3. Skin depth in gold at 3 THz:
    let skin_depth = ThzPolaritonicWaveguide::drude_skin_depth_m(3.0e12);
    assert!(
        skin_depth > 30e-9 && skin_depth < 80e-9,
        "Drude skin depth: {:.2e} m",
        skin_depth
    );
}
