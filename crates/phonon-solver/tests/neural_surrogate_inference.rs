//! Numerical verification of Neural Surrogate models:
//! analytical backpropagation Jacobians against finite-difference gradients,
//! and Polynomial Chaos Expansion (PCE) with Sobol variance indices.

use phonon_models::surrogate::{
    fit_pce_surrogate, train_mlp_surrogate, ActivationFunction, DenseLayer, MlpTrainingConfig,
    MultilayerPerceptron, NeuralSurrogateCompanion, PceTerm, PolynomialChaosExpansion,
};

#[test]
fn test_mlp_analytical_jacobian_vs_finite_difference() {
    // 2-input, 1-output network with non-linear SiLU/Tanh activations
    let layers = vec![
        DenseLayer::new(
            2,
            3,
            vec![0.8, -0.5, -1.2, 0.9, 0.4, 0.7],
            vec![0.1, -0.2, 0.05],
            ActivationFunction::Tanh,
        ),
        DenseLayer::new(
            3,
            2,
            vec![0.6, -0.8, 1.1, -0.3, 0.5, -0.7],
            vec![0.0, 0.1],
            ActivationFunction::SiLU,
        ),
        DenseLayer::new(
            2,
            1,
            vec![1.5, -1.0],
            vec![0.02],
            ActivationFunction::Linear,
        ),
    ];

    let mlp = MultilayerPerceptron::new(layers);
    assert_eq!(mlp.in_features(), 2);
    assert_eq!(mlp.out_features(), 1);

    let test_inputs = vec![
        vec![0.5, 1.2],
        vec![-0.8, 0.3],
        vec![1.5, -0.4],
        vec![0.0, 0.0],
    ];

    let eps = 1e-6;

    for x in test_inputs {
        let (y, analytical_jac) = mlp.forward_with_jacobian(&x);
        assert_eq!(y.len(), 1);
        assert_eq!(analytical_jac.len(), 1);
        assert_eq!(analytical_jac[0].len(), 2);

        // Finite difference verification
        for j in 0..2 {
            let mut x_plus = x.clone();
            x_plus[j] += eps;
            let mut x_minus = x.clone();
            x_minus[j] -= eps;

            let y_plus = mlp.forward(&x_plus);
            let y_minus = mlp.forward(&x_minus);

            let numerical_grad = (y_plus[0] - y_minus[0]) / (2.0 * eps);
            let analytical_grad = analytical_jac[0][j];

            let diff = (analytical_grad - numerical_grad).abs();
            assert!(
                diff < 1e-5,
                "Analytical Jacobian mismatch at input {:?}, dim {}: analytical={}, numerical={}, diff={}",
                x,
                j,
                analytical_grad,
                numerical_grad,
                diff
            );
        }
    }
}

#[test]
fn test_mlp_training_on_transistor_iv_curve() {
    // Generate synthetic NMOS I_ds = 0.5 * beta * (V_gs - V_th)^2 * (1 + lambda * V_ds) data
    let vth = 0.5;
    let beta = 1e-3;
    let lambda = 0.05;

    let mut x_train = Vec::new();
    let mut y_train = Vec::new();

    for v_ds_step in 0..10 {
        let v_ds = (v_ds_step as f64) * 0.2; // 0.0 to 1.8V
        for v_gs_step in 0..10 {
            let v_gs = (v_gs_step as f64) * 0.2; // 0.0 to 1.8V
            let v_ov = (v_gs - vth).max(0.0);
            let i_ds = 0.5 * beta * v_ov * v_ov * (1.0 + lambda * v_ds);

            x_train.push(vec![v_ds, v_gs]);
            y_train.push(vec![i_ds]);
        }
    }

    let config = MlpTrainingConfig {
        hidden_layers: vec![8, 8],
        activation: ActivationFunction::SiLU,
        learning_rate: 0.02,
        epochs: 150,
        batch_size: 16,
    };

    let trained_mlp = train_mlp_surrogate(&x_train, &y_train, &config);

    let surrogate = NeuralSurrogateCompanion::new_transistor("surrogate_nmos", trained_mlp);

    // Evaluate response at test bias
    let (i_ds, g_m, g_ds, _) = surrogate.evaluate_transistor(1.0, 1.2, 0.0);
    assert!(i_ds > 0.0, "Current should be positive: i_ds={}", i_ds);
    assert!(
        g_ds > 0.0,
        "Conductance g_ds must be positive: g_ds={}",
        g_ds
    );
    assert!(
        g_m >= 0.0,
        "Transconductance g_m must be non-negative: g_m={}",
        g_m
    );
}

#[test]
fn test_polynomial_chaos_expansion_and_sobol_indices() {
    // Exact function: f(x1, x2) = 2.0 + 3.0 * x1 + 4.0 * x2 + 1.5 * (1.5*x1^2 - 0.5)
    // where x1, x2 in [-1, 1]
    let terms = vec![
        PceTerm {
            degrees: vec![0, 0],
        },
        PceTerm {
            degrees: vec![1, 0],
        },
        PceTerm {
            degrees: vec![0, 1],
        },
        PceTerm {
            degrees: vec![2, 0],
        },
    ];
    let coefficients = vec![2.0, 3.0, 4.0, 1.5];
    let input_mins = vec![-1.0, -1.0];
    let input_maxs = vec![1.0, 1.0];

    let pce = PolynomialChaosExpansion::new(2, terms, coefficients, input_mins, input_maxs);

    // Evaluate at point [0.5, -0.5]
    let (val, grad) = pce.evaluate_with_gradient(&[0.5, -0.5]);
    let leg_p2 = 0.5 * (3.0 * 0.5 * 0.5 - 1.0);
    let expected = 2.0 + 3.0 * 0.5 + 4.0 * (-0.5) + 1.5 * leg_p2;
    assert!((val - expected).abs() < 1e-12);

    // Evaluate analytical gradient
    assert_eq!(grad.len(), 2);
    // d/dx1 = 3.0 + 1.5 * 3.0 * 0.5 = 5.25
    assert!((grad[0] - 5.25).abs() < 1e-10);
    // d/dx2 = 4.0
    assert!((grad[1] - 4.0).abs() < 1e-10);

    // Total variance and Sobol sensitivity indices
    let (total_var, sobol_indices) = pce.sobol_indices();
    assert!(total_var > 0.0);
    let s1 = sobol_indices[0];
    let s2 = sobol_indices[1];
    assert!(s1 > 0.0 && s1 < 1.0);
    assert!(s2 > 0.0 && s2 < 1.0);
    // Sum of independent orthogonal effects should be ~ 1.0
    assert!((s1 + s2 - 1.0).abs() < 1e-6);

    // Test PCE regression fitting
    let mut x_samples = Vec::new();
    let mut y_samples = Vec::new();
    for i in -5..=5 {
        for j in -5..=5 {
            let x1 = (i as f64) / 5.0;
            let x2 = (j as f64) / 5.0;
            let y = 1.0 + 2.0 * x1 - 1.5 * x2;
            x_samples.push(vec![x1, x2]);
            y_samples.push(y);
        }
    }

    let fitted_pce = fit_pce_surrogate(&x_samples, &y_samples, 2);
    assert_eq!(fitted_pce.dimension, 2);
    let test_val = fitted_pce.evaluate(&[0.2, -0.4]);
    let expected_test = 1.0 + 2.0 * 0.2 - 1.5 * (-0.4);
    assert!(
        (test_val - expected_test).abs() < 1e-4,
        "Fitted PCE error: got {}, expected {}",
        test_val,
        expected_test
    );
}
