#![allow(non_upper_case_globals)]
use num_complex::Complex32;
use ofdm::*;
use tap::{Pipe, Tap};
use utils::Analysis;

const block_size: usize = 32;
const guard_bands: bool = true;

// todo: implement these
const prefix_length: u32 = 0;
const training_blocks: u32 = 0;
const estimation_blocks: u32 = 0;
const preamble_blocks: u32 = 0;

fn main() {
    let source_data = utils::create_transmission::<block_size>();

    let received_data = source_data
        .as_ref()
        //     // 1) Encode the data
        .pipe(|data| {
            ofdm::encode!(data, guard_bands)
                .into_iter()
                .flat_map(|f| std::array::IntoIter::new(f))
                .collect::<Vec<Complex32>>()
        })
        .tap(|f| {
            dbg!(f.len());
        })
        // 2) Pass through the channel
        .pipe(|transmission| ofdm::channel!(transmission, snr: 1000.0))
        .tap(|f| {
            dbg!(f.len());
        })
        // 3) Receive and decode the samples
        .pipe(|samples| ofdm::decode!(samples, guard_bands))
        .pipe(|mut received| {
            // Only take as many samples as was sent
            let _ = received.split_off(block_size);
            received
        });

    &source_data.iter().zip(&received_data).for_each(|both| {
        println!("{:#012b} \n{:#012b}\n============", both.0, both.1);
    });

    // Compare the sent data to the original
    let analysis = Analysis::new(source_data.as_ref(), &received_data);

    dbg!(&analysis);
    assert!(analysis.num_errs == 0);
}
