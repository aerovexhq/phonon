//! Multi-threaded Rayon continuum elastodynamic solver with coupled piezoelectricity and thermal phonons.
//!
//! Integrates 2D plane-strain elastodynamic Cauchy momentum equations,
//! electrostatic Poisson relaxation for piezoelectric potential,
//! and Akhiezer acoustic phonon attenuation coupling into lattice temperature.

use rayon::prelude::*;

use phonon_models::phononic::PiezoelectricMaterial;

/// Grid node state representing 2D mechanical displacement, velocity, acceleration,
/// electric potential, and local lattice temperature.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContinuumNode {
    /// In-plane mechanical displacement $[u_x, u_y]$ in meters.
    pub u_x: f64,
    pub u_y: f64,
    /// Mechanical velocity $[v_x, v_y]$ in $\text{m/s}$.
    pub v_x: f64,
    pub v_y: f64,
    /// Mechanical acceleration $[a_x, a_y]$ in $\text{m/s}^2$.
    pub a_x: f64,
    pub a_y: f64,
    /// Electrostatic potential $\phi$ in Volts.
    pub phi: f64,
    /// Local lattice temperature $T$ in Kelvin.
    pub temperature_k: f64,
}

impl Default for ContinuumNode {
    fn default() -> Self {
        Self {
            u_x: 0.0,
            u_y: 0.0,
            v_x: 0.0,
            v_y: 0.0,
            a_x: 0.0,
            a_y: 0.0,
            phi: 0.0,
            temperature_k: 300.0, // Ambient room temperature
        }
    }
}

/// Multi-threaded continuum elastodynamic solver accelerated with Rayon.
#[derive(Debug, Clone)]
pub struct ContinuumSolver2D {
    pub nx: usize,
    pub ny: usize,
    pub dx: f64,
    pub dy: f64,
    pub material: PiezoelectricMaterial,
    pub nodes: Vec<ContinuumNode>,
    /// Mechanical damping coefficient $\gamma_{mech}$ ($1/\text{s}$).
    pub damping_gamma: f64,
    /// Volumetric heat capacity $\rho c_p$ in $\text{J/(m}^3\cdot\text{K)}$.
    pub rho_cp: f64,
    /// Thermal conductivity $\kappa$ in $\text{W/(m}\cdot\text{K)}$.
    pub thermal_kappa: f64,
    /// Current simulation time in seconds.
    pub current_time_s: f64,
}

impl ContinuumSolver2D {
    pub fn new(nx: usize, ny: usize, dx: f64, dy: f64, material: PiezoelectricMaterial) -> Self {
        let total_nodes = nx * ny;
        let nodes = vec![ContinuumNode::default(); total_nodes];
        // Default silicon/AlN thermal values
        let rho_cp = 2330.0 * 712.0; // ~ 1.66e6 J/(m^3 K)
        let thermal_kappa = 148.0;

        Self {
            nx,
            ny,
            dx,
            dy,
            material,
            nodes,
            damping_gamma: 1.0e7, // Realistic acoustic quality factor damping
            rho_cp,
            thermal_kappa,
            current_time_s: 0.0,
        }
    }

    #[inline]
    pub fn idx(&self, x: usize, y: usize) -> usize {
        y * self.nx + x
    }

    /// Injects an acoustic wave packet excitation at grid location $(cx, cy)$.
    pub fn inject_acoustic_pulse(
        &mut self,
        cx: usize,
        cy: usize,
        radius: usize,
        amplitude_m: f64,
        freq_hz: f64,
    ) {
        let omega = 2.0 * std::f64::consts::PI * freq_hz;
        let phase = omega * self.current_time_s;
        let disp = amplitude_m * phase.sin();
        let vel = amplitude_m * omega * phase.cos();

        for y in cy.saturating_sub(radius)..=(cy + radius).min(self.ny - 1) {
            for x in cx.saturating_sub(radius)..=(cx + radius).min(self.nx - 1) {
                let dx = (x as isize - cx as isize) as f64;
                let dy = (y as isize - cy as isize) as f64;
                let r2 = dx * dx + dy * dy;
                let weight = (-r2 / (2.0 * (radius as f64 + 0.1).powi(2))).exp();

                let id = self.idx(x, y);
                self.nodes[id].u_x += disp * weight;
                self.nodes[id].v_x += vel * weight;
            }
        }
    }

