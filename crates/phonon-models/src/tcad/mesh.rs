//! 1D/2D spatial discretization meshes, doping profiles, and contact boundary conditions.

use super::material::MaterialProperties;

/// Type of electrical contact boundary condition.
#[derive(Debug, Clone, PartialEq)]
pub enum ContactType {
    /// Ideal Ohmic contact (infinite surface recombination, charge neutrality).
    Ohmic,
    /// Schottky barrier contact with prescribed barrier height (eV).
    Schottky { barrier_height: f64 },
    /// Insulated MOS gate electrode with dielectric oxide.
    Gate {
        oxide_thickness: f64, // meters (e.g. 2e-9 m = 2 nm)
        oxide_rel_perm: f64,  // typically 3.9 for SiO2
        flatband_voltage: f64,
    },
}

/// Boundary electrical contact attached to a mesh point.
#[derive(Debug, Clone, PartialEq)]
pub struct Contact {
    pub name: String,
    pub node_index: usize,
    pub contact_type: ContactType,
}

/// 1D Finite Volume discretization mesh for semiconductor TCAD simulation.
#[derive(Debug, Clone, PartialEq)]
pub struct Mesh1D {
    /// Spatial node coordinates x_i in meters (strictly monotonically increasing).
    pub coords: Vec<f64>,
    /// Donor impurity concentration N_D(x_i) in m^-3.
    pub doping_donor: Vec<f64>,
    /// Acceptor impurity concentration N_A(x_i) in m^-3.
    pub doping_acceptor: Vec<f64>,
    /// Terminal contacts attached to mesh nodes.
    pub contacts: Vec<Contact>,
}

impl Mesh1D {
    /// Generates a uniform 1D mesh spanning [0.0, length] with N points.
    pub fn uniform(length: f64, num_points: usize) -> Self {
        assert!(num_points >= 2, "Mesh must contain at least 2 points");
        assert!(length > 0.0, "Mesh length must be strictly positive");

        let mut coords = Vec::with_capacity(num_points);
        let h = length / (num_points - 1) as f64;
        for i in 0..num_points {
            coords.push(i as f64 * h);
        }

        Self {
            coords,
            doping_donor: vec![0.0; num_points],
            doping_acceptor: vec![0.0; num_points],
            contacts: Vec::new(),
        }
    }

    /// Number of spatial mesh points.
    #[inline]
    pub fn num_points(&self) -> usize {
        self.coords.len()
    }

    /// Cell segment lengths h_i = x_{i+1} - x_i (length N - 1).
    pub fn step_lengths(&self) -> Vec<f64> {
        let n = self.coords.len();
        let mut h = Vec::with_capacity(n - 1);
        for i in 0..(n - 1) {
            let step = self.coords[i + 1] - self.coords[i];
            assert!(step > 0.0, "Mesh coordinates must be strictly increasing");
            h.push(step);
        }
        h
    }

    /// Dual box control volumes for finite volume integration.
    /// V_i = (h_{i-1} + h_i) / 2 for interior nodes.
    pub fn dual_box_volumes(&self) -> Vec<f64> {
        let n = self.coords.len();
        let h = self.step_lengths();
        let mut vol = vec![0.0; n];

        vol[0] = 0.5 * h[0];
        for i in 1..(n - 1) {
            vol[i] = 0.5 * (h[i - 1] + h[i]);
        }
        vol[n - 1] = 0.5 * h[n - 2];

        vol
    }

    /// Net active ionized doping C(x_i) = N_D(x_i) - N_A(x_i) in m^-3.
    #[inline]
    pub fn net_doping(&self, i: usize) -> f64 {
        self.doping_donor[i] - self.doping_acceptor[i]
    }

    /// Adds an electrical contact to a mesh node.
    pub fn add_contact(&mut self, name: &str, node_index: usize, contact_type: ContactType) {
        assert!(
            node_index < self.coords.len(),
            "Contact node index out of range"
        );
        self.contacts.push(Contact {
            name: name.to_string(),
            node_index,
            contact_type,
        });
    }

    /// Sets up a step PN junction profile:
    /// P-doped region for x <= junction_pos, N-doped region for x > junction_pos.
    pub fn set_pn_step_doping(&mut self, junction_pos: f64, p_acceptor: f64, n_donor: f64) {
        for (i, &x) in self.coords.iter().enumerate() {
            if x <= junction_pos {
                self.doping_acceptor[i] = p_acceptor;
                self.doping_donor[i] = 0.0;
            } else {
                self.doping_acceptor[i] = 0.0;
                self.doping_donor[i] = n_donor;
            }
        }
    }

    /// Sets up a 1D lateral MOSFET channel profile:
    /// N+ Source: x in [0, x_s_end] with donor conc n_sd
    /// P- Channel: x in (x_s_end, x_d_start) with acceptor conc p_sub
    /// N+ Drain: x in [x_d_start, L] with donor conc n_sd
    pub fn set_mosfet_lateral_doping(
        &mut self,
        x_s_end: f64,
        x_d_start: f64,
        n_sd: f64,
        p_sub: f64,
    ) {
        for (i, &x) in self.coords.iter().enumerate() {
            if x <= x_s_end || x >= x_d_start {
                self.doping_donor[i] = n_sd;
                self.doping_acceptor[i] = 0.0;
            } else {
                self.doping_donor[i] = 0.0;
                self.doping_acceptor[i] = p_sub;
            }
        }
    }

    /// Calculates thermodynamic equilibrium carrier densities and built-in potential at Ohmic node.
    pub fn ohmic_equilibrium(
        &self,
        node: usize,
        mat: &MaterialProperties,
        temp_k: f64,
    ) -> (f64, f64, f64) {
        let ni = mat.intrinsic_carrier_concentration(temp_k);
        let vt = MaterialProperties::thermal_voltage(temp_k);
        let c = self.net_doping(node);

        let n0 = if c >= 0.0 {
            0.5 * (c + (c * c + 4.0 * ni * ni).sqrt())
        } else {
            2.0 * ni * ni / (-c + (c * c + 4.0 * ni * ni).sqrt())
        };
        let p0 = (ni * ni) / n0;
        let psi_bi = vt * (n0 / ni).ln();

        (psi_bi, n0, p0)
    }
}
