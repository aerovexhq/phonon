#![deny(unsafe_code)]

//! Latin Hypercube Sampling (LHS), Monte Carlo sensitivity harvesting, and yield capability engine.

pub mod lhs;
pub mod yield_harvester;

pub use lhs::{
    erf, normal_quantile, standard_normal_cdf, LatinHypercubeSampler, LhsParameter,
    ParameterDistribution, SamplingMode, SplitMix64,
};

pub use yield_harvester::{
    compute_empirical_cdf, compute_histogram, compute_metrics, evaluate_circuit_objective,
    identify_process_corners, EmpiricalCdfPoint, HarvestMetrics, HarvestReport, HistogramBin,
    MonteCarloHarvester, ProcessCorners, YieldSpecification,
};
