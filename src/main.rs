extern crate core;

mod components;
mod constants;
mod ecs;
mod metrics;
mod web;


use crate::components::Args;
use crate::ecs::*;
use crate::metrics::*;

use clap::Parser;
use crossbeam_channel::unbounded;
use std::io;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{error, info, Level};
use tracing_subscriber::EnvFilter;

fn main() -> io::Result<()> {
    // Init the logging
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(true)
        .with_thread_ids(false)
        .with_env_filter(EnvFilter::new("info"));
    subscriber.init();

    // Parse command line arguments
    let args = Args::parse();

    // Determine optimal thread count based on event rate
    let thread_count = std::cmp::min(
        args.threads.unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(NonZeroUsize::get)
                .unwrap_or(1)
        }),
        std::cmp::max(1, (args.rate as usize + 999) / 1000) // Use at most 1 thread per 1000 events/sec
    );

    info!("AI Compliance ECS Demo");
    info!("Target processing rate: {} events/second", args.rate);
    info!("Using {} worker threads", thread_count);
    info!("Reporting interval: {} seconds", args.interval);

    // Calculate events per thread and batch with more efficient batching
    let events_per_thread = args.rate as usize / thread_count;
    let min_batch_size = 10;
    let events_per_batch = std::cmp::max(min_batch_size, events_per_thread / 10);

    // Calculate the actual events per second per thread
    let events_per_sec_per_thread = if thread_count > 0 {
        std::cmp::max(1, args.rate / thread_count as u32)
    } else {
        args.rate
    };

    info!("Events per thread: {}/sec, Events per batch: {}, Batch time target: {} ms",
             events_per_sec_per_thread,
             events_per_batch,
             if events_per_sec_per_thread > 0 {
                 (1000.0 * events_per_batch as f64 / events_per_sec_per_thread as f64) as u64
             } else {
                 1000
             });

    // Set up channels for metrics reporting
    let (metrics_sender, metrics_receiver) = unbounded();

    // Set up a stop signal for graceful shutdown
    let stop_signal = Arc::new(AtomicBool::new(false));

    // Create a shared metrics object for the web dashboard
    let shared_metrics = Arc::new(RwLock::new(ComplianceMetrics::default()));
    let web_metrics = shared_metrics.clone();

    // Launch worker threads
    let mut worker_handles = Vec::with_capacity(thread_count);
    for thread_id in 0..thread_count {
        let thread_sender = metrics_sender.clone();
        let thread_stop = stop_signal.clone();

        info!("Starting worker thread {} with rate {}/sec", thread_id, events_per_sec_per_thread);

        let handle = thread::spawn(move || {
            worker_thread(
                events_per_batch,
                events_per_sec_per_thread,
                thread_stop,
                thread_sender
            );
        });

        worker_handles.push(handle);
    }

    // Launch web dashboard in a separate async runtime
    let web_stop = Arc::clone(&stop_signal);  // Use Arc::clone to be explicit
    let web_metrics_clone = Arc::clone(&web_metrics);
    let web_handle = thread::spawn(move || {
        // Create a new tokio runtime for the web server
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Create a future that resolves when stop signal is set
            let web_stop_captured = Arc::clone(&web_stop); // Capture in async block
            let shutdown_signal = async move {
                while !web_stop_captured.load(Ordering::Relaxed) {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            };

            // Start the web dashboard
            web::start_server(web_metrics_clone, shutdown_signal).await;
        });
    });

    // Metrics aggregation variables
    let mut total_metrics = ComplianceMetrics::default();
    let mut last_report_time = Instant::now();
    let mut metrics_since_last = ComplianceMetrics::default();

    // Set up Ctrl+C handler for graceful shutdown
    let ctrl_c_stop = stop_signal.clone();
    ctrlc::set_handler(move || {
        info!("Received Ctrl+C, shutting down gracefully...");
        ctrl_c_stop.store(true, Ordering::Relaxed);
    }).expect("Error setting Ctrl+C handler");

    // Main loop: aggregate metrics and update shared state
    while !stop_signal.load(Ordering::Relaxed) {
        while let Ok(metrics) = metrics_receiver.try_recv() {
            total_metrics.merge(&metrics);
            metrics_since_last.merge(&metrics);
        }

        if last_report_time.elapsed() >= Duration::from_secs(args.interval) {
            let elapsed = last_report_time.elapsed();

            // Calculate and print the actual processing rate
            let events_per_sec = metrics_since_last.total_events as f64 / elapsed.as_secs_f64();
            info!("Processing rate: {:.2} events/second ({:.2}M/s)",
                     events_per_sec,
                     events_per_sec / 1_000_000.0);

            // Update metrics
            total_metrics.update_historical_data(metrics_since_last.total_events, elapsed);

            // Update shared metrics for web dashboard
            if let Ok(mut web_metrics_guard) = shared_metrics.try_write() {
                *web_metrics_guard = total_metrics.clone();
            }

            last_report_time = Instant::now();
            metrics_since_last = ComplianceMetrics::default();
        }

        // Sleep to avoid busy-waiting in the main thread
        thread::sleep(Duration::from_millis(50));
    }

    info!("Waiting for web dashboard to shut down...");
    web_handle.join().expect("Web dashboard thread panicked");

    info!("Waiting for worker threads to finish...");
    for (i, handle) in worker_handles.into_iter().enumerate() {
        if let Err(e) = handle.join() {
            error!("Worker thread {} panicked: {:?}", i, e);
        }
    }

    info!("Shutdown complete.");
    Ok(())
}