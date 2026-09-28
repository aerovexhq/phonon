//! Integration Tests: 3D Asset Mesh Geometry & Physical Mass Validation
//!
//! Validates:
//! 1. Mesh3D surface area and volume via divergence theorem for canonical primitives.
//! 2. Center of mass and 3x3 moment of inertia tensor computation.
//! 3. Procedural component catalog generation (Dipole Antenna, Patch Antenna,
//!    CubeSat Chassis, Quadrotor Frame, Finned Heat Sink, Landing Gear).

#![deny(unsafe_code)]

use phonon_models::assets::{
    create_cubesat_chassis, create_dipole_antenna, create_finned_heatsink, create_patch_antenna,
    create_quadrotor_frame, create_tactile_landing_gear, Mesh3D, Triangle3D, Vertex3D,
};
use phonon_models::em::Vector3D;

#[test]
fn test_mesh_volume_area_and_inertia_canonical_cube() {
    let mut cube = Mesh3D::new("UnitCube");
    let l = 1.0; // 1 meter cube
    let hl = l / 2.0;

    for z in [-hl, hl] {
        for y in [-hl, hl] {
            for x in [-hl, hl] {
                let n = Vector3D::new(x / hl, y / hl, z / hl).normalize();
                cube.vertices
                    .push(Vertex3D::new(Vector3D::new(x, y, z), n, (0.0, 0.0)));
            }
        }
    }

    let box_faces = [
        [0, 1, 3, 2],
        [4, 6, 7, 5],
        [0, 4, 5, 1],
        [2, 3, 7, 6],
        [0, 2, 6, 4],
        [1, 5, 7, 3],
    ];

    for face in &box_faces {
        cube.triangles
            .push(Triangle3D::new(face[0], face[1], face[2], 0));
        cube.triangles
            .push(Triangle3D::new(face[0], face[2], face[3], 0));
    }

    // 1. Surface Area = 6 * 1.0^2 = 6.0 m^2:
    let area = cube.compute_surface_area();
    assert!((area - 6.0).abs() < 1e-6);

    // 2. Volume = 1.0 m^3:
    let vol = cube.compute_volume();
    assert!((vol - 1.0).abs() < 1e-6);

    // 3. Center of mass = (0, 0, 0):
    let cm = cube.compute_center_of_mass();
    assert!(cm.x.abs() < 1e-6);
    assert!(cm.y.abs() < 1e-6);
    assert!(cm.z.abs() < 1e-6);

    // 4. Moment of inertia tensor for uniform density rho = 1000 kg/m^3 (Mass M = 1000 kg):
    // For a cube: Ixx = Iyy = Izz = 1/6 * M * L^2 = 1/6 * 1000 * 1.0 ~= 166.67 kg*m^2
    let inertia = cube.compute_inertia_tensor(1000.0);
    let ixx = inertia[0];
    let iyy = inertia[4];
    let izz = inertia[8];
    assert!((ixx - 166.67).abs() < 1.0);
    assert!((iyy - 166.67).abs() < 1.0);
    assert!((izz - 166.67).abs() < 1.0);

    // Off-diagonal terms must be zero for symmetric cube:
    assert!(inertia[1].abs() < 1e-4);
    assert!(inertia[2].abs() < 1e-4);
    assert!(inertia[5].abs() < 1e-4);
}

#[test]
fn test_catalog_components_creation_and_properties() {
    // 1. Dipole Antenna:
    let dipole = create_dipole_antenna(0.30, 0.003, 16);
    assert!(!dipole.vertices.is_empty());
    assert!(!dipole.triangles.is_empty());
    assert_eq!(dipole.submeshes.len(), 1);
    let aabb_dipole = dipole.compute_aabb();
    assert!((aabb_dipole.extents().z - 0.30).abs() < 0.01);

    // 2. Microstrip Patch Antenna:
    let patch = create_patch_antenna(0.08, 0.08, 0.0016, 0.04, 0.04);
    assert_eq!(patch.submeshes.len(), 2); // Substrate + Copper patch
    assert_eq!(patch.submeshes[0].name, "DielectricSubstrate");
    assert_eq!(patch.submeshes[1].name, "CopperRadiatingPatch");

    // 3. CubeSat 3U Chassis:
    let cubesat = create_cubesat_chassis(3);
    let aabb_cubesat = cubesat.compute_aabb();
    let extents = aabb_cubesat.extents();
    assert!((extents.x - 0.10).abs() < 1e-4);
    assert!((extents.y - 0.10).abs() < 1e-4);
    assert!((extents.z - 0.30).abs() < 1e-4); // 3U = 30 cm

    // 4. Quadrotor Frame:
    let quad = create_quadrotor_frame(0.50);
    assert!(quad.compute_surface_area() > 0.01);
    assert_eq!(quad.submeshes.len(), 1);

    // 5. Finned Heatsink:
    let heatsink = create_finned_heatsink(0.06, 0.06, 0.005, 0.015, 5);
    assert!(heatsink.triangles.len() >= 60);

    // 6. Tactile Landing Gear:
    let leg = create_tactile_landing_gear(0.15, 0.02);
    assert_eq!(leg.submeshes.len(), 2); // Strut + Footpad
}
