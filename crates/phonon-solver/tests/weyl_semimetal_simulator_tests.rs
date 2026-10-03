#![deny(unsafe_code)]

//! Test suite for Phase 336: Universal Topological Dirac & Weyl Semimetal Metamaterial Simulator.
//!
//! Verifies:
//! - Linear conical dispersion around 3D Weyl nodes.
//! - Type-I vs Type-II tilt classification (t < 1.0 vs t > 1.0).
//! - Berry curvature monopole charge and Nielsen-Ninomiya total neutrality.
//! - Surface Fermi arc open boundary termination between projected bulk nodes.
//! - Chiral anomaly quadratic magnetoconductance sigma(B) = sigma_0 + C_chiral * B^2.
//! - Acoustic beam splitting efficiency >= 95% and cross-talk isolation >= 30 dB.

use phonon_solver::weyl_semimetal::{
    ChiralAnomalyTransport, FermiArcSurface, WeylNode, WeylNodeType, WeylSemimetalModel,
};
use std::f64::consts::PI;

#[test]
fn test_linear_conical_dispersion_around_weyl_nodes() {
    let k0 = [0.4, -0.2, 0.1];
    let vf = [1200.0, 1200.0, 1200.0];
    let tilt = [0.0, 0.0, 0.0];
    let node = WeylNode::new(k0, 1, vf, tilt);

    // At the node vertex, energy splitting is zero
    let (e_minus_0, e_plus_0) = node.dispersion_at(k0);
    assert!(e_minus_0.abs() < 1e-12, "Valence band at node must be zero");
    assert!(e_plus_0.abs() < 1e-12, "Conduction band at node must be zero");

    // Test linear conical dispersion along kx, ky, kz
    let delta_k = 0.05;
    let k_x = [k0[0] + delta_k, k0[1], k0[2]];
    let (em_x, ep_x) = node.dispersion_at(k_x);
    let expected_delta_e = vf[0] * delta_k;

    assert!(
        (ep_x - expected_delta_e).abs() < 1e-9,
        "Conduction band must follow linear conical slope"
    );
    assert!(
        (em_x - (-expected_delta_e)).abs() < 1e-9,
        "Valence band must follow linear conical slope"
    );
    assert!(
        ((ep_x - em_x) - 2.0 * expected_delta_e).abs() < 1e-9,
        "Band splitting must be 2 * vf * delta_k"
    );

    // Test anisotropic Fermi velocity
    let vf_aniso = [1500.0, 800.0, 400.0];
    let node_aniso = WeylNode::new(k0, 1, vf_aniso, tilt);
    let k_y = [k0[0], k0[1] + delta_k, k0[2]];
    let (em_y, ep_y) = node_aniso.dispersion_at(k_y);
    assert!(
        ((ep_y - em_y) - 2.0 * vf_aniso[1] * delta_k).abs() < 1e-9,
        "Anisotropic splitting along ky must match vf_y"
    );
}

#[test]
fn test_type_i_vs_type_ii_tilt_classification() {
    let k0 = [0.0, 0.0, 0.0];
    let vf = [1000.0, 1000.0, 1000.0];

    // 1. Moderate tilt: t = 0.35 < 1.0 -> Type-I Weyl node
    let tilt_type1 = [350.0, 0.0, 0.0];
    let node_1 = WeylNode::new(k0, 1, vf, tilt_type1);
    assert!(
        (node_1.tilt_parameter() - 0.35).abs() < 1e-9,
        "Tilt parameter must be 0.35"
    );
    assert_eq!(
        node_1.node_type(),
        WeylNodeType::TypeI,
        "Node with t < 1.0 must be Type-I"
    );

    // 2. Overtilted cone: t = 1.45 > 1.0 -> Type-II Weyl node
    let tilt_type2 = [1450.0, 0.0, 0.0];
    let node_2 = WeylNode::new(k0, 1, vf, tilt_type2);
    assert!(
        (node_2.tilt_parameter() - 1.45).abs() < 1e-9,
        "Tilt parameter must be 1.45"
    );
    assert_eq!(
        node_2.node_type(),
        WeylNodeType::TypeII,
        "Node with t > 1.0 must be Type-II"
    );

    // 3. Multi-axis tilt vector
    let tilt_3d = [600.0, 800.0, 0.0]; // sqrt((0.6)^2 + (0.8)^2) = 1.0 exactly
    let node_crit = WeylNode::new(k0, 1, vf, tilt_3d);
    assert!(
        (node_crit.tilt_parameter() - 1.0).abs() < 1e-9,
        "Critically tilted parameter must be 1.0"
    );
}

