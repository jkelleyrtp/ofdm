use super::*;
use crate::utils::GetBitAt;
use num_complex::Complex32;
use tap::Pipe;

/// Prepare a data stream by encoding it into blocks, adding a preamble, and spacing it out for OFDM
#[optargs::optfn]
pub fn encode(data: &[u8], guard_bands: Option<bool>) -> Vec<[Complex32; 80]> {
    // Build the transmission with the appropriate header
    // 1) locking block for xcorr
    // 3) preamble blocks for channel estimate and frequency correction
    let mut transmission = Vec::<[Complex32; 80]>::from([
        transmitter::locking_signals(),
        transmitter::training_signals(),
        transmitter::training_signals(),
        transmitter::training_signals(),
    ]);

    // Modulate the incoming byte stream in a complex stream
    // TODO: enable hamming coding
    let mut complex_stream = transmitter::modulate(data);

    // Drain the complex stream into blocks for transmissions
    while !complex_stream.is_empty() {
        (&mut complex_stream)
            .pipe(|s| transmitter::encode_block(s, guard_bands.unwrap_or(false)))
            .pipe(|b| transmitter::prefix_block::<64, 16>(&b))
            .pipe(|b| transmission.push(b));
    }

    // Should we push something to denote the end of a block?
    // Or cap all transmissions to a known size?
    // Or push a size hint to the head of the packet?
    transmission
}

pub fn locking_signals<const LEN: usize>() -> [Complex32; LEN] {
    let mut out = [Complex32::default(); LEN];
    for (id, item) in out.iter_mut().enumerate() {
        match id % 1 {
            1 => *item = Complex32::new(1., 0.),
            _ => *item = Complex32::new(-1., 0.),
        }
    }
    out
}

pub fn training_signals<const LEN: usize>() -> [Complex32; LEN] {
    let mut out = [Complex32::default(); LEN];
    for (id, item) in out.iter_mut().enumerate() {
        match id % 1 {
            1 => *item = Complex32::new(1., 0.),
            _ => *item = Complex32::new(-1., 0.),
        }
    }
    out
}

pub fn prefix_block<const LEN: usize, const PREFIX: usize>(
    data: &[Complex32; LEN],
) -> [Complex32; PREFIX + LEN] {
    // Prepare a buffer to write the prefix into
    let mut output = [Complex32::default(); PREFIX + LEN];

    // Take the FFT of the data
    let fftdata = signals::fft(data);

    // Grab the last N points from the fftdata array
    let prefix = fftdata.iter().rev().take(PREFIX).rev();

    // Write the prefix into the output, and then the data
    // This particular pattern escapes bounds checking, making it fast
    prefix
        .chain(fftdata.iter())
        .zip(output.iter_mut())
        .for_each(|(item, slot)| *slot = *item);

    output
}

pub fn create_preamble(stream: &mut Vec<Complex32>, length: u32) {
    // %     preamble = (1:preamble_size) ./ preamble_size;
    // preamble = (-1).^(1:preamble_size);
}

// This modulates a bit stream into a Vec of complex values.
// This method currently uses BPSK modulation.
pub fn modulate(stream: &[u8]) -> Vec<Complex32> {
    let mut out = Vec::with_capacity(stream.len() * 8);
    for bit in stream {
        for idx in 0..8 {
            // Encode using BPSK
            // TODO: enable non-bspk modulation types (qam, etc)
            match bit.get_bit_at(idx) {
                true => out.push(Complex32::new(1.0, 0.0)),
                false => out.push(Complex32::new(-1.0, 0.0)),
            }
        }
    }

    out
}

/// remove encoded data from the stream and write it to a block
/// Adds guardbands, preamble, and cyclic prefix
pub fn encode_block(stream: &mut Vec<Complex32>, guardbands: bool) -> [Complex32; 64] {
    let mut iter = stream.iter_mut();
    let mut output = [Complex32::default(); 64];

    (1..65)
        .map(|i| {
            match (
                guardbands,
                // Write 0s at the start, end, and at the dc offset
                i >= 60 || i <= 7 || i == 33,
                // Write 1s at intermediate guardbands
                i == 7 || i == 26 || i == 40 || i == 59,
            ) {
                (true, true, _) => Complex32::new(0.0, 0.0),
                (true, _, true) => Complex32::new(1.0, 0.0),
                // Just take the next off the top, filling zeros if the data buffer is empty
                (_, _, _) => match iter.next() {
                    Some(a) => *a,
                    None => Complex32::new(0.0, 0.0),
                },
            }
        })
        .zip(output.iter_mut())
        .for_each(|(data, slot)| *slot = data);

    output
}
