use std::convert::TryInto;

use crate::utils::GetBitAt;
use crate::{packets::Header, signals::*};
use num::complex::Complex64;
use rand::{Rng, SeedableRng};
use tap::Pipe;

/// Prepare a data stream by encoding it into blocks, adding a preamble, and spacing it out for OFDM
#[optargs::optfn]
pub fn encode(
    data: &[u8],
    guard_bands: Option<bool>,
    modulation: Option<crate::ModulationScheme>,
) -> Vec<Complex64> {
    let guard_bands = guard_bands.unwrap_or(false);
    let modulation = modulation.unwrap_or(ModulationScheme::Bpsk);

    let mut out_stream = Vec::new();

    // Add the locking block
    for _ in 0..1 {
        out_stream.extend(locking_signal::<80>().iter());
    }

    // Add the preamble for frequency correction
    for _ in 0..4 {
        out_stream.extend(preamble::<80>().iter())
    }

    // Add the training signals for channel estimation
    for _ in 0..5 {
        out_stream.extend(prefix_block::<64, 16>(&mut training_signals::<64>()).iter());
    }

    // Add a header for the receiver to know how long the transmission is
    let header = Header::new(data.len() as u128);
    let header_bytes = bincode::serialize(&header).unwrap();

    // First, add the header, and then
    // Finally, add the transmission itself
    // Modulate the incoming byte stream in a complex stream
    // Drain the complex stream into blocks for transmissions
    let mut complex_stream = modulate(&header_bytes, &modulation)
        .into_iter()
        .chain(modulate(data, &modulation).into_iter())
        .peekable();

    while complex_stream.peek().is_some() {
        (&mut complex_stream)
            .pipe(|s| encode_block(s, guard_bands))
            .pipe(|mut b| prefix_block::<64, 16>(&mut b))
            .pipe(|b| out_stream.extend(b.iter()));
    }

    normalize(&mut out_stream);
    out_stream
}

pub fn locking_signal<const LEN: usize>() -> [Complex64; LEN] {
    let mut out = [Complex64::default(); LEN];

    for (idx, o) in out.iter_mut().enumerate() {
        let v = 0.5 * ((idx as f64) / (2.0 * LEN as f64) + 0.5);
        *o = Complex64::new(v, 0.0);
    }

    // The fft shift produces the best cross correlation
    out.fft_shift();

    out
}

/// Create a pseudrandom sequence of numbers
pub fn preamble<const LEN: usize>() -> [Complex64; LEN] {
    let mut rng = rand::rngs::StdRng::seed_from_u64(100);
    let mut out = [Complex64::default(); LEN];

    for o in out.iter_mut() {
        *o = Complex64::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)) * 0.25;
    }

    out
}

/// Create an alternating set of complex numbers.
/// This will be used as a known set of data to lock onto later.
pub fn training_signals<const LEN: usize>() -> [Complex64; LEN] {
    let mut rng = rand::rngs::StdRng::seed_from_u64(50);
    let mut out = [Complex64::default(); LEN];

    for o in out.iter_mut() {
        *o = Complex64::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)) * 1.0;
    }
    out
}

pub enum ModulationScheme {
    Bpsk,
    Qpsk,

    // Only 16 qam is implemented
    Qam,
}

// This modulates a bit stream into a Vec of complex values.
// This method currently uses BPSK modulation.
pub fn modulate(stream: &[u8], scheme: &ModulationScheme) -> Vec<Complex64> {
    let mut out = Vec::with_capacity(stream.len() * 8);

    match scheme {
        ModulationScheme::Bpsk => {
            for byte in stream {
                byte.to_bools().iter().for_each(|b| match b {
                    true => out.push(Complex64::new(1.0, 0.0)),
                    false => out.push(Complex64::new(-1.0, 0.0)),
                });
            }
        }

        // TODO
        ModulationScheme::Qpsk => {
            for byte in stream {
                byte.to_bools()
                    .array_chunks::<2>()
                    .for_each(|&[l, r]| match (l, r) {
                        (true, true) => out.push(Complex64::new(1.0, 1.0)),
                        (true, false) => out.push(Complex64::new(1.0, -1.0)),
                        (false, true) => out.push(Complex64::new(-1.0, 1.0)),
                        (false, false) => out.push(Complex64::new(-1.0, -1.0)),
                    });
            }
        }

        // TODO
        ModulationScheme::Qam => {}
    }

    out
}

/// remove encoded data from the stream and write it to a block
/// Adds guardbands, preamble, and cyclic prefix
pub fn encode_block(
    stream: &mut impl Iterator<Item = Complex64>,
    guard_bands: bool,
) -> SignalConst<64> {
    let mut out = [Complex64::default(); 64];

    for i in 0..64 {
        out[i] = match i {
            // dc offset, sidebands, just skip
            i if guard_bands && (i >= 59 || i <= 5 || i == 32) => Complex64::new(0.0, 0.0),

            // pilot tones
            i if guard_bands && (i == 6 || i == 25 || i == 39 || i == 58) => {
                Complex64::new(1.0, 0.0)
            }

            _ => stream.next().unwrap_or_else(|| Complex64::new(0.0, 0.0)),
        }
    }

    out
}

/// Encode the data with an FFT and then add a cyclic prefix
pub fn prefix_block<const LEN: usize, const PREFIX: usize>(
    fftdata: &mut [Complex64; LEN],
) -> SignalConst<{ PREFIX + LEN }> {
    fftdata.ifft();
    let mut out = [Complex64::default(); PREFIX + LEN];

    (&fftdata[(LEN - PREFIX)..])
        .iter()
        .chain(fftdata.iter())
        .enumerate()
        .for_each(|(idx, &f)| out[idx] = f);

    out
}

pub fn normalize(data: &mut Vec<Complex64>) -> &mut Vec<Complex64> {
    let mut max: f64 = 0.0;
    for f in data.iter() {
        max = f64::max(f.re, max);
        max = f64::max(f.im, max);
    }
    for f in data.iter_mut() {
        f.re = f.re / max;
        f.im = f.im / max;
    }
    data
}

#[cfg(test)]
mod tests {
    use crate::plots::stem_plot;

    use super::*;

    #[test]
    fn cyclic_prefix_works() {
        let mut i = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10].to_signal();
        let out = prefix_block::<10, 3>(&mut i);
        dbg!(out);
    }

    #[test]
    fn locking_signals_arent_crazy_high() {
        let sig = locking_signal::<80>();
        stem_plot(&sig);
    }

    #[test]
    fn bands_work() {
        let data = (0..52).collect::<Vec<_>>().to_signal();
        // let data: [Complex64; 52] = (0..52).collect::<Vec<_>>().to_signal().try_into().unwrap();

        let mut data_iter = data.into_iter();

        let mut out = encode_block(&mut data_iter, true);
        dbg!(out.reals());
        dbg!(out.fft_shift().reals());
    }
}
