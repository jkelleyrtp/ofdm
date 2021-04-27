use num_complex::Complex32;
use tap::Pipe;

#[optargs::optfn]
pub fn decode(
    samples: Vec<Complex32>,
    block_size: Option<u32>,
    prefix_length: Option<u32>,
    training_blocks: Option<u32>,
    estimation_blocks: Option<u32>,
    preamble_blocks: Option<u32>,
    guard_bands: Option<bool>,
) -> Vec<u8> {
    todo!()
}

fn demodulate(stream: Vec<Complex32>) -> Vec<u8> {
    todo!()
}
