#![deny(unsafe_code)]

//! Monte Carlo and Latin Hypercube yield harvester, statistical metrics analyzer, and process corner extractor.
//!
//! Provides distributed multi-threaded parameter sweeps over MNA companion models and circuit netlists
//! powered by Rayon, calculating sample statistics, process capability (Cp/Cpk), histogram densities,
//! and empirical CDF corner points.

use crate::error::SolverError;
use crate::mna::non_linear_solver::ModelContext;
use crate::monte_carlo::lhs::{LatinHypercubeSampler, LhsParameter, SamplingMode};
use crate::sensitivity::ObjectiveKind;
use crate::transient::{solve_transient, TransientOptions, TransientSolution};
use phonon_core::CircuitGraph;
use rayon::prelude::*;

/// Performance specification limits defining circuit pass/fail criteria.
#[derive(Debug, Clone, PartialEq)]
pub struct YieldSpecification {
    /// Designated metric identifier (e.g. "Terminal Voltage V(OUT)", "Bandwidth", "Delay").
    pub metric_name: String,
    /// Lower specification limit (LSL). Values below this fail the spec.
    pub lsl: Option<f64>,
    /// Upper specification limit (USL). Values above this fail the spec.
    pub usl: Option<f64>,
}

impl YieldSpecification {
    /// Constructs a new specification with optional LSL and USL bounds.
    pub fn new(metric_name: impl Into<String>, lsl: Option<f64>, usl: Option<f64>) -> Self {
        Self {
            metric_name: metric_name.into(),
            lsl,
            usl,
        }
    }

    /// Constructs a two-sided specification limit [LSL, USL].
    pub fn two_sided(metric_name: impl Into<String>, lsl: f64, usl: f64) -> Self {
        Self {
            metric_name: metric_name.into(),
            lsl: Some(lsl),
            usl: Some(usl),
        }
    }

    /// Constructs a lower-bounded specification limit [LSL, infinity).
    pub fn lower_limit(metric_name: impl Into<String>, lsl: f64) -> Self {
        Self {
            metric_name: metric_name.into(),
            lsl: Some(lsl),
            usl: None,
        }
    }

    /// Constructs an upper-bounded specification limit (-infinity, USL].
    pub fn upper_limit(metric_name: impl Into<String>, usl: f64) -> Self {
        Self {
            metric_name: metric_name.into(),
            lsl: None,
            usl: Some(usl),
        }
    }

    /// Evaluates whether a scalar performance value satisfies active specification limits.
    pub fn meets_spec(&self, value: f64) -> bool {
        if value.is_nan() {
            return false;
        }
        if let Some(lsl) = self.lsl {
            if value < lsl {
                return false;
            }
        }
        if let Some(usl) = self.usl {
            if value > usl {
                return false;
            }
        }
        true
    }
}

/// Comprehensive statistical summary for an evaluated performance metric.
#[derive(Debug, Clone, PartialEq)]
pub struct HarvestMetrics {
    /// Total number of evaluated sample iterations.
    pub sample_count: usize,
    /// Arithmetic sample mean.
    pub mean: f64,
    /// Sample standard deviation.
    pub std_dev: f64,
    /// Sample variance.
    pub variance: f64,
    /// Minimum observed metric value.
    pub min: f64,
    /// Maximum observed metric value.
    pub max: f64,
    /// Median value (50th percentile).
    pub median: f64,
    /// Third standardized moment measuring distribution asymmetry.
    pub skewness: f64,
    /// Fourth standardized moment (excess kurtosis, 0 for standard normal).
    pub kurtosis: f64,
    /// Percentage of samples meeting the specification limits [0.0%, 100.0%].
    pub yield_percentage: f64,
    /// Potential process capability index Cp = (USL - LSL) / (6 * sigma).
    pub cp: Option<f64>,
    /// Actual process capability index Cpk = min(CPU, CPL).
    pub cpk: Option<f64>,
}

/// Individual histogram bin containing sample count and probability density.
#[derive(Debug, Clone, PartialEq)]
pub struct HistogramBin {
    /// Lower boundary of the bin interval (inclusive).
    pub bin_min: f64,
    /// Upper boundary of the bin interval (exclusive, except for last bin).
    pub bin_max: f64,
    /// Number of sample points falling within this bin interval.
    pub count: usize,
    /// Normalized probability density: count / (N * bin_width).
    pub probability_density: f64,
}

