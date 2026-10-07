#![deny(unsafe_code)]

//! Twisted Acoustic Moiré Superlattice & Magic-Angle Flat Bands Engine.
//!
//! Models acoustic twistronics in bilayer phononic metamaterials, commensurate
//! moiré supercells, magic-angle kinetic energy quenching, flat-band dispersion,
//! and quantized valley Chern numbers in pure safe Rust.

use std::f64::consts::PI;

/// Configuration parameters for the acoustic moiré flat-band lattice.
#[derive(Debug, Clone, PartialEq)]
pub struct MoireFlatBandParams {
    /// Twist angle theta between layers in degrees (magic angle approx 1.08 deg).
    pub twist_angle_deg: f64,
    /// Monolayer unit cell lattice constant in mm.
    pub a_lattice_mm: f64,
    /// Interlayer AA-stacking acoustic tunneling coupling w0 in MHz.
    pub interlayer_w0_mhz: f64,
    /// Interlayer AB/BA-stacking acoustic tunneling coupling w1 in MHz.
    pub interlayer_w1_mhz: f64,
    /// Intralayer acoustic Dirac velocity v_F in MHz*mm.
    pub dirac_velocity_mhz_mm: f64,
    /// Central operating frequency in MHz.
    pub center_freq_mhz: f64,
}

impl Default for MoireFlatBandParams {
    fn default() -> Self {
        Self {
            twist_angle_deg: 1.08,
            a_lattice_mm: 2.0,
            interlayer_w0_mhz: 0.85,
            interlayer_w1_mhz: 1.25,
            dirac_velocity_mhz_mm: 15.0,
            center_freq_mhz: 25.0,
        }
    }
}

/// A single momentum point along the mini-Brillouin zone dispersion path.
#[derive(Debug, Clone, PartialEq)]
pub struct MoireCombBandPoint {
    /// Normalized k-path coordinate in [0, 1].
    pub k_dist: f64,
    /// Symmetry label (e.g. "Gamma_M", "M_M", "K_M").
    pub label: String,
    /// Lower valence mini-band frequency in MHz.
    pub lower_band_mhz: f64,
    /// Ultra-flat mid-band frequency in MHz.
    pub flat_band_mhz: f64,
    /// Upper conduction mini-band frequency in MHz.
    pub upper_band_mhz: f64,
}

/// A spatial sample in the real-space moiré interference pattern.
#[derive(Debug, Clone, PartialEq)]
pub struct MoireSpatialPoint {
    /// Real-space coordinate x in mm.
    pub pos_x_mm: f64,
    /// Real-space coordinate y in mm.
    pub pos_y_mm: f64,
    /// Local interlayer acoustic tunneling amplitude (w(r)).
    pub local_tunneling_mhz: f64,
    /// Stacking region type: 0=AA (symmetric), 1=AB, 2=BA.
    pub stacking_type: usize,
}

/// Evaluated metrics for the acoustic moiré flat-band superlattice.
#[derive(Debug, Clone, PartialEq)]
pub struct MoireFlatBandMetrics {
    /// Moiré superlattice period L_M = a / (2 * sin(theta / 2)) in mm.
    pub moire_period_lm_mm: f64,
    /// Bandwidth of the ultra-flat band in MHz (W_flat <= 0.5 MHz at magic angle).
    pub flat_bandwidth_mhz: f64,
    /// Acoustic bulk bandgap separating flat band from dispersive bands (MHz).
    pub bulk_bandgap_mhz: f64,
    /// Flatness ratio F = W_flat / Delta_gap (F <= 0.20 indicates extreme flattening).
    pub flatness_ratio: f64,
    /// Quantized valley Chern number C_v.
    pub valley_chern_number: f64,
    /// Effective acoustic mass divergence ratio m_eff / m_0.
    pub effective_mass_ratio: f64,
    /// Whether the twist angle is in the magic-angle flat-band regime.
    pub is_magic_angle: bool,
}

/// Solver for the twisted acoustic moiré flat-band superlattice.
#[derive(Debug, Clone)]
pub struct MoireFlatBandSolver {
    pub params: MoireFlatBandParams,
}

impl MoireFlatBandSolver {
    /// Creates a new solver with specified parameters.
    pub fn new(params: MoireFlatBandParams) -> Self {
        Self { params }
    }

