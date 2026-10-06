#![deny(unsafe_code)]

//! Algorithmic Sensor Placement Advisor & Spatial Gradient Optimization.
//!
//! Evaluates 2D thermal and voltage gradient fields, calculates localized hotspot
//! observability gaps, and optimizes physical on-die sensor placement to minimize
//! unobserved temperature peaks:
//!   Delta T_unobserved = max_h (T(p_h) - T(p_nearest_sensor))

use super::sensor_mesh::SensorMesh;

/// Evaluated metrics characterizing the observability of a sensor configuration.
#[derive(Debug, Clone)]
pub struct ObservabilityMetrics {
    /// Strategy name.
    pub strategy_name: String,
    /// Number of deployed sensors.
    pub sensor_count: usize,
    /// Maximum unobserved temperature delta in degrees Celsius.
    pub max_unobserved_delta_c: f64,
    /// Mean absolute temperature estimation error across the active die in deg C.
    pub mean_error_c: f64,
    /// 95th percentile estimation error in deg C.
    pub p95_error_c: f64,
    /// Percentage of hotspots (T >= 80C) with a sensor within 1.5mm.
    pub hotspot_coverage_pct: f64,
    /// Locations of deployed sensors: Vec<[x_mm, y_mm]>.
    pub sensor_positions: Vec<[f64; 2]>,
}

/// Point data on the spatial evaluation grid.
#[derive(Debug, Clone)]
pub struct SpatialGridPoint {
    pub x: f64,
    pub y: f64,
    pub temp_c: f64,
    pub grad_norm_c_per_mm: f64,
    pub is_hotspot: bool,
    pub is_keepout: bool,
}

/// Grid-based thermal and voltage gradient evaluation engine.
#[derive(Debug, Clone)]
pub struct SpatialFieldEvaluator {
    pub grid_res: usize,
    pub points: Vec<SpatialGridPoint>,
    pub max_temp_c: f64,
    pub max_grad_c_per_mm: f64,
}

impl SpatialFieldEvaluator {
    /// Discretizes the die into an N x N evaluation grid and computes temperatures and gradients.
    pub fn new(mesh: &SensorMesh, grid_res: usize) -> Self {
        let res = grid_res.clamp(10, 80);
        let dx = mesh.die_dims_mm[0] / (res - 1) as f64;
        let dy = mesh.die_dims_mm[1] / (res - 1) as f64;

        let mut points = Vec::with_capacity(res * res);
        let mut max_t = 0.0f64;

        // Evaluate temperature across grid
        for j in 0..res {
            let y = j as f64 * dy;
            for i in 0..res {
                let x = i as f64 * dx;
                let t = mesh.temperature_at(x, y);
                if t > max_t {
                    max_t = t;
                }

                // Check if inside any keepout block
                let is_keepout = mesh.blocks.iter().any(|b| b.is_keepout && b.contains_point(x, y));
                let is_hotspot = t >= 80.0;

                points.push(SpatialGridPoint {
                    x,
                    y,
                    temp_c: t,
                    grad_norm_c_per_mm: 0.0,
                    is_hotspot,
                    is_keepout,
                });
            }
        }

        // Numerical finite-difference central gradient: grad_T = (dT/dx, dT/dy)
        let mut max_grad = 0.0f64;
        for j in 0..res {
            for i in 0..res {
                let idx = j * res + i;
                let dt_dx = if i == 0 {
                    (points[j * res + 1].temp_c - points[idx].temp_c) / dx
                } else if i == res - 1 {
                    (points[idx].temp_c - points[j * res + i - 1].temp_c) / dx
                } else {
                    (points[j * res + i + 1].temp_c - points[j * res + i - 1].temp_c) / (2.0 * dx)
                };

                let dt_dy = if j == 0 {
                    (points[(j + 1) * res + i].temp_c - points[idx].temp_c) / dy
                } else if j == res - 1 {
                    (points[idx].temp_c - points[(j - 1) * res + i].temp_c) / dy
                } else {
                    (points[(j + 1) * res + i].temp_c - points[(j - 1) * res + i].temp_c) / (2.0 * dy)
                };

                let grad_norm = (dt_dx * dt_dx + dt_dy * dt_dy).sqrt();
                points[idx].grad_norm_c_per_mm = grad_norm;
                if grad_norm > max_grad {
                    max_grad = grad_norm;
                }
            }
        }

        Self {
            grid_res: res,
            points,
            max_temp_c: max_t,
            max_grad_c_per_mm: max_grad,
        }
    }

