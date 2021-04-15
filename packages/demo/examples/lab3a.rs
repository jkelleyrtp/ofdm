#![allow(non_upper_case_globals)]
use demo::*;

const block_size: u32 = 0;
const prefix_length: u32 = 0;
const training_blocks: u32 = 0;
const estimation_blocks: u32 = 0;
const preamble_blocks: u32 = 0;
const guard_bands: bool = false;

fn main() {
    // We create the transmission block
    Some(utils::create_transmission(block_size))
        .map(|data| {
            // Encode the data with the transmitter
            demo::transmit!(
                data,
                block_size,
                prefix_length,
                training_blocks,
                estimation_blocks,
                preamble_blocks,
                guard_bands,
            )
        })
        .map(|transmission| {
            // Pass the transmission through the channel
            channel!(transmission, snr: 0.0)
        })
        .map(|samples| {
            // Receive and decode the data
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
