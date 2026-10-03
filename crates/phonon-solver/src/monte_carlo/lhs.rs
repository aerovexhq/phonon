#![deny(unsafe_code)]

//! Latin Hypercube Sampling (LHS) and Monte Carlo parameter distribution generator.
//!
//! Provides stratified hypercube sampling ensuring exactly 1 sample per stratum per dimension,
//! support for Uniform, Gaussian, and LogNormal distributions, deterministic pseudo-random
//! number generation, and tolerance clamping.

use std::f64::consts::{PI, SQRT_2};

/// Deterministic 64-bit SplitMix pseudo-random number generator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// Creates a new PRNG with the specified 64-bit seed.
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x5EED_CAFE_BABE_F00D } else { seed },
        }
    }

    /// Generates the next pseudo-random 64-bit unsigned integer.
    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Generates the next pseudo-random f64 in the half-open interval [0.0, 1.0).
    #[inline]
    pub fn next_f64(&mut self) -> f64 {
        let val = self.next_u64() >> 11;
        (val as f64) * (1.0 / ((1u64 << 53) as f64))
    }

    /// Shuffles a slice in-place using the Fisher-Yates algorithm.
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        let len = slice.len();
        if len <= 1 {
            return;
        }
        for i in (1..len).rev() {
            let j = (self.next_u64() % ((i + 1) as u64)) as usize;
            slice.swap(i, j);
        }
    }
}

/// Computes the standard normal inverse cumulative distribution function (probit)
/// using Peter John Acklam's high-precision rational approximation (relative error < 1.15e-9).
pub fn normal_quantile(p: f64) -> f64 {
    let p = p.clamp(1e-15, 1.0 - 1e-15);

    const A1: f64 = -3.969683028665376e+01;
    const A2: f64 = 2.209460984245205e+02;
    const A3: f64 = -2.759285104469687e+02;
    const A4: f64 = 1.383577518672690e+02;
    const A5: f64 = -3.066479806614716e+01;
    const A6: f64 = 2.506628277459239e+00;

    const B1: f64 = -5.447609879822406e+01;
    const B2: f64 = 1.615858368580409e+02;
    const B3: f64 = -1.556989798598866e+02;
    const B4: f64 = 6.680131188771972e+01;
    const B5: f64 = -1.328068155288572e+01;

    const C1: f64 = -7.784894002430293e-03;
    const C2: f64 = -3.223964580411365e-01;
    const C3: f64 = -2.400758277161838e+00;
    const C4: f64 = -2.549732539343734e+00;
    const C5: f64 = 4.374664141464968e+00;
    const C6: f64 = 2.938163982698783e+00;

    const D1: f64 = 7.784695709041462e-03;
    const D2: f64 = 3.224671290700398e-01;
    const D3: f64 = 2.445134137142996e+00;
    const D4: f64 = 3.754408661907416e+00;

    const P_LOW: f64 = 0.02425;
    const P_HIGH: f64 = 1.0 - P_LOW;

    if p < P_LOW {
        let q = (-2.0 * p.ln()).sqrt();
        (((((C1 * q + C2) * q + C3) * q + C4) * q + C5) * q + C6)
            / ((((D1 * q + D2) * q + D3) * q + D4) * q + 1.0)
    } else if p <= P_HIGH {
        let q = p - 0.5;
        let r = q * q;
        (((((A1 * r + A2) * r + A3) * r + A4) * r + A5) * r + A6) * q
            / (((((B1 * r + B2) * r + B3) * r + B4) * r + B5) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((C1 * q + C2) * q + C3) * q + C4) * q + C5) * q + C6)
            / ((((D1 * q + D2) * q + D3) * q + D4) * q + 1.0)
    }
}

/// Computes the Gauss error function erf(x) via Abramowitz & Stegun 7.1.26 (error < 1.5e-7).
pub fn erf(x: f64) -> f64 {
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let abs_x = x.abs();
    let p = 0.3275911;
    let t = 1.0 / (1.0 + p * abs_x);
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let poly = ((((a5 * t + a4) * t + a3) * t + a2) * t + a1) * t;
    sign * (1.0 - poly * (-abs_x * abs_x).exp())
}

/// Standard normal cumulative distribution function Phi(z).
pub fn standard_normal_cdf(z: f64) -> f64 {
    0.5 * (1.0 + erf(z / SQRT_2))
}

/// Statistical probability distribution associated with a physical component parameter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParameterDistribution {
    /// Continuous uniform distribution between min and max bounds.
    Uniform(f64, f64),
    /// Normal Gaussian distribution parameterized by mean and standard deviation.
    Gaussian(f64, f64),
    /// Log-normal distribution parameterized by underlying normal mean and standard deviation.
    LogNormal(f64, f64),
}

