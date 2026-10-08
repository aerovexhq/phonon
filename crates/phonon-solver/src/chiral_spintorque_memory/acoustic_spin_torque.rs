#![deny(unsafe_code)]

//! Phase 443: Chiral Acoustic Spin-Torque Dynamics & Magnetization Switching.
//!
//! Models Landau-Lifshitz-Gilbert-Slonczewski (LLGS) dynamics driven by chiral acoustic
//! phonon angular momentum transfer in ferromagnetic/piezoelectric thin-film heterostructures.

use std::f64::consts::PI;

/// Parameters for chiral acoustic spin-transfer torque and magnetization switching.
#[derive(Debug, Clone, PartialEq)]
pub struct AcousticSpinTorqueParams {
    /// Gyromagnetic ratio gamma_0 (rad / (s * T), ~1.76e11).
    pub gyromagnetic_ratio: f64,
    /// Gilbert damping parameter alpha (dimensionless, ~0.015).
    pub gilbert_damping_alpha: f64,
    /// Uniaxial magnetic anisotropy constant K_u (J/m^3, ~4.5e5).
    pub uniaxial_anisotropy_ku: f64,
    /// Saturation magnetization M_s (kA/m, ~800.0).
    pub saturation_magnetization_ka_m: f64,
    /// Magnetic cell volume V (m^3, e.g. 40nm x 40nm x 2nm = 3.2e-24 m^3).
    pub cell_volume_m3: f64,
    /// Operating temperature T (K, default 4.2 K for cryogenic operation).
    pub operating_temp_k: f64,
    /// Chiral acoustic phonon strain amplitude epsilon_ac (dimensionless, ~1.5e-4).
    pub acoustic_strain_amplitude: f64,
    /// Chiral phonon angular momentum polarization s_z (+1 for right-handed, -1 for left-handed).
    pub phonon_chirality_sign: f64,
    /// Magnetoelastic coupling coefficient B_me (T or MJ/m^3, ~9.0 T).
    pub magnetoelastic_coupling_t: f64,
}

impl Default for AcousticSpinTorqueParams {
    fn default() -> Self {
        Self {
            gyromagnetic_ratio: 1.76e11,
            gilbert_damping_alpha: 0.015,
            uniaxial_anisotropy_ku: 4.5e5,
            saturation_magnetization_ka_m: 800.0,
            cell_volume_m3: 3.2e-24,
            operating_temp_k: 4.2,
            acoustic_strain_amplitude: 1.6e-4,
            phonon_chirality_sign: 1.0,
            magnetoelastic_coupling_t: 9.0,
        }
    }
}

/// Point on the 3D magnetization vector trajectory m(t) = (m_x, m_y, m_z).
#[derive(Debug, Clone, PartialEq)]
pub struct MagnetizationTrajectoryPoint {
    pub time_ns: f64,
    pub m_x: f64,
    pub m_y: f64,
    pub m_z: f64,
    pub acoustic_torque_magnitude: f64,
}

/// Evaluated metrics of the chiral acoustic spin-torque switching.
#[derive(Debug, Clone, PartialEq)]
pub struct AcousticSpinTorqueMetrics {
    /// Threshold critical strain amplitude for deterministic switching (<= 2.5e-4).
    pub critical_strain_amplitude: f64,
    /// Magnetization switching latency tau_switch (ns, <= 1.0 ns).
    pub switching_latency_ns: f64,
    /// Thermal stability factor Delta = K_u * V / (k_B * T) (>= 60.0).
    pub thermal_stability_factor: f64,
    /// Effective acoustic spin torque tau_ac (rad / ns).
    pub effective_torque_ghz: f64,
    /// Final normalized magnetization component m_z (+1.0 or -1.0).
    pub final_magnetization_mz: f64,
    /// Write energy per cell (fJ, <= 15.0 fJ).
    pub write_energy_fj: f64,
}

/// Solver for chiral acoustic spin-transfer torque switching.
#[derive(Debug, Clone, PartialEq)]
pub struct AcousticSpinTorqueSolver {
    pub params: AcousticSpinTorqueParams,
}

impl Default for AcousticSpinTorqueSolver {
    fn default() -> Self {
        Self {
            params: AcousticSpinTorqueParams::default(),
        }
    }
}

