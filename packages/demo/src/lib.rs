#![feature(const_generics)]
#![feature(const_evaluatable_checked)]
//! This program implements lab3 OFDM using the Rust programming language.
use num_complex::Complex32;
use tap::Pipe;

struct Transmitter {}

impl Transmitter {
    fn transmit(mut self) -> Vec<[Complex32; 80]> {
        todo!()
    }
}

/// Prepare a data stream by encoding it into blocks, adding a preamble, and spacing it out for OFDM
#[optargs::optfn]
pub fn transmit(data: &[u8], guard_bands: Option<bool>) -> Vec<[Complex32; 80]> {
    // Build the transmission with the appropriate header
    // |> 1 locking block for xcorr
    // |> 3 preamble blocks for channel estimate and frequency correction
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

#[optargs::optfn]
pub fn channel(transmission: Vec<Complex32>, snr: Option<f32>) -> Vec<Complex32> {
    todo!()
}

#[optargs::optfn]
pub fn receive(
    samples: Vec<Complex32>,
    block_size: Option<u32>,
    prefix_length: Option<u32>,
    training_blocks: Option<u32>,
    estimation_blocks: Option<u32>,
    preamble_blocks: Option<u32>,
    guard_bands: Option<bool>,
) -> Vec<f32> {
    todo!()
}

pub mod transmitter {
    use crate::utils::GetBitAt;

    use super::*;

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
        let fftdata = utils::fft(data);

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
}
pub mod receiver {}
pub mod utils {
    use super::*;

    pub trait GetBitAt {
        fn get_bit_at(self, n: u8) -> bool;
    }

    impl GetBitAt for u8 {
        fn get_bit_at(self, n: u8) -> bool {
            if n < 8 {
                self & (1 << n) != 0
            } else {
                false
            }
        }
    }

    pub fn create_transmission(packet_size: u32) -> Vec<u8> {
        let text = "abc123";
        let g = text.as_bytes();
        String::from(text).into_bytes();
        todo!()
    }

    pub fn normalize_received_samples(received: u32) {}

    pub fn estimate_error(data: u32, normalized: u32) {}

    /// Perform a fast fourier transform with constantly known array sizes
    pub fn fft<const LEN: usize>(data: &[Complex32; LEN]) -> [Complex32; LEN] {
        todo!()
    }
}

/// Important functions for doing signal processing not normally included in Rust.
/// These are included manually so they can be swapped out with the realtime implementation.
pub mod signals {
    fn xcorr() {}
    fn convolve() {}
    fn fft() {}
    fn ifft() {}
}

mod original {
    /*
    // todo!()
    // % Rehsape the data into the appropraite block sizes

    // let buffer =
    // block_data = zeros(...
    //     size(blockstream, 1), ... % Column length (with prefix if prefixenabled)
    //     size(blockstream, 2) + prefix_length ... % Number of rows
    //     );

    // % Encode the data with a prefix
    // kstop = size(blockstream, 1);
    // for k = 1:kstop
    //     % Simply concat the input and the end of the input to create a cyclical data array
    //     %
    //     % Approach illustrated in:
    //     %    https://dspillustrations.com/pages/posts/misc/the-cyclic-prefix-cp-in-ofdm.html
    //     %    symb1 = hstack([ofdm1[-NCP:], ofdm1])
    //     iffted = ifft(blockstream(k, :));

    //     % Apply the prefix
    //     % Grab out the last N samples where N = prefix_length
    //     pref = iffted(length(iffted) - prefix_length + 1:end);

    //     %         Assign the data
    //     block_data(k, :) = transpose([pref iffted]);
    // end
    */
    /*

    // // Generate locking features
    // // Generate signals to perform the channel estimation
    // // Generate a preamble for frequency correction
    // // Generate guardbands & dc offset

    // // Take in a datastream of 1s and 0s and convert it to 1s and -1s
    // //             bpsk_stream = bin_stream;
    // bpsk_stream = (bin_stream .* 2) - 1;

    // // Reshape the stream into an abitrary # of columns with a fixed size
    // // The number of blocks automatically expands to fit the input data
    // block_data = reshape(...
    //     bpsk_stream, [], self.block_size);

    // training_signals = repmat(...
    //     Utils.training_signals(self.block_size), self.training_blocks, 1);

    // // Preamble the data for frequency offset correction
    // // Make the preamble as wide as the current data is
    // // This scales with the guard band additions
    // preamble = repmat(...
    //     create_preamble(...
    //     self.block_size ...
    //     ), self.preamble_blocks, 1);

    // block_data = [preamble; block_data];

    // block_data = prefix_block(...
    //     [training_signals; block_data], self.prefix_length);

    // // Add guardbands, DC Offset, etc
    // if self.guard_bands
    //     // Instead of shrinking our channels down, we just make them bigger
    //     // That way, the packet size stays the same but the channel utilization increases

    //     // Currently ripped out to fix frequency shifting
    // end

    // // // Preamble the data for frequency offset correction
    // // // Make the preamble as wide as the current data is
    // // // This scales with the guard band additions
    // // preamble = repmat(...
    // //     create_preamble(...
    // //     self.block_size + self.prefix_length ...
    // //     ), self.preamble_blocks, 1);

    // // block_data = [preamble; block_data];

    // // Flatten the block structure down into a single stream
    // //             transpose is very important, for whatever erason
    // samples = reshape(transpose(block_data), 1, []);

    // // Add features to lock onto the signal easier
    // locking = Utils.locking_features();
    // samples = [locking samples];
    */
}