/// Key process corner boundaries identified from distribution statistics and empirical percentiles.
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessCorners {
    /// Theoretical -3-sigma process corner (mean - 3 * std_dev).
    pub minus_3sigma: f64,
    /// Nominal center point (sample mean).
    pub nominal: f64,
    /// Theoretical +3-sigma process corner (mean + 3 * std_dev).
    pub plus_3sigma: f64,
    /// Empirical 0.135% quantile corresponding to -3-sigma.
    pub empirical_minus_3sigma: f64,
    /// Empirical median corresponding to nominal.
    pub empirical_nominal: f64,
    /// Empirical 99.865% quantile corresponding to +3-sigma.
    pub empirical_plus_3sigma: f64,
}

/// Discrete point in an empirical cumulative distribution function (ECDF).
#[derive(Debug, Clone, PartialEq)]
pub struct EmpiricalCdfPoint {
    /// Scalar metric value.
    pub value: f64,
    /// Empirical cumulative probability P(X <= value) in (0.0, 1.0].
    pub cumulative_probability: f64,
}

/// Complete report returned by a Monte Carlo or Latin Hypercube parameter sweep.
#[derive(Debug, Clone, PartialEq)]
pub struct HarvestReport {
    /// Performance specification applied to compute yield.
    pub spec: YieldSpecification,
    /// Statistical moments and capability indices.
    pub metrics: HarvestMetrics,
    /// Histogram bins covering the sample range.
    pub histogram: Vec<HistogramBin>,
    /// Theoretical and empirical process corner points.
    pub corners: ProcessCorners,
    /// Empirical CDF points.
    pub empirical_cdf: Vec<EmpiricalCdfPoint>,
    /// Evaluated scalar performance metric values for each sample.
    pub sample_values: Vec<f64>,
    /// Parameter names corresponding to columns in `samples`.
    pub parameter_names: Vec<String>,
    /// Multi-dimensional sampled parameter matrix [sample_idx][param_idx].
    pub samples: Vec<Vec<f64>>,
    /// Number of passing samples.
    pub passed_count: usize,
    /// Number of failing samples.
    pub failed_count: usize,
}

/// Calculates statistical moments, capability indices, and yield for sample values.
pub fn compute_metrics(values: &[f64], spec: &YieldSpecification) -> HarvestMetrics {
    let n = values.len();
    if n == 0 {
        return HarvestMetrics {
            sample_count: 0,
            mean: 0.0,
            std_dev: 0.0,
            variance: 0.0,
            min: 0.0,
            max: 0.0,
            median: 0.0,
            skewness: 0.0,
            kurtosis: 0.0,
            yield_percentage: 0.0,
            cp: None,
            cpk: None,
        };
    }

    let mut sum = 0.0;
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    let mut passed = 0;

    for &v in values {
        sum += v;
        if v < min {
            min = v;
        }
        if v > max {
            max = v;
        }
        if spec.meets_spec(v) {
            passed += 1;
        }
    }

    let mean = sum / (n as f64);

    let mut var_sum = 0.0;
    let mut skew_sum = 0.0;
    let mut kurt_sum = 0.0;

    for &v in values {
        let diff = v - mean;
        let diff2 = diff * diff;
        var_sum += diff2;
        skew_sum += diff2 * diff;
        kurt_sum += diff2 * diff2;
    }

    let variance = if n > 1 {
        var_sum / ((n - 1) as f64)
    } else {
        0.0
    };
    let std_dev = variance.sqrt();

    let skewness = if std_dev > 1e-15 {
        (skew_sum / (n as f64)) / (std_dev * std_dev * std_dev)
    } else {
        0.0
    };

    let kurtosis = if std_dev > 1e-15 {
        ((kurt_sum / (n as f64)) / (std_dev * std_dev * std_dev * std_dev)) - 3.0
    } else {
        0.0
    };

    let yield_percentage = (passed as f64 / n as f64) * 100.0;

    // Median
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = if n % 2 == 1 {
        sorted[n / 2]
    } else {
        0.5 * (sorted[n / 2 - 1] + sorted[n / 2])
    };

    // Cp and Cpk capability indices
    let (cp, cpk) = match (spec.lsl, spec.usl) {
        (Some(lsl), Some(usl)) => {
            if std_dev > 1e-15 {
                let cp_val = (usl - lsl) / (6.0 * std_dev);
                let cpl = (mean - lsl) / (3.0 * std_dev);
                let cpu = (usl - mean) / (3.0 * std_dev);
                let cpk_val = cpl.min(cpu);
                (Some(cp_val), Some(cpk_val))
            } else {
                (None, None)
            }
        }
        (Some(lsl), None) => {
            if std_dev > 1e-15 {
                let cpl = (mean - lsl) / (3.0 * std_dev);
                (None, Some(cpl))
            } else {
                (None, None)
            }
        }
        (None, Some(usl)) => {
            if std_dev > 1e-15 {
                let cpu = (usl - mean) / (3.0 * std_dev);
                (None, Some(cpu))
            } else {
                (None, None)
            }
        }
        (None, None) => (None, None),
    };

    HarvestMetrics {
        sample_count: n,
        mean,
        std_dev,
        variance,
        min,
        max,
        median,
        skewness,
        kurtosis,
        yield_percentage,
        cp,
        cpk,
    }
}