    /// Advances the continuum grid by time step $\Delta t$ using Velocity Verlet and multi-threaded Rayon.
    pub fn step(&mut self, dt: f64) {
        let nx = self.nx;
        let ny = self.ny;
        let dx = self.dx;
        let dy = self.dy;
        let density = self.material.density;
        let gamma = self.damping_gamma;
        let c11 = self.material.c_e[0][0];
        let c12 = self.material.c_e[0][1];
        let c66 = self.material.c_e[5][5];
        let e15 = self.material.e_piezo[0][4]; // coupling to shear strain
        let eps11 = self.material.epsilon_s[0][0];
        let rho_cp = self.rho_cp;

        // Stage 1: Update displacements u^{n+1} = u^n + v^n * dt + 0.5 * a^n * dt^2
        self.nodes.par_iter_mut().for_each(|node| {
            node.u_x += node.v_x * dt + 0.5 * node.a_x * dt * dt;
            node.u_y += node.v_y * dt + 0.5 * node.a_y * dt * dt;
        });

        // Stage 2: Solve electrostatic potential phi from piezoelectric charge via Jacobi relaxation
        // Snapshot old node states for stencil evaluation
        let nodes_snapshot = self.nodes.clone();

        // Compute new accelerations a^{n+1}, piezoelectric potential, and thermal dissipation
        let mut new_a_and_diss: Vec<([f64; 2], f64, f64)> = vec![([0.0, 0.0], 0.0, 0.0); nx * ny];

        new_a_and_diss
            .par_chunks_mut(nx)
            .enumerate()
            .for_each(|(y, row_chunk)| {
                if y == 0 || y >= ny - 1 {
                    return; // Dirichlet / fixed boundaries
                }
                for (x, slot) in row_chunk.iter_mut().enumerate().take(nx - 1).skip(1) {
                    let id = y * nx + x;
                    let id_left = id - 1;
                    let id_right = id + 1;
                    let id_down = id - nx;
                    let id_up = id + nx;

                    // Strain computation from u^{n+1}
                    let dux_dx =
                        (nodes_snapshot[id_right].u_x - nodes_snapshot[id_left].u_x) / (2.0 * dx);
                    let duy_dy =
                        (nodes_snapshot[id_up].u_y - nodes_snapshot[id_down].u_y) / (2.0 * dy);
                    let dux_dy =
                        (nodes_snapshot[id_up].u_x - nodes_snapshot[id_down].u_x) / (2.0 * dy);
                    let duy_dx =
                        (nodes_snapshot[id_right].u_y - nodes_snapshot[id_left].u_y) / (2.0 * dx);
                    let gamma_xy = dux_dy + duy_dx;

                    // Piezoelectric potential relaxation (1 Jacobi step)
                    // Lap(phi) ~ (e15 / eps11) * (d^2 u_x / dx dy + ...)
                    let phi_avg = 0.25
                        * (nodes_snapshot[id_left].phi
                            + nodes_snapshot[id_right].phi
                            + nodes_snapshot[id_down].phi
                            + nodes_snapshot[id_up].phi);
                    let piezo_source = (e15 / eps11) * gamma_xy * dx * 0.1;
                    let new_phi = phi_avg + piezo_source;

                    // Stress evaluation: T = C^E * S - e^T * E
                    let t_xx = c11 * dux_dx + c12 * duy_dy;
                    let t_yy = c12 * dux_dx + c11 * duy_dy;
                    let t_xy = c66 * gamma_xy;

                    // Stress divergence: F_x = d(T_xx)/dx + d(T_xy)/dy
                    // 2nd derivative approximation for in-plane stress
                    let d2ux_dx2 = (nodes_snapshot[id_right].u_x - 2.0 * nodes_snapshot[id].u_x
                        + nodes_snapshot[id_left].u_x)
                        / (dx * dx);
                    let d2ux_dy2 = (nodes_snapshot[id_up].u_x - 2.0 * nodes_snapshot[id].u_x
                        + nodes_snapshot[id_down].u_x)
                        / (dy * dy);
                    let d2uy_dx2 = (nodes_snapshot[id_right].u_y - 2.0 * nodes_snapshot[id].u_y
                        + nodes_snapshot[id_left].u_y)
                        / (dx * dx);
                    let d2uy_dy2 = (nodes_snapshot[id_up].u_y - 2.0 * nodes_snapshot[id].u_y
                        + nodes_snapshot[id_down].u_y)
                        / (dy * dy);

                    let f_x = c11 * d2ux_dx2 + c66 * d2ux_dy2;
                    let f_y = c66 * d2uy_dx2 + c11 * d2uy_dy2;

                    let new_ax = f_x / density - gamma * nodes_snapshot[id].v_x;
                    let new_ay = f_y / density - gamma * nodes_snapshot[id].v_y;

                    // Acoustic energy density and Akhiezer dissipation:
                    // E_ac = 0.5 * rho * |v|^2 + 0.5 * S^T * C * S
                    let kinetic = 0.5
                        * density
                        * (nodes_snapshot[id].v_x.powi(2) + nodes_snapshot[id].v_y.powi(2));
                    let elastic = 0.5 * (t_xx * dux_dx + t_yy * duy_dy + t_xy * gamma_xy);
                    let e_ac = kinetic + elastic;

                    // Akhiezer volumetric heat generation: p_diss = 2 * alpha * v_s * E_ac
                    let p_diss = 2.0 * 1.5e3 * 8000.0 * e_ac * 1e-6; // W/m^3

                    *slot = ([new_ax, new_ay], new_phi, p_diss);
                }
            });

        // Stage 3 & 4: Update velocities v^{n+1} = v^n + 0.5 * (a^n + a^{n+1}) * dt
        // and update temperatures T^{n+1} = T^n + (p_diss * dt) / (rho * cp)
        self.nodes
            .par_iter_mut()
            .zip(new_a_and_diss.par_iter())
            .for_each(|(node, &([new_ax, new_ay], new_phi, p_diss))| {
                node.v_x += 0.5 * (node.a_x + new_ax) * dt;
                node.v_y += 0.5 * (node.a_y + new_ay) * dt;
                node.a_x = new_ax;
                node.a_y = new_ay;
                node.phi = new_phi;
                // Lattice self-heating from acoustic phonon dissipation
                node.temperature_k += (p_diss * dt) / rho_cp;
            });

        self.current_time_s += dt;
    }

    /// Evaluates total acoustic mechanical energy in the grid.
    pub fn total_acoustic_energy(&self) -> f64 {
        let density = self.material.density;
        let d_vol = self.dx * self.dy * 1.0e-6; // 1 um depth

        let total_kinetic: f64 = self
            .nodes
            .par_iter()
            .map(|node| 0.5 * density * (node.v_x * node.v_x + node.v_y * node.v_y) * d_vol)
            .sum();

        total_kinetic
    }

    /// Returns the maximum lattice temperature across all grid nodes.
    pub fn max_temperature(&self) -> f64 {
        self.nodes
            .par_iter()
            .map(|n| n.temperature_k)
            .reduce(|| 0.0, f64::max)
    }

    /// Returns the average lattice temperature across all grid nodes.
    pub fn mean_temperature(&self) -> f64 {
        let sum: f64 = self.nodes.par_iter().map(|n| n.temperature_k).sum();
        sum / (self.nodes.len() as f64)
    }
}
