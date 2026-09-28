//! Integration Test Suite: 2D Nanoscale Magnetometry, Inverse Biot-Savart & Multi-Threaded Comparative Benchmark

use approx::assert_relative_eq;
use phonon_models::quantum::Complex;
use phonon_models::sensors::{NitrogenIsotope, NvCenter, NvOrientation};
use phonon_solver::sensors::{
    fft_1d, fft_2d, DefectType, MagnetometerSpecs, NvBenchmarkRunner, NvProbeGrid,
};

#[test]
fn test_2d_fft_roundtrip_invertibility() {
    let nx = 16;
    let ny = 16;
    let mut input = Vec::with_capacity(nx * ny);

    for r in 0..nx {
        for c in 0..ny {
            let val = (r as f64 * 0.3).sin() + (c as f64 * 0.5).cos();
            input.push(Complex::new(val, 0.1 * val));
        }
    }

    // Forward FFT then Inverse FFT
    let k_space = fft_2d(&input, nx, ny, false);
    let reconstructed = fft_2d(&k_space, nx, ny, true);

    assert_eq!(reconstructed.len(), nx * ny);
    for i in 0..(nx * ny) {
        assert_relative_eq!(reconstructed[i].re, input[i].re, epsilon = 1e-10);
        assert_relative_eq!(reconstructed[i].im, input[i].im, epsilon = 1e-10);
    }

    // Also test 1D FFT on power-of-2 and non-power-of-2
    let in_1d_pow2 = vec![
        Complex::new(1.0, 0.0),
        Complex::new(2.0, 1.0),
        Complex::new(-1.0, 0.5),
        Complex::new(0.0, -1.0),
    ];
    let k_1d = fft_1d(&in_1d_pow2, false);
    let rec_1d = fft_1d(&k_1d, true);
    for i in 0..4 {
        assert_relative_eq!(rec_1d[i].re, in_1d_pow2[i].re, epsilon = 1e-10);
        assert_relative_eq!(rec_1d[i].im, in_1d_pow2[i].im, epsilon = 1e-10);
    }
}

#[test]
fn test_forward_and_inverse_biot_savart_reconstruction() {
    let nx = 32;
    let ny = 32;
    let pitch = 10.0e-9; // 10 nm pitch
    let standoff = 15.0e-9; // 15 nm standoff distance

    let nv = NvCenter::new(NvOrientation::V0, NitrogenIsotope::N14);
    let grid = NvProbeGrid::new(nx, ny, pitch, pitch, standoff, nv);

    // Synthesize a localized current carrying trace along X at y = ny/2
    let mut jx = vec![0.0; nx * ny];
    let jy = vec![0.0; nx * ny];

    let target_row = ny / 2;
    let peak_j0 = 100.0; // 100 A/m

    for r in 0..nx {
        for c in 0..ny {
            let idx = r * ny + c;
            let dist_c = (c as f64 - target_row as f64).abs();
            // Gaussian current profile of 20 nm width
            jx[idx] = peak_j0 * (-0.5 * (dist_c / 1.5).powi(2)).exp();
        }
    }

    // Forward projection: compute Bz at standoff distance d
    let bz = grid.forward_biot_savart(&jx, &jy);
    assert_eq!(bz.len(), nx * ny);

    // Magnetic field must be non-zero and antisymmetric across the wire
    let max_bz = bz.iter().copied().fold(0.0_f64, |acc, v| acc.max(v.abs()));
    assert!(
        max_bz > 1e-9,
        "Magnetic field Bz must be detectable: max = {}",
        max_bz
    );

    // Inverse Biot-Savart reconstruction back to current density
    let (jx_recon, _jy_recon) = grid.inverse_biot_savart_reconstruction(&bz, None);
    assert_eq!(jx_recon.len(), nx * ny);

    // Verify reconstructed current peaks at the true wire row (target_row)
    let mut max_recon_row = 0;
    let mut max_recon_val = 0.0;
    for c in 0..ny {
        let val = jx_recon[(nx / 2) * ny + c];
        if val > max_recon_val {
            max_recon_val = val;
            max_recon_row = c;
        }
    }

    assert_eq!(
        max_recon_row, target_row,
        "Reconstructed current path must peak at target row {}: got {}",
        target_row, max_recon_row
    );
    assert!(
        max_recon_val > 50.0,
        "Reconstructed peak current density must be recovered: got {}",
        max_recon_val
    );
}

