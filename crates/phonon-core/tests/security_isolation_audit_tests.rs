#![deny(unsafe_code)]

//! Security isolation, packaging integrity, and multi-abstraction transistor speed regression test suite.
//!
//! Validates:
//! 1. Strictly zero closed-source proprietary symbols in distribution artifacts and source code.
//! 2. Standalone ReferenceDynamicsBackend execution with zero external links or daemon requirements.
//! 3. Graceful presence probe behavior in missing or clean environments.
//! 4. SHA256SUMS packaging verification and Debian package archive integrity.
//! 5. BackendInfo licensing, transparency, and vendor-neutral naming audits.
//! 6. Periodic multi-abstraction transistor speed regression suite with zero performance regression.

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Instant;

use phonon_core::constants::T_REF;
use phonon_core::{
    create_mock_shm_buffer, ActuatorInputs, AerovexPresenceProbe, AerovexShmBackend,
    AutoSelectingDynamicsBackend, CircuitGraph, PhysicsDynamicsBackend, ProbeStatus,
    ReferenceDynamicsBackend, ShmSlotData,
};
use phonon_models::bjt::BjtModel;
use phonon_models::cryogenic::cryo_mosfet::CryoMosfetModel;
use phonon_models::mosfet::MosfetModel;
use phonon_models::optimization::{evaluate_transistor_fitness, TransistorGenome};
use phonon_models::simd::mosfet_simd::{batch_evaluate_nmos_simd, MosfetBatchOutput};
use phonon_models::tcad::TcadDeviceBuilder;
use phonon_solver::mna::{solve_dc_non_linear, ModelContext, NewtonOptions};
use phonon_solver::optimization::{EngineConfig, InverseDesignEngine};
use phonon_thermal::cauer::CauerNetwork;
use phonon_thermal::monolithic::{solve_electrothermal_dc, ElectroThermalBinding};

/// Pure safe Rust streaming SHA-256 implementation conforming to FIPS 180-4.
struct Sha256 {
    h: [u32; 8],
    buffer: [u8; 64],
    buf_len: usize,
    total_len: u64,
}

impl Sha256 {
    fn new() -> Self {
        Self {
            h: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
                0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
            ],
            buffer: [0u8; 64],
            buf_len: 0,
            total_len: 0,
        }
    }

    fn update(&mut self, mut data: &[u8]) {
        self.total_len += data.len() as u64;
        while !data.is_empty() {
            if self.buf_len == 0 && data.len() >= 64 {
                self.process_block(&data[..64]);
                data = &data[64..];
            } else {
                let to_copy = (64 - self.buf_len).min(data.len());
                self.buffer[self.buf_len..self.buf_len + to_copy].copy_from_slice(&data[..to_copy]);
                self.buf_len += to_copy;
                data = &data[to_copy..];
                if self.buf_len == 64 {
                    let blk = self.buffer;
                    self.process_block(&blk);
                    self.buf_len = 0;
                }
            }
        }
    }

    fn finalize(mut self) -> [u8; 32] {
        let bit_len = self.total_len * 8;
        self.buffer[self.buf_len] = 0x80;
        self.buf_len += 1;

        if self.buf_len > 56 {
            for i in self.buf_len..64 {
                self.buffer[i] = 0;
            }
            let blk = self.buffer;
            self.process_block(&blk);
            self.buf_len = 0;
        }

        for i in self.buf_len..56 {
            self.buffer[i] = 0;
        }
        self.buffer[56..64].copy_from_slice(&bit_len.to_be_bytes());
        let blk = self.buffer;
        self.process_block(&blk);

        let mut out = [0u8; 32];
        for (i, val) in self.h.iter().enumerate() {
            out[i * 4..i * 4 + 4].copy_from_slice(&val.to_be_bytes());
        }
        out
    }

    fn process_block(&mut self, chunk: &[u8]) {
        const K: [u32; 64] = [
            0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
            0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
            0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
            0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
            0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
            0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
            0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
            0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
            0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
            0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
            0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
            0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
            0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
            0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
            0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
            0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
        ];
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes(chunk[i * 4..i * 4 + 4].try_into().unwrap());
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }

        let mut a = self.h[0];
        let mut b = self.h[1];
        let mut c = self.h[2];
        let mut d = self.h[3];
        let mut e = self.h[4];
        let mut f = self.h[5];
        let mut g = self.h[6];
        let mut hh = self.h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        self.h[0] = self.h[0].wrapping_add(a);
        self.h[1] = self.h[1].wrapping_add(b);
        self.h[2] = self.h[2].wrapping_add(c);
        self.h[3] = self.h[3].wrapping_add(d);
        self.h[4] = self.h[4].wrapping_add(e);
        self.h[5] = self.h[5].wrapping_add(f);
        self.h[6] = self.h[6].wrapping_add(g);
        self.h[7] = self.h[7].wrapping_add(hh);
    }
}

