//! Command-line PPRZ telemetry decoder for offline recordings.

use std::{collections::BTreeMap, env, fs, process::ExitCode};

use pprz_messages::parse_dictionary;
use pprz_sim::replay;

fn main() -> ExitCode {
    let mut arguments = env::args_os();
    let program = arguments.next().unwrap_or_default();
    let (Some(recording), Some(dictionary)) = (arguments.next(), arguments.next()) else {
        eprintln!(
            "usage: {} <recorded-pprz-stream.bin> <messages.xml>",
            program.to_string_lossy()
        );
        return ExitCode::from(2);
    };
    if arguments.next().is_some() {
        eprintln!(
            "usage: {} <recorded-pprz-stream.bin> <messages.xml>",
            program.to_string_lossy()
        );
        return ExitCode::from(2);
    }

    let bytes = match fs::read(&recording) {
        Ok(bytes) => bytes,
        Err(error) => {
            return fail(&format!(
                "cannot read {}: {error}",
                recording.to_string_lossy()
            ));
        }
    };
    let xml = match fs::read_to_string(&dictionary) {
        Ok(xml) => xml,
        Err(error) => {
            return fail(&format!(
                "cannot read {}: {error}",
                dictionary.to_string_lossy()
            ));
        }
    };
    let dictionary = match parse_dictionary(&xml, "telemetry") {
        Ok(dictionary) => dictionary,
        Err(error) => return fail(&format!("cannot parse telemetry dictionary: {error:?}")),
    };

    let report = replay(bytes);
    let mut decoded_by_name = BTreeMap::new();
    let mut unknown = 0_usize;
    let mut malformed = 0_usize;
    for frame in &report.frames {
        match dictionary.decode(frame) {
            Ok(message) => *decoded_by_name.entry(message.name).or_insert(0_usize) += 1,
            Err(pprz_messages::DecodeError::UnknownMessage(_)) => unknown += 1,
            Err(_) => malformed += 1,
        }
    }
    println!("transport accepted: {}", report.frames.len());
    println!("transport rejected: {}", report.rejected_frames);
    println!(
        "dictionary decoded: {}",
        decoded_by_name.values().sum::<usize>()
    );
    println!("dictionary unknown: {unknown}");
    println!("dictionary malformed: {malformed}");
    for (name, count) in decoded_by_name {
        println!("{name}: {count}");
    }
    ExitCode::SUCCESS
}

fn fail(message: &str) -> ExitCode {
    eprintln!("{message}");
    ExitCode::from(1)
}