impl ParameterDistribution {
    /// Maps a probability quantile u in [0.0, 1.0] to a parameter value via inverse CDF.
    pub fn inverse_cdf(&self, u: f64) -> f64 {
        let u_clamped = u.clamp(1e-15, 1.0 - 1e-15);
        match self {
            Self::Uniform(min, max) => {
                let (lo, hi) = if min <= max { (*min, *max) } else { (*max, *min) };
                lo + u_clamped * (hi - lo)
            }
            Self::Gaussian(mean, std_dev) => {
                let s = std_dev.abs().max(1e-15);
                mean + s * normal_quantile(u_clamped)
            }
            Self::LogNormal(mu, sigma) => {
                let s = sigma.abs().max(1e-15);
                (mu + s * normal_quantile(u_clamped)).exp()
            }
        }
    }

    /// Evaluates cumulative distribution function F(x).
    pub fn cdf(&self, x: f64) -> f64 {
        match self {
            Self::Uniform(min, max) => {
                let (lo, hi) = if min <= max { (*min, *max) } else { (*max, *min) };
                if x <= lo {
                    0.0
                } else if x >= hi {
                    1.0
                } else {
                    (x - lo) / (hi - lo)
                }
            }
            Self::Gaussian(mean, std_dev) => {
                let s = std_dev.abs().max(1e-15);
                standard_normal_cdf((x - mean) / s)
            }
            Self::LogNormal(mu, sigma) => {
                if x <= 0.0 {
                    0.0
                } else {
                    let s = sigma.abs().max(1e-15);
                    standard_normal_cdf((x.ln() - mu) / s)
                }
            }
        }
    }

    /// Evaluates the probability density function f(x).
    pub fn pdf(&self, x: f64) -> f64 {
        match self {
            Self::Uniform(min, max) => {
                let (lo, hi) = if min <= max { (*min, *max) } else { (*max, *min) };
                if x >= lo && x <= hi && hi > lo {
                    1.0 / (hi - lo)
                } else {
                    0.0
                }
            }
            Self::Gaussian(mean, std_dev) => {
                let s = std_dev.abs().max(1e-15);
                let z = (x - mean) / s;
                (-0.5 * z * z).exp() / (s * (2.0 * PI).sqrt())
            }
            Self::LogNormal(mu, sigma) => {
                if x <= 0.0 {
                    0.0
                } else {
                    let s = sigma.abs().max(1e-15);
                    let z = (x.ln() - mu) / s;
                    (-0.5 * z * z).exp() / (x * s * (2.0 * PI).sqrt())
                }
            }
        }
    }

    /// Theoretical expected value (mean) of the distribution.
    pub fn mean(&self) -> f64 {
        match self {
            Self::Uniform(min, max) => 0.5 * (min + max),
            Self::Gaussian(mean, _) => *mean,
            Self::LogNormal(mu, sigma) => (mu + 0.5 * sigma * sigma).exp(),
        }
    }

    /// Theoretical variance of the distribution.
    pub fn variance(&self) -> f64 {
        match self {
            Self::Uniform(min, max) => {
                let range = (max - min).abs();
                (range * range) / 12.0
            }
            Self::Gaussian(_, std_dev) => std_dev * std_dev,
            Self::LogNormal(mu, sigma) => {
                let s2 = sigma * sigma;
                (s2.exp() - 1.0) * (2.0 * mu + s2).exp()
            }
        }
    }

    /// Theoretical standard deviation of the distribution.
    pub fn std_dev(&self) -> f64 {
        self.variance().sqrt()
    }
}

/// Tunable or uncertain circuit parameter designated for sampling.
#[derive(Debug, Clone, PartialEq)]
pub struct LhsParameter {
    /// Unique identifier / component designation (e.g. "R1", "C_LOAD", "M1_W").
    pub name: String,
    /// Baseline nominal value.
    pub nominal: f64,
    /// Statistical parameter distribution.
    pub distribution: ParameterDistribution,
    /// Relative percentage tolerance (+/- % relative to nominal).
    pub tolerance_pct: f64,
    /// Optional lower clamping limit preventing non-physical or negative values.
    pub min_clamp: Option<f64>,
    /// Optional upper clamping limit.
    pub max_clamp: Option<f64>,
}

impl LhsParameter {
    /// Creates a new parameter definition.
    pub fn new(
        name: impl Into<String>,
        nominal: f64,
        distribution: ParameterDistribution,
        tolerance_pct: f64,
    ) -> Self {
        Self {
            name: name.into(),
            nominal,
            distribution,
            tolerance_pct,
            min_clamp: None,
            max_clamp: None,
        }
    }

    /// Configures explicit lower and upper clamping bounds.
    pub fn with_clamping(mut self, min: f64, max: f64) -> Self {
        self.min_clamp = Some(min);
        self.max_clamp = Some(max);
        self
    }