    /// Evaluates flat-band and topological metrics.
    pub fn evaluate_metrics(&self) -> MoireFlatBandMetrics {
        let p = &self.params;
        let theta_rad = p.twist_angle_deg.to_radians().max(0.005);
        let moire_period_lm_mm = p.a_lattice_mm / (2.0 * (0.5 * theta_rad).sin());

        // Magic angle condition: theta approx 1.08 deg (or commensurate resonances)
        let magic_diff = (p.twist_angle_deg - 1.08).abs();
        let is_magic_angle = magic_diff <= 0.20;

        // Flat-band bandwidth: strongly suppressed near magic angle (down to ~0.25 MHz)
        let flat_bandwidth_mhz = if is_magic_angle {
            0.25 + 0.5 * magic_diff
        } else {
            0.25 + 1.8 * magic_diff
        }
        .min(4.5);

        // Bulk bandgap separating flat band from adjacent dispersive bands
        let bulk_bandgap_mhz = (2.0 * (p.interlayer_w1_mhz - p.interlayer_w0_mhz).abs() + 1.2).max(1.5);
        let flatness_ratio = (flat_bandwidth_mhz / bulk_bandgap_mhz).min(1.0);

        // Valley Chern number: quantized to +1.0 in topological regime
        let valley_chern_number = if p.interlayer_w1_mhz > p.interlayer_w0_mhz {
            1.0
        } else {
            0.0
        };

        // Effective mass divergence ratio m* / m0 = 1 / (d^2 E / dk^2)
        let effective_mass_ratio = (5.0 / flat_bandwidth_mhz.max(0.1)).max(1.0);

        MoireFlatBandMetrics {
            moire_period_lm_mm,
            flat_bandwidth_mhz,
            bulk_bandgap_mhz,
            flatness_ratio,
            valley_chern_number,
            effective_mass_ratio,
            is_magic_angle,
        }
    }

    /// Computes band dispersion across the mini-Brillouin zone path (Gamma -> M -> K -> Gamma).
    pub fn compute_band_dispersion(&self, points_per_segment: usize) -> Vec<MoireCombBandPoint> {
        let p = &self.params;
        let n = points_per_segment.max(10);
        let mut dispersion = Vec::with_capacity(3 * n);

        let metrics = self.evaluate_metrics();
        let f0 = p.center_freq_mhz;
        let gap = metrics.bulk_bandgap_mhz;
        let w_flat = metrics.flat_bandwidth_mhz;

        // Path: 0.0..0.33 (Gamma -> M), 0.33..0.66 (M -> K), 0.66..1.0 (K -> Gamma)
        for i in 0..=(3 * n) {
            let frac = i as f64 / ((3 * n) as f64);
            let (label, k_wave) = if frac <= 0.333 {
                let s = frac / 0.333;
                ("Gamma_M -> M_M", s * PI)
            } else if frac <= 0.666 {
                let s = (frac - 0.333) / 0.333;
                ("M_M -> K_M", PI + s * 0.5 * PI)
            } else {
                let s = (frac - 0.666) / 0.334;
                ("K_M -> Gamma_M", 1.5 * PI + s * 0.5 * PI)
            };

            // Dispersion formulas:
            // Flat band: strictly bounded within [-w_flat/2, w_flat/2]
            let flat_mod = 0.5 * w_flat * (2.0 * k_wave).cos();
            let flat_band_mhz = f0 + flat_mod;

            // Dispersive upper and lower mini-bands
            let disp = gap + 2.0 * (1.0 - (k_wave).cos().abs());
            let lower_band_mhz = f0 - disp;
            let upper_band_mhz = f0 + disp;

            dispersion.push(MoireCombBandPoint {
                k_dist: frac,
                label: label.to_string(),
                lower_band_mhz,
                flat_band_mhz,
                upper_band_mhz,
            });
        }

        dispersion
    }

    /// Generates 2D real-space moiré tunneling interference pattern.
    pub fn generate_spatial_profile(&self, grid_size: usize) -> Vec<MoireSpatialPoint> {
        let p = &self.params;
        let n = grid_size.max(12);
        let metrics = self.evaluate_metrics();
        let lm = metrics.moire_period_lm_mm;
        let mut points = Vec::with_capacity(n * n);

        let k_m = 2.0 * PI / lm;

        for iy in 0..n {
            let y = (iy as f64 / n as f64) * lm;
            for ix in 0..n {
                let x = (ix as f64 / n as f64) * lm;

                // 3 reciprocal lattice wavevectors of moiré pattern at 120 deg
                let b1 = k_m * x;
                let b2 = k_m * (-0.5 * x + (3.0f64.sqrt() / 2.0) * y);
                let b3 = k_m * (-0.5 * x - (3.0f64.sqrt() / 2.0) * y);

                let inter = (b1.cos() + b2.cos() + b3.cos()) / 3.0; // [-0.5, 1.0]

                let local_tunneling = p.interlayer_w0_mhz + (p.interlayer_w1_mhz - p.interlayer_w0_mhz) * (1.0 - inter).abs();
                let stacking_type = if inter > 0.6 {
                    0 // AA stacking
                } else if inter < -0.2 {
                    1 // AB stacking
                } else {
                    2 // BA stacking
                };

                points.push(MoireSpatialPoint {
                    pos_x_mm: x,
                    pos_y_mm: y,
                    local_tunneling_mhz: local_tunneling,
                    stacking_type,
                });
            }
        }

        points
    }
}
