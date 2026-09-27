use phonon_thermal::cauer::CauerNetwork;
use phonon_thermal::foster::FosterNetwork;

#[test]
fn test_cauer_ladder_steady_state() {
    // 3-stage Cauer ladder:
    // Stage 0 (Junction to Die): R_th0 = 0.5 K/W
    // Stage 1 (Die to Case):     R_th1 = 1.0 K/W
    // Stage 2 (Case to Ambient): R_th2 = 2.0 K/W
    // Total R_th,ja = 3.5 K/W
    let mut cauer = CauerNetwork::new();
    cauer.add_stage("Junction_Die", 0.5, 0.001);
    cauer.add_stage("Die_Case", 1.0, 0.01);
    cauer.add_stage("Case_Ambient", 2.0, 0.1);

    assert_eq!(cauer.num_nodes(), 3);
    assert!((cauer.total_thermal_resistance() - 3.5).abs() < 1e-12);

    let ambient_k = 300.0;
    let power_in = 10.0; // 10 Watts injected at junction (node 0)

    let temps = cauer.solve_steady_state(&[power_in], ambient_k);
    assert_eq!(temps.len(), 3);

    // Node 2 (Case): T_2 = T_amb + P * R_th2 = 300 + 10 * 2.0 = 320 K
    assert!((temps[2] - 320.0).abs() < 1e-9);
    // Node 1 (Die):  T_1 = T_2 + P * R_th1 = 320 + 10 * 1.0 = 330 K
    assert!((temps[1] - 330.0).abs() < 1e-9);
    // Node 0 (Junction): T_0 = T_1 + P * R_th0 = 330 + 10 * 0.5 = 335 K
    assert!((temps[0] - 335.0).abs() < 1e-9);
}

#[test]
fn test_foster_step_response_and_cauer_equivalence() {
    // 2-stage Foster model:
    // Stage 1: R1 = 1.0 K/W, tau1 = 0.01 s -> C1 = 0.01 F
    // Stage 2: R2 = 2.0 K/W, tau2 = 0.1 s  -> C2 = 0.05 F
    let mut foster = FosterNetwork::new();
    foster.add_stage(1.0, 0.01);
    foster.add_stage(2.0, 0.05);

    assert!((foster.total_thermal_resistance() - 3.0).abs() < 1e-12);

    let p0 = 5.0; // 5 Watts
                  // At t=0, delta_T = 0
    assert_eq!(foster.step_response_analytical(p0, 0.0), 0.0);

    // At t -> infinity, delta_T = P0 * R_th,total = 5.0 * 3.0 = 15.0 K
    let delta_t_inf = foster.step_response_analytical(p0, 10.0); // 10s >> 0.1s
    assert!((delta_t_inf - 15.0).abs() < 1e-6);

    // Convert Foster to Cauer and test frequency impedance matching
    let cauer = foster.to_cauer();
    assert!((cauer.total_thermal_resistance() - foster.total_thermal_resistance()).abs() < 1e-6);

    // At DC (s=0), both impedances must equal total R_th
    let z_foster_dc = foster.driving_point_impedance(0.0);
    let z_cauer_dc = cauer.driving_point_impedance(0.0);
    assert!((z_foster_dc - 3.0).abs() < 1e-6);
    assert!((z_cauer_dc - 3.0).abs() < 1e-6);

    // At intermediate frequencies (e.g. s = 10 rad/s), impedances must match closely
    let s = 10.0;
    let z_f = foster.driving_point_impedance(s);
    let z_c = cauer.driving_point_impedance(s);
    let rel_err = (z_f - z_c).abs() / z_f;
    assert!(
        rel_err < 0.05,
        "Impedance mismatch at s={s}: Foster={z_f}, Cauer={z_c}, rel_err={rel_err}"
    );
}

#[test]
fn test_cauer_transient_backward_euler() {
    let mut cauer = CauerNetwork::new();
    // 1-stage: R = 2.0 K/W, C = 0.5 J/K -> tau = R*C = 1.0 s
    cauer.add_stage("Die", 2.0, 0.5);

    let ambient_k = 300.0;
    let p_in = 10.0; // 10 Watts -> delta_T_inf = 20 K -> T_inf = 320 K
    let mut temps = vec![ambient_k];

    let dt = 0.05; // 50 ms steps
    let steps = 100; // 5 seconds = 5 * tau

    for _ in 0..steps {
        temps = cauer.step_transient_backward_euler(&temps, &[p_in], ambient_k, dt);
    }

    // After 5 tau, temperature should be > 99% of final value (320 K)
    assert!(
        temps[0] > 319.5 && temps[0] <= 320.0,
        "Final transient temperature was {}",
        temps[0]
    );
}
