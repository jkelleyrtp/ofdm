#![allow(non_upper_case_globals)]
use num_complex::Complex32;
use ofdm::*;
use tap::{Pipe, Tap};
use utils::Analysis;

const block_size: usize = 16;
const guard_bands: bool = true;
const timing_error: bool = true;

fn main() {
    let source_data = utils::create_transmission::<block_size>();

    source_data
        .as_ref()
        // 1) Encode the data
        .pipe(|data| ofdm::encode!(data, guard_bands))
        // 2) Pass through the channel
        .pipe(|transmission| ofdm::channel!(transmission, snr: 30.0, timing_error))
        // 3) Receive and decode the samples
        .pipe(|samples| ofdm::decode!(samples, guard_bands))
        // 4) post-process the data
        .pipe(|received| utils::trim_to(received, block_size))
        // 5) print out the analysis
        .pipe(|received_data| {
            // Print the data to the terminal
            utils::debug_data(source_data.as_ref(), &received_data);

            // Compare the sent data to the original
            dbg!(Analysis::new(source_data.as_ref(), &received_data));
        });
}
