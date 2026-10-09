#![deny(unsafe_code)]

//! Phase 462: Topological Moire Superlattice Flat-Band Acoustic Polariton Laser & Chiral Valley Sensor Network.
//!
//! Master coordinator and physics invariant audit engine integrating:
//! 1. Twisted bilayer acoustic moire superlattices with magic-angle flat-band formation (theta_m = 1.08 deg),
//!    group velocity quenching (v_F / v_0 <= 0.05), ultra-narrow flat-band bandwidth (Delta E_flat <= 1.5 MHz),
//!    Van Hove singularity density of states enhancement (rho / rho_0 >= 20.0), and AA-stacking localization (>= 80.0%).
//! 2. Flat-band acoustic polariton laser with macroscopic condensation into k = 0 mode, ultra-low threshold
//!    pump power (P_th <= 2.5 mW), Schawlow-Townes linewidth narrowing (Delta nu <= 15.0 kHz), and
//!    second-order coherence transition (|g^(2)(0) - 1.0| <= 0.05).
//! 3. Distributed chiral valley polariton sensor network with valley polarization isolation (ISO_v >= 35.0 dB),
//!    sub-nanostrain minimum detectable strain (epsilon_min <= 1.0e-8), and low-loss topological edge channel routing (IL <= 0.40 dB).

pub mod moire_superlattice;
pub mod polariton_laser;
pub mod valley_sensor_network;

pub use moire_superlattice::{
    MoireBandDispersionPoint, MoireSpatialProfilePoint, MoireSuperlatticeMetrics,
    MoireSuperlatticeParams, MoireSuperlatticeSolver,
};
pub use polariton_laser::{
    MoireLaserMetrics, MoireLaserParams, MoireLaserSolver, MoireLaserSpectrumPoint,
    PolaritonInputOutputCurvePoint,
};
pub use valley_sensor_network::{
    ValleyNodeSensorPoint, ValleySensorNetworkMetrics, ValleySensorNetworkParams,
    ValleySensorNetworkSolver, ValleyTransmissionSpectrumPoint,
};

/// 10-point rigorous physics audit report for Phase 462.
#[derive(Debug, Clone)]
pub struct MoireSuperlatticeAuditReport {
    /// 1. Magic angle Dirac velocity quenching v_F / v_0 <= 0.05 at theta = 1.08 deg.
    pub magic_angle_flat_band: bool,
    /// 2. Isolated flat-band bandwidth Delta E_flat <= 1.5 MHz.
    pub flat_band_bandwidth_quenching: bool,
    /// 3. Spatial acoustic energy confinement within AA stacking regions >= 80.0%.
    pub aa_site_spatial_confinement: bool,
    /// 4. Van Hove singularity acoustic DOS enhancement factor >= 20.0.
    pub van_hove_dos_enhancement: bool,
    /// 5. Polariton laser threshold pump power P_th <= 2.5 mW.
    pub polariton_laser_threshold: bool,
    /// 6. Second-order coherence transition |g^(2)(0) - 1.0| <= 0.05 above threshold.
    pub coherence_transition: bool,
    /// 7. Schawlow-Townes narrowed emission linewidth Delta nu_laser <= 15.0 kHz.
    pub lasing_linewidth_narrowing: bool,
    /// 8. Valley polarization crosstalk isolation ISO_v >= 35.0 dB.
    pub valley_crosstalk_isolation: bool,
    /// 9. Distributed sensor network strain sensitivity epsilon_min <= 1.0e-8.
    pub distributed_sensor_sensitivity: bool,
    /// 10. Inter-node topological edge channel routing insertion loss IL <= 0.40 dB.
    pub inter_node_insertion_loss: bool,
}

impl MoireSuperlatticeAuditReport {
    /// Returns the (passed_count, total_count) score.
    pub fn score(&self) -> (usize, usize) {
        let items = [
            self.magic_angle_flat_band,
            self.flat_band_bandwidth_quenching,
            self.aa_site_spatial_confinement,
            self.van_hove_dos_enhancement,
            self.polariton_laser_threshold,
            self.coherence_transition,
            self.lasing_linewidth_narrowing,
            self.valley_crosstalk_isolation,
            self.distributed_sensor_sensitivity,
            self.inter_node_insertion_loss,
        ];
        let passed = items.iter().filter(|&&v| v).count();
        (passed, items.len())
    }

    /// Returns true if all 10 physics audit criteria scored PASS.
    pub fn is_pass(&self) -> bool {
        let (passed, total) = self.score();
        passed == total
    }
}

/// Master coordinator processor for Moire Superlattice Polariton Laser & Valley Sensor Network (Phase 462).
#[derive(Debug, Clone)]
pub struct MoireSuperlatticeLaserProcessor {
    pub moire_params: MoireSuperlatticeParams,
    pub laser_params: MoireLaserParams,
    pub sensor_params: ValleySensorNetworkParams,
}

impl MoireSuperlatticeLaserProcessor {
    /// Creates a new master processor instance.
    pub fn new(
        moire_params: MoireSuperlatticeParams,
        laser_params: MoireLaserParams,
        sensor_params: ValleySensorNetworkParams,
    ) -> Self {
        Self {
            moire_params,
            laser_params,
            sensor_params,
        }
    }

    /// Runs a comprehensive 10-point physics audit across all subsystems.
    pub fn audit_system(&self) -> MoireSuperlatticeAuditReport {
        let moire_solver = MoireSuperlatticeSolver::new(self.moire_params.clone());
        let moire_m = moire_solver.evaluate_metrics();

        let laser_solver = MoireLaserSolver::new(self.laser_params.clone());
        let laser_m = laser_solver.evaluate_metrics();

        let sensor_solver = ValleySensorNetworkSolver::new(self.sensor_params.clone());
        let sensor_m = sensor_solver.evaluate_metrics();

        MoireSuperlatticeAuditReport {
            magic_angle_flat_band: moire_m.velocity_quenching_ratio <= 0.05,
            flat_band_bandwidth_quenching: moire_m.flat_band_bandwidth_mhz <= 1.5,
            aa_site_spatial_confinement: moire_m.aa_spatial_confinement_percent >= 80.0,
            van_hove_dos_enhancement: moire_m.dos_enhancement_factor >= 20.0,
            polariton_laser_threshold: laser_m.threshold_pump_power_mw <= 2.5,
            coherence_transition: (laser_m.second_order_coherence_g2 - 1.0).abs() <= 0.05,
            lasing_linewidth_narrowing: laser_m.lasing_linewidth_khz <= 15.0,
            valley_crosstalk_isolation: sensor_m.valley_crosstalk_isolation_db >= 35.0,
            distributed_sensor_sensitivity: sensor_m.minimum_detectable_strain <= 1.0e-8,
            inter_node_insertion_loss: sensor_m.inter_node_insertion_loss_db <= 0.40,
        }
    }
}
