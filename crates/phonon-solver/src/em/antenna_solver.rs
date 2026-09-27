//! Antenna Observation Sphere & Near-Field / Far-Field Electrodynamic Solver
//!
//! Evaluates radiated 3D vector fields ($\mathbf{E}$, $\mathbf{H}$, and Poynting vector $\mathbf{S}$)
//! over spherical observation envelopes surrounding discrete transmitters, with multi-threaded Rayon acceleration.
//! Validates total power radiation via numerical integration:
//! $$P_{rad} = \oint_{S} (\mathbf{S} \cdot \hat{\mathbf{n}}) dA$$

use phonon_core::constants::SPEED_OF_LIGHT;
use phonon_models::em::{DiscreteTransmitter, Vector3D, VACUUM_IMPEDANCE};
use rayon::prelude::*;
use std::f64::consts::PI;

/// Observation point on a 3D spherical measurement grid.
#[derive(Debug, Clone, PartialEq)]
pub struct ObservationPoint {
    /// 3D Cartesian position in world space.
    pub position: Vector3D,
    /// Distance from antenna phase center in meters.
    pub radius_m: f64,
    /// Polar elevation angle $\theta$ from $+Z$ in radians.
    pub theta_rad: f64,
    /// Azimuth angle $\phi$ from $+X$ in radians.
    pub phi_rad: f64,
    /// Root-Mean-Square (RMS) electric field magnitude in V/m.
    pub e_field_rms_v_per_m: f64,
    /// Root-Mean-Square (RMS) magnetic field magnitude in A/m.
    pub h_field_rms_a_per_m: f64,
    /// Time-averaged Poynting vector power density magnitude in $\text{W/m}^2$.
    pub poynting_density_w_per_m2: f64,
    /// Radiated electric field vector $\mathbf{E}(\mathbf{r})$.
    pub e_vector: Vector3D,
    /// Radiated magnetic field vector $\mathbf{H}(\mathbf{r})$.
    pub h_vector: Vector3D,
    /// Radiated Poynting power flux vector $\mathbf{S}(\mathbf{r})$.
    pub poynting_vector: Vector3D,
    /// Flag indicating whether point is in Fraunhofer far-field.
    pub is_far_field: bool,
}

/// Comprehensive observation sphere radiation result.
#[derive(Debug, Clone, PartialEq)]
pub struct RadiationSphereResult {
    /// Radius of measurement sphere in meters.
    pub sphere_radius_m: f64,
    /// Number of sampled observation points.
    pub total_points: usize,
    /// Total radiated RF power obtained via surface numerical integration $\oint \mathbf{S} \cdot d\mathbf{A}$ in Watts.
    pub integrated_power_watts: f64,
    /// Antenna input power multiplied by radiation efficiency ($\eta_{rad} P_{in}$) in Watts.
    pub theoretical_radiated_power_watts: f64,
    /// Power conservation relative error $|P_{int} - P_{theo}| / P_{theo}$.
    pub power_conservation_error: f64,
    /// Maximum observed electric field magnitude in V/m.
    pub max_e_field_v_per_m: f64,
    /// Peak observed antenna gain in dBi.
    pub peak_gain_dbi: f64,
    /// All sampled observation point records.
    pub observation_points: Vec<ObservationPoint>,
}

/// High-performance electrodynamic antenna field solver.
#[derive(Debug, Clone, Default)]
pub struct AntennaElectrodynamicSolver;

impl AntennaElectrodynamicSolver {
    /// Creates a new electrodynamic antenna solver.
    pub fn new() -> Self {
        Self
    }

