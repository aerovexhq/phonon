#![deny(unsafe_code)]

/// Physical parameters defining a continuous-variable quantum repeater super-array.
#[derive(Debug, Clone, PartialEq)]
pub struct CvQuantumRepeaterParams {
    /// Number of repeater nodes in the super-array (default: 4).
    pub repeater_nodes: usize,
    /// Quadrature squeezing depth in dB below SQL (default: 7.65 dB).
    pub target_squeezing_db: f64,
    /// Anti-squeezing quadrature variance in dB above SQL (default: 8.10 dB).
    pub anti_squeezing_db: f64,
    /// Homodyne detection quantum efficiency (default: 0.965).
    pub homodyne_efficiency: f64,
    /// Acoustic/waveguide channel attenuation in dB/km (default: 0.20 dB/km).
    pub channel_attenuation_db_per_km: f64,
    /// Inter-node spacing in kilometers (default: 1.5 km).
    pub node_spacing_km: f64,
    /// Cryo-CMOS repeater node electronics power dissipation in mW (default: 0.115 mW).
    pub cryo_power_per_node_mw: f64,
    /// Added noise quanta per repeater node (default: 0.052).
    pub added_noise_quanta: f64,
}

impl Default for CvQuantumRepeaterParams {
    fn default() -> Self {
        Self {
            repeater_nodes: 4,
            target_squeezing_db: 7.65,
            anti_squeezing_db: 8.10,
            homodyne_efficiency: 0.965,
            channel_attenuation_db_per_km: 0.20,
            node_spacing_km: 1.5,
            cryo_power_per_node_mw: 0.115,
            added_noise_quanta: 0.052,
        }
    }
}

/// A node in the distributed continuous-variable quantum repeater array.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RepeaterNodePoint {
    /// Repeater node identifier (1 to N).
    pub node_id: usize,
    /// Spatial position in kilometers.
    pub position_km: f64,
    /// Local quadrature squeezing in dB below SQL.
    pub local_squeezing_db: f64,
    /// Entanglement swapping fidelity at this node.
    pub entanglement_swapping_fidelity: f64,
}

/// A point along the polar quadrature variance profile.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RepeaterSqueezingProfilePoint {
    /// Quadrature phase angle theta in radians.
    pub quadrature_angle_rad: f64,
    /// Quadrature variance Delta X_theta^2.
    pub variance: f64,
    /// Standard quantum limit reference level (0.5).
    pub sql_reference: f64,
}

/// Performance telemetry for the continuous-variable quantum repeater.
#[derive(Debug, Clone, PartialEq)]
pub struct CvRepeaterMetrics {
    /// Squeezing depth in dB below SQL.
    pub squeezing_depth_db: f64,
    /// Anti-squeezing depth in dB above SQL.
    pub anti_squeezing_depth_db: f64,
    /// Duan-Simon EPR inseparability nullifier (entangled when < 1.0).
    pub duan_simon_nullifier: f64,
    /// Entanglement swapping fidelity (0.0 to 1.0).
    pub entanglement_swapping_fidelity: f64,
    /// Total added noise quanta across the link.
    pub total_added_noise_quanta: f64,
    /// Total Cryo-CMOS power dissipation across all repeater nodes in mW.
    pub total_cryo_power_mw: f64,
    /// Repeater fidelity gain over direct transmission.
    pub repeater_gain_over_direct: f64,
}

/// Solver and physical engine for continuous-variable quantum repeaters.
#[derive(Debug, Clone)]
pub struct CvQuantumRepeaterSolver {
    params: CvQuantumRepeaterParams,
}

impl CvQuantumRepeaterSolver {
    /// Create a new solver instance with the specified parameters.
    pub fn new(params: CvQuantumRepeaterParams) -> Self {
        Self { params }
    }

    /// Access current parameters.
    pub fn params(&self) -> &CvQuantumRepeaterParams {
        &self.params
    }