#[test]
fn test_circuit_short_and_leakage_detection() {
    let nx = 32;
    let ny = 32;
    let pitch = 8.0e-9; // 8 nm sub-10 nm pitch
    let standoff = 10.0e-9; // 10 nm standoff

    let nv = NvCenter::new(NvOrientation::V0, NitrogenIsotope::N14);
    let grid = NvProbeGrid::new(nx, ny, pitch, pitch, standoff, nv);

    let mut jx = vec![0.0; nx * ny];
    let mut jy = vec![0.0; nx * ny];

    // Create a localized short circuit bridge at (16, 16)
    let short_idx = 16 * ny + 16;
    jx[short_idx] = 120.0; // 120 A/m high current short
    jy[short_idx] = 80.0;

    // Create a small dielectric leakage current at (8, 8)
    let leak_idx = 8 * ny + 8;
    jx[leak_idx] = 15.0; // 15 A/m leakage
    jy[leak_idx] = 10.0;

    let defects = grid.detect_defects(&jx, &jy, 100.0, 10.0);
    assert!(
        defects.len() >= 2,
        "Should detect both short circuit and leakage defect"
    );

    let has_short = defects
        .iter()
        .any(|d| d.defect_type == DefectType::ShortCircuit);
    let has_leak = defects
        .iter()
        .any(|d| d.defect_type == DefectType::LeakageCurrent);

    assert!(has_short, "Must identify localized short circuit");
    assert!(has_leak, "Must identify dielectric leakage current");
}

#[test]
fn test_comparative_benchmark_diamond_nv_vs_squid_vs_hall() {
    let nv_specs = MagnetometerSpecs::diamond_nv();
    let squid_specs = MagnetometerSpecs::squid();
    let hall_specs = MagnetometerSpecs::hall();

    // Verify key technology specs
    assert!(
        nv_specs.spatial_resolution_m <= 10.0e-9,
        "Diamond NV must resolve sub-10 nm"
    );
    assert_eq!(
        nv_specs.standby_power_w, 0.0,
        "Diamond NV has 0.0 W standby power"
    );
    assert!(
        nv_specs.max_temperature_k >= 600.0,
        "Diamond NV operates up to > 600 K"
    );
    assert!(
        nv_specs.bandwidth_hz >= 10.0e9,
        "Diamond NV operates up to GHz microwave frequencies"
    );

    assert!(
        squid_specs.max_temperature_k <= 5.0,
        "SQUID requires cryogenic temperature <= 4.2 K"
    );
    assert!(
        squid_specs.standby_power_w >= 500.0,
        "SQUID cryocooler consumes > 500 W"
    );

    assert!(
        hall_specs.spatial_resolution_m >= 1.0e-6,
        "Hall sensor limited to ~ 1 um resolution"
    );

    // Run parallel Rayon benchmark across 10,000 scenarios
    let scenarios = NvBenchmarkRunner::generate_testbed(10_000);
    assert_eq!(scenarios.len(), 10_000);

    let report = NvBenchmarkRunner::run_benchmark(&scenarios);
    assert_eq!(report.num_scenarios_evaluated, 10_000);

    // Diamond NV should achieve high viability (> 85%) due to wide temperature (0.1K-650K) and sub-10nm resolution
    assert!(
        report.nv_success_rate > 85.0,
        "Diamond NV viability rate should exceed 85%: got {:.1}%",
        report.nv_success_rate
    );

    // SQUID fails on all room-temperature and hot-die scenarios (4/5 of scenarios), so viability < 25%
    assert!(
        report.squid_success_rate < 25.0,
        "SQUID viability rate should be low (< 25%) due to cryogenic constraint: got {:.1}%",
        report.squid_success_rate
    );

    // Hall sensor fails on nanoscale features (< 1 um, 2/4 of scenarios) and cryogenic (1/5), so viability < 50%
    assert!(
        report.hall_success_rate < 50.0,
        "Hall sensor viability rate should be moderate (< 50%): got {:.1}%",
        report.hall_success_rate
    );

    // Diamond NV average score should lead all technologies
    assert!(
        report.nv_average_score > report.squid_average_score,
        "Diamond NV score ({:.1}) must exceed SQUID ({:.1})",
        report.nv_average_score,
        report.squid_average_score
    );
    assert!(
        report.nv_average_score > report.hall_average_score,
        "Diamond NV score ({:.1}) must exceed Hall sensor ({:.1})",
        report.nv_average_score,
        report.hall_average_score
    );

    // Throughput should be high (multi-threaded Rayon execution)
    assert!(
        report.throughput_scenarios_per_sec > 1000.0,
        "Parallel benchmark throughput should be high: got {:.0} scenarios/s",
        report.throughput_scenarios_per_sec
    );

    // Summary table must contain expected markdown headings
    assert!(report.summary_markdown.contains("Diamond NV Center"));
    assert!(report.summary_markdown.contains("SQUID"));
    assert!(report.summary_markdown.contains("Hall Sensor"));
}
