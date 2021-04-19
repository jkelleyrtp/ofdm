#![allow(non_upper_case_globals)]
use demo::*;
use num_complex::Complex32;

const block_size: u32 = 0;
const prefix_length: u32 = 0;
const training_blocks: u32 = 0;
const estimation_blocks: u32 = 0;
const preamble_blocks: u32 = 0;
const guard_bands: bool = false;

fn main() {
    use tap::Pipe;

    utils::create_transmission(block_size)
        // Encode the data
        .pipe(|data| {
            demo::transmit!(data: data.as_ref(), guard_bands)
                .into_iter()
                .flat_map(|f| std::array::IntoIter::new(f))
                .collect::<Vec<Complex32>>()
        })
        // Pass through the channel
        .pipe(|transmission| channel!(transmission, snr: 30.0))
        // Receive and decode the samples
        .pipe(|samples| {
            receive!(
                samples,
                block_size,
                prefix_length,
                training_blocks,
                estimation_blocks,
                preamble_blocks,
                guard_bands,
            )
        });
}
