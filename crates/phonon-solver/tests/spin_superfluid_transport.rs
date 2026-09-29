//! Integration tests for Spin Superfluidity & Long-Range Non-Local Spin Transport.

use phonon_models::magnon_bec::{HeavyMetalElectrode, SpinSuperfluidChannel};
use phonon_solver::magnon_bec::SpinSuperfluidTransportSolver;

#[test]
fn test_spin_superfluid_landau_velocity_and_current() {
    let channel = SpinSuperfluidChannel::default();

    let v_c = channel.landau_critical_velocity_m_per_s();
    assert!(
        v_c > 50.0 && v_c < 2000.0,
        "Landau critical velocity should be ~ 100-1000 m/s, got {:.1}",
        v_c
    );

    // Superfluid velocity for moderate phase gradient dphi/dx = 5e4 rad/m
    let dphi_dx = 5.0e4;
    let v_s = channel.superfluid_velocity_m_per_s(dphi_dx);
    assert!(
        v_s > 0.0 && v_s < v_c,
        "v_s should be subcritical, got v_s={:.1}, v_c={:.1}",
        v_s,
        v_c
    );

    // Spin current density
    let j_s = channel.spin_current_density_j_per_m2(dphi_dx);
    assert!(j_s > 0.0, "Spin current density should be positive");
}

#[test]
fn test_long_range_superfluid_vs_diffusive_transport() {
    let channel = SpinSuperfluidChannel::default();
    let electrode = HeavyMetalElectrode::default();
    let solver = SpinSuperfluidTransportSolver::new();

    // At short distance L = 5 um: comparable
    let t_super_short = channel.superfluid_transmission_factor(5.0e-6);
    let t_diff_short = channel.diffusive_transmission_factor(5.0e-6);
    assert!(t_super_short > 0.8 && t_diff_short > 0.3);

    // At long distance L = 50 um (10x lambda_s):
    // Diffusive: exp(-10) ~ 4.5e-5
    // Superfluid: 1 / (1 + 1) = 0.50
    // Advantage > 10,000x!
    let long_channel = SpinSuperfluidChannel {
        channel_length_m: 50.0e-6,
        ..Default::default()
    };

    let res = solver.solve_transport(&long_channel, &electrode, 1.0e-3, 5.0e4);
    assert!(res.is_below_critical_velocity);
    assert!(res.superfluid_transmission > 0.40);
    assert!(res.diffusive_transmission < 1.0e-4);
    assert!(
        res.transmission_advantage > 1000.0,
        "Advantage at 50 um must exceed 1000x, got {:.1}x",
        res.transmission_advantage
    );

    // Non-local detector voltages
    assert!(
        res.non_local_voltage_superfluid_v > 1.0e-9,
        "Superfluid ISHE voltage should be > 1 nV"
    );
    assert!(res.non_local_voltage_superfluid_v > res.non_local_voltage_diffusive_v * 1000.0);
}
