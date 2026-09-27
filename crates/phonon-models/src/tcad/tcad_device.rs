//! High-level TCAD device representation and fluent device construction builder.

use super::material::SemiconductorMaterial;
use super::mesh::{ContactType, Mesh1D};
use super::poisson_dd::{PoissonDriftDiffusionSolver, TcadState1D};
use std::sync::RwLock;

/// A physical semiconductor device with spatial discretization mesh and PDD solver.
#[derive(Debug)]
pub struct TcadDevice {
    pub name: String,
    pub material: SemiconductorMaterial,
    pub mesh: Mesh1D,
    pub cross_section_area: f64, // m^2
    pub solver: PoissonDriftDiffusionSolver,
    pub state_cache: RwLock<Option<TcadState1D>>,
    pub is_mosfet: bool,
}

impl Clone for TcadDevice {
    fn clone(&self) -> Self {
        let cached = self.state_cache.read().ok().and_then(|g| g.clone());
        Self {
            name: self.name.clone(),
            material: self.material.clone(),
            mesh: self.mesh.clone(),
            cross_section_area: self.cross_section_area,
            solver: self.solver.clone(),
            state_cache: RwLock::new(cached),
            is_mosfet: self.is_mosfet,
        }
    }
}

impl TcadDevice {
    /// Creates a new TCAD device from a mesh and material properties.
    pub fn new(
        name: &str,
        material: SemiconductorMaterial,
        mesh: Mesh1D,
        cross_section_area: f64,
    ) -> Self {
        Self {
            name: name.to_string(),
            material,
            mesh,
            cross_section_area,
            solver: PoissonDriftDiffusionSolver::new(),
            state_cache: RwLock::new(None),
            is_mosfet: false,
        }
    }

    /// Evaluates 2-terminal diode I-V characteristics: returns (I_D, g_D).
    /// I_D: current entering anode in Amperes.
    /// g_D: differential conductance dI_D/dV_D in Siemens.
    pub fn evaluate_diode(&self, v_d: f64, temp_k: f64) -> (f64, f64) {
        let mat = self.material.properties();
        let mut state = if let Ok(guard) = self.state_cache.read() {
            if let Some(ref st) = *guard {
                st.clone()
            } else {
                self.solver.initialize_equilibrium(&self.mesh, &mat, temp_k)
            }
        } else {
            self.solver.initialize_equilibrium(&self.mesh, &mat, temp_k)
        };

        let contact_biases = [v_d, 0.0];
        let _ = self
            .solver
            .solve(&self.mesh, &mat, temp_k, &mut state, &contact_biases);
        if let Ok(mut guard) = self.state_cache.write() {
            *guard = Some(state.clone());
        }

        let (j_n, j_p) = PoissonDriftDiffusionSolver::calculate_current_densities(
            &self.mesh, &mat, temp_k, &state,
        );

        // Terminal current entering anode (interface 0)
        let j_total = if !j_n.is_empty() {
            -(j_n[0] + j_p[0])
        } else {
            0.0
        };
        let i_d = self.cross_section_area * j_total;

        // Perturbation for small-signal conductance dI/dV
        let delta_v = 1.0e-3; // 1 mV
        let mut state_pert = state;
        let pert_biases = [v_d + delta_v, 0.0];
        let _ = self
            .solver
            .solve(&self.mesh, &mat, temp_k, &mut state_pert, &pert_biases);
        let (j_n_pert, j_p_pert) = PoissonDriftDiffusionSolver::calculate_current_densities(
            &self.mesh,
            &mat,
            temp_k,
            &state_pert,
        );
        let j_total_pert = if !j_n_pert.is_empty() {
            -(j_n_pert[0] + j_p_pert[0])
        } else {
            0.0
        };
        let i_d_pert = self.cross_section_area * j_total_pert;

        let g_d = ((i_d_pert - i_d) / delta_v).max(1e-14);

        (i_d, g_d)
    }

