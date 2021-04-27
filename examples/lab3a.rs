#![allow(non_upper_case_globals)]
use num_complex::Complex32;
use ofdm::*;
use utils::Analysis;

const block_size: u32 = 0;
const prefix_length: u32 = 0;
const training_blocks: u32 = 0;
const estimation_blocks: u32 = 0;
const preamble_blocks: u32 = 0;
const guard_bands: bool = false;

fn main() {
    use tap::Pipe;

    let source_data = utils::create_transmission(block_size);

    let received_data = source_data
        .as_slice()
        // 1) Encode the data
        .pipe(|data: &[u8]| {
            ofdm::encode!(data, guard_bands)
                .into_iter()
                .flat_map(|f| std::array::IntoIter::new(f))
                .collect::<Vec<Complex32>>()
        })
        // 2) Pass through the channel
        .pipe(|transmission| ofdm::channel!(transmission, snr: 30.0))
        // 3) Receive and decode the samples
        .pipe(|samples| {
            ofdm::decode!(
                samples,
                block_size,
                prefix_length,
                training_blocks,
                estimation_blocks,
                preamble_blocks,
                guard_bands,
            )
        });

    // Compare the sent data to the original
    let analysis = Analysis::new(&source_data, &received_data);
}
