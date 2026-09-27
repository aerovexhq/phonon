//! Integration tests for Electrochemical Metallization (ECM) filament redox kinetics and conductance quantization.

use phonon_models::relay::{
    EcmCellModel, EcmCellParameters, EcmConductionState, EcmSwitchingMode, G_0, R_0,
};

#[test]
fn test_butler_volmer_ion_hopping_exponential_field_scaling() {
    let cell = EcmCellModel::new(EcmCellParameters::default());

    // Evaluate ion hopping velocity at low field vs high programming field
    let v_low = 0.10; // 100 mV (sub-threshold)
    let v_high = 1.20; // 1.2 V (strong programming field)

    let vel_low = cell.evaluate_ion_drift_velocity(v_low);
    let vel_high = cell.evaluate_ion_drift_velocity(v_high);

    // Butler-Volmer sinh(q*a*E / 2*kB*T) gives super-linear/exponential velocity surge
    assert!(vel_low > 0.0);
    assert!(
        vel_high > vel_low * 10.0,
        "Expected >10x surge, was {} vs {}",
        vel_high,
        vel_low
    );
}

#[test]
fn test_ecm_filament_growth_and_conductance_quantization() {
    let mut cell = EcmCellModel::new(EcmCellParameters::default());

    // Pristine state is HighResistanceOff
    assert_eq!(
        cell.conduction_state(),
        EcmConductionState::HighResistanceOff
    );
    assert!(cell.resistance_ohm() > 1.0e10);

    // Apply positive programming pulse to migrate cations and nucleate bridge
    for _ in 0..100 {
        cell.step(0.60, 2.0e-6);
    }

    // Must bridge the solid electrolyte gap
    assert!(cell.state.filament_length_ratio >= 0.999);
    assert!(cell.state.atomic_channels >= 1);

    // Check conductance quantization scale: G_0 ~ 77.48 uS (R_0 ~ 12.9 kOhm)
    let g = cell.conductance_s();
    assert!(
        g >= 0.5 * G_0,
        "Conductance was {} S, expected >= 0.5 * G_0",
        g
    );

    let r = cell.resistance_ohm();
    assert!(r < R_0 * 2.0);
}

#[test]
fn test_ecm_negative_bias_reset_dissolution() {
    let mut cell = EcmCellModel::new(EcmCellParameters::default());

    // 1. Program to low-resistance state (SET)
    for _ in 0..100 {
        cell.step(0.60, 2.0e-6);
    }
    assert!(cell.resistance_ohm() < 1000.0);

    // 2. Apply negative erase pulse to dissolve filament (RESET)
    for _ in 0..200 {
        cell.step(-0.50, 2.0e-6); // -500 mV reset pulse
    }

    // Filament must break, returning cell to high-resistance off-state
    assert!(cell.state.filament_length_ratio < 0.99);
    assert!(cell.state.atomic_channels == 0);
    assert!(
        cell.resistance_ohm() > 1.0e6,
        "Resistance after RESET was {} Ohm",
        cell.resistance_ohm()
    );
}

#[test]
fn test_ecm_volatile_threshold_vs_non_volatile_memory() {
    // Volatile threshold switch: dissolves spontaneously at zero bias
    let volatile_params = EcmCellParameters {
        mode: EcmSwitchingMode::VolatileThreshold,
        critical_radius_m: 5.0e-9, // High surface tension threshold
        ..Default::default()
    };

    let mut volatile_cell = EcmCellModel::new(volatile_params);
    // Program briefly
    for _ in 0..80 {
        volatile_cell.step(0.60, 2.0e-6);
    }

    // Now hold at 0V: spontaneous surface tension drives dissolution
    for _ in 0..500 {
        volatile_cell.step(0.0, 5.0e-6);
    }
    assert!(
        volatile_cell.resistance_ohm() > 1.0e8,
        "Volatile cell should have dissolved at 0V, but R was {} Ohm",
        volatile_cell.resistance_ohm()
    );

    // Non-volatile cell: stable retention at 0V
    let nv_params = EcmCellParameters {
        mode: EcmSwitchingMode::NonVolatileMemory,
        ..Default::default()
    };
    let mut nv_cell = EcmCellModel::new(nv_params);
    for _ in 0..100 {
        nv_cell.step(0.60, 2.0e-6);
    }
    let r_set = nv_cell.resistance_ohm();
    for _ in 0..500 {
        nv_cell.step(0.0, 5.0e-6);
    }
    let r_retained = nv_cell.resistance_ohm();
    assert!(
        (r_retained - r_set).abs() < 50.0,
        "Non-volatile cell should retain state"
    );
}