    /// Solves the 3D radiated electromagnetic field vectors across a spherical observation shell
    /// around a discrete transmitter at radius `sphere_radius_m` with polar/azimuth angular resolutions.
    pub fn evaluate_radiation_sphere(
        &self,
        transmitter: &DiscreteTransmitter,
        sphere_radius_m: f64,
        theta_steps: usize,
        phi_steps: usize,
    ) -> RadiationSphereResult {
        let r = sphere_radius_m.max(0.001);
        let n_th = theta_steps.max(8);
        let n_ph = phi_steps.max(16);

        let d_theta = PI / (n_th as f64);
        let d_phi = (2.0 * PI) / (n_ph as f64);

        let far_field_boundary = transmitter.antenna.far_field_distance_m();
        let freq_hz = transmitter.carrier_frequency_hz();
        let wavelength = SPEED_OF_LIGHT / freq_hz;
        let k = 2.0 * PI / wavelength;
        let p_in = transmitter.power_into_antenna_watts();
        let eff = transmitter.antenna.radiation_efficiency();
        let p_rad_theo = p_in * eff;

        // Generate angular coordinates
        let mut angle_pairs = Vec::with_capacity(n_th * n_ph);
        for i in 0..n_th {
            let theta = (i as f64 + 0.5) * d_theta; // Center of annular ring
            for j in 0..n_ph {
                let phi = (j as f64) * d_phi;
                angle_pairs.push((theta, phi));
            }
        }

        // Parallel Rayon evaluation across all observation points
        let observation_points: Vec<ObservationPoint> = angle_pairs
            .par_iter()
            .map(|&(theta, phi)| {
                let sin_th = theta.sin();
                let cos_th = theta.cos();
                let sin_ph = phi.sin();
                let cos_ph = phi.cos();

                // Direction vector from transmitter phase center
                let dir = Vector3D::new(sin_th * cos_ph, sin_th * sin_ph, cos_th);
                let obs_pos = transmitter.position + dir * r;

                // Antenna power gain along this direction
                let gain_lin = transmitter.antenna.gain_linear(theta, phi);

                // Near-field vs far-field field calculation
                let is_far_field = r >= far_field_boundary;

                // In far-field: S = P_in * G(theta, phi) / (4 * pi * r^2)
                // In near-field: Include reactive induction term (1 + (1 / (k*r)^2))
                let kr = k * r;
                let near_field_correction = if is_far_field || kr > 10.0 {
                    1.0
                } else {
                    1.0 + 1.0 / (kr * kr)
                };

                let s_mag = (p_in * gain_lin / (4.0 * PI * r * r)) * near_field_correction;
                let e_rms = (s_mag * VACUUM_IMPEDANCE).sqrt();
                let h_rms = e_rms / VACUUM_IMPEDANCE;

                // Electric field polarization unit vector
                // For Z-oriented dipole: E is in the theta direction
                let e_dir = Vector3D::new(cos_th * cos_ph, cos_th * sin_ph, -sin_th);
                // Magnetic field is in the phi direction: H = dir x E
                let h_dir = Vector3D::new(-sin_ph, cos_ph, 0.0);

                let e_vec = e_dir * e_rms;
                let h_vec = h_dir * h_rms;
                let s_vec = dir * s_mag;

                ObservationPoint {
                    position: obs_pos,
                    radius_m: r,
                    theta_rad: theta,
                    phi_rad: phi,
                    e_field_rms_v_per_m: e_rms,
                    h_field_rms_a_per_m: h_rms,
                    poynting_density_w_per_m2: s_mag,
                    e_vector: e_vec,
                    h_vector: h_vec,
                    poynting_vector: s_vec,
                    is_far_field,
                }
            })
            .collect();

        // Numerical surface integration: P_rad = sum_ij [ S_ij * r^2 * sin(theta_i) * d_theta * d_phi ]
        let mut integrated_p_rad = 0.0;
        let mut max_e_field = 0.0;
        let mut max_gain_lin = 0.0;

        for pt in &observation_points {
            let da = r * r * pt.theta_rad.sin() * d_theta * d_phi;
            integrated_p_rad += pt.poynting_density_w_per_m2 * da;
            if pt.e_field_rms_v_per_m > max_e_field {
                max_e_field = pt.e_field_rms_v_per_m;
            }
            let g = transmitter.antenna.gain_linear(pt.theta_rad, pt.phi_rad);
            if g > max_gain_lin {
                max_gain_lin = g;
            }
        }

        let power_error = if p_rad_theo > 0.0 {
            (integrated_p_rad - p_rad_theo).abs() / p_rad_theo
        } else {
            0.0
        };

        let peak_gain_dbi = 10.0 * max_gain_lin.max(1e-12).log10();

        RadiationSphereResult {
            sphere_radius_m: r,
            total_points: observation_points.len(),
            integrated_power_watts: integrated_p_rad,
            theoretical_radiated_power_watts: p_rad_theo,
            power_conservation_error: power_error,
            max_e_field_v_per_m: max_e_field,
            peak_gain_dbi,
            observation_points,
        }
    }
}
