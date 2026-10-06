#![deny(unsafe_code)]

//! 3D Oblique Ion Track Ray Intersection, Multi-Die Chiplet Stack & Correlated MBU/SEL Engine.
//!
//! Models 2.5D/3D heterogeneous chiplet layouts (compute die, HBM/SRAM memory stack, active interposer),
//! calculating oblique 3D ray intercepts, sensitive cell node upsets, correlated Multi-Bit Upsets (MBU),
//! and parasitic thyristor Single Event Latchup (SEL) triggering margins.

use super::ion_track::{IncidentTrajectory, IonTrackProfile};

/// Individual die layer in a 2.5D/3D heterogeneous chiplet package.
#[derive(Debug, Clone)]
pub struct DieLayer3D {
    pub die_id: usize,
    pub name: String,
    /// Center physical position in package in micrometers [x, y, z].
    pub center_um: [f64; 3],
    /// Die dimensions: width (X), length (Y), thickness (Z) in micrometers.
    pub dimensions_um: [f64; 3],
    /// Sensitive bit cell node pitch in micrometers (e.g. 0.8 um for 7nm SRAM).
    pub cell_pitch_um: f64,
    /// Critical charge threshold Q_crit in fC required to cause a bit flip (SEU).
    pub q_crit_fc: f64,
    /// Single Event Latchup critical charge threshold Q_sel in fC (e.g. 120 fC).
    pub q_sel_fc: f64,
}

impl DieLayer3D {
    /// Bounding box minimum coordinates [x_min, y_min, z_min].
    pub fn min_corner(&self) -> [f64; 3] {
        [
            self.center_um[0] - self.dimensions_um[0] * 0.5,
            self.center_um[1] - self.dimensions_um[1] * 0.5,
            self.center_um[2] - self.dimensions_um[2] * 0.5,
        ]
    }

    /// Bounding box maximum coordinates [x_max, y_max, z_max].
    pub fn max_corner(&self) -> [f64; 3] {
        [
            self.center_um[0] + self.dimensions_um[0] * 0.5,
            self.center_um[1] + self.dimensions_um[1] * 0.5,
            self.center_um[2] + self.dimensions_um[2] * 0.5,
        ]
    }

    /// Check if a point (x, y, z) is inside the die volume.
    pub fn contains_point(&self, p: [f64; 3]) -> bool {
        let min = self.min_corner();
        let max = self.max_corner();
        p[0] >= min[0] && p[0] <= max[0]
            && p[1] >= min[1] && p[1] <= max[1]
            && p[2] >= min[2] && p[2] <= max[2]
    }
}

/// Result of an oblique ion track ray piercing an individual die layer.
#[derive(Debug, Clone)]
pub struct DieHitResult {
    pub die_id: usize,
    pub die_name: String,
    /// Ray entry point into die [x, y, z] in micrometers.
    pub entry_point_um: [f64; 3],
    /// Ray exit point from die [x, y, z] in micrometers.
    pub exit_point_um: [f64; 3],
    /// Oblique track path length through die in micrometers.
    pub track_length_um: f64,
    /// Total deposited charge inside die sensitive volume in fC.
    pub deposited_charge_fc: f64,
    /// Number of flipped bit cells (Multi-Bit Upset multiplicity).
    pub upset_cell_count: usize,
    /// Whether Single Event Latchup (SEL) was triggered in this die.
    pub sel_triggered: bool,
    /// Transient pulse duration tau_set in picoseconds.
    pub set_pulse_duration_ps: f64,
}

/// Package-level 3D Multi-Die Chiplet Radiation Intercept Engine.
#[derive(Debug, Clone)]
pub struct MultiDieMbuEngine {
    pub dies: Vec<DieLayer3D>,
    pub track_model: IonTrackProfile,
}

