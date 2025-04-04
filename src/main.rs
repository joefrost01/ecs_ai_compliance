mod components;
mod constants;
mod ecs;
mod metrics;
mod ui;

use crate::components::Args;
use crate::ecs::*;
use crate::metrics::*;
use crate::ui::dashboard::Dashboard;
use crate::ui::tui::setup_terminal;

use clap::Parser;
use crossbeam_channel::unbounded;
use std::io;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

fn main() -> io::Result<()> {
    // Parse command line arguments
    let args = Args::parse();

    // Determine optimal thread count
    let thread_count = args.threads.unwrap_or_else(|| {
        thread::available_parallelism()
            .map(NonZeroUsize::get)
            .unwrap_or(1)
    });

    println!("AI Compliance ECS Demo");
    println!("Target processing rate: {} events/second", args.rate);
    println!("Using {} worker threads", thread_count);
    println!("Reporting interval: {} seconds", args.interval);
    println!("Starting TUI dashboard...");

    // Calculate events per thread per batch
    let events_per_thread = args.rate as usize / thread_count;
    let events_per_batch = events_per_thread / 100; // Split into smaller batches for responsiveness

    // Set up channels for metrics reporting
    let (metrics_sender, metrics_receiver) = unbounded();

    // Channel for dashboard commands
    let (cmd_sender, cmd_receiver) = unbounded();

    // Set up stop signal
    let stop_signal = Arc::new(AtomicBool::new(false));

    // Launch worker threads
    let mut worker_handles = Vec::with_capacity(thread_count);
    for _ in 0..thread_count {
        let thread_sender = metrics_sender.clone();
        let thread_stop = stop_signal.clone();

        let handle = thread::spawn(move || {
            worker_thread(events_per_batch, thread_stop, thread_sender);
        });

        worker_handles.push(handle);
    }

    // Set up metrics reporting
    let mut total_metrics = ComplianceMetrics::default();
    let mut last_report_time = Instant::now();
    let mut metrics_since_last = ComplianceMetrics::default();

    // Set up Ctrl+C handler
    let ctrl_c_stop = stop_signal.clone();
    ctrlc::set_handler(move || {
        ctrl_c_stop.store(true, Ordering::Relaxed);
    }).expect("Error setting Ctrl+C handler");

    // Launch TUI dashboard in a separate thread
    let dashboard_stop = stop_signal.clone();
    let dashboard_handle = thread::spawn(move || {
        let mut terminal = setup_terminal().expect("Failed to setup terminal");
        let mut dashboard = Dashboard::new();

        while !dashboard_stop.load(Ordering::Relaxed) && !dashboard.should_quit {
            // Process any UI commands
            while let Ok(cmd) = cmd_receiver.try_recv() {
                dashboard.handle_command(cmd);
            }

            // Update the UI
            dashboard.render(&mut terminal).expect("Failed to render dashboard");

            // Handle input (with timeout to keep UI responsive)
            if crossterm::event::poll(Duration::from_millis(100)).unwrap() {
                if let crossterm::event::Event::Key(key) = crossterm::event::read().unwrap() {
                    dashboard.handle_key_event(key);

                    if dashboard.should_quit {
                        dashboard_stop.store(true, Ordering::Relaxed);
                    }
                }
            }
        }

        // Clean up terminal
        ui::tui::restore_terminal(&mut terminal).expect("Failed to restore terminal");
    });

    // Main metrics processing loop
    while !stop_signal.load(Ordering::Relaxed) {
        // Collect and aggregate metrics from worker threads
        while let Ok(metrics) = metrics_receiver.try_recv() {
            total_metrics.merge(&metrics);
            metrics_since_last.merge(&metrics);
        }

        // Update metrics at interval
        if last_report_time.elapsed() >= Duration::from_secs(args.interval) {
            let elapsed = last_report_time.elapsed();

            // Update historical data
            total_metrics.update_historical_data(metrics_since_last.total_events, elapsed);

            // Send metrics to the dashboard
            cmd_sender.send(ui::dashboard::DashboardCommand::UpdateMetrics(total_metrics.clone())).unwrap();

            last_report_time = Instant::now();
            metrics_since_last = ComplianceMetrics::default();
        }

        // Sleep a bit to not saturate CPU
        thread::sleep(Duration::from_millis(50));
    }

    // Wait for dashboard thread to finish
    dashboard_handle.join().unwrap();

    // Wait for worker threads to finish
    for handle in worker_handles {
        handle.join().unwrap();
    }

    println!("Shutdown complete.");
    Ok(())
}