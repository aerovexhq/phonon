#![deny(unsafe_code)]

//! Automated test suite for Latin Hypercube Sampling (LHS), Monte Carlo yield harvesting,
//! distribution generators, statistical capability indices, and high-throughput parallel execution.

use phonon_solver::monte_carlo::{
    compute_empirical_cdf, compute_histogram, compute_metrics, identify_process_corners,
    LatinHypercubeSampler, LhsParameter, MonteCarloHarvester, ParameterDistribution, SamplingMode,
    YieldSpecification,
};
use std::time::Instant;

#[test]
fn test_lhs_stratification_exact_single_sample_per_stratum() {
    let n_samples = 200;
    let parameters = vec![
        LhsParameter::uniform("R1", 1000.0, 10.0), // [900, 1100]
        LhsParameter::uniform("R2", 50.0, 20.0),   // [40, 60]
        LhsParameter::uniform("C1", 100.0, 5.0),   // [95, 105]
        LhsParameter::gaussian("V1", 5.0, 6.0),    // Gaussian mean 5.0, std_dev 0.1
    ];

    let sampler = LatinHypercubeSampler::new(0x1234_5678_9ABC_DEF0)
        .with_mode(SamplingMode::LatinHypercube);
    let samples = sampler.sample(&parameters, n_samples);

    assert_eq!(samples.len(), n_samples);

    for (dim, param) in parameters.iter().enumerate() {
        let mut stratum_counts = vec![0usize; n_samples];

        for row in &samples {
            let val = row[dim];
            // Evaluate CDF to get the theoretical quantile u in [0, 1]
            let u = param.distribution.cdf(val);
            let stratum_idx = ((u * (n_samples as f64)).floor() as usize).min(n_samples - 1);
            stratum_counts[stratum_idx] += 1;
        }

        // Verify Latin Hypercube stratification invariant: exactly 1 sample per stratum
        for (stratum_idx, &count) in stratum_counts.iter().enumerate() {
            assert_eq!(
                count, 1,
                "Dimension {} ({}) stratum {} expected count 1 but got {}",
                dim, param.name, stratum_idx, count
            );
        }
    }
}

#[test]
fn test_distribution_generators_uniform_gaussian_lognormal() {
    // 1. Uniform distribution
    let u_dist = ParameterDistribution::Uniform(10.0, 20.0);
    assert_eq!(u_dist.mean(), 15.0);
    assert!((u_dist.variance() - 100.0 / 12.0).abs() < 1e-12);
    assert!((u_dist.cdf(10.0) - 0.0).abs() < 1e-12);
    assert!((u_dist.cdf(15.0) - 0.5).abs() < 1e-12);
    assert!((u_dist.cdf(20.0) - 1.0).abs() < 1e-12);
    assert!((u_dist.inverse_cdf(0.0) - 10.0).abs() < 1e-6);
    assert!((u_dist.inverse_cdf(0.5) - 15.0).abs() < 1e-6);
    assert!((u_dist.inverse_cdf(1.0) - 20.0).abs() < 1e-6);
    assert!((u_dist.pdf(15.0) - 0.1).abs() < 1e-12);

    // 2. Gaussian distribution
    let g_dist = ParameterDistribution::Gaussian(100.0, 5.0);
    assert_eq!(g_dist.mean(), 100.0);
    assert_eq!(g_dist.variance(), 25.0);
    assert_eq!(g_dist.std_dev(), 5.0);
    assert!((g_dist.cdf(100.0) - 0.5).abs() < 1e-6);
    assert!((g_dist.cdf(105.0) - 0.8413447).abs() < 1e-4);
    assert!((g_dist.cdf(95.0) - 0.1586552).abs() < 1e-4);
    assert!((g_dist.inverse_cdf(0.5) - 100.0).abs() < 1e-6);
    assert!((g_dist.inverse_cdf(0.8413447) - 105.0).abs() < 1e-3);
    assert!((g_dist.inverse_cdf(0.1586552) - 95.0).abs() < 1e-3);

    // 3. LogNormal distribution
    let mu = 1.0f64.ln(); // 0.0
    let sigma = 0.2;
    let ln_dist = ParameterDistribution::LogNormal(mu, sigma);
    assert!((ln_dist.cdf(1.0) - 0.5).abs() < 1e-6);
    assert!((ln_dist.inverse_cdf(0.5) - 1.0).abs() < 1e-6);
    assert!(ln_dist.mean() > 1.0); // e^(0 + 0.04/2) = e^0.02 > 1.0
    assert!(ln_dist.pdf(1.0) > 0.0);
}

