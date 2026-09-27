use phonon_thermal::boundary::ThermalBoundary;
use phonon_thermal::grid2d::{silicon_thermal_conductivity, ThermalGrid2D};

#[test]
fn test_silicon_thermal_conductivity_temperature_dependence() {
    // Silicon kappa at 300K is ~148 W/(m*K)
    let kappa_300 = silicon_thermal_conductivity(300.0);
    assert!((kappa_300 - 148.0).abs() < 1.0);

    // As temperature increases, lattice phonon scattering increases, so kappa drops:
    let kappa_400 = silicon_thermal_conductivity(400.0);
    let kappa_500 = silicon_thermal_conductivity(500.0);

    assert!(kappa_400 < kappa_300);
    assert!(kappa_500 < kappa_400);

    // At 400 K: kappa ~ 148 * (400/300)^(-1.33) ~ 101 W/(m*K)
    assert!(kappa_400 > 95.0 && kappa_400 < 105.0);
}

#[test]
fn test_2d_fourier_diffusion_hotspot() {
    // 10x10 grid on a 1 mm x 1 mm x 200 um silicon die
    let nx = 10;
    let ny = 10;
    let width = 1e-3; // 1 mm
    let height = 1e-3; // 1 mm
    let thickness = 200e-6; // 200 um
    let init_temp = 300.0;

    let mut grid = ThermalGrid2D::new(nx, ny, width, height, thickness, init_temp);

    // Bottom (y=0) is attached to ideal 300 K heatsink (Dirichlet)
    grid.boundary_south = ThermalBoundary::Dirichlet {
        temperature_kelvin: 300.0,
    };
    // North, East, West are insulated (Neumann adiabatic)
    grid.boundary_north = ThermalBoundary::Neumann {
        heat_flux_w_m2: 0.0,
    };
    grid.boundary_east = ThermalBoundary::Neumann {
        heat_flux_w_m2: 0.0,
    };
    grid.boundary_west = ThermalBoundary::Neumann {
        heat_flux_w_m2: 0.0,
    };

    // Inject 1.0 Watt localized hotspot near top center (x = 0.5 mm, y = 0.8 mm)
    grid.inject_heat_power(0.5e-3, 0.8e-3, 1.0);

    // Relax to steady state
    let iters = grid
        .solve_steady_state(1e-4, 500)
        .expect("Thermal steady-state relaxation must converge");
    assert!(iters > 0);

    let max_t = grid.max_temperature();
    let sampled_hotspot = grid.sample_temperature(0.5e-3, 0.8e-3);
    let sampled_bottom = grid.sample_temperature(0.5e-3, 0.05e-3);

    // Hotspot temperature must be elevated above 300 K
    assert!(max_t > 305.0, "Hotspot max_t={max_t} must be elevated");
    assert!(
        sampled_hotspot > 305.0,
        "sampled_hotspot={sampled_hotspot} must be elevated"
    );

    // Temperature near bottom heatsink must be close to 300 K
    assert!(
        sampled_bottom > 299.9 && sampled_bottom < 305.0,
        "sampled_bottom={sampled_bottom}"
    );

    // Hotspot must be hotter than bottom
    assert!(sampled_hotspot > sampled_bottom);
}

#[test]
fn test_2d_fourier_transient_diffusion() {
    let mut grid = ThermalGrid2D::new(6, 6, 1e-3, 1e-3, 200e-6, 300.0);
    grid.boundary_south = ThermalBoundary::Dirichlet {
        temperature_kelvin: 300.0,
    };

    // Apply continuous heat pulse of 2.0 W at center
    grid.inject_heat_power(0.5e-3, 0.5e-3, 2.0);

    let t_initial = grid.sample_temperature(0.5e-3, 0.5e-3);
    assert_eq!(t_initial, 300.0);

    // Step explicit diffusion for small time steps (microsecond scale for stability)
    let dt = 1e-5; // 10 microseconds
    for _ in 0..50 {
        grid.step_transient_explicit(dt);
    }

    let t_transient = grid.sample_temperature(0.5e-3, 0.5e-3);
    // Center temperature must have heated up monotonically
    assert!(t_transient > t_initial);
    assert!(!t_transient.is_nan());
}
