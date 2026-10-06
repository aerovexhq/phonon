#![deny(unsafe_code)]

//! Automated Test Suite for Phase 376:
//! Directional Cosmic Heavy Ion Radiation Track & 3D Anisotropic Shielding Co-Simulator.

use phonon_solver::directional_radiation::{
    DirectionalRadiationCoSimulator, HeavyIonSpecies, IncidentTrajectory,
    IonTrackProfile, MultiDieMbuEngine, SpacecraftShieldingModel,
};
use std::f64::consts::PI;
use std::time::Instant;

#[test]
fn test_heavy_ion_species_and_bragg_peak_let() {
    let proton = HeavyIonSpecies::Proton;
    let iron = HeavyIonSpecies::IronFe56;

    assert_eq!(proton.atomic_number(), 1);
    assert_eq!(proton.mass_number(), 1);
    assert_eq!(iron.atomic_number(), 26);
    assert_eq!(iron.mass_number(), 56);

    // Heavy iron stopping power must be dramatically higher than proton
    assert!(iron.peak_stopping_power_let() > proton.peak_stopping_power_let() * 20.0);
    assert!(iron.peak_stopping_power_let() > 70.0);

    let traj_normal = IncidentTrajectory {
        theta_rad: 0.0,
        phi_rad: 0.0,
        energy_mev_per_nuc: 150.0,
        species: HeavyIonSpecies::IronFe56,
    };
    assert!((traj_normal.path_elongation_factor() - 1.0).abs() < 1e-10);

    let traj_oblique = IncidentTrajectory {
        theta_rad: 60.0 * PI / 180.0, // 60 degrees -> sec(60 deg) = 2.0
        phi_rad: 0.0,
        energy_mev_per_nuc: 150.0,
        species: HeavyIonSpecies::IronFe56,
    };
    assert!((traj_oblique.path_elongation_factor() - 2.0).abs() < 1e-3);
}

#[test]
fn test_charge_deposition_and_radial_carrier_profile() {
    let profile = IonTrackProfile::default();
    let traj = IncidentTrajectory {
        theta_rad: 30.0 * PI / 180.0,
        phi_rad: 45.0 * PI / 180.0,
        energy_mev_per_nuc: 200.0,
        species: HeavyIonSpecies::Silicon,
    };

    let let_val = profile.let_at_depth_um(&traj, 10.0);
    assert!(let_val > 10.0, "LET must be positive and substantial for Silicon ion");

    let dq_dz = profile.charge_density_fc_per_um(&traj, 10.0);
    assert!(dq_dz > 100.0, "Linear charge deposition rate must exceed 100 fC/um for Silicon");

    // Total charge integral over 15 um silicon layer
    let total_q = profile.total_deposited_charge_fc(&traj, 0.0, 15.0, 20);
    assert!(total_q > 500.0, "Total deposited charge must exceed 500 fC");

    // Radial carrier concentration profile
    let n_center = profile.radial_carrier_density_cm3(&traj, 0.0, 10.0);
    let n_outer = profile.radial_carrier_density_cm3(&traj, 150.0, 10.0);
    assert!(n_center > n_outer * 5.0, "Radial profile must decay exponentially away from track core");
}

#[test]
fn test_anisotropic_shielding_ray_tracing_attenuation() {
    let model = SpacecraftShieldingModel::default();

    // Direction through hull only
    let areal_hull = model.effective_areal_mass_g_cm2(0.0, 0.0);
    assert!(areal_hull > 0.5, "Hull areal mass must be positive");

    // Direction through fuel tank shadow (theta = 0.6*pi, phi = pi)
    let areal_tank = model.effective_areal_mass_g_cm2(PI * 0.6, PI);
    assert!(
        areal_tank > areal_hull * 3.0,
        "Propellant tank shadow must provide substantial extra areal mass shielding"
    );

    // Directional attenuation factor
    let atten_tank = model.directional_attenuation_factor(PI * 0.6, PI);
    let atten_hull = model.directional_attenuation_factor(0.0, 0.0);
    assert!(atten_tank < atten_hull, "Tank shadow must attenuate more flux than bare hull");

    // Mission TID
    let tid_tank = model.localized_mission_tid_krad(PI * 0.6, PI);
    let tid_hull = model.localized_mission_tid_krad(0.0, 0.0);
    assert!(tid_tank < tid_hull);

    // Polar sweep points
    let sweep = model.compute_polar_azimuth_sweep(PI * 0.6, 24);
    assert_eq!(sweep.len(), 24);

    // Tradeoff curves
    let (al_curve, gz_curve) = model.compute_tid_thickness_tradeoff(20.0, 16);
    assert_eq!(al_curve.len(), 16);
    assert_eq!(gz_curve.len(), 16);
}

#[test]
fn test_multi_die_oblique_ray_intersection_and_mbu() {
    let mbu_engine = MultiDieMbuEngine::default();
    let traj = IncidentTrajectory {
        theta_rad: 25.0 * PI / 180.0,
        phi_rad: 30.0 * PI / 180.0,
        energy_mev_per_nuc: 250.0,
        species: HeavyIonSpecies::IronFe56,
    };

    // Ray passing from above (z=150) through HBM memory (x=1500, y=0, z=120) and down to interposer (z=10)
    let ray_origin = [1500.0, 0.0, 150.0];
    let hits = mbu_engine.trace_incident_ray(&traj, ray_origin);

    assert!(!hits.is_empty(), "Ray must hit at least one die in the stack");
    let hbm_hit = hits.iter().find(|h| h.die_id == 2);
    assert!(hbm_hit.is_some(), "Ray must hit HBM memory die");

    if let Some(h) = hbm_hit {
        assert!(h.track_length_um > 10.0);
        assert!(h.deposited_charge_fc > 50.0);
        assert!(h.upset_cell_count >= 1, "Dense HBM die must suffer bit flips");
        assert!(h.set_pulse_duration_ps > 5.0);
    }
}

#[test]
fn test_directional_radiation_co_simulator_recompute() {
    let mut sim = DirectionalRadiationCoSimulator::new(
        IncidentTrajectory::default(),
        SpacecraftShieldingModel::default(),
        MultiDieMbuEngine::default(),
    );

    assert!(sim.latest_telemetry.peak_let_mev_cm2_mg > 50.0);
    assert!(sim.latest_telemetry.localized_mission_tid_krad > 0.0);
    assert!(sim.latest_telemetry.shielding_attenuation_percent > 0.0);

    // Recompute with adjusted origin
    sim.trajectory.species = HeavyIonSpecies::Alpha;
    sim.recompute([0.0, 0.0, 150.0]);
    assert!(sim.latest_telemetry.peak_let_mev_cm2_mg < 20.0, "Alpha LET must be significantly lower than Fe-56");
}

#[test]
fn test_fast_initialization_cold_boot_latency() {
    let start = Instant::now();
    let sim = DirectionalRadiationCoSimulator::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "DirectionalRadiationCoSimulator::new_fast must execute in < 5ms, took {:?}",
        elapsed
    );
    assert_eq!(sim.latest_telemetry.peak_let_mev_cm2_mg, 78.4);
    assert_eq!(sim.latest_telemetry.total_mbu_flipped_cells, 6);
    assert_eq!(sim.latest_telemetry.pierced_die_count, 2);
}