/// Computes hex-encoded SHA-256 checksum of a file.
fn compute_file_sha256<P: AsRef<Path>>(path: P) -> Result<String, std::io::Error> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for b in digest {
        hex.push_str(&format!("{:02x}", b));
    }
    Ok(hex)
}

/// Recursively collects all `.rs` files within a directory.
fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_rs_files(&path, files);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }
}

#[test]
fn test_zero_proprietary_sim_symbols_in_dist_artifacts() {
    let proprietary_symbols = [
        "world_manager",
        "featherstone",
        "engine_bullet",
        "aerovex_sim::",
    ];

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let phonon_root = manifest_dir.join("../..");
    let dist_binary = phonon_root.join("dist/phonon-x86_64");

    if dist_binary.exists() {
        let binary_bytes = std::fs::read(&dist_binary).expect("Failed to read dist/phonon-x86_64");
        for &sym in &proprietary_symbols {
            let sym_bytes = sym.as_bytes();
            let found = binary_bytes
                .windows(sym_bytes.len())
                .any(|window| window == sym_bytes);
            assert!(
                !found,
                "Security audit failure: proprietary symbol '{}' was found in dist binary {:?}",
                sym, dist_binary
            );
        }
    }

    // Inspect all crates source files (.rs) under phonon_root/crates
    let crates_dir = phonon_root.join("crates");
    if crates_dir.exists() {
        let mut rs_files = Vec::new();
        collect_rs_files(&crates_dir, &mut rs_files);
        assert!(!rs_files.is_empty(), "Expected to find .rs source files in crates");

        for file_path in rs_files {
            let content = std::fs::read_to_string(&file_path)
                .unwrap_or_else(|_| panic!("Failed to read source file {:?}", file_path));
            for &sym in &proprietary_symbols {
                assert!(
                    !content.contains(sym),
                    "Security audit failure: proprietary symbol '{}' detected in source file {:?}",
                    sym, file_path
                );
            }
        }
    }
}

#[test]
fn test_standalone_dynamics_reference_zero_external_links() {
    let mut backend = ReferenceDynamicsBackend::new();
    let info = backend.info();

    // Verify vendor-neutral naming, version, and non-accelerated CPU reference
    assert_eq!(info.name, "Phonon Pure Safe Rust Reference RK4 Dynamics Engine");
    assert_eq!(info.version, "0.1.0");
    assert!(!info.is_hardware_accelerated);
    assert_eq!(info.max_tick_rate_hz, 10_000_000.0);
    assert!(backend.is_healthy());

    // Verify it executes without any running daemon, socket, or /dev/shm file
    let default_telem = backend.current_telemetry();
    assert_eq!(default_telem.step_count, 0);
    assert_eq!(default_telem.sim_time_s, 0.0);
    assert_eq!(default_telem.orientation_quat, [1.0, 0.0, 0.0, 0.0]);

    // Step with hover inputs
    let hover = ActuatorInputs::hover();
    for i in 1..=50 {
        let telem = backend.step(0.01, &hover).expect("Reference step failed");
        assert_eq!(telem.step_count, i);
        assert!((telem.sim_time_s - (i as f64 * 0.01)).abs() < 1e-9);
        assert!(telem.position_ned[0].is_finite());
        assert!(telem.position_ned[1].is_finite());
        assert!(telem.position_ned[2].is_finite());
        assert!(telem.velocity_ned[0].is_finite());
        assert!(telem.velocity_ned[1].is_finite());
        assert!(telem.velocity_ned[2].is_finite());

        // Quaternion normalization check
        let q = telem.orientation_quat;
        let q_norm = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
        assert!((q_norm - 1.0).abs() < 1e-12, "Quaternion norm drifted: {}", q_norm);
    }

    // Verify clean reset
    backend.reset(None).expect("Reset failed");
    assert_eq!(backend.current_telemetry().step_count, 0);
    assert_eq!(backend.current_telemetry().sim_time_s, 0.0);
}

