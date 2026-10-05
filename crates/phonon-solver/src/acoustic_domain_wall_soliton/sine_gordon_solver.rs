#![deny(unsafe_code)]

use std::f64::consts::PI;

/// Parameters defining the Sine-Gordon non-linear acoustic domain wall lattice.
#[derive(Debug, Clone)]
pub struct SineGordonParams {
    /// Number of spatial grid points along the domain.
    pub grid_points: usize,
    /// Physical length of the 1D acoustic domain in meters.
    pub length_m: f64,
    /// Acoustic sound speed in m/s (c_0).
    pub speed_of_sound: f64,
    /// Characteristic pinning/potential frequency in rad/s (omega_0).
    pub omega_0: f64,
    /// Acoustic damping coefficient in 1/s (gamma).
    pub damping_gamma: f64,
    /// Courant-Friedrichs-Lewy (CFL) stability factor (e.g. 0.35).
    pub cfl_factor: f64,
}

impl Default for SineGordonParams {
    fn default() -> Self {
        Self {
            grid_points: 256,
            length_m: 0.05, // 50 mm
            speed_of_sound: 1500.0, // 1500 m/s
            omega_0: 2.0 * PI * 40_000.0, // 40 kHz characteristic resonance
            damping_gamma: 0.0, // Undamped conservative baseline
            cfl_factor: 0.35,
        }
    }
}

impl SineGordonParams {
    /// Grid spacing dx in meters.
    #[inline]
    pub fn dx(&self) -> f64 {
        self.length_m / (self.grid_points.max(2) - 1) as f64
    }

    /// Characteristic domain wall thickness lambda_dw = c_0 / omega_0 in meters.
    #[inline]
    pub fn domain_wall_width(&self) -> f64 {
        if self.omega_0 > 1e-12 {
            self.speed_of_sound / self.omega_0
        } else {
            1.0
        }
    }

    /// Stable numerical time step dt = CFL * dx / c_0.
    #[inline]
    pub fn stable_dt(&self) -> f64 {
        self.cfl_factor * self.dx() / self.speed_of_sound
    }

    /// Analytical rest energy of a single kink E_0 = 8 * c_0 * omega_0.
    #[inline]
    pub fn kink_rest_energy(&self) -> f64 {
        8.0 * self.speed_of_sound * self.omega_0
    }
}

/// Soliton solution type initialized into the domain.
#[derive(Debug, Clone)]
pub enum SolitonKind {
    /// Single moving kink (topological charge Q = +1).
    MovingKink {
        x0: f64,
        velocity_ratio: f64, // v / c_0 in (-0.95, +0.95)
    },
    /// Single moving antikink (topological charge Q = -1).
    MovingAntikink {
        x0: f64,
        velocity_ratio: f64,
    },
    /// Kink-antikink collision (topological charge Q = 0).
    KinkAntikinkCollision {
        x_left: f64,
        v_left_ratio: f64,
        x_right: f64,
        v_right_ratio: f64,
    },
    /// Bound pulsating breather soliton (topological charge Q = 0).
    Breather {
        x0: f64,
        frequency_ratio: f64, // omega / omega_0 in (0.1, 0.95)
    },
    /// Static domain wall.
    StaticWall {
        x0: f64,
    },
}

/// Dynamic state of the Sine-Gordon acoustic lattice.
#[derive(Debug, Clone)]
pub struct SineGordonState {
    /// Spatial grid coordinates in meters.
    pub x: Vec<f64>,
    /// Phase field / acoustic displacement phi(x, t) in radians.
    pub phi: Vec<f64>,
    /// Time derivative / velocity field dphi/dt in rad/s.
    pub dphi_dt: Vec<f64>,
    /// Local Hamiltonian energy density H(x) in J/m.
    pub energy_density: Vec<f64>,
    /// Total integrated Hamiltonian energy in Joules.
    pub total_energy: f64,
    /// Initial total energy for drift evaluation.
    pub initial_energy: f64,
    /// Topological winding charge Q = (phi(L) - phi(0)) / (2*pi).
    pub topological_charge: f64,
    /// Elapsed simulation time in seconds.
    pub time_s: f64,
    /// Active soliton preset kind.
    pub kind: SolitonKind,
}

