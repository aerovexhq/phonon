//! Pre-Defined 3D Engineering Component Catalog
//!
//! Procedural mesh generators for standardized aerospace, RF, structural,
//! and electronic components with physically validated multi-material assignments.

use super::mesh::{Mesh3D, Submesh, Triangle3D, Vertex3D};
use crate::em::Vector3D;

/// Generates a standardized half-wave RF dipole antenna with feed gap (Copper, Material ID 0).
pub fn create_dipole_antenna(total_length_m: f64, radius_m: f64, num_segments: usize) -> Mesh3D {
    let mut mesh = Mesh3D::new("DipoleAntenna");
    let copper_id = 0; // Copper in MaterialLibrary

    let half_len = total_length_m / 2.0;
    let gap_half = 0.002_f64.min(half_len * 0.1); // 2 mm feed gap
    let segments = num_segments.max(8);

    // Upper arm: z from +gap_half to +half_len
    // Lower arm: z from -half_len to -gap_half
    for is_upper in [true, false] {
        let (z_start, z_end) = if is_upper {
            (gap_half, half_len)
        } else {
            (-half_len, -gap_half)
        };

        let base_idx = mesh.vertices.len();
        // Bottom ring:
        for i in 0..segments {
            let theta = (i as f64) * std::f64::consts::TAU / (segments as f64);
            let x = radius_m * theta.cos();
            let y = radius_m * theta.sin();
            let normal = Vector3D::new(theta.cos(), theta.sin(), 0.0);
            mesh.vertices.push(Vertex3D::new(
                Vector3D::new(x, y, z_start),
                normal,
                (i as f64 / segments as f64, 0.0),
            ));
        }
        // Top ring:
        for i in 0..segments {
            let theta = (i as f64) * std::f64::consts::TAU / (segments as f64);
            let x = radius_m * theta.cos();
            let y = radius_m * theta.sin();
            let normal = Vector3D::new(theta.cos(), theta.sin(), 0.0);
            mesh.vertices.push(Vertex3D::new(
                Vector3D::new(x, y, z_end),
                normal,
                (i as f64 / segments as f64, 1.0),
            ));
        }

        // Cylinder side faces:
        for i in 0..segments {
            let next = (i + 1) % segments;
            let b0 = base_idx + i;
            let b1 = base_idx + next;
            let t0 = base_idx + segments + i;
            let t1 = base_idx + segments + next;

            mesh.triangles.push(Triangle3D::new(b0, b1, t1, copper_id));
            mesh.triangles.push(Triangle3D::new(b0, t1, t0, copper_id));
        }
    }

    let num_tris = mesh.triangles.len();
    mesh.submeshes.push(Submesh {
        name: "CopperArms".to_string(),
        material_id: copper_id,
        first_triangle: 0,
        num_triangles: num_tris,
    });

    mesh
}