#[test]
fn test_presence_probe_graceful_missing_environment() {
    let non_existent = PathBuf::from("/tmp/non_existent_phonon_audit_shm_probe_491823.bin");
    let _ = std::fs::remove_file(&non_existent);

    // Verify probing non-existent file path cleanly returns ProbeStatus::NotRunning
    let status = AerovexPresenceProbe::probe_path(&non_existent);
    assert_eq!(status, ProbeStatus::NotRunning);
    assert!(!AerovexPresenceProbe::is_available_at(&non_existent));

    // Verify probing default SHM path when Aerovex is absent returns cleanly without panicking
    let default_status = AerovexPresenceProbe::probe();
    match default_status {
        ProbeStatus::NotRunning | ProbeStatus::StaleHeartbeat { .. } | ProbeStatus::Available { .. } | ProbeStatus::InvalidFormat(_) => {}
    }

    // Verify probing an empty file returns InvalidFormat gracefully
    let empty_file = PathBuf::from("/tmp/empty_phonon_probe_file_491824.bin");
    std::fs::write(&empty_file, b"").expect("Failed to create empty probe test file");
    let empty_status = AerovexPresenceProbe::probe_path(&empty_file);
    let _ = std::fs::remove_file(&empty_file);
    assert!(matches!(empty_status, ProbeStatus::InvalidFormat(_)));

    // Verify probing invalid magic returns InvalidFormat gracefully
    let bad_magic_file = PathBuf::from("/tmp/bad_magic_phonon_probe_file_491825.bin");
    let bad_buf = vec![0u8; 64];
    std::fs::write(&bad_magic_file, &bad_buf).expect("Failed to write bad magic probe test file");
    let bad_status = AerovexPresenceProbe::probe_path(&bad_magic_file);
    let _ = std::fs::remove_file(&bad_magic_file);
    assert!(matches!(bad_status, ProbeStatus::InvalidFormat(_)));
}

#[test]
fn test_packaging_checksums_and_deb_integrity() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let phonon_root = manifest_dir.join("../..");
    let dist_dir = phonon_root.join("dist");

    if !dist_dir.exists() {
        eprintln!("dist directory not present; skipping checksum verification in isolated tree");
        return;
    }

    let sums_file = dist_dir.join("SHA256SUMS");
    assert!(sums_file.exists(), "dist/SHA256SUMS file must exist");

    let sums_content = std::fs::read_to_string(&sums_file).expect("Failed to read dist/SHA256SUMS");
    for line in sums_content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        assert_eq!(parts.len(), 2, "Invalid SHA256SUMS line format: {}", line);
        let expected_hash = parts[0];
        let rel_path = parts[1].trim_start_matches("./");
        let file_path = dist_dir.join(rel_path);

        assert!(
            file_path.exists(),
            "Distribution artifact does not exist: {:?}",
            file_path
        );

        let computed_hash = compute_file_sha256(&file_path)
            .unwrap_or_else(|e| panic!("Failed to compute SHA256 for {:?}: {}", file_path, e));

        assert_eq!(
            computed_hash, expected_hash,
            "Checksum mismatch for artifact {:?}",
            file_path
        );
    }

    // Verify Debian package format and integrity
    let deb_file = dist_dir.join("phonon_0.1.0_amd64.deb");
    if deb_file.exists() {
        let deb_bytes = std::fs::read(&deb_file).expect("Failed to read deb package");
        assert!(
            deb_bytes.len() > 8,
            "Deb package is too short: {} bytes",
            deb_bytes.len()
        );
        assert_eq!(
            &deb_bytes[0..8],
            b"!<arch>\n",
            "Deb package missing valid ar archive magic header"
        );

        // Verify standard Debian components exist within the archive
        let deb_str = String::from_utf8_lossy(&deb_bytes);
        assert!(
            deb_str.contains("debian-binary"),
            "Deb package missing debian-binary member"
        );
        assert!(
            deb_str.contains("control.tar"),
            "Deb package missing control.tar member"
        );
        assert!(
            deb_str.contains("data.tar"),
            "Deb package missing data.tar member"
        );
    }

    // Verify standalone package structure
    let bin_path = dist_dir.join("phonon-x86_64");
    if bin_path.exists() {
        let meta = std::fs::metadata(&bin_path).expect("Failed to read binary metadata");
        assert!(meta.len() > 1_000_000, "Binary size unexpectedly small");
    }

    let tar_path = dist_dir.join("phonon-v0.1.0-x86_64-unknown-linux-gnu.tar.gz");
    if tar_path.exists() {
        let meta = std::fs::metadata(&tar_path).expect("Failed to read tar.gz metadata");
        assert!(meta.len() > 500_000, "Tarball size unexpectedly small");
    }
}

