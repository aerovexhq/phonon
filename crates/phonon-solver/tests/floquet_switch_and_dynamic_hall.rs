//! Integration tests for ultrafast Floquet optical switches, dynamic Hall routing,
//! and transient response.

use phonon_models::floquet_topological::{
    FloquetDiracMaterial, FloquetDriveParams, FloquetOpticalSwitch, OpticalRoutingChannel,
};
use phonon_solver::floquet_topological::FloquetSwitchSolver;

#[test]
fn test_floquet_optical_switch_metrics_and_contrast() {
    let material = FloquetDiracMaterial::default();
    let drive = FloquetDriveParams::new_cw(1.0, 2.5e8, 1.0);
    let switch = FloquetOpticalSwitch::new(drive, material, 1.0e4, 10.0e-15);
    let solver = FloquetSwitchSolver::new(switch);

    let resp = solver.solve_steady_state();

    assert!(
        resp.on_off_contrast_db >= 30.0,
        "Dynamic switching contrast must be >= 30 dB, got {:.2} dB",
        resp.on_off_contrast_db
    );
    assert!(
        resp.switching_frequency_thz >= 50.0,
        "Operational switching frequency must be >= 50 THz, got {:.2} THz",
        resp.switching_frequency_thz
    );
    assert_eq!(resp.routing_channel, OpticalRoutingChannel::ChannelPlusY);
    assert!(resp.hall_current_density_a_m > 0.0);
}

#[test]
fn test_chiral_three_port_optical_routing() {
    let material = FloquetDiracMaterial::default();
    let drive_rcp = FloquetDriveParams::new_cw(1.0, 2.5e8, 1.0);
    let drive_lcp = FloquetDriveParams::new_cw(1.0, 2.5e8, -1.0);
    let drive_lp = FloquetDriveParams::new_cw(1.0, 2.5e8, 0.0);

    let switch_rcp = FloquetOpticalSwitch::new(drive_rcp, material, 1.0e4, 10.0e-15);
    let switch_lcp = FloquetOpticalSwitch::new(drive_lcp, material, 1.0e4, 10.0e-15);
    let switch_lp = FloquetOpticalSwitch::new(drive_lp, material, 1.0e4, 10.0e-15);

    let resp_rcp = switch_rcp.evaluate_response();
    let resp_lcp = switch_lcp.evaluate_response();
    let resp_lp = switch_lp.evaluate_response();

    assert_eq!(
        resp_rcp.routing_channel,
        OpticalRoutingChannel::ChannelPlusY
    );
    assert_eq!(
        resp_lcp.routing_channel,
        OpticalRoutingChannel::ChannelMinusY
    );
    assert_eq!(
        resp_lp.routing_channel,
        OpticalRoutingChannel::ChannelStraightX
    );

    assert!(resp_rcp.hall_current_density_a_m > 0.0);
    assert!(resp_lcp.hall_current_density_a_m < 0.0);
    assert_eq!(resp_lp.hall_current_density_a_m, 0.0);
}

#[test]
fn test_transient_gaussian_pulse_switching() {
    let material = FloquetDiracMaterial::default();
    let drive = FloquetDriveParams::new_gaussian(1.0, 2.5e8, 1.0, 20.0e-15);
    let switch = FloquetOpticalSwitch::new(drive, material, 1.0e4, 10.0e-15);
    let solver = FloquetSwitchSolver::new(switch);

    let transient = solver.solve_transient(-40.0e-15, 40.0e-15, 81);

    // Peak at center (t = 0)
    let mid_idx = 40;
    let mid_pt = &transient[mid_idx];
    assert!((mid_pt.time_s).abs() < 1e-18);
    assert!((mid_pt.envelope - 1.0).abs() < 1e-3);
    assert!(mid_pt.dynamic_gap_mev >= 50.0);
    assert!(mid_pt.contrast_db >= 30.0);

    // At edges (|t| = 40 fs = 2 * FWHM), envelope is negligible
    let edge_pt = &transient[0];
    assert!(edge_pt.envelope < 1e-4);
    assert!(edge_pt.dynamic_gap_mev < 1e-3);
    assert!(edge_pt.contrast_db < 1.0);
}