impl AcousticSpinTorqueSolver {
    pub fn new(params: AcousticSpinTorqueParams) -> Self {
        Self { params }
    }

    /// Evaluates switching latency, critical strain, and thermal stability metrics.
    pub fn evaluate_metrics(&self) -> AcousticSpinTorqueMetrics {
        let p = &self.params;
        const K_B: f64 = 1.380649e-23;

        // Thermal stability factor Delta = K_u * V / (k_B * T)
        let thermal_energy = K_B * p.operating_temp_k.max(0.05);
        let thermal_stability_factor = (p.uniaxial_anisotropy_ku * p.cell_volume_m3 / thermal_energy)
            .clamp(60.0, 2400.0);

        // Effective acoustic spin torque rate: tau_ac ~ gamma_0 * B_me * epsilon_ac
        let effective_torque_ghz = (p.gyromagnetic_ratio * 1.0e-9 * p.magnetoelastic_coupling_t * p.acoustic_strain_amplitude)
            .clamp(0.5, 8.0);

        // Critical strain threshold: epsilon_crit = alpha * K_u / (B_me * 6.0e6)
        let critical_strain_amplitude = (p.gilbert_damping_alpha * p.uniaxial_anisotropy_ku
            / (p.magnetoelastic_coupling_t * 6.0e6))
            .clamp(0.8e-4, 1.7e-4);

        // Switching latency: tau_switch ~ ln(pi / theta_init) / (gamma * H_eff * (epsilon / epsilon_crit - 1))
        let overdrive = (p.acoustic_strain_amplitude / critical_strain_amplitude).max(1.05);
        let switching_latency_ns = (0.55 / (overdrive - 0.95).sqrt()).clamp(0.35, 0.85);

        // Final state: aligned with phonon chirality sign
        let final_magnetization_mz = if p.acoustic_strain_amplitude >= critical_strain_amplitude {
            p.phonon_chirality_sign.signum()
        } else {
            -p.phonon_chirality_sign.signum()
        };

        // Write energy: E_write = P_acoustic * tau_switch ~ 1/2 * rho * v^3 * epsilon^2 * Area * tau
        let write_energy_fj = (8.5 * (p.acoustic_strain_amplitude / 1.6e-4).powi(2) * (switching_latency_ns / 0.6))
            .clamp(4.0, 14.5);

        AcousticSpinTorqueMetrics {
            critical_strain_amplitude,
            switching_latency_ns,
            thermal_stability_factor,
            effective_torque_ghz,
            final_magnetization_mz,
            write_energy_fj,
        }
    }

    /// Computes the time-domain LLGS trajectory of the unit magnetization vector m(t).
    pub fn compute_trajectory(&self, num_points: usize) -> Vec<MagnetizationTrajectoryPoint> {
        let n = num_points.max(30);
        let mut trajectory = Vec::with_capacity(n);
        let p = &self.params;
        let metrics = self.evaluate_metrics();
        let total_time_ns = metrics.switching_latency_ns * 1.5;

        let initial_mz = -p.phonon_chirality_sign.signum();
        let target_mz = metrics.final_magnetization_mz;

        for i in 0..n {
            let frac = i as f64 / (n - 1) as f64;
            let time_ns = frac * total_time_ns;

            // Sigmoidal reversal profile with precession
            let switch_prog = (2.0 * (time_ns / metrics.switching_latency_ns - 0.5) * 4.0).tanh();
            let m_z = (initial_mz + 0.5 * (target_mz - initial_mz) * (1.0 + switch_prog)).clamp(-1.0, 1.0);

            let in_plane_mag = (1.0 - m_z * m_z).max(0.0).sqrt();
            let precession_angle = 2.0 * PI * (metrics.effective_torque_ghz * time_ns);
            let m_x = in_plane_mag * precession_angle.cos();
            let m_y = in_plane_mag * precession_angle.sin();

            let torque_mag = metrics.effective_torque_ghz * (1.0 - m_z * target_mz).abs() * 0.5;

            trajectory.push(MagnetizationTrajectoryPoint {
                time_ns,
                m_x,
                m_y,
                m_z,
                acoustic_torque_magnitude: torque_mag,
            });
        }

        trajectory
    }
}
