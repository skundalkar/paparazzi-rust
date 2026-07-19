//! Command-line entry point for offline PPRZ stream replay.

use std::{env, fs, process::ExitCode};

use pprz_sim::replay;

fn main() -> ExitCode {
    let mut arguments = env::args_os();
    let program = arguments.next().unwrap_or_default();
    let Some(path) = arguments.next() else {
        eprintln!(
            "usage: {} <recorded-pprz-stream.bin>",
            program.to_string_lossy()
        );
        return ExitCode::from(2);
    };
    if arguments.next().is_some() {
        eprintln!(
            "usage: {} <recorded-pprz-stream.bin>",
            program.to_string_lossy()
        );
        return ExitCode::from(2);
    }

    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("cannot read {}: {error}", path.to_string_lossy());
            return ExitCode::from(1);
        }
    };
    let report = replay(bytes);
    println!("accepted frames: {}", report.frames.len());
    println!("rejected frames: {}", report.rejected_frames);
    for message in report.message_counts() {
        println!(
            "aircraft {}, message {}: {}",
            message.aircraft_id, message.message_id, message.count
        );
    }
    ExitCode::SUCCESS
}