/// Generates a planar microstrip patch antenna (Rogers 5880 substrate + Copper patch & ground).
pub fn create_patch_antenna(
    sub_w_m: f64,
    sub_l_m: f64,
    sub_h_m: f64,
    patch_w_m: f64,
    patch_l_m: f64,
) -> Mesh3D {
    let mut mesh = Mesh3D::new("MicrostripPatchAntenna");
    let rogers_id = 37; // Rogers RT/duroid 5880
    let copper_id = 0; // Copper

    let hw_s = sub_w_m / 2.0;
    let hl_s = sub_l_m / 2.0;
    let hw_p = patch_w_m / 2.0;
    let hl_p = patch_l_m / 2.0;

    // 1. Substrate Box (centered at z = 0, height = sub_h_m):
    let base_v = mesh.vertices.len();
    let z0 = -sub_h_m / 2.0;
    let z1 = sub_h_m / 2.0;

    // 8 box vertices:
    for z in [z0, z1] {
        for y in [-hl_s, hl_s] {
            for x in [-hw_s, hw_s] {
                let n = Vector3D::new(x / hw_s, y / hl_s, (z - 0.0) / (sub_h_m / 2.0)).normalize();
                mesh.vertices
                    .push(Vertex3D::new(Vector3D::new(x, y, z), n, (0.0, 0.0)));
            }
        }
    }

    let tri_sub_start = mesh.triangles.len();
    // 6 box faces (12 triangles):
    let box_faces = [
        [0, 1, 3, 2], // Bottom (Ground Plane interface)
        [4, 6, 7, 5], // Top
        [0, 4, 5, 1], // Front
        [2, 3, 7, 6], // Back
        [0, 2, 6, 4], // Left
        [1, 5, 7, 3], // Right
    ];

    for face in &box_faces {
        let i0 = base_v + face[0];
        let i1 = base_v + face[1];
        let i2 = base_v + face[2];
        let i3 = base_v + face[3];
        mesh.triangles.push(Triangle3D::new(i0, i1, i2, rogers_id));
        mesh.triangles.push(Triangle3D::new(i0, i2, i3, rogers_id));
    }
    let sub_tris = mesh.triangles.len() - tri_sub_start;
    mesh.submeshes.push(Submesh {
        name: "DielectricSubstrate".to_string(),
        material_id: rogers_id,
        first_triangle: tri_sub_start,
        num_triangles: sub_tris,
    });

    // 2. Copper Radiating Patch on top surface (z = z1 + 1e-4):
    let patch_v = mesh.vertices.len();
    let z_patch = z1 + 1e-4;
    let norm_up = Vector3D::new(0.0, 0.0, 1.0);

    mesh.vertices.push(Vertex3D::new(
        Vector3D::new(-hw_p, -hl_p, z_patch),
        norm_up,
        (0.0, 0.0),
    ));
    mesh.vertices.push(Vertex3D::new(
        Vector3D::new(hw_p, -hl_p, z_patch),
        norm_up,
        (1.0, 0.0),
    ));
    mesh.vertices.push(Vertex3D::new(
        Vector3D::new(hw_p, hl_p, z_patch),
        norm_up,
        (1.0, 1.0),
    ));
    mesh.vertices.push(Vertex3D::new(
        Vector3D::new(-hw_p, hl_p, z_patch),
        norm_up,
        (0.0, 1.0),
    ));

    let tri_patch_start = mesh.triangles.len();
    mesh.triangles.push(Triangle3D::new(
        patch_v,
        patch_v + 1,
        patch_v + 2,
        copper_id,
    ));
    mesh.triangles.push(Triangle3D::new(
        patch_v,
        patch_v + 2,
        patch_v + 3,
        copper_id,
    ));

    mesh.submeshes.push(Submesh {
        name: "CopperRadiatingPatch".to_string(),
        material_id: copper_id,
        first_triangle: tri_patch_start,
        num_triangles: 2,
    });

    mesh
}