    /// Evaluates 4-terminal lateral MOSFET characteristics: returns (I_DS, g_m, g_ds, g_mbs).
    pub fn evaluate_mosfet(
        &self,
        v_ds: f64,
        v_gs: f64,
        v_bs: f64,
        temp_k: f64,
    ) -> (f64, f64, f64, f64) {
        let mat = self.material.properties();
        let mut state = if let Ok(guard) = self.state_cache.read() {
            if let Some(ref st) = *guard {
                st.clone()
            } else {
                self.solver.initialize_equilibrium(&self.mesh, &mat, temp_k)
            }
        } else {
            self.solver.initialize_equilibrium(&self.mesh, &mat, temp_k)
        };

        // Terminal order: [Source=0, Gate=v_gs, Drain=v_ds, Bulk=v_bs]
        let base_biases = [0.0, v_gs, v_ds, v_bs];
        let _ = self
            .solver
            .solve(&self.mesh, &mat, temp_k, &mut state, &base_biases);
        if let Ok(mut guard) = self.state_cache.write() {
            *guard = Some(state.clone());
        }

        let (j_n, j_p) = PoissonDriftDiffusionSolver::calculate_current_densities(
            &self.mesh, &mat, temp_k, &state,
        );
        let last_idx = j_n.len().saturating_sub(1);
        let j_drain = if !j_n.is_empty() {
            -(j_n[last_idx] + j_p[last_idx])
        } else {
            0.0
        };
        let i_ds = (self.cross_section_area * j_drain).max(0.0);

        let delta_v = 1.0e-3;

        // Perturb V_GS -> g_m
        let mut state_g = state.clone();
        let _ = self.solver.solve(
            &self.mesh,
            &mat,
            temp_k,
            &mut state_g,
            &[0.0, v_gs + delta_v, v_ds, v_bs],
        );
        let (j_ng, j_pg) = PoissonDriftDiffusionSolver::calculate_current_densities(
            &self.mesh, &mat, temp_k, &state_g,
        );
        let i_ds_g = (self.cross_section_area * -(j_ng[last_idx] + j_pg[last_idx])).max(0.0);
        let g_m = ((i_ds_g - i_ds) / delta_v).max(1e-14);

        // Perturb V_DS -> g_ds
        let mut state_d = state.clone();
        let _ = self.solver.solve(
            &self.mesh,
            &mat,
            temp_k,
            &mut state_d,
            &[0.0, v_gs, v_ds + delta_v, v_bs],
        );
        let (j_nd, j_pd) = PoissonDriftDiffusionSolver::calculate_current_densities(
            &self.mesh, &mat, temp_k, &state_d,
        );
        let i_ds_d = (self.cross_section_area * -(j_nd[last_idx] + j_pd[last_idx])).max(0.0);
        let g_ds = ((i_ds_d - i_ds) / delta_v).max(1e-14);

        // Perturb V_BS -> g_mbs
        let mut state_b = state;
        let _ = self.solver.solve(
            &self.mesh,
            &mat,
            temp_k,
            &mut state_b,
            &[0.0, v_gs, v_ds, v_bs + delta_v],
        );
        let (j_nb, j_pb) = PoissonDriftDiffusionSolver::calculate_current_densities(
            &self.mesh, &mat, temp_k, &state_b,
        );
        let i_ds_b = (self.cross_section_area * -(j_nb[last_idx] + j_pb[last_idx])).max(0.0);
        let g_mbs = ((i_ds_b - i_ds) / delta_v).abs().max(1e-15);

        (i_ds, g_m, g_ds, g_mbs)
    }
}

/// Fluent builder for constructing physical TCAD devices from first principles.
pub struct TcadDeviceBuilder {
    name: String,
    material: SemiconductorMaterial,
    cross_section_area: f64,
    length: f64,
    num_points: usize,
    // Diode-specific
    p_acceptor_doping: f64,
    n_donor_doping: f64,
    junction_ratio: f64,
    // MOSFET-specific
    is_mosfet: bool,
    oxide_thickness: f64,
    oxide_rel_perm: f64,
    flatband_voltage: f64,
}

