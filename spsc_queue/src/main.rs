use chrono::Local;
use std::{env, process, sync::mpsc, thread, time::Duration};

use spsc_queue::{config, consumer, producer};

/// Generates a timestamped CSV filename using the configured prefix
/// and current local time.
fn get_timestamped_filename() -> String {
    let now = Local::now();
    format!(
        "{}_{}.csv",
        config::FILENAME_PREFIX,
        now.format("%Y-%m-%d_%H-%M-%S")
    )
}

/// Entry point for the SPSC Queue application
///
/// - Parses command-line arguments for the COMPORT, duration in seconds, and
///   an optional output CSV filename.
/// - Acquires a system and display `keepawake` guard.
/// - Spawns a single-producer single-consumer (SPSC) pipeline
///   using `sync::mspc`.
///
/// ## Command Line Arguments
///
/// - `<COMPORT>` Serial Port Identifier (e.g `COM4` or `COM10`). Prefixed with
///   `\\.\` for Windows compatibility on COMPORTs at or above COM10.
/// - `<DURATION (s)>` Recording duration in seconds, parsed as `u64`.
/// - `<OUTPUT_FILENAME?>` Optional output filename for CSV file. Defaults to
///   `config::DEFAULT_OUTPUT_FILENAME`.
///
/// ## Exit Codes and Panics
///
/// - Exits with `-1` if fewer than 2 arguments are passed
///   through the command-line.
/// - Logs a warning to `stderr` and continues without a guard if `keepawake`
///   fails to initialise.
/// - Logs panics from worker threads if `join` returns `Err`.
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("\nMissing required arguments!\n");

        println!("Usage:   spsc_queue <COMPORT> <DURATION (s)> <OUTPUT_FILENAME?>");
        println!("Example: spsc_queue COM4 120");
        println!("Example: spsc_queue COM4 120 test_run\n");

        process::exit(-1);
    }

    // Windows requires \\.\ prefix before COMPORT 10 or above
    let comport = format!("\\\\.\\{}", args[1]);

    // Parse duration arg str->u64->Duration::secs
    let duration_secs: u64 = match args[2].parse() {
        Ok(secs) => secs,
        Err(_) => {
            eprintln!("Failed to parse duration '{}'.", args[2]);
            process::exit(-1);
        }
    };
    let duration = Duration::from_secs(duration_secs);

    // `.csv` extension appended automatically
    let output_filename = if args.len() > 3 {
        format!("{}.csv", args[3])
    } else {
        get_timestamped_filename()
    };

    println!("Recording {comport} to {output_filename} for {duration_secs} seconds.\n");

    // https://docs.rs/keepawake/latest/keepawake/
    let _awake = match keepawake::Builder::default()
        .display(true)
        .idle(true)
        .reason("Recording serial port data!")
        .app_name("spsc_queue")
        .create()
    {
        Ok(guard) => Some(guard),
        Err(err) => {
            eprintln!("Warning: Failed to initialise keepawake guard: {err}");
            None
        }
    };

    let (tx, rx) = mpsc::channel();

    // tx is moved into the producer thread
    // `rx.recv()` in consumer unblocks when `tx` is dropped
    let producer_handle = thread::spawn(move || {
        producer::reader_thread(comport, duration, tx);
    });

    // rx is moved into the consumer thread
    let consumer_handle = thread::spawn(move || {
        consumer::writer_thread(&output_filename, rx);
    });

    if let Err(err) = producer_handle.join() {
        eprintln!("Producer thread panicked: {err:?}");
    }

    if let Err(err) = consumer_handle.join() {
        eprintln!("Consumer thread panicked: {err:?}");
    }

    println!("Recording completed!");
    drop(_awake);
}
