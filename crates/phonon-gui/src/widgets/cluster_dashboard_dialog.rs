#![deny(unsafe_code)]

//! Interactive Distributed Cloud Parameter Sweep Cluster Dashboard dialog for Phonon Visual Studio.
//!
//! Provides real-time cluster worker topology monitoring, work-stealing scheduling,
//! fault injection and resilience telemetry, aggregated moment harvesting, and distribution histograms.

use egui::{Color32, Frame, ProgressBar, RichText, ScrollArea, Stroke, Ui, Vec2};
use egui_plot::{Bar, BarChart, Plot, VLine};
use phonon_solver::cluster::{
    ClusterDispatchQueue, ClusterSweepResult, NodeStatus, RpcMessage, SimulatedClusterWorker,
    SimulationType, SweepSample, WorkerNodeInfo,
};

/// Interactive modal dialog for distributed cluster parameter sweep orchestration and telemetry.
#[derive(Debug, Clone, PartialEq)]
pub struct ClusterDashboardDialog {
    /// Visibility flag of the dialog window.
    pub is_open: bool,
    /// Central work-stealing scheduler and node registry.
    pub scheduler: ClusterDispatchQueue,
    /// Simulated hardware worker daemon instances.
    pub workers: Vec<SimulatedClusterWorker>,
    /// Selected simulation solver category.
    pub sim_type: SimulationType,
    /// Total parameter sweep sample count.
    pub total_samples: usize,
    /// Sample chunk size per dispatch batch.
    pub batch_size: usize,
    /// Flag indicating whether a sweep is currently active and executing.
    pub is_running: bool,
    /// Current job identifier string.
    pub current_job_id: String,
    /// Cumulative count of completed samples for active sweep.
    pub completed_samples: usize,
    /// Total samples targeted for active sweep.
    pub total_job_samples: usize,
    /// Finalized or live aggregated sweep results.
    pub active_result: Option<ClusterSweepResult>,
    /// System status message displayed in banner.
    pub status_msg: String,
    /// Lower specification limit enabled flag.
    pub enable_lsl: bool,
    /// Lower specification limit value.
    pub lsl_value: f64,
    /// Upper specification limit enabled flag.
    pub enable_usl: bool,
    /// Upper specification limit value.
    pub usl_value: f64,
    /// Virtual cluster simulation clock time in seconds.
    pub current_time_s: f64,
    /// Next dynamic worker node counter for unique ID generation.
    pub next_worker_id: usize,
    /// Flag signaling to the parent application that a sweep should run.
    pub run_requested: bool,
}

impl Default for ClusterDashboardDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl ClusterDashboardDialog {
    /// Creates a new cluster dashboard dialog initialized with a default 4-node topology.
    pub fn new() -> Self {
        let mut scheduler = ClusterDispatchQueue::new(5.0);
        let mut workers = Vec::with_capacity(8);

        // Default 4 heterogeneous worker nodes
        let defaults = [
            ("node-alpha", "192.168.1.101:9040", 8, 16384, 1.2),
            ("node-beta", "192.168.1.102:9040", 16, 32768, 1.5),
            ("node-gamma", "192.168.1.103:9040", 4, 8192, 0.9),
            ("node-delta", "192.168.1.104:9040", 8, 16384, 1.0),
        ];

        for (id, addr, cores, mem, speed) in defaults {
            let info = WorkerNodeInfo::new(id, addr, cores, mem);
            scheduler.add_node(info);
            let worker = SimulatedClusterWorker::new(id, addr, cores, mem).with_speed(speed);
            workers.push(worker);
            scheduler.heartbeat(id, 0.0, 3.0, 512.0);
        }

        Self {
            is_open: false,
            scheduler,
            workers,
            sim_type: SimulationType::MonteCarlo,
            total_samples: 2000,
            batch_size: 100,
            is_running: false,
            current_job_id: String::new(),
            completed_samples: 0,
            total_job_samples: 0,
            active_result: None,
            status_msg: "Cluster ready: 4 worker nodes online (36 aggregate cores).".to_string(),
            enable_lsl: true,
            lsl_value: 2.38,
            enable_usl: true,
            usl_value: 2.62,
            current_time_s: 0.0,
            next_worker_id: 5,
            run_requested: false,
        }
    }