/// High-accuracy 4th-order Runge-Kutta numerical solver for the Sine-Gordon PDE.
#[derive(Debug, Clone)]
pub struct SineGordonSolver {
    pub params: SineGordonParams,
    pub state: SineGordonState,
}

impl SineGordonSolver {
    /// Construct a new solver with default parameters and a moving kink preset.
    pub fn new(params: SineGordonParams, kind: SolitonKind) -> Self {
        let n = params.grid_points;
        let dx = params.dx();
        let mut x = Vec::with_capacity(n);
        for i in 0..n {
            x.push(i as f64 * dx - params.length_m * 0.5);
        }

        let mut solver = Self {
            params,
            state: SineGordonState {
                x,
                phi: vec![0.0; n],
                dphi_dt: vec![0.0; n],
                energy_density: vec![0.0; n],
                total_energy: 0.0,
                initial_energy: 0.0,
                topological_charge: 0.0,
                time_s: 0.0,
                kind: kind.clone(),
            },
        };

        solver.initialize_soliton(kind);
        solver
    }

    /// Initialize the acoustic phase field according to the specified soliton kind.
    pub fn initialize_soliton(&mut self, kind: SolitonKind) {
        self.state.kind = kind.clone();
        self.state.time_s = 0.0;
        let c0 = self.params.speed_of_sound;
        let omega0 = self.params.omega_0;
        let lambda_dw = self.params.domain_wall_width();
        let _n = self.params.grid_points;

        self.state.phi.clear();
        self.state.dphi_dt.clear();

        match kind {
            SolitonKind::MovingKink { x0, velocity_ratio } => {
                let u = velocity_ratio.clamp(-0.95, 0.95);
                let v = u * c0;
                let gamma_l = 1.0 / (1.0 - u * u).sqrt();
                let width = lambda_dw / gamma_l;

                for &xi in &self.state.x {
                    let arg = (xi - x0) / width;
                    let exp_val = arg.exp();
                    let phi_val = 4.0 * exp_val.atan();
                    let sech_val = 1.0 / arg.cosh();
                    let dphi_val = -2.0 * v / width * sech_val;

                    self.state.phi.push(phi_val);
                    self.state.dphi_dt.push(dphi_val);
                }
            }
            SolitonKind::MovingAntikink { x0, velocity_ratio } => {
                let u = velocity_ratio.clamp(-0.95, 0.95);
                let v = u * c0;
                let gamma_l = 1.0 / (1.0 - u * u).sqrt();
                let width = lambda_dw / gamma_l;

                for &xi in &self.state.x {
                    let arg = -(xi - x0) / width;
                    let exp_val = arg.exp();
                    let phi_val = 4.0 * exp_val.atan();
                    let sech_val = 1.0 / arg.cosh();
                    let dphi_val = 2.0 * v / width * sech_val;

                    self.state.phi.push(phi_val);
                    self.state.dphi_dt.push(dphi_val);
                }
            }
            SolitonKind::KinkAntikinkCollision {
                x_left,
                v_left_ratio,
                x_right,
                v_right_ratio,
            } => {
                let u_l = v_left_ratio.clamp(-0.95, 0.95);
                let v_l = u_l * c0;
                let gamma_l = 1.0 / (1.0 - u_l * u_l).sqrt();
                let width_l = lambda_dw / gamma_l;

                let u_r = v_right_ratio.clamp(-0.95, 0.95);
                let v_r = u_r * c0;
                let gamma_r = 1.0 / (1.0 - u_r * u_r).sqrt();
                let width_r = lambda_dw / gamma_r;

                for &xi in &self.state.x {
                    let arg_kink = (xi - x_left) / width_l;
                    let phi_k = 4.0 * arg_kink.exp().atan();
                    let dphi_k = -2.0 * v_l / width_l / arg_kink.cosh();

                    let arg_anti = -(xi - x_right) / width_r;
                    let phi_a = 4.0 * arg_anti.exp().atan();
                    let dphi_a = 2.0 * v_r / width_r / arg_anti.cosh();

                    // Superposition of kink (0 -> 2pi) and antikink (2pi -> 0)
                    let phi_val = phi_k + phi_a - 2.0 * PI;
                    let dphi_val = dphi_k + dphi_a;

                    self.state.phi.push(phi_val);
                    self.state.dphi_dt.push(dphi_val);
                }
            }
            SolitonKind::Breather { x0, frequency_ratio } => {
                let om_ratio = frequency_ratio.clamp(0.1, 0.95);
                let eta = (1.0 - om_ratio * om_ratio).sqrt();
                let width = lambda_dw / eta;

                for &xi in &self.state.x {
                    let arg_x = (xi - x0) / width;
                    // At t = 0, sin(omega * t) = 0, so phi = 0
                    // dphi/dt = 4 * omega * eta / (cosh(arg_x) * om_ratio)
                    let phi_val = 0.0;
                    let dphi_val = 4.0 * (om_ratio * omega0) * eta / (arg_x.cosh() * om_ratio);

                    self.state.phi.push(phi_val);
                    self.state.dphi_dt.push(dphi_val);
                }
            }
            SolitonKind::StaticWall { x0 } => {
                let width = lambda_dw;
                for &xi in &self.state.x {
                    let arg = (xi - x0) / width;
                    let phi_val = 4.0 * arg.exp().atan();
                    self.state.phi.push(phi_val);
                    self.state.dphi_dt.push(0.0);
                }
            }
        }

        self.recompute_metrics();
        self.state.initial_energy = self.state.total_energy;
    }