#[test]
fn test_berry_curvature_monopole_charge_and_nielsen_ninomiya_neutrality() {
    let k0 = [0.0, 0.0, 0.0];
    let vf = [1.0, 1.0, 1.0];
    let tilt = [0.0, 0.0, 0.0];

    let node_plus = WeylNode::new(k0, 1, vf, tilt);
    let node_minus = WeylNode::new(k0, -1, vf, tilt);

    // Berry curvature vector direction test: Omega(k) = C * k / (2 |k|^3)
    let test_k = [0.2, 0.0, 0.0];
    let omega_plus = node_plus.berry_curvature(test_k);
    let omega_minus = node_minus.berry_curvature(test_k);
    assert!(
        omega_plus[0] > 0.0,
        "Positive chirality must have outward pointing Berry curvature"
    );
    assert!(
        omega_minus[0] < 0.0,
        "Negative chirality must have inward pointing Berry curvature"
    );
    assert!(
        (omega_plus[0] + omega_minus[0]).abs() < 1e-12,
        "Opposite chiralities must have opposite Berry curvature"
    );

    // Numerical Berry flux over enclosing sphere: \oint Omega · dS = 2 * pi * C
    let radius = 0.15;
    let flux_plus = node_plus.berry_flux_sphere(radius, 40, 80);
    let flux_minus = node_minus.berry_flux_sphere(radius, 40, 80);

    let expected_flux_plus = 2.0 * PI;
    let expected_flux_minus = -2.0 * PI;

    assert!(
        (flux_plus - expected_flux_plus).abs() < 0.05,
        "Berry flux over sphere must integrate to +2pi: got {}",
        flux_plus
    );
    assert!(
        (flux_minus - expected_flux_minus).abs() < 0.05,
        "Berry flux over sphere must integrate to -2pi: got {}",
        flux_minus
    );

    // Nielsen-Ninomiya theorem validation for multi-node metamaterial models
    let model_2node = WeylSemimetalModel::new_trs_broken_pair(0.8, vf, tilt, 1.0);
    assert_eq!(model_2node.nodes.len(), 2);
    assert_eq!(model_2node.total_chirality(), 0);
    assert!(
        model_2node.satisfies_nielsen_ninomiya(),
        "2-node model must satisfy Nielsen-Ninomiya neutrality"
    );

    let model_4node = WeylSemimetalModel::new_inversion_broken_quad(0.8, vf, tilt, 1.0);
    assert_eq!(model_4node.nodes.len(), 4);
    assert_eq!(model_4node.total_chirality(), 0);
    assert!(
        model_4node.satisfies_nielsen_ninomiya(),
        "4-node model must satisfy Nielsen-Ninomiya neutrality"
    );

    let model_dirac = WeylSemimetalModel::new_dirac_semimetal(0.8, vf, 1.0);
    assert_eq!(model_dirac.nodes.len(), 4);
    assert_eq!(model_dirac.total_chirality(), 0);
    assert!(
        model_dirac.satisfies_nielsen_ninomiya(),
        "Dirac model must satisfy Nielsen-Ninomiya neutrality"
    );
}

#[test]
fn test_fermi_arc_open_boundary_termination_between_projected_nodes() {
    let w_plus = [0.55, 0.0];
    let w_minus = [-0.55, 0.0];
    let curvature = 0.22;
    let xi_0 = 1.2;
    let lattice_a = 1.0;

    let surface = FermiArcSurface::new(w_plus, w_minus, curvature, xi_0, lattice_a);
    let trajectory = surface.generate_arc_trajectory(100);

    assert_eq!(trajectory.len(), 100);

    // Confirm open contour (first != last)
    assert!(
        surface.is_open_contour(&trajectory),
        "Fermi arc must form an open contour, distinct from a closed Fermi surface"
    );

    // Confirm abrupt termination at projected bulk Weyl nodes
    assert!(
        surface.confirms_abrupt_termination(&trajectory, 1e-9),
        "Fermi arc must terminate abruptly at projected Weyl points W+ and W-"
    );

    // First point must be W+
    let first = &trajectory[0];
    assert!((first.kx - w_plus[0]).abs() < 1e-9);
    assert!((first.ky - w_plus[1]).abs() < 1e-9);

    // Last point must be W-
    let last = &trajectory[trajectory.len() - 1];
    assert!((last.kx - w_minus[0]).abs() < 1e-9);
    assert!((last.ky - w_minus[1]).abs() < 1e-9);

    // Test surface state exponential penetration
    let z = 2.4;
    let xi = 1.2;
    let psi_0 = surface.surface_wavefunction(0.0, xi);
    let psi_z = surface.surface_wavefunction(z, xi);
    let expected_ratio = (-z / xi).exp();

    assert!(
        ((psi_z / psi_0) - expected_ratio).abs() < 1e-9,
        "Surface wavefunction must decay exponentially into the bulk: psi(z) ~ exp(-z / xi)"
    );

    // Integrated probability density over large z must approach unity
    let integrated = surface.integrate_probability_density(xi, 10.0 * xi);
    assert!(
        (integrated - 1.0).abs() < 1e-6,
        "Integrated probability density must normalize to 1"
    );
}

