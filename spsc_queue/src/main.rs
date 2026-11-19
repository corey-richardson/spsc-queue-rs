use std::{env, process, sync::mpsc, thread, time::Duration};

pub mod config;
pub mod consumer;
pub mod message;
pub mod producer;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("\nMissing required arguments!\n");

        println!("Usage:   spsc_queue <COMPORT> <DURATION (s)> <OUTPUT_FILENAME?>");
        println!("Example: spsc_queue COM4 120");
        println!("Example: spsc_queue COM4 120 test_run\n");

        process::exit(-1);
    }

    let comport = format!("\\\\.\\{}", args[1]);

    let duration_secs: u64 = match args[2].parse() {
        Ok(secs) => secs,
        Err(_) => {
            eprintln!("Failed to parse duration '{}'.", args[2]);
            process::exit(-1);
        }
    };
    let duration = Duration::from_secs(duration_secs);

    let output_filename = if args.len() > 3 {
        format!("{}.csv", args[3])
    } else {
        config::DEFAULT_OUTPUT_FILENAME.to_string()
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

    let producer_handle = thread::spawn(move || {
        producer::reader_thread(comport, duration, tx);
    });

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