    /// Dynamically provisions and registers a new worker node into the cluster topology.
    pub fn add_worker_node(&mut self) {
        let id = format!("node-worker-{}", self.next_worker_id);
        let addr = format!("192.168.1.{}:9040", 100 + self.next_worker_id);
        self.next_worker_id += 1;

        let cores = 8;
        let mem = 16384;
        let info = WorkerNodeInfo::new(&id, &addr, cores, mem);
        self.scheduler.add_node(info);

        let worker = SimulatedClusterWorker::new(&id, &addr, cores, mem).with_speed(1.1);
        self.scheduler.heartbeat(&id, self.current_time_s, 2.0, 400.0);
        self.workers.push(worker);

        let total_cores: usize = self.workers.iter().map(|w| w.info.core_count).sum();
        self.status_msg = format!(
            "Worker '{}' added. Total cluster capacity: {} nodes ({} cores).",
            id,
            self.workers.len(),
            total_cores
        );
    }

    /// Simulates sudden node failure or network dropout on the first available busy or online worker.
    pub fn simulate_dropout(&mut self) -> Option<String> {
        let candidate_id = self
            .workers
            .iter()
            .find(|w| w.info.status != NodeStatus::Offline)
            .map(|w| w.info.id.clone())?;

        // Mark worker offline
        if let Some(w) = self.workers.iter_mut().find(|w| w.info.id == candidate_id) {
            w.info.status = NodeStatus::Offline;
            w.current_cpu_pct = 0.0;
        }

        // Inform scheduler of node failure to trigger fault-tolerant batch recovery
        let recovered = self.scheduler.handle_node_failure(&candidate_id);
        if let Some(node) = self.scheduler.get_node_mut(&candidate_id) {
            node.status = NodeStatus::Offline;
        }

        self.status_msg = format!(
            "Fault simulated: worker '{}' dropped offline. {} in-flight batches safely re-queued.",
            candidate_id, recovered
        );

        Some(candidate_id)
    }

    /// Dispatches a parameter sweep job across the cluster.
    pub fn dispatch_sweep(&mut self) {
        let job_id = format!("job-sweep-{:.0}", self.current_time_s * 1000.0);
        self.current_job_id = job_id.clone();
        self.completed_samples = 0;
        self.total_job_samples = self.total_samples;
        self.active_result = None;

        // Generate parameter sweep samples
        let samples: Vec<SweepSample> = (0..self.total_samples as u64)
            .map(|i| {
                let mut sample = SweepSample::new(i);
                match self.sim_type {
                    SimulationType::MonteCarlo => {
                        let tol = ((i as f64 * 17.0) % 100.0 - 50.0) * 1.5;
                        sample.parameter_overrides.insert("VIN".to_string(), 5.0);
                        sample.parameter_overrides.insert("R1".to_string(), 1000.0);
                        sample.parameter_overrides.insert("R2".to_string(), 1000.0 + tol);
                    }
                    SimulationType::SParameterSweep => {
                        let f = 4.5e9 + (i as f64 / self.total_samples as f64) * 1.0e9;
                        sample.parameter_overrides.insert("freq".to_string(), f);
                    }
                    SimulationType::DcCornerSweep => {
                        sample.parameter_overrides.insert("VDD".to_string(), 3.3);
                        sample.parameter_overrides.insert("TEMP".to_string(), 300.0);
                    }
                }
                sample
            })
            .collect();

        self.scheduler.dispatch_sweep(
            &job_id,
            samples,
            self.batch_size,
            self.sim_type,
            self.current_time_s,
        );
        self.is_running = true;
        self.status_msg = format!(
            "Cluster sweep '{}' dispatched: {} samples across {} batches.",
            job_id,
            self.total_samples,
            (self.total_samples + self.batch_size - 1) / self.batch_size
        );
    }

