use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::time::Instant;

use crate::config;
use crate::message::Message;

pub fn writer_thread(output_filename: &str, rx: Receiver<Message>) {
    if let Err(err) = run_writer(output_filename, rx) {
        eprintln!("Writer Error: {err}");
    }
}

fn run_writer(output_filename: impl AsRef<Path>, rx: Receiver<Message>) -> io::Result<()> {
    let file = File::create(output_filename)?;
    let mut writer = BufWriter::new(file);

    writeln!(
        writer,
        "SyncByte,Byte1,Byte2,SensorValue1,Byte3,Byte4,SensorValue2,CS\n",
    )?;

    let mut last_display_time = Instant::now();

    for message in rx.iter() {
        write!(writer, "{message}")?;

        let now = Instant::now();
        if now.duration_since(last_display_time) >= config::PRINT_SUBSAMPLE_RATE {
            print!("{message}");
            last_display_time = now;
        }
    }

    writer.flush()?;
    Ok(())
}