impl Default for MultiDieMbuEngine {
    fn default() -> Self {
        // Typical 3D stacked system: Base Interposer (z=0), Host Compute SoC (z=60), HBM3/SRAM Top (z=120)
        let interposer = DieLayer3D {
            die_id: 0,
            name: "Active Silicon Interposer".to_string(),
            center_um: [0.0, 0.0, 10.0],
            dimensions_um: [8000.0, 8000.0, 20.0],
            cell_pitch_um: 2.5,
            q_crit_fc: 8.5,
            q_sel_fc: 140.0,
        };

        let compute_soc = DieLayer3D {
            die_id: 1,
            name: "Host Compute SoC Chiplet".to_string(),
            center_um: [-1500.0, 0.0, 60.0],
            dimensions_um: [4000.0, 4000.0, 40.0],
            cell_pitch_um: 0.9,
            q_crit_fc: 3.2,
            q_sel_fc: 95.0,
        };

        let hbm_memory = DieLayer3D {
            die_id: 2,
            name: "Stacked HBM/SRAM Memory Die".to_string(),
            center_um: [1500.0, 0.0, 120.0],
            dimensions_um: [3000.0, 4000.0, 35.0],
            cell_pitch_um: 0.75,
            q_crit_fc: 2.1, // Highly sensitive ultra-dense bit cells
            q_sel_fc: 85.0,
        };

        Self {
            dies: vec![interposer, compute_soc, hbm_memory],
            track_model: IonTrackProfile::default(),
        }
    }
}

impl MultiDieMbuEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Trace an oblique incident heavy ion ray through the 3D chiplet stack.
    ///
    /// Ray equation: r(s) = origin + s * direction, with s >= 0 (penetrating downward).
    pub fn trace_incident_ray(
        &self,
        trajectory: &IncidentTrajectory,
        ray_origin_um: [f64; 3],
    ) -> Vec<DieHitResult> {
        let dir = trajectory.direction_vector();
        let mut hit_results = Vec::new();

        for die in &self.dies {
            let min = die.min_corner();
            let max = die.max_corner();

            // Ray-AABB intersection algorithm
            let mut t_min = f64::NEG_INFINITY;
            let mut t_max = f64::INFINITY;

            for axis in 0..3 {
                let d = -dir[axis]; // Downward ray into substrate
                if d.abs() > 1e-9 {
                    let mut t1 = (min[axis] - ray_origin_um[axis]) / d;
                    let mut t2 = (max[axis] - ray_origin_um[axis]) / d;
                    if t1 > t2 {
                        std::mem::swap(&mut t1, &mut t2);
                    }
                    t_min = t_min.max(t1);
                    t_max = t_max.min(t2);
                } else if ray_origin_um[axis] < min[axis] || ray_origin_um[axis] > max[axis] {
                    t_min = 1.0;
                    t_max = -1.0; // No hit
                }
            }

            if t_min <= t_max && t_max >= 0.0 {
                let entry_s = t_min.max(0.0);
                let exit_s = t_max;
                let track_len = (exit_s - entry_s).max(0.1);

                let p_entry = [
                    ray_origin_um[0] - dir[0] * entry_s,
                    ray_origin_um[1] - dir[1] * entry_s,
                    ray_origin_um[2] - dir[2] * entry_s,
                ];

                let p_exit = [
                    ray_origin_um[0] - dir[0] * exit_s,
                    ray_origin_um[1] - dir[1] * exit_s,
                    ray_origin_um[2] - dir[2] * exit_s,
                ];

                // Calculate deposited charge along the segment inside this die
                let avg_depth = (p_entry[2] + p_exit[2]) * 0.5;
                let dq_dz = self.track_model.charge_density_fc_per_um(trajectory, avg_depth.abs());
                let dep_charge = dq_dz * track_len;

                // MBU multiplicity: cells spanned along path + radial diffusion core
                let r_core_um = trajectory.species.core_ionization_radius_nm() * 0.001;
                let effective_cross_section_um = (track_len * (trajectory.theta_rad.sin().abs())).max(die.cell_pitch_um);
                let cells_along_path = (effective_cross_section_um / die.cell_pitch_um).ceil() as usize;
                let radial_cells = ((2.0 * r_core_um) / die.cell_pitch_um).ceil().max(1.0) as usize;

                let upset_count = if dep_charge >= die.q_crit_fc {
                    (cells_along_path * radial_cells).max(1)
                } else {
                    0
                };

                let sel = dep_charge >= die.q_sel_fc;

                // Transient pulse duration: tau = Q_dep / I_drive (assuming 1.5 mA drive current)
                let set_ps = (dep_charge * 1e-15 / 1.5e-3) * 1e12;

                hit_results.push(DieHitResult {
                    die_id: die.die_id,
                    die_name: die.name.clone(),
                    entry_point_um: p_entry,
                    exit_point_um: p_exit,
                    track_length_um: track_len,
                    deposited_charge_fc: dep_charge,
                    upset_cell_count: upset_count,
                    sel_triggered: sel,
                    set_pulse_duration_ps: set_ps,
                });
            }
        }

        hit_results
    }
}