#[test]
fn test_chiral_anomaly_quadratic_magnetoconductance() {
    let sigma_0 = 1.25;
    let c_chiral = 0.20;
    let transport = ChiralAnomalyTransport::new(sigma_0, c_chiral, 25.0, 0.4);

    // At zero magnetic field, sigma(0) = sigma_0
    let sigma_zero = transport.magnetoconductance(0.0, 0.0);
    assert!(
        (sigma_zero - sigma_0).abs() < 1e-12,
        "Zero-field conductance must equal sigma_0"
    );

    // For parallel fields (theta = 0), test quadratic enhancement: sigma(B) = sigma_0 + C_chiral * B^2
    let b1 = 2.0;
    let b2 = 4.0;
    let sigma_b1 = transport.magnetoconductance(b1, 0.0);
    let sigma_b2 = transport.magnetoconductance(b2, 0.0);

    let expected_b1 = sigma_0 + c_chiral * (b1 * b1);
    let expected_b2 = sigma_0 + c_chiral * (b2 * b2);

    assert!(
        (sigma_b1 - expected_b1).abs() < 1e-12,
        "Conductance at B=2 T must match quadratic prediction"
    );
    assert!(
        (sigma_b2 - expected_b2).abs() < 1e-12,
        "Conductance at B=4 T must match quadratic prediction"
    );

    // Quadratic scaling test: delta_sigma(4T) / delta_sigma(2T) must be (4/2)^2 = 4.0
    let delta_1 = sigma_b1 - sigma_0;
    let delta_2 = sigma_b2 - sigma_0;
    assert!(
        ((delta_2 / delta_1) - 4.0).abs() < 1e-12,
        "Magnetoconductance enhancement must be strictly quadratic in B"
    );

    // Negative longitudinal magnetoresistance: rho(B) = 1 / sigma(B) decreases with B
    let rho_zero = transport.magnetoresistance(0.0, 0.0);
    let rho_b1 = transport.magnetoresistance(b1, 0.0);
    let rho_b2 = transport.magnetoresistance(b2, 0.0);

    assert!(
        rho_b1 < rho_zero,
        "Longitudinal magnetoresistance must decrease with magnetic field (NLMR)"
    );
    assert!(
        rho_b2 < rho_b1,
        "Longitudinal magnetoresistance must continue decreasing quadratically"
    );

    // Transverse configuration: E perpendicular to B (theta = pi/2) cancels chiral pumping
    let sigma_perp = transport.magnetoconductance(b2, PI * 0.5);
    assert!(
        (sigma_perp - sigma_0).abs() < 1e-12,
        "Chiral anomaly enhancement must vanish when E is perpendicular to B"
    );
}

#[test]
fn test_acoustic_beam_splitting_efficiency_and_isolation() {
    let transport = ChiralAnomalyTransport::default();
    let b_field = 3.5;

    // 1. Chiral valley +1: routed into Port 1
    let result_plus = transport.route_chiral_beam(1, b_field);
    assert!(
        result_plus.port1_transmission >= 0.95,
        "Port 1 transmission efficiency {} must be >= 95%",
        result_plus.port1_transmission
    );
    assert!(
        result_plus.isolation_db >= 30.0,
        "Cross-talk isolation {} dB must be >= 30 dB",
        result_plus.isolation_db
    );
    assert!(
        result_plus.crosstalk_transmission <= 0.001,
        "Cross-talk leakage {} must be <= 0.1%",
        result_plus.crosstalk_transmission
    );

    // 2. Chiral valley -1: routed into Port 2
    let result_minus = transport.route_chiral_beam(-1, b_field);
    assert!(
        result_minus.port2_transmission >= 0.95,
        "Port 2 transmission efficiency {} must be >= 95%",
        result_minus.port2_transmission
    );
    assert!(
        result_minus.isolation_db >= 30.0,
        "Cross-talk isolation {} dB must be >= 30 dB",
        result_minus.isolation_db
    );
    assert!(
        result_minus.crosstalk_transmission <= 0.001,
        "Cross-talk leakage {} must be <= 0.1%",
        result_minus.crosstalk_transmission
    );

    // 3. Unpolarized / Dirac mode (chirality 0): 50/50 split
    let result_zero = transport.route_chiral_beam(0, b_field);
    assert!(
        (result_zero.port1_transmission - 0.495).abs() < 0.02,
        "Unpolarized beam must split equally between ports"
    );
    assert!(
        (result_zero.port2_transmission - 0.495).abs() < 0.02,
        "Unpolarized beam must split equally between ports"
    );
}
