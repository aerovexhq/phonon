//! 2D Hexagonal (honeycomb) superconducting nanowire array geometry,
//! tri-junction connectivity, and orientation-dependent topological Bogoliubov-de Gennes parameters.
//!
//! Formulates:
//! - Nanowire segments on a hexagonal lattice with 120-degree tri-junctions.
//! - Directional Rashba spin-orbit coupling $\vec{\alpha}_R \times \hat{e}_\parallel$.
//! - In-plane Zeeman magnetic field decomposition and orientation-dependent topological gap.
//! - Local electrostatic depletion gate electrodes tuning chemical potential profiles.
//! - Majorana zero mode (MZM) localization, coherence lengths, and hybridization energy.

use std::f64::consts::PI;

/// Physical fundamental constants (SI / eV units).
pub const HBAR_EV_S: f64 = 6.582_119_569e-16; // eV * s
pub const ELECTRON_MASS_KG: f64 = 9.109_383_7e-31; // kg
pub const BOHR_MAGNETON_EV_T: f64 = 5.788_381_806_6e-5; // eV / T
pub const ELEMENTARY_CHARGE_C: f64 = 1.602_176_634e-19; // C

/// Semiconductor material parameters for proximity-induced topological superconductivity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MajoranaMaterialParams {
    /// Effective electron mass ratio $m^* / m_0$ (e.g. 0.026 for InAs, 0.014 for InSb).
    pub effective_mass_ratio: f64,
    /// Rashba spin-orbit coupling parameter $\alpha_R$ in $\text{eV}\cdot\text{nm}$ (typically 0.02 - 0.05).
    pub rashba_alpha_ev_nm: f64,
    /// Proximity-induced s-wave superconducting pairing gap $\Delta_0$ in meV (typically 0.2 - 0.5 meV).
    pub induced_gap_mev: f64,
    /// Effective Landé g-factor $g^*$ (typically 10 - 50).
    pub g_factor: f64,
    /// Fermi velocity $v_F$ in m/s (typically $\sim 10^5\text{ m/s}$).
    pub fermi_velocity_m_s: f64,
}

impl MajoranaMaterialParams {
    /// Standard epitaxial InAs/Al parameters.
    pub fn inas_al() -> Self {
        Self {
            effective_mass_ratio: 0.026,
            rashba_alpha_ev_nm: 0.03, // 0.3 eV * Angstrom = 0.03 eV * nm
            induced_gap_mev: 0.25,
            g_factor: 15.0,
            fermi_velocity_m_s: 2.5e5,
        }
    }

    /// High-mobility InSb/Al or InSb/NbTiN parameters.
    pub fn insb_al() -> Self {
        Self {
            effective_mass_ratio: 0.014,
            rashba_alpha_ev_nm: 0.05,
            induced_gap_mev: 0.35,
            g_factor: 40.0,
            fermi_velocity_m_s: 3.5e5,
        }
    }

    /// Spin-orbit energy $E_{SO} = \frac{m^* \alpha_R^2}{2 \hbar^2}$ in meV.
    pub fn spin_orbit_energy_mev(&self) -> f64 {
        let m_eff = self.effective_mass_ratio * ELECTRON_MASS_KG;
        let alpha_si = self.rashba_alpha_ev_nm * 1e-9 * ELEMENTARY_CHARGE_C; // J * m
        let hbar_si: f64 = 1.054_571_817e-34; // J * s
        let e_so_joules = (m_eff * alpha_si.powi(2)) / (2.0 * hbar_si.powi(2));
        (e_so_joules / ELEMENTARY_CHARGE_C) * 1e3 // in meV
    }

    /// Spin-orbit length $\ell_{SO} = \frac{\hbar^2}{m^* \alpha_R}$ in nm.
    pub fn spin_orbit_length_nm(&self) -> f64 {
        let m_eff = self.effective_mass_ratio * ELECTRON_MASS_KG;
        let alpha_si = self.rashba_alpha_ev_nm * 1e-9 * ELEMENTARY_CHARGE_C;
        let hbar_si: f64 = 1.054_571_817e-34;
        let l_so_m = hbar_si.powi(2) / (m_eff * alpha_si);
        l_so_m * 1e9
    }
}