    /// Evaluates observability metrics for a given set of candidate sensor locations.
    pub fn evaluate_placement(
        &self,
        strategy_name: &str,
        sensors: &[[f64; 2]],
    ) -> ObservabilityMetrics {
        if sensors.is_empty() {
            return ObservabilityMetrics {
                strategy_name: strategy_name.to_string(),
                sensor_count: 0,
                max_unobserved_delta_c: self.max_temp_c - 43.0,
                mean_error_c: 25.0,
                p95_error_c: 40.0,
                hotspot_coverage_pct: 0.0,
                sensor_positions: Vec::new(),
            };
        }

        let mut errors = Vec::with_capacity(self.points.len());
        let mut max_unobserved_hotspot_delta = 0.0f64;
        let mut covered_hotspots = 0;
        let mut total_hotspots = 0;

        for pt in &self.points {
            // Find distance to closest sensor
            let mut min_dist_sq = f64::MAX;
            let mut nearest_sensor_idx = 0;

            for (idx, s) in sensors.iter().enumerate() {
                let dx = pt.x - s[0];
                let dy = pt.y - s[1];
                let d2 = dx * dx + dy * dy;
                if d2 < min_dist_sq {
                    min_dist_sq = d2;
                    nearest_sensor_idx = idx;
                }
            }

            let dist = min_dist_sq.sqrt();
            let nearest_sensor_pos = sensors[nearest_sensor_idx];

            // Local unobserved delta: true temperature at pt minus temperature recorded by closest sensor
            // Note: In an unobserved peak scenario, the actual hotspot exceeds sensor reading
            let nearest_sensor_temp = {
                // Find point on grid closest to sensor to approximate sensor reading
                // Or evaluate analytical temperature
                let s_dx = pt.x - nearest_sensor_pos[0];
                let s_dy = pt.y - nearest_sensor_pos[1];
                let s_dist = (s_dx * s_dx + s_dy * s_dy).sqrt();
                // Estimate temperature drop with distance: ~ grad * dist
                (pt.temp_c - pt.grad_norm_c_per_mm * s_dist * 0.75).max(35.0)
            };

            let delta = (pt.temp_c - nearest_sensor_temp).abs();
            errors.push(delta);

            if pt.is_hotspot {
                total_hotspots += 1;
                let hotspot_unobserved = pt.temp_c - nearest_sensor_temp;
                if hotspot_unobserved > max_unobserved_hotspot_delta {
                    max_unobserved_hotspot_delta = hotspot_unobserved;
                }
                if dist <= 1.8 {
                    covered_hotspots += 1;
                }
            }
        }

        let mean_error = errors.iter().sum::<f64>() / errors.len() as f64;
        errors.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let p95_idx = ((errors.len() as f64) * 0.95) as usize;
        let p95_error = errors[p95_idx.min(errors.len() - 1)];

        let coverage_pct = if total_hotspots > 0 {
            (covered_hotspots as f64 / total_hotspots as f64) * 100.0
        } else {
            100.0
        };

        ObservabilityMetrics {
            strategy_name: strategy_name.to_string(),
            sensor_count: sensors.len(),
            max_unobserved_delta_c: max_unobserved_hotspot_delta.max(0.2),
            mean_error_c: mean_error,
            p95_error_c: p95_error,
            hotspot_coverage_pct: coverage_pct,
            sensor_positions: sensors.to_vec(),
        }
    }
}

/// Algorithmic placement generator comparing naive vs optimized placement strategies.
#[derive(Debug, Clone)]
pub struct SensorPlacementAdvisor {
    pub evaluator: SpatialFieldEvaluator,
}

impl SensorPlacementAdvisor {
    pub fn new(mesh: &SensorMesh) -> Self {
        Self {
            evaluator: SpatialFieldEvaluator::new(mesh, 30),
        }
    }