    /// Constructs a uniformly distributed parameter centered at nominal +/- tolerance_pct.
    pub fn uniform(name: impl Into<String>, nominal: f64, tolerance_pct: f64) -> Self {
        let delta = (nominal * tolerance_pct / 100.0).abs();
        let min = nominal - delta;
        let max = nominal + delta;
        Self {
            name: name.into(),
            nominal,
            distribution: ParameterDistribution::Uniform(min, max),
            tolerance_pct,
            min_clamp: Some(0.0),
            max_clamp: None,
        }
    }

    /// Constructs a Gaussian parameter with +/-3-sigma span matching tolerance_pct.
    pub fn gaussian(name: impl Into<String>, nominal: f64, tolerance_pct: f64) -> Self {
        let sigma = ((nominal * tolerance_pct / 100.0).abs()) / 3.0;
        Self {
            name: name.into(),
            nominal,
            distribution: ParameterDistribution::Gaussian(nominal, sigma),
            tolerance_pct,
            min_clamp: Some(0.0),
            max_clamp: None,
        }
    }

    /// Constructs a Log-Normal parameter with +/-3-sigma span matching tolerance_pct.
    pub fn log_normal(name: impl Into<String>, nominal: f64, tolerance_pct: f64) -> Self {
        let mu = if nominal > 0.0 { nominal.ln() } else { 0.0 };
        let sigma = (tolerance_pct / 100.0) / 3.0;
        Self {
            name: name.into(),
            nominal,
            distribution: ParameterDistribution::LogNormal(mu, sigma),
            tolerance_pct,
            min_clamp: Some(0.0),
            max_clamp: None,
        }
    }

    /// Clamps a generated sample value within active bounds.
    pub fn clamp(&self, val: f64) -> f64 {
        let mut v = val;
        if let Some(min) = self.min_clamp {
            v = v.max(min);
        }
        if let Some(max) = self.max_clamp {
            v = v.min(max);
        }
        v
    }

    /// Returns the baseline nominal value.
    pub fn nominal_value(&self) -> f64 {
        self.nominal
    }
}

/// Sampling strategy mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SamplingMode {
    /// Latin Hypercube Sampling: stratified grid partitioning with permutation ensuring 1 sample per stratum.
    LatinHypercube,
    /// Standard independent pseudo-random Monte Carlo sampling.
    MonteCarlo,
}

/// Stratified Latin Hypercube and Monte Carlo multi-dimensional parameter sampler.
#[derive(Debug, Clone)]
pub struct LatinHypercubeSampler {
    /// Seed used for reproducible deterministic sampling.
    pub seed: u64,
    /// Active sampling mode.
    pub mode: SamplingMode,
}

impl Default for LatinHypercubeSampler {
    fn default() -> Self {
        Self {
            seed: 0x5EED_C0DE_1234_5678,
            mode: SamplingMode::LatinHypercube,
        }
    }
}

impl LatinHypercubeSampler {
    /// Constructs a new sampler with the specified seed.
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            mode: SamplingMode::LatinHypercube,
        }
    }

    /// Configures the sampling mode.
    pub fn with_mode(mut self, mode: SamplingMode) -> Self {
        self.mode = mode;
        self
    }

    /// Generates N multi-dimensional sample vectors across D parameters.
    ///
    /// When `mode` is `LatinHypercube`, the probability space [0, 1] is partitioned into N equal strata.
    /// Each stratum k in [k/N, (k+1)/N] is sampled exactly once per dimension, and strata indices are
    /// permuted independently for each dimension.
    pub fn sample(&self, parameters: &[LhsParameter], n_samples: usize) -> Vec<Vec<f64>> {
        if n_samples == 0 || parameters.is_empty() {
            return Vec::new();
        }

        let num_params = parameters.len();
        let mut samples = vec![vec![0.0; num_params]; n_samples];
        let mut rng = SplitMix64::new(self.seed);

        for (d, param) in parameters.iter().enumerate() {
            match self.mode {
                SamplingMode::LatinHypercube => {
                    // Generate N stratified quantiles strictly inside [k/N, (k+1)/N)
                    let mut u_vals = Vec::with_capacity(n_samples);
                    for k in 0..n_samples {
                        let r = rng.next_f64() * 0.999998 + 0.000001;
                        let u_k = (k as f64 + r) / (n_samples as f64);
                        u_vals.push(u_k);
                    }

                    // Permute stratum quantiles across the N sample vectors
                    rng.shuffle(&mut u_vals);

                    for i in 0..n_samples {
                        let u = u_vals[i];
                        let raw_val = param.distribution.inverse_cdf(u);
                        samples[i][d] = param.clamp(raw_val);
                    }
                }
                SamplingMode::MonteCarlo => {
                    for i in 0..n_samples {
                        let r = rng.next_f64() * 0.999998 + 0.000001;
                        let raw_val = param.distribution.inverse_cdf(r);
                        samples[i][d] = param.clamp(raw_val);
                    }
                }
            }
        }

        samples
    }
}
