use phonon_models::moire::{BistritzerMacDonaldModel, MoireLattice, Vector2D};
use phonon_solver::moire::MoireHamiltonianSolver;

#[test]
fn test_moire_lattice_geometry_and_magic_angle() {
    let lattice = MoireLattice::magic_angle();

    // At \u{03b8} = 1.08 deg:
    assert!((lattice.theta_deg - 1.08).abs() < 1e-6);
    // L_M = a_0 / (2 sin(\u{03b8}/2)) \u{2248} 0.246 / (2 * sin(0.54 deg)) \u{2248} 13.05 nm
    assert!(
        lattice.moire_period_nm > 12.8 && lattice.moire_period_nm < 13.3,
        "Expected L_M \u{2248} 13.05 nm, got {:.3} nm",
        lattice.moire_period_nm
    );

    // Verify q-vectors magnitude: |q_1| = |q_2| = |q_3| = k_\u{03b8}
    let k_th = lattice.k_theta_inv_nm;
    assert!(
        (lattice.q_vectors[0].norm() - k_th).abs() < 1e-6,
        "|q1| mismatch"
    );
    assert!(
        (lattice.q_vectors[1].norm() - k_th).abs() < 1e-6,
        "|q2| mismatch"
    );
    assert!(
        (lattice.q_vectors[2].norm() - k_th).abs() < 1e-6,
        "|q3| mismatch"
    );

    // Sum of q-vectors must vanish identically: q1 + q2 + q3 = 0
    let sum_q_x = lattice.q_vectors[0].x + lattice.q_vectors[1].x + lattice.q_vectors[2].x;
    let sum_q_y = lattice.q_vectors[0].y + lattice.q_vectors[1].y + lattice.q_vectors[2].y;
    assert!(
        sum_q_x.abs() < 1e-12,
        "q-vector sum x non-zero: {}",
        sum_q_x
    );
    assert!(
        sum_q_y.abs() < 1e-12,
        "q-vector sum y non-zero: {}",
        sum_q_y
    );
}

#[test]
fn test_dirac_velocity_quenching() {
    let model = BistritzerMacDonaldModel::new_relaxed(1.08);

    // Check dimensionless parameter \u{03b1} = w_AB / (hbar * v_F * k_\u{03b8})
    // For magic angle, \u{03b1} \u{2248} 0.585 \u{2248} 1/\u{221a}3 = 0.577
    assert!(
        model.alpha > 0.50 && model.alpha < 0.65,
        "Expected \u{03b1} \u{2248} 0.58, got {:.4}",
        model.alpha
    );

    // Analytical velocity ratio |1 - 3\u{03b1}^2| / (1 + 6\u{03b1}^2) must be strongly quenched (< 0.05):
    let vf_ratio = model.normalized_fermi_velocity();
    assert!(
        vf_ratio < 0.05,
        "Expected Dirac velocity ratio < 0.05, got {:.4}",
        vf_ratio
    );

    // Numerical gradient test using eigensolver near Dirac point:
    let solver = MoireHamiltonianSolver::new(model.clone());
    let k_dirac = model.lattice.k_m_point();
    let (_vf_num, vf_num_ratio) = solver.calculate_dirac_velocity(k_dirac, 0.0001);

    assert!(
        vf_num_ratio < 0.10,
        "Numerical velocity ratio exceeds 0.10: {:.4}",
        vf_num_ratio
    );
}

#[test]
fn test_flatband_bandwidth_and_remote_isolation() {
    let model = BistritzerMacDonaldModel::new_relaxed(1.08);
    let solver = MoireHamiltonianSolver::new(model.clone());

    // Generate high-symmetry momentum points along \u{0393} - K_M - M_M - \u{0393}:
    let gamma = model.lattice.gamma_point();
    let k_m = model.lattice.k_m_point();
    let m_m = model.lattice.m_m_point();

    let n_pts = 30;
    let mut k_path = Vec::new();

    // \u{0393} -> K_M
    for i in 0..=n_pts {
        let frac = (i as f64) / (n_pts as f64);
        k_path.push(Vector2D::new(
            gamma.x + frac * (k_m.x - gamma.x),
            gamma.y + frac * (k_m.y - gamma.y),
        ));
    }
    // K_M -> M_M
    for i in 0..=n_pts {
        let frac = (i as f64) / (n_pts as f64);
        k_path.push(Vector2D::new(
            k_m.x + frac * (m_m.x - k_m.x),
            k_m.y + frac * (m_m.y - k_m.y),
        ));
    }
    // M_M -> \u{0393}
    for i in 0..=n_pts {
        let frac = (i as f64) / (n_pts as f64);
        k_path.push(Vector2D::new(
            m_m.x + frac * (gamma.x - m_m.x),
            m_m.y + frac * (gamma.y - m_m.y),
        ));
    }

    let bandwidth_ev = solver.calculate_bandwidth_ev(&k_path);
    let bandwidth_mev = bandwidth_ev * 1000.0;

    // Flat-band bandwidth must be < 10 meV:
    assert!(
        bandwidth_mev < 10.0,
        "Bandwidth W = {:.2} meV exceeds 10 meV threshold",
        bandwidth_mev
    );

    // Remote bandgap isolation:
    let sol_gamma = solver.solve(gamma);
    let lower_gap_mev = sol_gamma.lower_remote_gap_ev * 1000.0;
    let upper_gap_mev = sol_gamma.upper_remote_gap_ev * 1000.0;

    assert!(
        lower_gap_mev > 15.0,
        "Lower remote gap {:.2} meV below 15 meV",
        lower_gap_mev
    );
    assert!(
        upper_gap_mev > 15.0,
        "Upper remote gap {:.2} meV below 15 meV",
        upper_gap_mev
    );
}