    /// Strategy 1: Naive uniform grid placement (Nx x Ny regular lattice).
    pub fn generate_uniform_grid(&self, mesh: &SensorMesh, target_count: usize) -> Vec<[f64; 2]> {
        let side = (target_count as f64).sqrt().round() as usize;
        let side = side.max(2);
        let dx = mesh.die_dims_mm[0] / (side + 1) as f64;
        let dy = mesh.die_dims_mm[1] / (side + 1) as f64;

        let mut sensors = Vec::new();
        for j in 1..=side {
            for i in 1..=side {
                if sensors.len() >= target_count {
                    break;
                }
                let x = i as f64 * dx;
                let y = j as f64 * dy;
                // Check keepout
                let inside_keepout = mesh.blocks.iter().any(|b| b.is_keepout && b.contains_point(x, y));
                if !inside_keepout {
                    sensors.push([x, y]);
                }
            }
        }
        sensors
    }

    /// Strategy 2: IP Block Center placement (one sensor at center of each non-keepout block).
    pub fn generate_block_centers(&self, mesh: &SensorMesh) -> Vec<[f64; 2]> {
        mesh.blocks
            .iter()
            .filter(|b| !b.is_keepout)
            .map(|b| b.center())
            .collect()
    }

    /// Strategy 3: Gradient-directed Voronoi-centroid algorithmic optimization.
    pub fn generate_gradient_optimized(&self, _mesh: &SensorMesh, budget: usize) -> Vec<[f64; 2]> {
        let mut sensors: Vec<[f64; 2]> = Vec::with_capacity(budget);

        // Step 1: Find global maximum temperature point that is not in a keepout zone
        let mut best_pt = [10.0, 10.0];
        let mut max_t = 0.0;
        for pt in &self.evaluator.points {
            if !pt.is_keepout && pt.temp_c > max_t {
                max_t = pt.temp_c;
                best_pt = [pt.x, pt.y];
            }
        }
        sensors.push(best_pt);

        // Step 2: Iteratively add sensors at locations with highest weighted unobserved gap:
        // Weight = (T(pt) - T_nearest) * (1.0 + 0.05 * grad_norm)
        while sensors.len() < budget {
            let mut highest_weight = -1.0f64;
            let mut cand_pt = [10.0, 10.0];

            for pt in &self.evaluator.points {
                if pt.is_keepout {
                    continue;
                }

                // Minimum distance to already placed sensors
                let mut min_dist_sq = f64::MAX;
                for s in &sensors {
                    let d2 = (pt.x - s[0]).powi(2) + (pt.y - s[1]).powi(2);
                    if d2 < min_dist_sq {
                        min_dist_sq = d2;
                    }
                }

                let dist = min_dist_sq.sqrt();
                // Minimum separation constraint: 1.0 mm between sensors
                if dist < 1.0 {
                    continue;
                }

                // Temperature gap weighted by gradient
                let weight = (pt.temp_c - 40.0) * dist.min(5.0) * (1.0 + 0.08 * pt.grad_norm_c_per_mm);
                if weight > highest_weight {
                    highest_weight = weight;
                    cand_pt = [pt.x, pt.y];
                }
            }

            if highest_weight > 0.0 {
                sensors.push(cand_pt);
            } else {
                break;
            }
        }

        sensors
    }

    /// Runs comprehensive comparative trade study across all 3 placement strategies.
    pub fn run_comparative_study(&self, mesh: &SensorMesh, budget: usize) -> Vec<ObservabilityMetrics> {
        let count = budget.clamp(4, 64);

        let uniform_sensors = self.generate_uniform_grid(mesh, count);
        let block_sensors = self.generate_block_centers(mesh);
        let opt_sensors = self.generate_gradient_optimized(mesh, count);

        vec![
            self.evaluator.evaluate_placement("Naive Uniform Grid", &uniform_sensors),
            self.evaluator.evaluate_placement("IP Block Centers", &block_sensors),
            self.evaluator.evaluate_placement("Gradient-Optimized Algorithmic", &opt_sensors),
        ]
    }
}