/// Generates histogram bins and probability densities for sample values.
pub fn compute_histogram(values: &[f64], n_bins: usize) -> Vec<HistogramBin> {
    let n = values.len();
    if n == 0 || n_bins == 0 {
        return Vec::new();
    }

    let mut min = values[0];
    let mut max = values[0];
    for &v in values {
        if v < min {
            min = v;
        }
        if v > max {
            max = v;
        }
    }

    let (range_min, range_max) = if (max - min).abs() < 1e-12 {
        (min - 0.5, max + 0.5)
    } else {
        (min, max)
    };

    let bin_width = (range_max - range_min) / (n_bins as f64);
    let mut counts = vec![0usize; n_bins];

    for &v in values {
        let idx = ((v - range_min) / bin_width).floor() as isize;
        let bin_idx = if idx < 0 {
            0
        } else if (idx as usize) >= n_bins {
            n_bins - 1
        } else {
            idx as usize
        };
        counts[bin_idx] += 1;
    }

    let mut bins = Vec::with_capacity(n_bins);
    for i in 0..n_bins {
        let b_min = range_min + (i as f64) * bin_width;
        let b_max = range_min + ((i + 1) as f64) * bin_width;
        let count = counts[i];
        let probability_density = if bin_width > 0.0 && n > 0 {
            (count as f64) / ((n as f64) * bin_width)
        } else {
            0.0
        };
        bins.push(HistogramBin {
            bin_min: b_min,
            bin_max: b_max,
            count,
            probability_density,
        });
    }

    bins
}