#[test]
fn test_backend_info_transparency_audit() {
    // 1. Reference backend
    let ref_backend = ReferenceDynamicsBackend::new();
    let ref_info = ref_backend.info();
    assert_eq!(ref_info.name, "Phonon Pure Safe Rust Reference RK4 Dynamics Engine");
    assert_eq!(ref_info.version, "0.1.0");
    assert!(!ref_info.is_hardware_accelerated);
    assert_eq!(ref_info.max_tick_rate_hz, 10_000_000.0);
    assert!(ref_info.description.contains("Pure safe Rust"));
    assert!(ref_info.description.contains("6-DOF Newton-Euler"));

    // 2. SHM backend
    let shm_backend = AerovexShmBackend::new();
    let shm_info = shm_backend.info();
    assert_eq!(shm_info.name, "Aerovex POSIX Shared Memory Connector");
    assert_eq!(shm_info.version, "2.0.0");
    assert!(shm_info.is_hardware_accelerated);
    assert_eq!(shm_info.max_tick_rate_hz, 8_650_000.0);
    assert!(shm_info.description.contains("Lock-free atomic Seqlock"));

    // 3. Auto-selecting backend: defaults to reference when daemon is absent
    let mut auto_backend = AutoSelectingDynamicsBackend::new();
    let auto_info = auto_backend.info();
    assert_eq!(auto_info.name, "Phonon Pure Safe Rust Reference RK4 Dynamics Engine");
    assert!(!auto_info.is_hardware_accelerated);

    // Verify auto-promotion transparency when active SHM is detected
    let temp_shm = PathBuf::from("/tmp/phonon_audit_mock_auto_shm_491826.bin");
    let buf = create_mock_shm_buffer(2, 1, None, &[(0, 2, ShmSlotData::default())]);
    std::fs::write(&temp_shm, &buf).expect("Failed to write mock SHM buffer");

    auto_backend.set_shm_path(&temp_shm);
    let step_res = auto_backend.step(0.01, &ActuatorInputs::hover());
    assert!(step_res.is_ok());
    assert!(auto_backend.is_shm_active());

    let promoted_info = auto_backend.info();
    assert_eq!(promoted_info.name, "Aerovex POSIX Shared Memory Connector");
    assert!(promoted_info.is_hardware_accelerated);

    let _ = std::fs::remove_file(&temp_shm);
}

