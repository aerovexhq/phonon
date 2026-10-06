#![deny(unsafe_code)]

//! Statistical Defect Density, Clustering & Wafer-Scale Yield Models.
//!
//! Formulates classical and modern yield estimation models:
//! - Poisson Model (uniform spatial distribution, no clustering)
//! - Murphy Model (triangular defect probability distribution)
//! - Seeds Model (simple empirical hyperbola)
//! - Negative Binomial / Stapper Model (gamma-distributed defect clustering)
//! - Radial Defect Density Profile with edge exclusion multiplier

/// Defect density parameters for wafer yield projection.
#[derive(Debug, Clone)]
pub struct DefectYieldParams {
    /// Nominal defect density at wafer center in defects/cm^2 (typically 0.05 to 0.30).
    pub d0_center_cm2: f64,
    /// Radial edge defect escalation factor (e.g. 4.0 = defect density increases at edge).
    pub kappa_edge: f64,
    /// Edge defect profile polynomial exponent (typically 4.0 for sharp edge rise).
    pub edge_exponent: f64,
    /// Defect clustering parameter alpha for Negative Binomial model (typically 0.5 to 4.0).
    /// As alpha -> infinity, Negative Binomial converges to Poisson.
    pub cluster_alpha: f64,
    /// Critical area factor (fraction of die active area vulnerable to fatal defect, typically 0.6 to 0.85).
    pub critical_area_factor: f64,
}

impl Default for DefectYieldParams {
    fn default() -> Self {
        Self {
            d0_center_cm2: 0.12,
            kappa_edge: 4.5,
            edge_exponent: 4.0,
            cluster_alpha: 2.0,
            critical_area_factor: 0.75,
        }
    }
}

impl DefectYieldParams {
    /// Evaluates localized defect density D0(r) in defects/cm^2 at distance r from center.
    pub fn defect_density_at_radius(&self, r_mm: f64, r_active_mm: f64) -> f64 {
        if r_active_mm <= 0.0 {
            return self.d0_center_cm2;
        }
        let r_norm = (r_mm / r_active_mm).clamp(0.0, 1.2);
        self.d0_center_cm2 * (1.0 + self.kappa_edge * r_norm.powf(self.edge_exponent))
    }

    /// Poisson yield: Y = exp(-D0 * A_crit)
    pub fn poisson_yield(&self, d0: f64, die_area_cm2: f64) -> f64 {
        let a_crit = die_area_cm2 * self.critical_area_factor;
        let lambda = d0 * a_crit;
        (-lambda).exp()
    }

    /// Murphy yield (triangular distribution): Y = ((1 - exp(-D0 * A)) / (D0 * A))^2
    pub fn murphy_yield(&self, d0: f64, die_area_cm2: f64) -> f64 {
        let a_crit = die_area_cm2 * self.critical_area_factor;
        let lambda = d0 * a_crit;
        if lambda < 1.0e-6 {
            1.0 - lambda + (5.0 / 12.0) * lambda * lambda
        } else {
            let term = (1.0 - (-lambda).exp()) / lambda;
            term * term
        }
    }

    /// Seeds yield: Y = 1 / (1 + D0 * A)
    pub fn seeds_yield(&self, d0: f64, die_area_cm2: f64) -> f64 {
        let a_crit = die_area_cm2 * self.critical_area_factor;
        let lambda = d0 * a_crit;
        1.0 / (1.0 + lambda)
    }

    /// Negative Binomial (Stapper) yield: Y = (1 + D0 * A / alpha)^(-alpha)
    pub fn negative_binomial_yield(&self, d0: f64, die_area_cm2: f64) -> f64 {
        let a_crit = die_area_cm2 * self.critical_area_factor;
        let alpha = self.cluster_alpha.max(0.01);
        let lambda = d0 * a_crit;
        (1.0 + lambda / alpha).powf(-alpha)
    }
}

/// A comparison point along the die area curve comparing 4 yield models.
#[derive(Debug, Clone)]
pub struct YieldCurvePoint {
    pub die_area_cm2: f64,
    pub poisson_yield: f64,
    pub murphy_yield: f64,
    pub seeds_yield: f64,
    pub neg_bin_yield: f64,
}

/// Generates yield vs die area curves from 0.1 cm^2 to max_area_cm2.
pub fn generate_yield_curves(
    params: &DefectYieldParams,
    max_area_cm2: f64,
    steps: usize,
) -> Vec<YieldCurvePoint> {
    let mut points = Vec::with_capacity(steps);
    let min_area = 0.05;
    let step_size = (max_area_cm2 - min_area) / (steps.max(2) - 1) as f64;

    for i in 0..steps {
        let a = min_area + i as f64 * step_size;
        let d0 = params.d0_center_cm2;
        points.push(YieldCurvePoint {
            die_area_cm2: a,
            poisson_yield: params.poisson_yield(d0, a),
            murphy_yield: params.murphy_yield(d0, a),
            seeds_yield: params.seeds_yield(d0, a),
            neg_bin_yield: params.negative_binomial_yield(d0, a),
        });
    }

    points
}

/// Deterministic pseudo-random number generator for defect simulation across die grid.
pub struct DefectRng {
    state: u64,
}

impl DefectRng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x853c49e6748fea9b } else { seed },
        }
    }

    /// 64-bit XorShift pseudo-random number generator.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    /// Uniform float in [0.0, 1.0).
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Samples a Poisson-distributed integer with mean lambda using Knuth's algorithm.
    pub fn sample_poisson(&mut self, lambda: f64) -> u32 {
        if lambda <= 0.0 {
            return 0;
        }
        if lambda > 30.0 {
            // Gaussian approximation for large lambda
            let u1 = self.next_f64().max(1.0e-12);
            let u2 = self.next_f64();
            let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
            let val = lambda + lambda.sqrt() * z;
            return val.max(0.0).round() as u32;
        }

        let l = (-lambda).exp();
        let mut k = 0;
        let mut p = 1.0;
        loop {
            k += 1;
            p *= self.next_f64();
            if p <= l {
                break;
            }
        }
        k - 1
    }
}