/// Computes the empirical cumulative distribution function points from sample values.
pub fn compute_empirical_cdf(values: &[f64]) -> Vec<EmpiricalCdfPoint> {
    if values.is_empty() {
        return Vec::new();
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let n = sorted.len();
    let mut cdf = Vec::with_capacity(n);
    for (i, &val) in sorted.iter().enumerate() {
        let p = ((i + 1) as f64) / (n as f64);
        cdf.push(EmpiricalCdfPoint {
            value: val,
            cumulative_probability: p,
        });
    }

    cdf
}

/// Identifies theoretical and empirical process corner points (-3sigma, nominal, +3sigma).
pub fn identify_process_corners(values: &[f64], metrics: &HarvestMetrics) -> ProcessCorners {
    if values.is_empty() {
        return ProcessCorners {
            minus_3sigma: 0.0,
            nominal: 0.0,
            plus_3sigma: 0.0,
            empirical_minus_3sigma: 0.0,
            empirical_nominal: 0.0,
            empirical_plus_3sigma: 0.0,
        };
    }

    let minus_3sigma = metrics.mean - 3.0 * metrics.std_dev;
    let nominal = metrics.mean;
    let plus_3sigma = metrics.mean + 3.0 * metrics.std_dev;

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = sorted.len();

    let idx_minus_3s = (((n as f64) * 0.00135).floor() as usize).min(n - 1);
    let idx_nominal = n / 2;
    let idx_plus_3s = (((n as f64) * 0.99865).floor() as usize).min(n - 1);

    ProcessCorners {
        minus_3sigma,
        nominal,
        plus_3sigma,
        empirical_minus_3sigma: sorted[idx_minus_3s],
        empirical_nominal: sorted[idx_nominal],
        empirical_plus_3sigma: sorted[idx_plus_3s],
    }
}

/// Evaluates a circuit objective functional from a transient simulation solution.
pub fn evaluate_circuit_objective(
    solution: &TransientSolution,
    objective: &ObjectiveKind,
    _graph: &CircuitGraph,
) -> f64 {
    if solution.steps.is_empty() {
        return 0.0;
    }

    match objective {
        ObjectiveKind::TerminalVoltage { node } => {
            let last = &solution.steps[solution.steps.len() - 1];
            if *node == 0 {
                0.0
            } else if *node < last.voltages.len() {
                last.voltages[*node]
            } else {
                0.0
            }
        }
        ObjectiveKind::IntegralSquaredError { node, target_v } => {
            let mut integral = 0.0;
            for i in 0..solution.steps.len() - 1 {
                let s0 = &solution.steps[i];
                let s1 = &solution.steps[i + 1];
                let dt = s1.time - s0.time;
                let v0 = if *node < s0.voltages.len() {
                    s0.voltages[*node]
                } else {
                    0.0
                };
                let v1 = if *node < s1.voltages.len() {
                    s1.voltages[*node]
                } else {
                    0.0
                };
                let err0 = (v0 - target_v).powi(2);
                let err1 = (v1 - target_v).powi(2);
                integral += 0.5 * dt * (err0 + err1);
            }
            integral
        }
        ObjectiveKind::EnergyDissipated => {
            let mut total_energy = 0.0;
            for i in 0..solution.steps.len() - 1 {
                let s0 = &solution.steps[i];
                let s1 = &solution.steps[i + 1];
                let dt = s1.time - s0.time;
                let mut p0 = 0.0;
                let mut p1 = 0.0;
                for (v0, v1) in s0.voltages.iter().zip(s1.voltages.iter()) {
                    p0 += v0 * v0;
                    p1 += v1 * v1;
                }
                total_energy += 0.5 * dt * (p0 + p1);
            }
            total_energy
        }
        ObjectiveKind::PeakOvershoot { node } => {
            let mut max_v = f64::NEG_INFINITY;
            for step in &solution.steps {
                let v = if *node < step.voltages.len() {
                    step.voltages[*node]
                } else {
                    0.0
                };
                if v > max_v {
                    max_v = v;
                }
            }
            if max_v.is_infinite() {
                0.0
            } else {
                max_v
            }
        }
        ObjectiveKind::DelayToThreshold { node, threshold } => {
            for i in 0..solution.steps.len() - 1 {
                let s0 = &solution.steps[i];
                let s1 = &solution.steps[i + 1];
                let v0 = if *node < s0.voltages.len() {
                    s0.voltages[*node]
                } else {
                    0.0
                };
                let v1 = if *node < s1.voltages.len() {
                    s1.voltages[*node]
                } else {
                    0.0
                };
                if (v0 <= *threshold && v1 >= *threshold) || (v0 >= *threshold && v1 <= *threshold)
                {
                    let fraction = if (v1 - v0).abs() > 1e-14 {
                        (threshold - v0) / (v1 - v0)
                    } else {
                        0.5
                    };
                    return s0.time + fraction * (s1.time - s0.time);
                }
            }
            solution.steps.last().map(|s| s.time).unwrap_or(0.0)
        }
    }
}

/// Distributed multi-threaded parameter space exploration and Monte Carlo harvester engine.
#[derive(Debug, Clone)]
pub struct MonteCarloHarvester {
    /// Active parameters undergoing sampling.
    pub parameters: Vec<LhsParameter>,
    /// Sampling strategy mode (LHS vs Monte Carlo).
    pub sampling_mode: SamplingMode,
    /// Seed for reproducible PRNG sweeps.
    pub seed: u64,
    /// Number of histogram bins to generate.
    pub bin_count: usize,
}

impl MonteCarloHarvester {
    /// Constructs a new harvester for the given parameter list.
    pub fn new(parameters: Vec<LhsParameter>) -> Self {
        Self {
            parameters,
            sampling_mode: SamplingMode::LatinHypercube,
            seed: 0x5EED_CAFE_1234_5678,
            bin_count: 25,
        }
    }

    /// Sets the sampling strategy mode.
    pub fn with_sampling_mode(mut self, mode: SamplingMode) -> Self {
        self.sampling_mode = mode;
        self
    }

    /// Sets the random seed for reproducible sweeps.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Configures the number of histogram partition bins.
    pub fn with_bins(mut self, bin_count: usize) -> Self {
        self.bin_count = bin_count.max(2);
        self
    }

    /// Executes high-throughput parallel Rayon batch evaluation using a custom closure or companion model.
    pub fn harvest_custom<F>(
        &self,
        spec: &YieldSpecification,
        n_samples: usize,
        eval_fn: F,
    ) -> HarvestReport
    where
        F: Fn(&[f64]) -> f64 + Sync + Send,
    {
        let sampler = LatinHypercubeSampler::new(self.seed).with_mode(self.sampling_mode);
        let sample_matrix = sampler.sample(&self.parameters, n_samples);

        let values: Vec<f64> = sample_matrix
            .par_iter()
            .map(|row| eval_fn(row))
            .collect();

        let metrics = compute_metrics(&values, spec);
        let histogram = compute_histogram(&values, self.bin_count);
        let corners = identify_process_corners(&values, &metrics);
        let empirical_cdf = compute_empirical_cdf(&values);

        let passed_count = values.iter().filter(|&&v| spec.meets_spec(v)).count();
        let failed_count = values.len() - passed_count;
        let parameter_names = self.parameters.iter().map(|p| p.name.clone()).collect();

        HarvestReport {
            spec: spec.clone(),
            metrics,
            histogram,
            corners,
            empirical_cdf,
            sample_values: values,
            parameter_names,
            samples: sample_matrix,
            passed_count,
            failed_count,
        }
    }

    /// Evaluates pre-generated sample vectors in parallel with Rayon.
    pub fn harvest_samples<F>(
        &self,
        samples: &[Vec<f64>],
        spec: &YieldSpecification,
        eval_fn: F,
    ) -> HarvestReport
    where
        F: Fn(&[f64]) -> f64 + Sync + Send,
    {
        let values: Vec<f64> = samples
            .par_iter()
            .map(|row| eval_fn(row))
            .collect();

        let metrics = compute_metrics(&values, spec);
        let histogram = compute_histogram(&values, self.bin_count);
        let corners = identify_process_corners(&values, &metrics);
        let empirical_cdf = compute_empirical_cdf(&values);

        let passed_count = values.iter().filter(|&&v| spec.meets_spec(v)).count();
        let failed_count = values.len() - passed_count;
        let parameter_names = self.parameters.iter().map(|p| p.name.clone()).collect();

        HarvestReport {
            spec: spec.clone(),
            metrics,
            histogram,
            corners,
            empirical_cdf,
            sample_values: values,
            parameter_names,
            samples: samples.to_vec(),
            passed_count,
            failed_count,
        }
    }

    /// Parallel Rayon batch sweep simulating N parameter samples over circuit netlists / MNA companion models.
    pub fn harvest_circuit(
        &self,
        graph: &CircuitGraph,
        context: &ModelContext,
        options: &TransientOptions,
        objective: &ObjectiveKind,
        spec: &YieldSpecification,
        n_samples: usize,
    ) -> Result<HarvestReport, SolverError> {
        let sampler = LatinHypercubeSampler::new(self.seed).with_mode(self.sampling_mode);
        let sample_matrix = sampler.sample(&self.parameters, n_samples);

        let results: Result<Vec<f64>, SolverError> = sample_matrix
            .par_iter()
            .map(|row| {
                let mut modified_graph = graph.clone();
                for (param_idx, param) in self.parameters.iter().enumerate() {
                    let val = row[param_idx];
                    let _ = modified_graph.update_component_value(&param.name, val);
                }
                let sol = solve_transient(&modified_graph, context, options)?;
                let obj_val = evaluate_circuit_objective(&sol, objective, &modified_graph);
                Ok(obj_val)
            })
            .collect();

        let values = results?;
        let metrics = compute_metrics(&values, spec);
        let histogram = compute_histogram(&values, self.bin_count);
        let corners = identify_process_corners(&values, &metrics);
        let empirical_cdf = compute_empirical_cdf(&values);

        let passed_count = values.iter().filter(|&&v| spec.meets_spec(v)).count();
        let failed_count = values.len() - passed_count;
        let parameter_names = self.parameters.iter().map(|p| p.name.clone()).collect();

        Ok(HarvestReport {
            spec: spec.clone(),
            metrics,
            histogram,
            corners,
            empirical_cdf,
            sample_values: values,
            parameter_names,
            samples: sample_matrix,
            passed_count,
            failed_count,
        })
    }
}