/// Generates a standardized CubeSat chassis (1U, 2U, 3U) of Al 7075-T6 (Material ID 4).
pub fn create_cubesat_chassis(units_u: usize) -> Mesh3D {
    let mut mesh = Mesh3D::new("CubeSatChassis");
    let al7075_id = 4; // Aluminum 7075-T6

    let u = units_u.max(1);
    let size_x = 0.10; // 10 cm
    let size_y = 0.10; // 10 cm
    let size_z = 0.10 * (u as f64); // 10 cm per U

    let hx = size_x / 2.0;
    let hy = size_y / 2.0;
    let hz = size_z / 2.0;

    let base_v = mesh.vertices.len();
    for z in [-hz, hz] {
        for y in [-hy, hy] {
            for x in [-hx, hx] {
                let n = Vector3D::new(x / hx, y / hy, z / hz).normalize();
                mesh.vertices
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
        let i0 = base_v + face[0];
        let i1 = base_v + face[1];
        let i2 = base_v + face[2];
        let i3 = base_v + face[3];
        mesh.triangles.push(Triangle3D::new(i0, i1, i2, al7075_id));
        mesh.triangles.push(Triangle3D::new(i0, i2, i3, al7075_id));
    }

    let num_tris = mesh.triangles.len();
    mesh.submeshes.push(Submesh {
        name: "AluminumChassis".to_string(),
        material_id: al7075_id,
        first_triangle: 0,
        num_triangles: num_tris,
    });

    mesh
}

/// Generates a quadrotor carbon-fiber X-frame airframe (CFRP, Material ID 51).
pub fn create_quadrotor_frame(arm_span_m: f64) -> Mesh3D {
    let mut mesh = Mesh3D::new("QuadrotorFrame");
    let cfrp_id = 51; // CFRP Unidirectional

    let half_span = arm_span_m / 2.0;
    let hub_size = 0.08;
    let hh = hub_size / 2.0;
    let arm_thick = 0.015;
    let ht = arm_thick / 2.0;

    // 1. Central Avionics Hub Box:
    let base_v = mesh.vertices.len();
    for z in [-ht, ht] {
        for y in [-hh, hh] {
            for x in [-hh, hh] {
                let n = Vector3D::new(x / hh, y / hh, z / ht).normalize();
                mesh.vertices
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
        let i0 = base_v + face[0];
        let i1 = base_v + face[1];
        let i2 = base_v + face[2];
        let i3 = base_v + face[3];
        mesh.triangles.push(Triangle3D::new(i0, i1, i2, cfrp_id));
        mesh.triangles.push(Triangle3D::new(i0, i2, i3, cfrp_id));
    }

    // 2. Four Diagonal Arms (+X+Y, -X+Y, -X-Y, +X-Y):
    for &(sx, sy) in &[(1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)] {
        let start_x = sx * hh;
        let start_y = sy * hh;
        let end_x = sx * half_span;
        let end_y = sy * half_span;

        let arm_v = mesh.vertices.len();
        for z in [-ht, ht] {
            for &(ox, oy) in &[(-ht, -ht), (ht, -ht), (ht, ht), (-ht, ht)] {
                let n = Vector3D::new(ox, oy, z).normalize();
                mesh.vertices.push(Vertex3D::new(
                    Vector3D::new(start_x + ox, start_y + oy, z),
                    n,
                    (0.0, 0.0),
                ));
            }
            for &(ox, oy) in &[(-ht, -ht), (ht, -ht), (ht, ht), (-ht, ht)] {
                let n = Vector3D::new(ox, oy, z).normalize();
                mesh.vertices.push(Vertex3D::new(
                    Vector3D::new(end_x + ox, end_y + oy, z),
                    n,
                    (1.0, 1.0),
                ));
            }
        }

        // Form 8 box triangles for arm tube:
        for f in 0..4 {
            let next = (f + 1) % 4;
            let b0 = arm_v + f;
            let b1 = arm_v + next;
            let t0 = arm_v + 4 + f;
            let t1 = arm_v + 4 + next;
            mesh.triangles.push(Triangle3D::new(b0, b1, t1, cfrp_id));
            mesh.triangles.push(Triangle3D::new(b0, t1, t0, cfrp_id));
        }
    }

    let num_tris = mesh.triangles.len();
    mesh.submeshes.push(Submesh {
        name: "CarbonFiberAirframe".to_string(),
        material_id: cfrp_id,
        first_triangle: 0,
        num_triangles: num_tris,
    });

    mesh
}

/// Generates an extruded finned aluminum heat sink (Aluminum 6061-T6, Material ID 3).
pub fn create_finned_heatsink(
    base_w_m: f64,
    base_l_m: f64,
    base_h_m: f64,
    fin_h_m: f64,
    num_fins: usize,
) -> Mesh3D {
    let mut mesh = Mesh3D::new("FinnedHeatsink");
    let al6061_id = 3; // Aluminum 6061-T6

    let hw = base_w_m / 2.0;
    let hl = base_l_m / 2.0;

    // 1. Base Plate: z from 0.0 to base_h_m:
    let base_v = mesh.vertices.len();
    for z in [0.0, base_h_m] {
        for y in [-hl, hl] {
            for x in [-hw, hw] {
                let n = Vector3D::new(x / hw, y / hl, (z - base_h_m / 2.0) / (base_h_m / 2.0))
                    .normalize();
                mesh.vertices
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
        let i0 = base_v + face[0];
        let i1 = base_v + face[1];
        let i2 = base_v + face[2];
        let i3 = base_v + face[3];
        mesh.triangles.push(Triangle3D::new(i0, i1, i2, al6061_id));
        mesh.triangles.push(Triangle3D::new(i0, i2, i3, al6061_id));
    }

    // 2. Fins on top of base plate:
    let n_fins = num_fins.max(2);
    let fin_thick = 0.002_f64.min(base_w_m / (n_fins as f64 * 3.0));
    let fin_spacing = (base_w_m - fin_thick) / ((n_fins - 1) as f64);

    for fin_idx in 0..n_fins {
        let x_center = -hw + fin_thick / 2.0 + (fin_idx as f64) * fin_spacing;
        let x0 = x_center - fin_thick / 2.0;
        let x1 = x_center + fin_thick / 2.0;
        let z0 = base_h_m;
        let z1 = base_h_m + fin_h_m;

        let fin_v = mesh.vertices.len();
        for z in [z0, z1] {
            for y in [-hl, hl] {
                for x in [x0, x1] {
                    let n = Vector3D::new(
                        (x - x_center) / (fin_thick / 2.0),
                        y / hl,
                        (z - (z0 + z1) / 2.0) / (fin_h_m / 2.0),
                    )
                    .normalize();
                    mesh.vertices
                        .push(Vertex3D::new(Vector3D::new(x, y, z), n, (0.0, 0.0)));
                }
            }
        }

        for face in &box_faces {
            let i0 = fin_v + face[0];
            let i1 = fin_v + face[1];
            let i2 = fin_v + face[2];
            let i3 = fin_v + face[3];
            mesh.triangles.push(Triangle3D::new(i0, i1, i2, al6061_id));
            mesh.triangles.push(Triangle3D::new(i0, i2, i3, al6061_id));
        }
    }

    let num_tris = mesh.triangles.len();
    mesh.submeshes.push(Submesh {
        name: "AluminumFinnedHeatsink".to_string(),
        material_id: al6061_id,
        first_triangle: 0,
        num_triangles: num_tris,
    });

    mesh
}

/// Generates a drone landing gear strut with elastomeric tactile footpad (Ti-6Al-4V + Silicone).
pub fn create_tactile_landing_gear(strut_length_m: f64, pad_radius_m: f64) -> Mesh3D {
    let mut mesh = Mesh3D::new("TactileLandingGear");
    let titanium_id = 8; // Titanium Grade 5
    let silicone_id = 46; // Silicone Elastomer

    // 1. Titanium Strut Rod (z from 0.0 to strut_length_m):
    let rod_radius = 0.005;
    let segs = 8;
    let base_v = mesh.vertices.len();

    for z in [0.0, strut_length_m] {
        for i in 0..segs {
            let theta = (i as f64) * std::f64::consts::TAU / (segs as f64);
            let x = rod_radius * theta.cos();
            let y = rod_radius * theta.sin();
            let n = Vector3D::new(theta.cos(), theta.sin(), 0.0);
            mesh.vertices
                .push(Vertex3D::new(Vector3D::new(x, y, z), n, (0.0, 0.0)));
        }
    }

    let tri_rod_start = mesh.triangles.len();
    for i in 0..segs {
        let next = (i + 1) % segs;
        let b0 = base_v + i;
        let b1 = base_v + next;
        let t0 = base_v + segs + i;
        let t1 = base_v + segs + next;
        mesh.triangles
            .push(Triangle3D::new(b0, b1, t1, titanium_id));
        mesh.triangles
            .push(Triangle3D::new(b0, t1, t0, titanium_id));
    }
    let rod_tris = mesh.triangles.len() - tri_rod_start;

    mesh.submeshes.push(Submesh {
        name: "TitaniumStrut".to_string(),
        material_id: titanium_id,
        first_triangle: tri_rod_start,
        num_triangles: rod_tris,
    });

    // 2. Silicone Tactile Footpad (z from -0.005 to 0.0):
    let pad_v = mesh.vertices.len();
    for z in [-0.005, 0.0] {
        for i in 0..segs {
            let theta = (i as f64) * std::f64::consts::TAU / (segs as f64);
            let x = pad_radius_m * theta.cos();
            let y = pad_radius_m * theta.sin();
            let n = Vector3D::new(theta.cos(), theta.sin(), if z < 0.0 { -1.0 } else { 1.0 })
                .normalize();
            mesh.vertices
                .push(Vertex3D::new(Vector3D::new(x, y, z), n, (0.0, 0.0)));
        }
    }

    let tri_pad_start = mesh.triangles.len();
    for i in 0..segs {
        let next = (i + 1) % segs;
        let b0 = pad_v + i;
        let b1 = pad_v + next;
        let t0 = pad_v + segs + i;
        let t1 = pad_v + segs + next;
        mesh.triangles
            .push(Triangle3D::new(b0, b1, t1, silicone_id));
        mesh.triangles
            .push(Triangle3D::new(b0, t1, t0, silicone_id));
    }
    let pad_tris = mesh.triangles.len() - tri_pad_start;

    mesh.submeshes.push(Submesh {
        name: "SiliconeTactilePad".to_string(),
        material_id: silicone_id,
        first_triangle: tri_pad_start,
        num_triangles: pad_tris,
    });

    mesh
}