    /// Advances virtual simulation clock by `dt_s`, assigning batches and stepping workers.
    pub fn step_cluster(&mut self, dt_s: f64) {
        self.current_time_s += dt_s;

        // 1. Send heartbeats from active workers and check timeouts
        for w in &self.workers {
            if w.info.status != NodeStatus::Offline {
                self.scheduler.heartbeat(
                    &w.info.id,
                    self.current_time_s,
                    w.current_cpu_pct,
                    w.current_mem_mb,
                );
            }
        }
        self.scheduler.mark_offline_timed_out(self.current_time_s);

        // 2. Dispatch batches to ready workers via work-stealing scheduler
        if self.is_running {
            for w in self.workers.iter_mut() {
                if w.is_ready() {
                    if let Some(batch) = self.scheduler.assign_next_batch(&w.info.id) {
                        w.receive_batch(batch);
                    }
                }
            }
        }

        // 3. Step workers and collect finished batches
        let mut completed_results = Vec::new();
        for w in self.workers.iter_mut() {
            if let Some(res) = w.step(dt_s) {
                completed_results.push(res);
            }
        }

        for res in completed_results {
            if let RpcMessage::BatchResult { sample_values, .. } = &res {
                self.completed_samples += sample_values.len();
            }
            self.scheduler.record_batch_result(&res, self.current_time_s);
        }

        // 4. Check if sweep completed
        if self.is_running && self.scheduler.is_sweep_complete() {
            self.is_running = false;
            let lsl = if self.enable_lsl { Some(self.lsl_value) } else { None };
            let usl = if self.enable_usl { Some(self.usl_value) } else { None };

            if let Some(res) = self.scheduler.finalize_result(
                &self.current_job_id,
                lsl,
                usl,
                self.current_time_s,
            ) {
                self.status_msg = format!(
                    "Cluster sweep finished in {:.2}s. Speedup: {:.1}x across {} nodes. Yield: {:.1}%.",
                    res.execution_time_s,
                    res.speedup_factor,
                    self.scheduler.available_node_count(),
                    res.yield_percentage
                );
                self.active_result = Some(res);
            }
        }
    }

    /// Renders the modal cluster dashboard window.
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        // If sweep is running, automatically step cluster clock each frame
        if self.is_running {
            self.step_cluster(0.05);
            ctx.request_repaint();
        }

