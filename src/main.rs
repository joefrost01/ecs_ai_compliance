mod components;
mod constants;
mod ecs;
mod metrics;
mod ui;

use crate::components::Args;
use crate::ecs::*;
use crate::metrics::*;
use crate::ui::dashboard::Dashboard;
use crate::ui::tui::{setup_terminal, restore_terminal};

use clap::Parser;
use crossbeam_channel::unbounded;
use std::io;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Main entry point for the AI Compliance ECS Demo application.
fn main() -> io::Result<()> {
    // Parse command line arguments.
    let args = Args::parse();

    // Determine optimal number of worker threads.
    let thread_count = args.threads.unwrap_or_else(|| {
        std::thread::available_parallelism()
            .map(NonZeroUsize::get)
            .unwrap_or(1)
    });

    println!("AI Compliance ECS Demo");
    println!("Target processing rate: {} events/second", args.rate);
    println!("Using {} worker threads", thread_count);
    println!("Reporting interval: {} seconds", args.interval);
    println!("Starting TUI dashboard...");

    // Calculate events per thread and per batch.
    let events_per_thread = args.rate as usize / thread_count;
    let events_per_batch = events_per_thread / 100;

    // Set up channels for metrics reporting and dashboard commands.
    let (metrics_sender, metrics_receiver) = unbounded();
    let (cmd_sender, cmd_receiver) = unbounded();

    // Set up a stop signal for graceful shutdown.
    let stop_signal = Arc::new(AtomicBool::new(false));

    // Launch worker threads.
    let mut worker_handles = Vec::with_capacity(thread_count);
    for _ in 0..thread_count {
        let thread_sender = metrics_sender.clone();
        let thread_stop = stop_signal.clone();
        let handle = thread::spawn(move || {
            worker_thread(events_per_batch, thread_stop, thread_sender);
        });
        worker_handles.push(handle);
    }

    // Metrics aggregation variables.
    let mut total_metrics = ComplianceMetrics::default();
    let mut last_report_time = Instant::now();
    let mut metrics_since_last = ComplianceMetrics::default();

    // Set up Ctrl+C handler for graceful shutdown.
    let ctrl_c_stop = stop_signal.clone();
    ctrlc::set_handler(move || {
        ctrl_c_stop.store(true, Ordering::Relaxed);
    }).expect("Error setting Ctrl+C handler");

    // Launch the TUI dashboard in a separate thread.
    let dashboard_stop = stop_signal.clone();
    let dashboard_handle = thread::spawn(move || {
        let mut terminal = setup_terminal().expect("Failed to setup terminal");
        let mut dashboard = Dashboard::new();
        while !dashboard_stop.load(Ordering::Relaxed) && !dashboard.should_quit {
            // Process incoming dashboard commands.
            while let Ok(cmd) = cmd_receiver.try_recv() {
                dashboard.handle_command(cmd);
            }
            // Render the dashboard UI.
            if let Err(e) = dashboard.render(&mut terminal) {
                eprintln!("Dashboard render error: {:?}", e);
            }
            // Poll for key events with a timeout.
            if crossterm::event::poll(Duration::from_millis(100)).unwrap_or(false) {
                if let crossterm::event::Event::Key(key) = crossterm::event::read().unwrap() {
                    dashboard.handle_key_event(key);
                    if dashboard.should_quit {
                        dashboard_stop.store(true, Ordering::Relaxed);
                    }
                }
            }
        }
        // Restore terminal settings upon exit.
        if let Err(e) = restore_terminal(&mut terminal) {
            eprintln!("Error restoring terminal: {:?}", e);
        }
    });

    // Main loop: aggregate metrics and send dashboard updates.
    while !stop_signal.load(Ordering::Relaxed) {
        while let Ok(metrics) = metrics_receiver.try_recv() {
            total_metrics.merge(&metrics);
            metrics_since_last.merge(&metrics);
        }
        if last_report_time.elapsed() >= Duration::from_secs(args.interval) {
            let elapsed = last_report_time.elapsed();
            total_metrics.update_historical_data(metrics_since_last.total_events, elapsed);
            if let Err(e) = cmd_sender.send(ui::dashboard::DashboardCommand::UpdateMetrics(total_metrics.clone())) {
                eprintln!("Error sending dashboard command: {:?}", e);
            }
            last_report_time = Instant::now();
            metrics_since_last = ComplianceMetrics::default();
        }
        thread::sleep(Duration::from_millis(50));
    }

    // Wait for the dashboard thread to finish.
    dashboard_handle.join().expect("Dashboard thread panicked");
    // Wait for all worker threads to finish.
    for handle in worker_handles {
        handle.join().expect("Worker thread panicked");
    }

    println!("Shutdown complete.");
    Ok(())
}
