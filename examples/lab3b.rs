#![allow(non_upper_case_globals)]
use colored::Colorize;
use ofdm::*;
use tap::{Pipe, Tap};

const num_bytes: usize = 400;
const guard_bands: bool = false;
const timing_error: bool = true;
const ecc_enabled: bool = false;
const modulation: ModulationScheme = ModulationScheme::Qpsk;

fn main() {
    ofdm::logging::set_up_logging("ofdm");
    let source_data = utils::create_transmission_text(num_bytes, ecc_enabled);

    (&source_data)
        // 1) Encode the data
        .pipe(|data| ofdm::encode!(data, guard_bands, modulation))
        // 2) Pass through the channel
        .pipe(|transmission| ofdm::channel!(transmission, snr: 30.0, timing_error))
        // 3) Receive and decode the samples
        .pipe(|samples| ofdm::decode!(samples, guard_bands, modulation).expect("Failed to decode"))
        // 5) print out the analysis
        .pipe(|received_data| {
            // Print the bit data to the terminal
            // utils::debug_data(source_data.as_ref(), &received_data);

            // Compare the sent data to the original
            dbg!(utils::Analysis::new(source_data.as_ref(), &received_data));
            // Convert the bytes back into the original message
            println!(
                "{}",
                utils::decipher_transmission_text(num_bytes, received_data, ecc_enabled)
                    .expect("Failed to decode text from transmission")
                    .green()
            );
        });
}
