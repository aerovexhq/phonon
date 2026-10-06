#![deny(unsafe_code)]

//! Test suite for Phase 378: Aerospace Thermal-Vacuum Radiation Dissipation & Orbital Cycling Co-Simulator.

use phonon_solver::thermal_vacuum::{
    CarrierFreezeoutModel, CryogenicDopantKind, CryogenicKinkModel, MicroBumpGeometry,
    OrbitalCyclingSimulator, OrbitalMissionKind, SolderAlloyKind, SubthresholdSteepeningModel,
    SurfaceCoatingKind, ThermalVacuumCoSimulator, VacuumRadiationModel,
};
use std::time::Instant;

#[test]
fn test_stefan_boltzmann_radiation_and_coating_emissivity() {
    let gold_rad = VacuumRadiationModel::new(SurfaceCoatingKind::PolishedGold, 0.05, 2.725);
    let black_rad = VacuumRadiationModel::new(SurfaceCoatingKind::BlackAnodize, 0.05, 2.725);

    assert_eq!(gold_rad.coating.emissivity(), 0.04);
    assert_eq!(black_rad.coating.emissivity(), 0.96);

    // Radiative flux at 300K in darkness (solar flux = 0)
    let mut dark_gold = gold_rad.clone();
    dark_gold.solar_flux_w_m2 = 0.0;
    let mut dark_black = black_rad.clone();
    dark_black.solar_flux_w_m2 = 0.0;

    let flux_gold = dark_gold.net_heat_rejection_flux_w_m2(300.0);
    let flux_black = dark_black.net_heat_rejection_flux_w_m2(300.0);

    assert!(flux_gold > 0.0, "Gold must reject heat to 2.7K sink");
    assert!(flux_black > flux_gold * 20.0, "Black anodize must reject >20x more heat than gold");

    // Stefan-Boltzmann T^4 scaling: doubling temperature increases emission by ~16x
    let flux_black_300k = dark_black.net_heat_rejection_flux_w_m2(300.0);
    let flux_black_600k = dark_black.net_heat_rejection_flux_w_m2(600.0);
    let ratio = flux_black_600k / flux_black_300k;
    assert!(
        (ratio - 16.0).abs() < 1.0,
        "T^4 Stefan-Boltzmann emission scaling expected ~16x, got {}",
        ratio
    );
}

#[test]
fn test_equilibrium_temperature_and_transient_cooldown() {
    let white_rad = VacuumRadiationModel::new(SurfaceCoatingKind::WhiteThermalPaint, 0.1, 2.725);
    let t_eq_k = white_rad.equilibrium_temperature_k();
    let t_eq_c = white_rad.equilibrium_temperature_c();

    assert!(t_eq_k > 200.0 && t_eq_k < 400.0, "Equilibrium temp must be reasonable in 1 AU sunlight");
    assert!((t_eq_c - (t_eq_k - 273.15)).abs() < 1e-4);

    // Transient cooldown in eclipse
    let mut eclipse_rad = white_rad.clone();
    eclipse_rad.solar_flux_w_m2 = 0.0;
    eclipse_rad.albedo_flux_w_m2 = 0.0;

    let cooldown = eclipse_rad.simulate_vacuum_cooldown(350.0, 100.0, 0.0, 3600.0, 100);
    assert_eq!(cooldown.len(), 100);
    assert!(cooldown[0].1 > cooldown[99].1, "Surface must cool down over time in vacuum");
    assert!(cooldown[99].1 > -273.15, "Cannot cool below absolute zero");
}

#[test]
fn test_orbital_thermal_cycling_and_coffin_manson_fatigue() {
    let mut sim = OrbitalCyclingSimulator::default();
    sim.mission = OrbitalMissionKind::LowEarthOrbit;

    let profile = sim.simulate_orbital_profile(100);
    assert_eq!(profile.len(), 100);

    let t_max = sim.peak_orbit_temperature_c();
    let t_min = sim.minimum_orbit_temperature_c();
    let delta_t = sim.orbital_temperature_swing_k();

    assert!(t_max > t_min, "Peak orbit temp must exceed minimum eclipse temp");
    assert!((delta_t - (t_max - t_min)).abs() < 1e-4);

    // Coffin-Manson low-cycle fatigue
    let nf = sim.projected_fatigue_cycles();
    assert!(nf > 1_000.0, "Micro-bump should survive thousands of orbital thermal cycles: {}", nf);

    let lifetime_years = sim.projected_lifetime_years();
    assert!(lifetime_years > 1.0, "Projected lifetime should exceed 1 year: {}", lifetime_years);

    // Indium micro-bump comparison
    let mut indium_bump = MicroBumpGeometry::default();
    indium_bump.alloy = SolderAlloyKind::PureIndium;
    let indium_nf = indium_bump.cycles_to_failure_coffin_manson(delta_t);
    assert!(indium_nf > 0.0);
}