/// In-plane Zeeman magnetic field configuration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InPlaneMagneticField {
    /// Field magnitude $B$ in Tesla (typically 0.5 - 2.0 T).
    pub magnitude_tesla: f64,
    /// Azimuthal angle $\theta_B$ in radians relative to x-axis.
    pub angle_rad: f64,
}

impl InPlaneMagneticField {
    pub fn new(magnitude_tesla: f64, angle_rad: f64) -> Self {
        Self {
            magnitude_tesla,
            angle_rad,
        }
    }

    /// Zeeman splitting energy $E_Z = \frac{1}{2} g^* \mu_B B$ in meV.
    pub fn zeeman_energy_mev(&self, g_factor: f64) -> f64 {
        0.5 * g_factor * (BOHR_MAGNETON_EV_T * 1e3) * self.magnitude_tesla
    }

    /// Parallel component of Zeeman energy along a wire oriented at angle $\theta_w$.
    pub fn parallel_zeeman_energy_mev(&self, g_factor: f64, wire_angle_rad: f64) -> f64 {
        let total_ez = self.zeeman_energy_mev(g_factor);
        total_ez * (self.angle_rad - wire_angle_rad).cos().abs()
    }

    /// Perpendicular component of Zeeman energy along a wire oriented at angle $\theta_w$.
    pub fn perpendicular_zeeman_energy_mev(&self, g_factor: f64, wire_angle_rad: f64) -> f64 {
        let total_ez = self.zeeman_energy_mev(g_factor);
        total_ez * (self.angle_rad - wire_angle_rad).sin().abs()
    }
}

/// An individual nanowire arm/segment in the hexagonal network.
#[derive(Debug, Clone, PartialEq)]
pub struct NanowireSegment {
    /// Unique segment identifier.
    pub id: usize,
    /// Start vertex ID.
    pub start_vertex: usize,
    /// End vertex ID.
    pub end_vertex: usize,
    /// Physical length of the segment in nanometers (typically 500 - 2000 nm).
    pub length_nm: f64,
    /// Orientation angle $\theta_w$ in radians in the 2D plane.
    pub orientation_angle_rad: f64,
    /// Discretized electrostatic gate voltages controlling chemical potential:
    /// `[mu_start, mu_center, mu_end]` in meV.
    pub gate_chemical_potentials_mev: [f64; 3],
}

impl NanowireSegment {
    pub fn new(
        id: usize,
        start_vertex: usize,
        end_vertex: usize,
        length_nm: f64,
        orientation_angle_rad: f64,
    ) -> Self {
        Self {
            id,
            start_vertex,
            end_vertex,
            length_nm,
            orientation_angle_rad,
            gate_chemical_potentials_mev: [0.0, 0.0, 0.0],
        }
    }

    /// Sets gate chemical potentials uniformly or piecewise across the segment.
    pub fn set_gates(&mut self, mu_start: f64, mu_center: f64, mu_end: f64) {
        self.gate_chemical_potentials_mev = [mu_start, mu_center, mu_end];
    }

    /// Interpolates local chemical potential $\mu(s)$ for normalized coordinate $s \in [0, 1]$.
    pub fn local_chemical_potential_mev(&self, s: f64) -> f64 {
        let s_clamped = s.clamp(0.0, 1.0);
        if s_clamped <= 0.5 {
            let t = s_clamped * 2.0;
            (1.0 - t) * self.gate_chemical_potentials_mev[0]
                + t * self.gate_chemical_potentials_mev[1]
        } else {
            let t = (s_clamped - 0.5) * 2.0;
            (1.0 - t) * self.gate_chemical_potentials_mev[1]
                + t * self.gate_chemical_potentials_mev[2]
        }
    }

    /// Evaluates if the segment at coordinate $s$ satisfies the topological criterion:
    /// $$E_{Z,\parallel} > \sqrt{\Delta_0^2 + \mu(s)^2}$$
    pub fn is_topological(
        &self,
        s: f64,
        materials: &MajoranaMaterialParams,
        field: &InPlaneMagneticField,
    ) -> bool {
        let ez_par =
            field.parallel_zeeman_energy_mev(materials.g_factor, self.orientation_angle_rad);
        let mu = self.local_chemical_potential_mev(s);
        let threshold = (materials.induced_gap_mev.powi(2) + mu.powi(2)).sqrt();
        ez_par > threshold
    }