#[test]
fn test_high_throughput_parallel_rayon_batch_10000_samples() {
    let parameters = vec![
        LhsParameter::uniform("R1", 1000.0, 5.0),
        LhsParameter::uniform("R2", 2000.0, 5.0),
        LhsParameter::gaussian("C1", 1.0e-6, 10.0),
        LhsParameter::gaussian("C2", 2.2e-6, 10.0),
        LhsParameter::uniform("VIN", 5.0, 2.0),
    ];

    let harvester = MonteCarloHarvester::new(parameters)
        .with_sampling_mode(SamplingMode::LatinHypercube)
        .with_seed(0xABCD_EF01_2345_6789)
        .with_bins(30);

    let spec = YieldSpecification::two_sided("VOUT", 3.2, 3.5);
    let sample_count = 10_000;

    let start = Instant::now();
    let report = harvester.harvest_custom(&spec, sample_count, |p| {
        // Evaluate analog divider companion model with capacitive loading transfer
        let r1 = p[0];
        let r2 = p[1];
        let c1 = p[2];
        let c2 = p[3];
        let vin = p[4];
        let v_divider = vin * (r2 / (r1 + r2));
        let cap_ratio = c1 / (c1 + c2);
        v_divider * (0.95 + 0.1 * cap_ratio)
    });
    let duration = start.elapsed();

    // Verify throughput requirement: 10,000 samples under 200ms
    assert!(
        duration.as_millis() < 200,
        "Parallel batch execution took {:?}, exceeding 200ms threshold",
        duration
    );

    assert_eq!(report.metrics.sample_count, sample_count);
    assert!(report.metrics.mean > 3.0 && report.metrics.mean < 3.6);
    assert!(report.metrics.std_dev > 0.0);
    assert!(report.metrics.yield_percentage > 0.0);
}

#[test]
fn test_statistical_metrics_mean_std_dev_cp_cpk_yield() {
    // Construct synthetic data with known mean 10.0 and known std dev
    let n = 1000;
    let mut values = Vec::with_capacity(n);
    for i in 0..n {
        // Uniform grid roughly in [8.0, 12.0]
        let v = 8.0 + 4.0 * (i as f64) / (n as f64);
        values.push(v);
    }

    let spec = YieldSpecification::two_sided("TEST_METRIC", 8.5, 11.5);
    let metrics = compute_metrics(&values, &spec);

    assert_eq!(metrics.sample_count, n);
    assert!((metrics.mean - 10.0).abs() < 0.05);
    // Theoretical std dev of U(8, 12) is 4 / sqrt(12) = 1.1547
    assert!((metrics.std_dev - 1.1547).abs() < 0.05);
    assert!(metrics.min >= 8.0);
    assert!(metrics.max <= 12.0);
    assert!((metrics.median - 10.0).abs() < 0.05);

    // Yield: range [8.5, 11.5] inside [8.0, 12.0] is 3.0 / 4.0 = 75.0%
    assert!((metrics.yield_percentage - 75.0).abs() < 1.0);

    // Process capability Cp and Cpk
    assert!(metrics.cp.is_some());
    assert!(metrics.cpk.is_some());
    let cp = metrics.cp.unwrap();
    let cpk = metrics.cpk.unwrap();
    // Cp = (11.5 - 8.5) / (6 * 1.155) = 3.0 / 6.93 ~ 0.433
    assert!((cp - 0.433).abs() < 0.05);
    // Symmetric bounds around mean 10.0, so Cpk ~ Cp
    assert!((cpk - cp).abs() < 0.05);
}

#[test]
fn test_histogram_bin_summation_and_cdf_monotonicity() {
    let mut values = Vec::new();
    for i in 0..500 {
        values.push((i as f64).sin() * 10.0 + 50.0);
    }

    let n_bins = 25;
    let bins = compute_histogram(&values, n_bins);
    assert_eq!(bins.len(), n_bins);

    // 1. Verify histogram bin count summation equals total sample count
    let total_bin_count: usize = bins.iter().map(|b| b.count).sum();
    assert_eq!(
        total_bin_count,
        values.len(),
        "Histogram bin counts must sum to sample count"
    );

    // 2. Verify probability density integral sums to approximately 1.0
    let total_prob_mass: f64 = bins
        .iter()
        .map(|b| b.probability_density * (b.bin_max - b.bin_min))
        .sum();
    assert!(
        (total_prob_mass - 1.0).abs() < 1e-4,
        "Total probability mass should integrate to 1.0, got {}",
        total_prob_mass
    );

    // 3. Verify Empirical CDF monotonicity
    let cdf = compute_empirical_cdf(&values);
    assert_eq!(cdf.len(), values.len());

    for window in cdf.windows(2) {
        let prev = &window[0];
        let curr = &window[1];
        assert!(
            curr.value >= prev.value,
            "CDF values must be monotonically non-decreasing: {} < {}",
            curr.value,
            prev.value
        );
        assert!(
            curr.cumulative_probability >= prev.cumulative_probability,
            "CDF cumulative probabilities must be monotonically non-decreasing"
        );
    }

    // 4. Verify Process Corners ordering
    let dummy_spec = YieldSpecification::two_sided("METRIC", 40.0, 60.0);
    let metrics = compute_metrics(&values, &dummy_spec);
    let corners = identify_process_corners(&values, &metrics);

    assert!(corners.minus_3sigma <= corners.nominal);
    assert!(corners.nominal <= corners.plus_3sigma);
    assert!(corners.empirical_minus_3sigma <= corners.empirical_nominal);
    assert!(corners.empirical_nominal <= corners.empirical_plus_3sigma);
}