    /// Acceleration evaluation: d^2 phi / dt^2 = c_0^2 d^2 phi / dx^2 - omega_0^2 sin(phi) - gamma dphi/dt.
    fn compute_acceleration(
        &self,
        phi: &[f64],
        dphi_dt: &[f64],
        acc: &mut [f64],
    ) {
        let n = phi.len();
        let dx = self.params.dx();
        let c0 = self.params.speed_of_sound;
        let c0_sq = c0 * c0;
        let inv_dx_sq = 1.0 / (dx * dx);
        let omega0_sq = self.params.omega_0 * self.params.omega_0;
        let gamma = self.params.damping_gamma;

        // Interior points
        for i in 1..n - 1 {
            let laplacian = (phi[i + 1] - 2.0 * phi[i] + phi[i - 1]) * inv_dx_sq;
            let potential_force = -omega0_sq * phi[i].sin();
            let damping = -gamma * dphi_dt[i];
            acc[i] = c0_sq * laplacian + potential_force + damping;
        }

        // Neumann / Zero-curvature boundaries
        if n > 2 {
            // Left boundary
            let lap_0 = 2.0 * (phi[1] - phi[0]) * inv_dx_sq;
            acc[0] = c0_sq * lap_0 - omega0_sq * phi[0].sin() - gamma * dphi_dt[0];

            // Right boundary
            let lap_n = 2.0 * (phi[n - 2] - phi[n - 1]) * inv_dx_sq;
            acc[n - 1] = c0_sq * lap_n - omega0_sq * phi[n - 1].sin() - gamma * dphi_dt[n - 1];
        }
    }