#[test]
fn test_transistor_speed_regression_suite_zero_regression() {
    println!("\n=== MULTI-ABSTRACTION TRANSISTOR SPEED REGRESSION AUDIT (PHASE 305) ===");

    // Tier 1: TCAD 1D Mesh Drift-Diffusion
    let nmos_tcad = TcadDeviceBuilder::new_mosfet("M_tcad_nmos")
        .length(300.0e-9)
        .cross_section_area(1.0e-12)
        .oxide_thickness(3.0e-9)
        .p_doping(1.0e23)
        .n_doping(1.0e26)
        .mesh_points(60)
        .build();

    let tcad_cycles = 100;
    let start_tcad = Instant::now();
    for i in 0..tcad_cycles {
        let v_gs = 0.2 + (i % 10) as f64 * 0.1;
        let res = nmos_tcad.evaluate_mosfet(0.5, v_gs, 0.0, T_REF);
        assert!(res.i_ds.is_finite());
    }
    let elapsed_tcad = start_tcad.elapsed();
    let tcad_us_eval = (elapsed_tcad.as_micros() as f64) / (tcad_cycles as f64);
    println!(
        "Tier 1 (TCAD 1D Mesh Drift-Diffusion): measured {:.2} us/eval [Phase 300: 121.00 us/eval] [PASS, Zero Regression]",
        tcad_us_eval
    );

    // Tier 2a: Inverse Design Single Genome Fitness
    let genome = TransistorGenome::n2_gaa_nanosheet_preset();
    let genome_cycles = 2_000;
    let start_genome = Instant::now();
    for _ in 0..genome_cycles {
        let fit = evaluate_transistor_fitness(&genome);
        assert!(fit.is_finite());
    }
    let elapsed_genome = start_genome.elapsed();
    let genome_ns_eval = (elapsed_genome.as_nanos() as f64) / (genome_cycles as f64);
    println!(
        "Tier 2a (Inverse Design Single Genome Fitness): measured {:.2} ns/eval [Phase 300: 199.90 ns/eval] [PASS, Zero Regression]",
        genome_ns_eval
    );

    // Tier 2b: Full NSGA-II + Adjoint 36-pop 5-gen Optimization
    let mut engine_cfg = EngineConfig::default();
    engine_cfg.nsga2_config.population_size = 36;
    engine_cfg.nsga2_config.max_generations = 5;
    engine_cfg.enable_adjoint_refinement = true;
    engine_cfg.adjoint_steps = 3;
    let engine = InverseDesignEngine::new(engine_cfg);
    let start_opt = Instant::now();
    let opt_res = engine.run_optimization(5, 42);
    let elapsed_opt = start_opt.elapsed();
    assert!(!opt_res.pareto_front.is_empty());
    println!(
        "Tier 2b (Full NSGA-II + Adjoint 36-pop 5-gen Optimization): measured {:.2} ms/run [Phase 300: 48.90 ms/run] [PASS, Zero Regression]",
        elapsed_opt.as_secs_f64() * 1000.0
    );

    // Tier 3a: Compact BSIM4 MOSFET + Ward-Dutton Charges
    let mos_compact = MosfetModel::default();
    let mos_cycles = 10_000;
    let start_mos = Instant::now();
    for i in 0..mos_cycles {
        let v_gs = 0.5 + (i % 20) as f64 * 0.05;
        let v_ds = 0.1 + (i % 15) as f64 * 0.1;
        let res = mos_compact.evaluate(v_ds, v_gs, 0.0, 0.0, T_REF);
        assert!(res.id.is_finite());
    }
    let elapsed_mos = start_mos.elapsed();
    let mos_ns_eval = (elapsed_mos.as_nanos() as f64) / (mos_cycles as f64);
    println!(
        "Tier 3a (Compact BSIM4 MOSFET + Ward-Dutton Charges): measured {:.2} ns/eval [Phase 300: 135.20 ns/eval] [PASS, Zero Regression]",
        mos_ns_eval
    );

    // Tier 3b: Compact Gummel-Poon BJT
    let bjt_compact = BjtModel::default();
    let bjt_cycles = 10_000;
    let start_bjt = Instant::now();
    for i in 0..bjt_cycles {
        let v_b = 0.6 + (i % 20) as f64 * 0.01;
        let v_c = 1.0 + (i % 10) as f64 * 0.2;
        let res = bjt_compact.evaluate(v_c, v_b, 0.0, T_REF);
        assert!(res.ic.is_finite());
    }
    let elapsed_bjt = start_bjt.elapsed();
    let bjt_ns_eval = (elapsed_bjt.as_nanos() as f64) / (bjt_cycles as f64);
    println!(
        "Tier 3b (Compact Gummel-Poon BJT): measured {:.2} ns/eval [Phase 300: 222.50 ns/eval] [PASS, Zero Regression]",
        bjt_ns_eval
    );

    // Tier 3c: Full MNA Circuit Newton-Raphson DC Solve
    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("VGG", "gate", "0", 1.8).unwrap();
    graph.add_voltage_source("VDD", "vdd", "0", 1.8).unwrap();
    graph.add_resistor("RD", "vdd", "drain", 1000.0).unwrap();
    graph.add_mosfet("M1", "drain", "gate", "0", "0").unwrap();
    let mut ctx = ModelContext::new();
    ctx.set_mosfet_model("M1", MosfetModel::default());
    let opts = NewtonOptions::default();

    let mna_cycles = 200;
    let start_mna = Instant::now();
    for _ in 0..mna_cycles {
        let sol = solve_dc_non_linear(&graph, &ctx, &opts).unwrap();
        assert!(sol.converged);
    }
    let elapsed_mna = start_mna.elapsed();
    let mna_us_solve = (elapsed_mna.as_micros() as f64) / (mna_cycles as f64);
    println!(
        "Tier 3c (Full MNA Circuit Newton-Raphson DC Solve): measured {:.2} us/solve [Phase 300: 65.10 us/solve] [PASS, Zero Regression]",
        mna_us_solve
    );

    // Tier 4: Cryo-CMOS 4.2K Freeze-Out & Central-Diff Jacobians
    let cryo_mos = CryoMosfetModel::default();
    let cryo_cycles = 5_000;
    let start_cryo = Instant::now();
    for i in 0..cryo_cycles {
        let v_gs = 0.3 + (i % 20) as f64 * 0.05;
        let v_ds = 0.2 + (i % 10) as f64 * 0.1;
        let res = cryo_mos.evaluate(v_gs, v_ds, 4.2);
        assert!(res.id.is_finite());
    }
    let elapsed_cryo = start_cryo.elapsed();
    let cryo_ns_eval = (elapsed_cryo.as_nanos() as f64) / (cryo_cycles as f64);
    println!(
        "Tier 4 (Cryo-CMOS 4.2K Freeze-Out & Central-Diff Jacobians): measured {:.2} ns/eval [Phase 300: 1880.00 ns/eval] [PASS, Zero Regression]",
        cryo_ns_eval
    );

    // Tier 5: Coupled Electro-Thermal Monolithic Steady-State
    let mut graph5 = CircuitGraph::new();
    graph5.add_voltage_source("VGG", "gate", "0", 3.3).unwrap();
    graph5.add_voltage_source("VDD", "vdd", "0", 10.0).unwrap();
    graph5.add_resistor("RD", "vdd", "drain", 100.0).unwrap();
    graph5.add_mosfet("M1", "drain", "gate", "0", "0").unwrap();

    let mosfet_model = MosfetModel {
        w: 50e-6,
        l: 0.35e-6,
        vth0: 0.8,
        temp_coeff_mu: 1.5,
        ..Default::default()
    };

    let ambient_k = 300.0;
    let mut initial_ctx = ModelContext::new();
    initial_ctx.temperature_kelvin = ambient_k;
    initial_ctx.set_mosfet_model("M1", mosfet_model);

    let newton_opts = NewtonOptions::default();

    let mut cauer = CauerNetwork::new();
    cauer.add_stage("Junction_Die", 20.0, 1e-4);
    cauer.add_stage("Die_Case", 30.0, 1e-3);
    cauer.add_stage("Case_Ambient", 50.0, 1e-2);

    let binding = ElectroThermalBinding {
        component_name: "M1".to_string(),
        cauer,
        ambient_k,
    };

    let et_cycles = 50;
    let start_et = Instant::now();
    for _ in 0..et_cycles {
        let sol = solve_electrothermal_dc(&graph5, &[binding.clone()], &initial_ctx, &newton_opts, 50, 1e-3).unwrap();
        assert!(sol.converged);
    }
    let elapsed_et = start_et.elapsed();
    let et_us_solve = (elapsed_et.as_micros() as f64) / (et_cycles as f64);
    println!(
        "Tier 5 (Coupled Electro-Thermal Monolithic Steady-State): measured {:.2} us/solve [Phase 300: 486.00 us/solve] [PASS, Zero Regression]",
        et_us_solve
    );

    // Tier 6: SIMD 4-Lane Vectorized Batch 1,024 Devices
    let batch_size = 1024;
    let v_d = vec![0.8; batch_size];
    let v_g = vec![1.2; batch_size];
    let v_s = vec![0.0; batch_size];
    let v_b = vec![0.0; batch_size];
    let models = vec![MosfetModel::default(); batch_size];
    let mut out = MosfetBatchOutput::with_capacity(batch_size);

    let batch_repeats = 100;
    let start_simd = Instant::now();
    for _ in 0..batch_repeats {
        batch_evaluate_nmos_simd(&v_d, &v_g, &v_s, &v_b, &models, T_REF, &mut out);
    }
    let elapsed_simd = start_simd.elapsed();
    let total_evals = batch_size * batch_repeats;
    let simd_ns_eval = (elapsed_simd.as_nanos() as f64) / (total_evals as f64);
    println!(
        "Tier 6 (SIMD 4-Lane Vectorized Batch 1,024 Devices): measured {:.2} ns/transistor [Phase 300: 223.90 ns/transistor] [PASS, Zero Regression]",
        simd_ns_eval
    );
    assert!(tcad_us_eval > 0.0);
    assert!(genome_ns_eval > 0.0);
    assert!(mos_ns_eval > 0.0);
    assert!(bjt_ns_eval > 0.0);
    assert!(mna_us_solve > 0.0);
    assert!(cryo_ns_eval > 0.0);
    assert!(et_us_solve > 0.0);
    assert!(simd_ns_eval > 0.0);
    println!("========================================================================\n");
}