    /// Effective topological gap $\Delta_{top}(s)$ in meV protecting the Majorana zero modes:
    pub fn topological_gap_mev(
        &self,
        s: f64,
        materials: &MajoranaMaterialParams,
        field: &InPlaneMagneticField,
    ) -> f64 {
        if !self.is_topological(s, materials, field) {
            0.0
        } else {
            let ez_par =
                field.parallel_zeeman_energy_mev(materials.g_factor, self.orientation_angle_rad);
            let e_so = materials.spin_orbit_energy_mev();
            let delta = materials.induced_gap_mev;
            let denom = (ez_par.powi(2) + e_so.powi(2)).sqrt().max(1e-6);
            (delta * e_so / denom).min(delta)
        }
    }

    /// Majorana coherence localization length $\xi_M \approx \frac{\alpha_R}{\Delta_{top}}$ in nanometers.
    pub fn majorana_coherence_length_nm(
        &self,
        s: f64,
        materials: &MajoranaMaterialParams,
        field: &InPlaneMagneticField,
    ) -> f64 {
        let gap_mev = self.topological_gap_mev(s, materials, field);
        if gap_mev <= 1e-4 {
            f64::INFINITY
        } else {
            let alpha_mev_nm = materials.rashba_alpha_ev_nm * 1e3;
            (alpha_mev_nm / gap_mev).clamp(50.0, 500.0)
        }
    }
}

/// A tri-junction vertex where 3 hexagonal nanowire arms meet at 120-degree angles.
#[derive(Debug, Clone, PartialEq)]
pub struct TriJunction {
    /// Vertex index.
    pub vertex_id: usize,
    /// 2D Cartesian spatial position $(x, y)$ in nanometers.
    pub position_nm: (f64, f64),
    /// IDs of the three connected nanowire segments: `[arm0, arm1, arm2]`.
    pub connected_segments: [usize; 3],
    /// Central vertex barrier potential $\mu_{junc}$ in meV (positive = barrier / pinch-off, zero = open).
    pub junction_barrier_mev: f64,
}

impl TriJunction {
    pub fn new(
        vertex_id: usize,
        position_nm: (f64, f64),
        connected_segments: [usize; 3],
        junction_barrier_mev: f64,
    ) -> Self {
        Self {
            vertex_id,
            position_nm,
            connected_segments,
            junction_barrier_mev,
        }
    }

    /// Evaluates vertex transmission probability $T_{junc}$ between arms through the barrier:
    pub fn junction_transmission(&self, induced_gap_mev: f64) -> f64 {
        let arg = (self.junction_barrier_mev / induced_gap_mev.max(1e-4)).clamp(-30.0, 30.0);
        1.0 / (1.0 + arg.exp())
    }
}

/// A localized Majorana Zero Mode (MZM) in the array.
#[derive(Debug, Clone, PartialEq)]
pub struct MajoranaZeroMode {
    /// Majorana operator index $i$ ($\gamma_i$).
    pub index: usize,
    /// Host segment ID.
    pub segment_id: usize,
    /// Normalized coordinate $s \in [0, 1]$ along the host segment.
    pub position_s: f64,
    /// Majorana localization length $\xi_M$ in nm.
    pub localization_length_nm: f64,
}

impl MajoranaZeroMode {
    pub fn new(
        index: usize,
        segment_id: usize,
        position_s: f64,
        localization_length_nm: f64,
    ) -> Self {
        Self {
            index,
            segment_id,
            position_s: position_s.clamp(0.0, 1.0),
            localization_length_nm,
        }
    }

    /// Spatial 2D position in nanometers given the segment geometry.
    pub fn cartesian_position(
        &self,
        segment: &NanowireSegment,
        start_coord: (f64, f64),
    ) -> (f64, f64) {
        let dist = self.position_s * segment.length_nm;
        (
            start_coord.0 + dist * segment.orientation_angle_rad.cos(),
            start_coord.1 + dist * segment.orientation_angle_rad.sin(),
        )
    }

    /// Majorana hybridization energy $\delta E_{ij}$ with another mode separated by distance $d$ in nm:
    pub fn hybridization_energy_mev(
        &self,
        separation_nm: f64,
        induced_gap_mev: f64,
        fermi_wavelength_nm: f64,
    ) -> f64 {
        let xi = self.localization_length_nm.max(10.0);
        let k_f = 2.0 * PI / fermi_wavelength_nm.max(1.0);
        let decay = (-separation_nm / xi).exp();
        let osc = (k_f * separation_nm).cos().abs();
        induced_gap_mev * decay * osc
    }
}