impl TcadDeviceBuilder {
    /// Starts building a 1D physical P-N junction diode.
    pub fn new_pn_junction(name: &str) -> Self {
        Self {
            name: name.to_string(),
            material: SemiconductorMaterial::Silicon,
            cross_section_area: 1.0e-8, // 100 um x 100 um = 1e-8 m^2
            length: 2.0e-6,             // 2 um length
            num_points: 50,
            p_acceptor_doping: 1.0e23, // 1e17 cm^-3 = 1e23 m^-3
            n_donor_doping: 1.0e22,    // 1e16 cm^-3 = 1e22 m^-3
            junction_ratio: 0.5,
            is_mosfet: false,
            oxide_thickness: 2.0e-9,
            oxide_rel_perm: 3.9,
            flatband_voltage: 0.0,
        }
    }

    /// Starts building a physical lateral MOSFET structure.
    pub fn new_mosfet(name: &str) -> Self {
        Self {
            name: name.to_string(),
            material: SemiconductorMaterial::Silicon,
            cross_section_area: 1.0e-12, // 1 um width x 1 um depth
            length: 300.0e-9,            // 300 nm length (source + channel + drain)
            num_points: 60,
            p_acceptor_doping: 1.0e23, // P-substrate channel: 1e17 cm^-3
            n_donor_doping: 1.0e26,    // N+ Source/Drain: 1e20 cm^-3
            junction_ratio: 0.33,
            is_mosfet: true,
            oxide_thickness: 3.0e-9, // 3 nm SiO2 gate oxide
            oxide_rel_perm: 3.9,
            flatband_voltage: -0.1,
        }
    }

    pub fn material(mut self, mat: SemiconductorMaterial) -> Self {
        self.material = mat;
        self
    }

    /// Sets the material to a first-principles ChemicalMaterial, automatically extracting
    /// permittivity, gate dielectric thickness/properties, and contact flatband voltage.
    pub fn chemical_material(mut self, chem: crate::chemistry::ChemicalMaterial) -> Self {
        if let Some((ref dielectric, thickness)) = chem.dielectric {
            self.oxide_thickness = thickness;
            self.oxide_rel_perm = dielectric.relative_permittivity;
        }
        if let Some(ref contact) = chem.contact {
            self.flatband_voltage =
                contact.flatband_offset_v(&chem.bandstructure, 0.0, phonon_core::constants::T_REF);
        }
        self.material = SemiconductorMaterial::Chemical(Box::new(chem));
        self
    }

    pub fn length(mut self, length: f64) -> Self {
        self.length = length;
        self
    }

    pub fn cross_section_area(mut self, area: f64) -> Self {
        self.cross_section_area = area;
        self
    }

    pub fn mesh_points(mut self, n: usize) -> Self {
        self.num_points = n;
        self
    }

    pub fn p_doping(mut self, na: f64) -> Self {
        self.p_acceptor_doping = na;
        self
    }

    pub fn n_doping(mut self, nd: f64) -> Self {
        self.n_donor_doping = nd;
        self
    }

    pub fn oxide_thickness(mut self, tox: f64) -> Self {
        self.oxide_thickness = tox;
        self
    }

    /// Builds the configured TCAD physical device.
    pub fn build(self) -> TcadDevice {
        let mut mesh = Mesh1D::uniform(self.length, self.num_points);

        if !self.is_mosfet {
            // P-N Diode
            let j_pos = self.length * self.junction_ratio;
            mesh.set_pn_step_doping(j_pos, self.p_acceptor_doping, self.n_donor_doping);
            mesh.add_contact("Anode", 0, ContactType::Ohmic);
            mesh.add_contact("Cathode", self.num_points - 1, ContactType::Ohmic);
        } else {
            // Lateral NMOS
            let x_s = self.length * 0.25;
            let x_d = self.length * 0.75;
            mesh.set_mosfet_lateral_doping(x_s, x_d, self.n_donor_doping, self.p_acceptor_doping);
            let mid_node = self.num_points / 2;

            mesh.add_contact("Source", 0, ContactType::Ohmic);
            mesh.add_contact(
                "Gate",
                mid_node,
                ContactType::Gate {
                    oxide_thickness: self.oxide_thickness,
                    oxide_rel_perm: self.oxide_rel_perm,
                    flatband_voltage: self.flatband_voltage,
                },
            );
            mesh.add_contact("Drain", self.num_points - 1, ContactType::Ohmic);
        }

        let mut dev = TcadDevice::new(&self.name, self.material, mesh, self.cross_section_area);
        dev.is_mosfet = self.is_mosfet;
        dev
    }
}
