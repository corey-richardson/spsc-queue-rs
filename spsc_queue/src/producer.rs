use std::collections::VecDeque;
use std::io::{self, ErrorKind};
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};

use crate::config;
use crate::message::Message;

pub fn reader_thread(comport: String, duration: Duration, tx: Sender<Message>) {
    if let Err(err) = run_reader(comport, duration, &tx) {
        eprintln!("Reader Error: {err}")
    }
}

fn run_reader(
    comport: String,
    duration: Duration,
    tx: &Sender<Message>,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    let mut port = serialport::new(comport, config::BAUD_RATE)
        .data_bits(config::DATA_BITS)
        .parity(config::PARITY)
        .stop_bits(config::STOP_BITS)
        .timeout(Duration::from_millis(50))
        .open()?;

    println!("Serial Port Open!\n\n");
    thread::sleep(Duration::from_millis(1000));

    port.clear(serialport::ClearBuffer::All)?;

    let mut receive_data_buffer = [0u8; config::SERIAL_BUFFER_SIZE];

    let mut slider: VecDeque<u8> = VecDeque::with_capacity(config::PACKET_SIZE);

    while start_time.elapsed() < duration {
        match port.read(&mut receive_data_buffer) {
            Ok(bytes_read) if bytes_read > 0 => {
                for &byte in &receive_data_buffer[..bytes_read] {
                    slider.push_back(byte);

                    if slider.len() == config::PACKET_SIZE {
                        if slider.front().copied() == Some(config::SYNC_BYTE) {
                            let received_message = Message {
                                bytes: [slider[0], slider[1], slider[2], slider[3], slider[4]],
                                checksum: slider[5],
                            };

                            if received_message.verify() {
                                if tx.send(received_message).is_err() {
                                    return Ok(());
                                }
                                slider.clear();
                                continue;
                            }
                        }

                        slider.pop_front();
                    }
                }
            }
            Ok(_) => {}
            Err(ref e) if e.kind() == ErrorKind::TimedOut => {}
            Err(e) => return Err(io::Error::from(e).into()),
        }
    }

    Ok(())
}