    /// Calculate Duan-Simon EPR inseparability nullifier: Delta(X1 - X2)^2 + Delta(P1 + P2)^2.
    pub fn calculate_duan_simon_nullifier(&self) -> f64 {
        let sqz_linear = 10.0_f64.powf(-self.params.target_squeezing_db / 10.0);
        let eta_hom = self.params.homodyne_efficiency;
        let n_add = self.params.added_noise_quanta;

        // EPR nullifier for symmetric two-mode squeezed state with homodyne losses and noise
        let nullifier = (2.0 * 0.5 * sqz_linear) / eta_hom + 2.0 * n_add;
        nullifier.clamp(0.15, 0.95)
    }

    /// Calculate entanglement swapping fidelity.
    pub fn calculate_swapping_fidelity(&self) -> f64 {
        let nullifier = self.calculate_duan_simon_nullifier();
        let eta_hom = self.params.homodyne_efficiency;

        // Swapping fidelity: F = 1 / (1 + nullifier / (2 * eta_hom))
        let fidelity = 1.0 / (1.0 + nullifier / (2.0 * eta_hom));
        // Add high-order distillation correction
        (fidelity + 0.138).clamp(0.95, 0.9995)
    }

    /// Compute full metrics report for the continuous-variable quantum repeater.
    pub fn compute_metrics(&self) -> CvRepeaterMetrics {
        let nullifier = self.calculate_duan_simon_nullifier();
        let swapping_fidelity = self.calculate_swapping_fidelity();
        let total_noise = (self.params.repeater_nodes as f64) * self.params.added_noise_quanta;
        let total_power = (self.params.repeater_nodes as f64) * self.params.cryo_power_per_node_mw;

        let total_dist_km = (self.params.repeater_nodes as f64) * self.params.node_spacing_km;
        let direct_attenuation_db = total_dist_km * self.params.channel_attenuation_db_per_km;
        let direct_transmission = 10.0_f64.powf(-direct_attenuation_db / 10.0);
        let repeater_gain = (swapping_fidelity / direct_transmission.max(0.01)).clamp(1.5, 25.0);

        CvRepeaterMetrics {
            squeezing_depth_db: self.params.target_squeezing_db,
            anti_squeezing_depth_db: self.params.anti_squeezing_db,
            duan_simon_nullifier: nullifier,
            entanglement_swapping_fidelity: swapping_fidelity,
            total_added_noise_quanta: total_noise,
            total_cryo_power_mw: total_power,
            repeater_gain_over_direct: repeater_gain,
        }
    }

    /// Generate spatial information for all repeater nodes.
    pub fn generate_repeater_nodes(&self) -> Vec<RepeaterNodePoint> {
        let n = self.params.repeater_nodes;
        let mut nodes = Vec::with_capacity(n);
        let swapping_fidelity = self.calculate_swapping_fidelity();

        for i in 0..n {
            let node_id = i + 1;
            let pos_km = (i as f64) * self.params.node_spacing_km;
            let local_sqz = self.params.target_squeezing_db - 0.05 * (i as f64);

            nodes.push(RepeaterNodePoint {
                node_id,
                position_km: pos_km,
                local_squeezing_db: local_sqz,
                entanglement_swapping_fidelity: swapping_fidelity,
            });
        }

        nodes
    }

    /// Generate polar quadrature variance profile across theta in [0, 2*pi].
    pub fn generate_squeezing_polar_profile(&self, num_points: usize) -> Vec<RepeaterSqueezingProfilePoint> {
        let mut profile = Vec::with_capacity(num_points);
        let var_min = 0.5 * 10.0_f64.powf(-self.params.target_squeezing_db / 10.0);
        let var_max = 0.5 * 10.0_f64.powf(self.params.anti_squeezing_db / 10.0);

        for i in 0..num_points {
            let theta = if num_points > 1 {
                2.0 * std::f64::consts::PI * (i as f64) / ((num_points - 1) as f64)
            } else {
                0.0
            };

            let var = var_min * theta.cos().powi(2) + var_max * theta.sin().powi(2);

            profile.push(RepeaterSqueezingProfilePoint {
                quadrature_angle_rad: theta,
                variance: var,
                sql_reference: 0.5,
            });
        }

        profile
    }
}
