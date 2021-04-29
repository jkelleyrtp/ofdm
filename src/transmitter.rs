use super::*;
use crate::signals::*;
use crate::utils::GetBitAt;
use num_complex::Complex32;
use tap::Pipe;

/// Prepare a data stream by encoding it into blocks, adding a preamble, and spacing it out for OFDM
#[optargs::optfn]
pub fn encode(data: &[u8], guard_bands: Option<bool>) -> Vec<Complex32> {
    let guard_bands = guard_bands.unwrap_or(false);

    // Build the transmission with the appropriate header
    let mut transmission = Vec::<[Complex32; 80]>::from([
        // xcorr
        transmitter::locking_signals(),
        // preamble for channel estimate and frequency correction
        transmitter::training_signals(),
        transmitter::training_signals(),
        transmitter::training_signals(),
    ]);

    // Modulate the incoming byte stream in a complex stream
    // TODO: enable hamming coding
    let mut modulated = transmitter::modulate(data);
    let mut complex_stream = modulated.drain(..).peekable();

    // Drain the complex stream into blocks for transmissions
    while complex_stream.peek().is_some() {
        (&mut complex_stream)
            .pipe(|s| transmitter::encode_block(s, guard_bands))
            .pipe(|mut b| transmitter::prefix_block::<64, 16>(&mut b))
            .pipe(|b| transmission.push(b));
    }

    // Should we push something to denote the end of a block?
    // Or cap all transmissions to a known size?
    // Or push a size hint to the head of the packet?
    transmission
        .into_iter()
        .flat_map(|f| std::array::IntoIter::new(f))
        .collect::<Vec<Complex32>>()
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

// This modulates a bit stream into a Vec of complex values.
// This method currently uses BPSK modulation.
pub fn modulate(stream: &[u8]) -> Vec<Complex32> {
    let mut out = Vec::with_capacity(stream.len() * 8);
    for bit in stream {
        bit.to_bools().iter().for_each(|b| match b {
            true => out.push(Complex32::new(1.0, 0.0)),
            false => out.push(Complex32::new(-1.0, 0.0)),
        });
    }

    out
}

/// remove encoded data from the stream and write it to a block
/// Adds guardbands, preamble, and cyclic prefix
pub fn encode_block(
    stream: &mut impl Iterator<Item = Complex32>,
    guard_bands: bool,
) -> [Complex32; 64] {
    let mut output = [Complex32::default(); 64];

    (0..64)
        .map(|i| {
            match (
                guard_bands,
                // Write 0s at the start, end, and at the dc offset
                i >= 59 || i <= 6 || i == 32,
                // Write 1s at intermediate guardbands
                i == 6 || i == 25 || i == 39 || i == 58,
            ) {
                (true, true, _) => Complex32::new(0.0, 0.0),
                (true, _, true) => Complex32::new(1.0, 0.0),
                // Just take the next off the top, filling zeros if the data buffer is empty
                (_, _, _) => match stream.next() {
                    Some(a) => a,
                    None => Complex32::new(0.0, 0.0),
                },
            }
        })
        .zip(output.iter_mut())
        .for_each(|(data, slot)| *slot = data);

    output
}

/// Encode the data with an FFT and then add a cyclic prefix
pub fn prefix_block<const LEN: usize, const PREFIX: usize>(
    fftdata: &mut [Complex32; LEN],
) -> [Complex32; PREFIX + LEN] {
    // Take the FFT of the data
    fftdata.ifft();

    // Grab the last N points from the fftdata array
    let prefix = fftdata.iter().rev().take(PREFIX).rev();
    assert_eq!(prefix.len(), PREFIX);

    // Prepare a buffer to write the prefix into
    let mut output = [Complex32::default(); PREFIX + LEN];

    // Write the prefix into the output, and then the data
    // This particular pattern escapes bounds checking, making it fast
    prefix
        .chain(fftdata.iter())
        .zip(output.iter_mut())
        .for_each(|(item, slot)| *slot = *item);

    output
}