#[test]
fn test_cryogenic_carrier_freezeout() {
    let freezeout = CarrierFreezeoutModel {
        dopant: CryogenicDopantKind::PhosphorusInSilicon,
        nominal_doping_cm3: 1.0e17,
        effective_dos_300k_cm3: 2.8e19,
        compensation_doping_cm3: 1.0e15,
    };

    let eta_300k = freezeout.ionized_carrier_fraction(300.0);
    let eta_77k = freezeout.ionized_carrier_fraction(77.0);
    let eta_4k = freezeout.ionized_carrier_fraction(4.2);

    assert!(eta_300k > 0.90, "At 300K dopants should be almost fully ionized, got {}", eta_300k);
    assert!(eta_77k < eta_300k, "At 77K partial freeze-out must occur, got {}", eta_77k);
    assert!(eta_4k < 0.001, "At 4.2K severe freeze-out must occur (<0.1%), got {}", eta_4k);

    let sweep = freezeout.simulate_freezeout_curve(4.2, 50);
    assert_eq!(sweep.len(), 50);
    assert!(sweep[0].1 <= sweep[49].1, "Carrier ionization fraction must monotonically increase with T");
}

#[test]
fn test_subthreshold_slope_steepening_and_cryogenic_kink() {
    let ss_model = SubthresholdSteepeningModel::default();

    let s_300k = ss_model.subthreshold_swing_mv_per_dec(300.0);
    let s_77k = ss_model.subthreshold_swing_mv_per_dec(77.0);
    let s_4k = ss_model.subthreshold_swing_mv_per_dec(4.2);

    assert!(s_300k > 60.0, "Room temperature subthreshold swing should exceed 60 mV/dec, got {}", s_300k);
    assert!(s_77k < 20.0, "Liquid nitrogen (77K) subthreshold swing must be < 20 mV/dec, got {}", s_77k);
    assert!(s_4k < 5.0, "Liquid helium (4.2K) subthreshold swing must be < 5 mV/dec, got {}", s_4k);

    // Cryogenic kink model
    let kink = CryogenicKinkModel {
        temp_k: 4.2,
        threshold_voltage_v: 0.65,
        beta_ma_per_v2: 2.4,
        kink_onset_voltage_v: 1.0,
        kink_intensity_factor: 0.35,
    };

    let iv_curve = kink.simulate_id_vds_curve(1.2, 2.5, 60);
    assert_eq!(iv_curve.len(), 60);

    // Verify upward kink: slope dI_d/dV_ds above V_kink (1.0V) should be greater than just below
    let id_0_8v = kink.drain_current_ma(1.2, 0.8);
    let id_1_0v = kink.drain_current_ma(1.2, 1.0);
    let id_1_8v = kink.drain_current_ma(1.2, 1.8);

    let slope_before_kink = (id_1_0v - id_0_8v) / 0.2;
    let slope_after_kink = (id_1_8v - id_1_0v) / 0.8;

    assert!(
        slope_after_kink > slope_before_kink,
        "Cryogenic impact-ionization kink must increase slope dId/dVds (before: {}, after: {})",
        slope_before_kink,
        slope_after_kink
    );
}

#[test]
fn test_thermal_vacuum_co_simulator_cold_boot_and_telemetry() {
    let start = Instant::now();
    let mut co_sim = ThermalVacuumCoSimulator::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "ThermalVacuumCoSimulator::new_fast() must complete in sub-5ms (took {:?})",
        elapsed
    );

    let report = co_sim.report();
    assert!(report.orbital_temp_swing_k > 0.0);
    assert!(report.subthreshold_swing_77k_mv_per_dec < 20.0);
    assert!(report.subthreshold_swing_4k_mv_per_dec < 5.0);

    let updated = co_sim.recompute();
    assert!(updated.projected_cycles_to_failure > 0.0);
    assert!(updated.carrier_ionization_77k_pct < 100.0);
}
