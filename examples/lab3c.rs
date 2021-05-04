#![allow(non_upper_case_globals)]
#![feature(slice_as_chunks)]
use colored::Colorize;
use num::complex::Complex64;
use ofdm::*;
use std::fs::File;
use std::io::prelude::*;

// Just enough for the whole poem
const num_bytes: usize = 500;
const guard_bands: bool = true;
const ecc_enabled: bool = true;
const modulation: ModulationScheme = ModulationScheme::Bpsk;

fn transmit(path: &str) {
    let data = utils::create_transmission_text(num_bytes, ecc_enabled);

    let samples: Vec<Complex64> = ofdm::encode!(data: data.as_ref(), guard_bands, modulation);

    crate::plots::stem_plot(&samples);

    let mut file = File::create(path).unwrap();
    file.write_all(&utils::sig_to_bytes(samples)).unwrap();
}

fn receive(path: &str, start: Option<usize>, stop: Option<usize>) {
    let mut file = File::open(path).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    let samples = unsafe { utils::bytes_to_sig(buf) };

    let samples = samples[start.unwrap_or(0)..stop.unwrap_or_else(|| samples.len())].to_vec();

    dbg!(samples.len());

    let received_data = ofdm::decode!(samples, guard_bands, modulation).expect("Failed to decode");

    // Debug the output
    let source_data = utils::create_transmission_text(num_bytes, ecc_enabled);

    // Print the data to the terminal
    // utils::debug_data(source_data.as_ref(), &received_data[..]);

    // Compare the sent data to the original
    dbg!(utils::Analysis::new(source_data.as_ref(), &received_data));

    // Convert the bytes back into the original message
    println!(
        "{}",
        utils::decipher_transmission_text(num_bytes, received_data, ecc_enabled)
            .unwrap()
            .green()
    );
}

/// Run lab3c
#[derive(argh::FromArgs)]
struct CmdArgs {
    /// create the transmission file
    #[argh(option)]
    transmit: Option<String>,

    /// decode the samples from file
    #[argh(option)]
    receive: Option<String>,

    /// start slice of the received data
    #[argh(option)]
    start: Option<usize>,

    /// stop slice of received data
    #[argh(option)]
    stop: Option<usize>,
}

fn main() {
    ofdm::logging::set_up_logging("ofdm");
    let cfg: CmdArgs = argh::from_env();
    match (cfg.transmit, cfg.receive) {
        (Some(t), None) => transmit(&t),
        (None, Some(t)) => receive(&t, cfg.start, cfg.stop),
        _ => panic!("Not a valid argument combination, specify transmit or receive, but not both"),
    }
}