        let mut is_open = self.is_open;
        egui::Window::new("Distributed Cloud Parameter Sweep Cluster Engine")
            .open(&mut is_open)
            .resizable(true)
            .default_size(Vec2::new(880.0, 700.0))
            .show(ctx, |ui| {
                self.render_content(ui);
            });
        self.is_open = is_open;
    }

    /// Renders the internal panels, topology cards, and histogram plot.
    pub fn render_content(&mut self, ui: &mut Ui) {
        // Status Bar Banner
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(&self.status_msg)
                    .color(Color32::from_rgb(180, 215, 250))
                    .size(12.5),
            );
        });
        ui.separator();

        ScrollArea::vertical().show(ui, |ui| {
            // 1. Live Telemetry & Progress Meter
            ui.heading("Live Telemetry & Execution Progress");
            ui.add_space(4.0);

            let progress_fraction = if self.total_job_samples > 0 {
                (self.completed_samples as f32 / self.total_job_samples as f32).clamp(0.0, 1.0)
            } else {
                0.0
            };

            ui.add(
                ProgressBar::new(progress_fraction)
                    .text(format!(
                        "{}/{} Samples Completed ({:.1}%)",
                        self.completed_samples,
                        self.total_job_samples,
                        progress_fraction * 100.0
                    ))
                    .animate(self.is_running),
            );

            ui.add_space(4.0);

            // Throughput & Speedup Meter
            let active_nodes = self.scheduler.available_node_count();
            let total_throughput: f64 = self
                .workers
                .iter()
                .filter(|w| w.info.status != NodeStatus::Offline)
                .map(|w| {
                    self.scheduler
                        .get_node(&w.info.id)
                        .map(|n| n.active_throughput_jobs_sec)
                        .unwrap_or(0.0)
                })
                .sum();

            let speedup = self
                .active_result
                .as_ref()
                .map(|r| r.speedup_factor)
                .unwrap_or_else(|| {
                    if self.is_running {
                        (active_nodes as f64 * 0.92).max(1.0)
                    } else {
                        1.0
                    }
                });

            ui.horizontal(|ui| {
                ui.label(RichText::new("Cluster Throughput:").strong());
                ui.label(
                    RichText::new(format!("{:.1} samples/sec", total_throughput))
                        .color(Color32::from_rgb(100, 220, 140)),
                );

                ui.separator();
                ui.label(RichText::new("Parallel Speedup:").strong());
                ui.label(
                    RichText::new(format!("{:.2}x", speedup))
                        .color(Color32::from_rgb(250, 200, 80)),
                );

                ui.separator();
                ui.label(RichText::new("Active Topology:").strong());
                ui.label(format!("{}/{} Nodes Online", active_nodes, self.workers.len()));
            });

            ui.add_space(8.0);
            ui.separator();

            // 2. Cluster Dispatch Panel
            ui.heading("Cluster Dispatch Panel");
            ui.horizontal(|ui| {
                ui.label("Simulation Type:");
                let current_type_name = self.sim_type.as_str();
                egui::ComboBox::from_id_salt("cluster_sim_type_combo")
                    .selected_text(current_type_name)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.sim_type,
                            SimulationType::MonteCarlo,
                            "Monte Carlo",
                        );
                        ui.selectable_value(
                            &mut self.sim_type,
                            SimulationType::SParameterSweep,
                            "S-Parameter Sweep",
                        );
                        ui.selectable_value(
                            &mut self.sim_type,
                            SimulationType::DcCornerSweep,
                            "DC Corner Sweep",
                        );
                    });

                ui.separator();
                ui.label("Total Samples:");
                egui::ComboBox::from_id_salt("cluster_sample_count_combo")
                    .selected_text(format!("{}", self.total_samples))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.total_samples, 500, "500 samples");
                        ui.selectable_value(&mut self.total_samples, 2000, "2,000 samples");
                        ui.selectable_value(&mut self.total_samples, 5000, "5,000 samples");
                        ui.selectable_value(&mut self.total_samples, 10000, "10,000 samples");
                    });

                ui.separator();
                ui.label("Batch Size:");
                egui::ComboBox::from_id_salt("cluster_batch_size_combo")
                    .selected_text(format!("{}", self.batch_size))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.batch_size, 50, "50 / batch");
                        ui.selectable_value(&mut self.batch_size, 100, "100 / batch");
                        ui.selectable_value(&mut self.batch_size, 250, "250 / batch");
                    });
            });

            // Specification Limits & Action Buttons
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.enable_lsl, "LSL:");
                if self.enable_lsl {
                    ui.add(egui::DragValue::new(&mut self.lsl_value).speed(0.01));
                }

                ui.checkbox(&mut self.enable_usl, "USL:");
                if self.enable_usl {
                    ui.add(egui::DragValue::new(&mut self.usl_value).speed(0.01));
                }

                ui.separator();

                let dispatch_btn = ui.add_enabled(
                    !self.is_running,
                    egui::Button::new(RichText::new("Dispatch Cluster Sweep").strong()),
                );
                if dispatch_btn.clicked() {
                    self.dispatch_sweep();
                }

                if ui.button("Add Worker Node").clicked() {
                    self.add_worker_node();
                }

                if ui.button("Simulate Node Dropout").clicked() {
                    self.simulate_dropout();
                }
            });

            ui.add_space(8.0);
            ui.separator();

            // 3. Node Topology Grid / Cards
            ui.heading("Worker Node Topology & Telemetry");
            ui.add_space(4.0);

            let available_width = ui.available_width();
            let card_width = (available_width / 2.0 - 8.0).max(180.0);

            let node_count = self.workers.len();
            for chunk_indices in (0..node_count).collect::<Vec<_>>().chunks(2) {
                ui.horizontal(|ui| {
                    for &idx in chunk_indices {
                        let worker = &self.workers[idx];
                        let info = self
                            .scheduler
                            .get_node(&worker.info.id)
                            .cloned()
                            .unwrap_or_else(|| worker.info.clone());

                        Frame::group(ui.style())
                            .fill(Color32::from_rgb(20, 26, 36))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(45, 55, 75)))
                            .inner_margin(6.0)
                            .show(ui, |ui| {
                                ui.set_width(card_width);

                                // Node ID and Status Badge
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(&info.id).strong().size(13.0));
                                    let (badge_color, badge_text) = match info.status {
                                        NodeStatus::Online => {
                                            (Color32::from_rgb(46, 160, 67), "Online")
                                        }
                                        NodeStatus::Busy => {
                                            (Color32::from_rgb(219, 171, 9), "Busy")
                                        }
                                        NodeStatus::Idle => {
                                            (Color32::from_rgb(88, 166, 255), "Idle")
                                        }
                                        NodeStatus::Offline => {
                                            (Color32::from_rgb(248, 81, 73), "Offline")
                                        }
                                    };
                                    ui.label(
                                        RichText::new(badge_text)
                                            .color(badge_color)
                                            .strong()
                                            .size(11.0),
                                    );
                                });

                                ui.label(
                                    RichText::new(&info.address)
                                        .color(Color32::from_rgb(130, 145, 165))
                                        .size(10.5),
                                );

                                ui.horizontal(|ui| {
                                    ui.label(format!("{} Cores", info.core_count));
                                    ui.label("|");
                                    ui.label(format!("{} MB RAM", info.memory_mb));
                                    ui.label("|");
                                    ui.label(format!("{} Jobs", info.jobs_completed));
                                });

                                // CPU Utilization Bar
                                ui.horizontal(|ui| {
                                    ui.label("CPU:");
                                    let cpu_frac = (worker.current_cpu_pct / 100.0).clamp(0.0, 1.0) as f32;
                                    ui.add(
                                        ProgressBar::new(cpu_frac)
                                            .text(format!("{:.1}%", worker.current_cpu_pct)),
                                    );
                                });
                            });
                    }
                });
                ui.add_space(4.0);
            }

            ui.add_space(8.0);
            ui.separator();

            // 4. Aggregated Results & Histogram
            ui.heading("Aggregated Results & Distribution Histogram");
            ui.add_space(4.0);

            if let Some(res) = &self.active_result {
                // Statistics readout
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Mean:").strong());
                    ui.label(format!("{:.4}", res.mean));

                    ui.separator();
                    ui.label(RichText::new("Std Dev:").strong());
                    ui.label(format!("{:.4}", res.std_dev));

                    ui.separator();
                    ui.label(RichText::new("Min:").strong());
                    ui.label(format!("{:.4}", res.min));

                    ui.separator();
                    ui.label(RichText::new("Max:").strong());
                    ui.label(format!("{:.4}", res.max));

                    ui.separator();
                    ui.label(RichText::new("Yield:").strong());
                    ui.label(
                        RichText::new(format!("{:.1}%", res.yield_percentage))
                            .color(Color32::from_rgb(60, 200, 120))
                            .strong(),
                    );
                });

                ui.add_space(4.0);

                // Distribution histogram plot
                let bars: Vec<Bar> = res
                    .bins
                    .iter()
                    .map(|(b_min, b_max, count)| {
                        let center = 0.5 * (b_min + b_max);
                        let width = (b_max - b_min).abs();
                        let in_spec = (!self.enable_lsl || center >= self.lsl_value)
                            && (!self.enable_usl || center <= self.usl_value);
                        let color = if in_spec {
                            Color32::from_rgb(60, 160, 240)
                        } else {
                            Color32::from_rgb(230, 70, 70)
                        };
                        Bar::new(center, *count as f64)
                            .width(width * 0.95)
                            .fill(color)
                    })
                    .collect();

                let bar_chart = BarChart::new("Sample Distribution", bars);

                Plot::new("cluster_sweep_histogram_plot")
                    .height(180.0)
                    .allow_zoom(true)
                    .allow_drag(true)
                    .show(ui, |plot_ui| {
                        plot_ui.bar_chart(bar_chart);

                        if self.enable_lsl {
                            plot_ui.vline(
                                VLine::new("LSL", self.lsl_value)
                                    .color(Color32::from_rgb(240, 80, 80))
                                    .stroke(Stroke::new(2.0, Color32::from_rgb(240, 80, 80))),
                            );
                        }

                        if self.enable_usl {
                            plot_ui.vline(
                                VLine::new("USL", self.usl_value)
                                    .color(Color32::from_rgb(240, 80, 80))
                                    .stroke(Stroke::new(2.0, Color32::from_rgb(240, 80, 80))),
                            );
                        }

                        plot_ui.vline(
                            VLine::new("Mean", res.mean)
                                .color(Color32::from_rgb(80, 220, 120))
                                .stroke(Stroke::new(1.5, Color32::from_rgb(80, 220, 120))),
                        );
                    });
            } else {
                ui.label(
                    RichText::new("No sweep results harvested yet. Click 'Dispatch Cluster Sweep' to start execution.")
                        .italics()
                        .color(Color32::from_rgb(140, 155, 175)),
                );
            }
        });
    }
}