/// Complete 2D hexagonal superconducting nanowire array.
#[derive(Debug, Clone, PartialEq)]
pub struct HexagonalSuperconductingArray {
    /// Material parameters.
    pub materials: MajoranaMaterialParams,
    /// Applied in-plane Zeeman field.
    pub magnetic_field: InPlaneMagneticField,
    /// Arm pitch length in nanometers.
    pub arm_length_nm: f64,
    /// Nanowire segments.
    pub segments: Vec<NanowireSegment>,
    /// Tri-junction vertices.
    pub junctions: Vec<TriJunction>,
    /// Active Majorana zero modes hosted in the array.
    pub majorana_modes: Vec<MajoranaZeroMode>,
}

impl HexagonalSuperconductingArray {
    /// Creates a canonical Y-junction (minimal 3-arm tri-junction) network for braiding.
    /// Arms point along angles $0^\circ$, $120^\circ$, $240^\circ$.
    pub fn create_canonical_tri_junction(
        arm_length_nm: f64,
        field_magnitude_tesla: f64,
        field_angle_rad: f64,
    ) -> Self {
        let materials = MajoranaMaterialParams::inas_al();
        let magnetic_field = InPlaneMagneticField::new(field_magnitude_tesla, field_angle_rad);

        let angles = [0.0, 2.0 * PI / 3.0, 4.0 * PI / 3.0];
        let mut segments = Vec::new();
        for (i, &ang) in angles.iter().enumerate() {
            segments.push(NanowireSegment::new(i, 0, i + 1, arm_length_nm, ang));
        }

        let junctions = vec![TriJunction::new(0, (0.0, 0.0), [0, 1, 2], 0.0)];

        Self {
            materials,
            magnetic_field,
            arm_length_nm,
            segments,
            junctions,
            majorana_modes: Vec::new(),
        }
    }

    /// Creates a hexagonal plaquette (honeycomb cell) array with 6 vertices, 6 perimeter edges,
    /// plus 6 external connecting arms (total 12 segments).
    pub fn create_honeycomb_cell(
        arm_length_nm: f64,
        field_magnitude_tesla: f64,
        field_angle_rad: f64,
    ) -> Self {
        let materials = MajoranaMaterialParams::inas_al();
        let magnetic_field = InPlaneMagneticField::new(field_magnitude_tesla, field_angle_rad);

        let mut segments = Vec::new();
        let mut junctions = Vec::new();

        let mut vertex_coords = Vec::new();
        for i in 0..6 {
            let theta = (i as f64) * PI / 3.0;
            vertex_coords.push((arm_length_nm * theta.cos(), arm_length_nm * theta.sin()));
        }

        for i in 0..6 {
            let next = (i + 1) % 6;
            let dx = vertex_coords[next].0 - vertex_coords[i].0;
            let dy = vertex_coords[next].1 - vertex_coords[i].1;
            let ang = dy.atan2(dx);
            segments.push(NanowireSegment::new(i, i, next, arm_length_nm, ang));
        }

        for i in 0..6 {
            let theta = (i as f64) * PI / 3.0;
            segments.push(NanowireSegment::new(6 + i, i, 6 + i, arm_length_nm, theta));
        }

        for (i, &coord) in vertex_coords.iter().enumerate().take(6) {
            let prev_edge = (i + 5) % 6;
            let next_edge = i;
            let ext_arm = 6 + i;
            junctions.push(TriJunction::new(
                i,
                coord,
                [prev_edge, next_edge, ext_arm],
                0.0,
            ));
        }

        Self {
            materials,
            magnetic_field,
            arm_length_nm,
            segments,
            junctions,
            majorana_modes: Vec::new(),
        }
    }

    /// Evaluates the global minimum topological gap $\Delta_{min}$ across all active topological segments.
    pub fn global_minigap_mev(&self) -> f64 {
        let mut min_gap = f64::INFINITY;
        for seg in &self.segments {
            for &s in &[0.1, 0.5, 0.9] {
                if seg.is_topological(s, &self.materials, &self.magnetic_field) {
                    let g = seg.topological_gap_mev(s, &self.materials, &self.magnetic_field);
                    if g < min_gap {
                        min_gap = g;
                    }
                }
            }
        }
        if min_gap.is_infinite() {
            0.0
        } else {
            min_gap
        }
    }
}
