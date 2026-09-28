//! Integration tests for quantum detection efficiency, dark count rate, timing jitter, and HBT anti-bunching.

use phonon_models::snspd::{NanowireGeometry, QuantumTelemetryModel};
use phonon_solver::snspd::{CoincidenceConfig, CoincidenceSolver};

#[test]
fn test_internal_quantum_efficiency_saturation() {
    let geom = NanowireGeometry::standard_nbn();
    let telemetry = QuantumTelemetryModel::standard_nbn(&geom);

    let i_c = geom.operational_critical_current_ua();

    // At low bias (e.g. 0.40 * I_c), efficiency is near zero
    let eta_low = telemetry.internal_quantum_efficiency(&geom, 0.40 * i_c, 1550.0);
    assert!(
        eta_low < 0.05,
        "Low bias efficiency should be near zero, got {}",
        eta_low
    );

    // At operating bias near 0.95 * I_c, efficiency saturates near unity (> 98%)
    let eta_high = telemetry.internal_quantum_efficiency(&geom, 0.95 * i_c, 1550.0);
    assert!(
        eta_high >= 0.98,
        "Near-critical bias efficiency should exceed 98%, got {}",
        eta_high
    );

    // Shorter wavelength (850 nm) has higher photon energy, so efficiency turns on at lower bias
    let eta_850_mid = telemetry.internal_quantum_efficiency(&geom, 0.60 * i_c, 850.0);
    let eta_1550_mid = telemetry.internal_quantum_efficiency(&geom, 0.60 * i_c, 1550.0);
    assert!(
        eta_850_mid > eta_1550_mid,
        "850 nm should have higher efficiency than 1550 nm at intermediate bias"
    );
}

#[test]
fn test_dark_count_rate_suppression() {
    let geom = NanowireGeometry::standard_nbn();
    let telemetry = QuantumTelemetryModel::standard_nbn(&geom);

    let i_c = geom.operational_critical_current_ua();

    // At operating bias I_b <= 0.90 * I_c and T = 2.0 K, dark count rate should be < 1.0 cps
    let dcr_op = telemetry.dark_count_rate_cps(&geom, 0.88 * i_c);
    assert!(
        dcr_op < 1.0,
        "Dark count rate should be < 1.0 cps at 88% bias, got {} cps",
        dcr_op
    );

    // DCR increases sharply as bias approaches I_c
    let dcr_critical = telemetry.dark_count_rate_cps(&geom, 0.98 * i_c);
    assert!(
        dcr_critical > dcr_op,
        "DCR should increase as bias approaches I_c"
    );
}

#[test]
fn test_timing_jitter_sub_50ps() {
    let geom = NanowireGeometry::standard_nbn();
    let telemetry = QuantumTelemetryModel::standard_nbn(&geom);

    let geom_jitter = telemetry.geometric_jitter_ps(&geom);
    let noise_jitter = telemetry.noise_jitter_ps();
    let lat_jitter = telemetry.latency_dispersion_ps;
    let total_jitter = telemetry.total_timing_jitter_ps(&geom);

    assert!(
        geom_jitter > 0.0 && geom_jitter < 20.0,
        "Geometric jitter should be ~5-15 ps, got {}",
        geom_jitter
    );
    assert!(
        noise_jitter > 0.0 && noise_jitter < 25.0,
        "Noise jitter should be ~10-20 ps, got {}",
        noise_jitter
    );
    assert!(
        lat_jitter > 0.0 && lat_jitter < 15.0,
        "Latency jitter should be ~5-10 ps, got {}",
        lat_jitter
    );

    // Sub-50 ps criterion
    assert!(
        total_jitter < 50.0,
        "Total RMS timing jitter must be < 50 ps, got {} ps",
        total_jitter
    );
    assert!(
        total_jitter > 10.0,
        "Total RMS jitter should be physically > 10 ps, got {} ps",
        total_jitter
    );
}

#[test]
fn test_hbt_single_photon_antibunching() {
    let config = CoincidenceConfig {
        max_delay_ps: 1500.0,
        bin_width_ps: 30.0,
        total_time_ns: 5000.0,
    };
    let solver = CoincidenceSolver::new(config);

    // Simulate anti-bunched single-photon pulses at repetition period T_rep = 20 ns = 20,000 ps
    // When a single photon arrives at the 50:50 beam splitter, it is randomly routed to either Ch1 or Ch2,
    // so no coincidences occur at tau = 0!
    let mut ch1_times = Vec::new();
    let mut ch2_times = Vec::new();

    let num_events = 250usize;
    let t_rep_ps = 20_000.0;

    for i in 0..num_events {
        let base_t = (i as f64 + 1.0) * t_rep_ps;
        // Deterministic pseudo-random split
        let hash = (i as u64).wrapping_mul(0x5851f42d4c957f2d).wrapping_add(1);
        if hash.is_multiple_of(2) {
            ch1_times.push(base_t + 10.0);
        } else {
            ch2_times.push(base_t + 10.0);
        }
    }

    let result = solver.evaluate_coincidences(&ch1_times, &ch2_times);

    // Zero-delay coincidences should be zero or negligible for ideal single-photon source
    assert!(
        result.g2_zero < 0.5,
        "g^(2)(0) must be < 0.5 for single photon anti-bunching, got {}",
        result.g2_zero
    );
    assert!(
        result.is_single_photon,
        "Single photon state should be verified"
    );
    assert!(
        result.antibunching_contrast > 0.5,
        "Anti-bunching contrast should exceed 0.5, got {}",
        result.antibunching_contrast
    );
}