    /// Single time integration step using 4th-order Runge-Kutta (RK4).
    pub fn step(&mut self, dt: f64) {
        let n = self.params.grid_points;

        let mut k1_phi = vec![0.0; n];
        let mut k1_v = vec![0.0; n];
        let mut k2_phi = vec![0.0; n];
        let mut k2_v = vec![0.0; n];
        let mut k3_phi = vec![0.0; n];
        let mut k3_v = vec![0.0; n];
        let mut k4_phi = vec![0.0; n];
        let mut k4_v = vec![0.0; n];

        let mut tmp_phi = vec![0.0; n];
        let mut tmp_v = vec![0.0; n];

        // Stage 1
        for i in 0..n {
            k1_phi[i] = self.state.dphi_dt[i];
        }
        self.compute_acceleration(&self.state.phi, &self.state.dphi_dt, &mut k1_v);

        // Stage 2
        let dt_half = 0.5 * dt;
        for i in 0..n {
            tmp_phi[i] = self.state.phi[i] + dt_half * k1_phi[i];
            tmp_v[i] = self.state.dphi_dt[i] + dt_half * k1_v[i];
            k2_phi[i] = tmp_v[i];
        }
        self.compute_acceleration(&tmp_phi, &tmp_v, &mut k2_v);

        // Stage 3
        for i in 0..n {
            tmp_phi[i] = self.state.phi[i] + dt_half * k2_phi[i];
            tmp_v[i] = self.state.dphi_dt[i] + dt_half * k2_v[i];
            k3_phi[i] = tmp_v[i];
        }
        self.compute_acceleration(&tmp_phi, &tmp_v, &mut k3_v);

        // Stage 4
        for i in 0..n {
            tmp_phi[i] = self.state.phi[i] + dt * k3_phi[i];
            tmp_v[i] = self.state.dphi_dt[i] + dt * k3_v[i];
            k4_phi[i] = tmp_v[i];
        }
        self.compute_acceleration(&tmp_phi, &tmp_v, &mut k4_v);

        // Update state
        let dt_sixth = dt / 6.0;
        for i in 0..n {
            self.state.phi[i] += dt_sixth * (k1_phi[i] + 2.0 * k2_phi[i] + 2.0 * k3_phi[i] + k4_phi[i]);
            self.state.dphi_dt[i] += dt_sixth * (k1_v[i] + 2.0 * k2_v[i] + 2.0 * k3_v[i] + k4_v[i]);
        }

        self.state.time_s += dt;
        self.recompute_metrics();
    }

    /// Advance solver by `steps` internal stable time steps.
    pub fn step_n(&mut self, steps: usize) {
        let dt = self.params.stable_dt();
        for _ in 0..steps {
            self.step(dt);
        }
    }

    /// Recompute energy density, integrated total energy, and topological charge.
    pub fn recompute_metrics(&mut self) {
        let n = self.params.grid_points;
        let dx = self.params.dx();
        let c0_sq = self.params.speed_of_sound * self.params.speed_of_sound;
        let omega0_sq = self.params.omega_0 * self.params.omega_0;
        let inv_2dx = 1.0 / (2.0 * dx);

        self.state.energy_density.resize(n, 0.0);
        let mut total_e = 0.0;

        for i in 0..n {
            let kinetic = 0.5 * self.state.dphi_dt[i] * self.state.dphi_dt[i];
            let gradient = if i == 0 {
                (self.state.phi[1] - self.state.phi[0]) / dx
            } else if i == n - 1 {
                (self.state.phi[n - 1] - self.state.phi[n - 2]) / dx
            } else {
                (self.state.phi[i + 1] - self.state.phi[i - 1]) * inv_2dx
            };
            let elastic = 0.5 * c0_sq * gradient * gradient;
            let potential = omega0_sq * (1.0 - self.state.phi[i].cos());

            let density = kinetic + elastic + potential;
            self.state.energy_density[i] = density;
            total_e += density * dx;
        }

        self.state.total_energy = total_e;

        // Topological winding charge Q = (phi[N-1] - phi[0]) / (2 * pi)
        if n >= 2 {
            let delta_phi = self.state.phi[n - 1] - self.state.phi[0];
            self.state.topological_charge = delta_phi / (2.0 * PI);
        } else {
            self.state.topological_charge = 0.0;
        }
    }

    /// Relative energy drift compared to initial energy: |E(t) - E_0| / E_0.
    pub fn relative_energy_drift(&self) -> f64 {
        if self.state.initial_energy.abs() > 1e-12 {
            (self.state.total_energy - self.state.initial_energy).abs() / self.state.initial_energy
        } else {
            0.0
        }
    }
}
