use num::complex::Complex64;

use crate::utils;
use crate::{packets::Header, plots};
use crate::{plots::stem_plot, signals::*, transmitter};
use transmitter::ModulationScheme;

#[optargs::optfn]
pub fn decode(
    mut samples: Vec<num::complex::Complex64>,
    guard_bands: Option<bool>,
    modulation: Option<crate::ModulationScheme>,
) -> anyhow::Result<Vec<u8>> {
    log::debug!("Decoding...");

    let guard_bands = guard_bands.unwrap_or_else(|| false);

    // hardcode the delay from the signal
    // we should do this with xcorr but that's a bit slow, unfortunately
    let (idxmax, cross) = samples.xcorr_fft(transmitter::locking_signal::<80>());
    let offset = dbg!(idxmax as i32 - (((cross.len() - 1) / 2) as i32 + 1));

    // plots::stem_plot(&cross);

    let samples = samples.split_off(offset as usize);

    if samples.len() < 800 {
        return Err(anyhow::anyhow!("Input not long enough, bailing early"));
    } else {
        dbg!(samples.len());
    }
    // plots::constellation(&samples);

    // Create an iterator over all the chunks we received
    // We need to pad the final chunk because we added zeros to fill in the gaps made by guardbands
    let mut chunks = split_into_chunks(samples);

    // Calculate the frequency offset
    let f_delta = dbg!(frequency_correction(&chunks[3], &chunks[4]));

    utils::write_to_numpy_file(&chunks[6], "preq_correction_3a");

    // Apply the frequency offset
    let mut sample_id = 0;
    for chunk in &mut chunks {
        for sample in chunk {
            *sample *= (Complex64::new(0.0, -1.0) * f_delta * (sample_id as f64)).exp();
            sample_id += 1;
        }
    }

    utils::write_to_numpy_file(&chunks[6], "post_correction_3a");

    assert_eq!(sample_id, chunks.len() * 80);

    let h_k = estimate_channel(&chunks[5..10]);

    utils::write_to_numpy_file(&h_k, "hk_estimate_3a");
    // stem_plot(&h_k);

    dbg!(&f_delta);

    let mut out_stream = Vec::new();
    for chunk in &chunks[10..] {
        let mut unprefixed = unprefix_block(chunk);

        // Apply the channel correction
        for (o, h) in unprefixed.iter_mut().zip(h_k.iter()) {
            *o /= h;
        }

        // Decode the block and push it into the output stream
        decode_block(unprefixed, &h_k, guard_bands, &mut out_stream);
    }

    utils::write_to_numpy_file(&out_stream, "no_phaseoffset");
    // utils::write_to_numpy_file(&out_stream, "with_phasoffset");
    // utils::write_to_numpy_file(&out_stream, "decoded_3a");

    // plots::constellation(&out_stream[..230 * 8]);

    //  Demodulate the output stream into bytes
    let mut decoded = demodulate(out_stream, modulation.unwrap_or(ModulationScheme::Bpsk));

    // Parse off the header
    let header_len =
        bincode::serialized_size(&crate::packets::Header { packet_length: 0 }).unwrap();
    let header_bytes = decoded.drain(0..header_len as usize).collect::<Vec<_>>();
    let header: Header = bincode::deserialize(&header_bytes).unwrap();

    // Trim the received data to the header
    // In the future, we'll want to do this earlier to not waste computation
    decoded.truncate(header.packet_length as usize);

    Ok(decoded)
}

/// Remove the cyclic prefix and then write into the buffer
pub fn unprefix_block(input: &[Complex64; 80]) -> [Complex64; 64] {
    let mut output = [Complex64::default(); 64];
    output.copy_from_slice(&input[16..]);
    output.fft();
    output
}

pub fn decode_block(
    input: [Complex64; 64],
    hk: &[Complex64; 64],
    guard_bands: bool,
    output: &mut Vec<Complex64>,
) {
    let mut input_iter = std::array::IntoIter::new(input).enumerate();

    let pilot_count: f64 = 4.0;

    let mut phase_offset: f64 = 0.0;
    let mut samples_counted = 0;

    while let Some((i, next)) = input_iter.next() {
        match i {
            // dc offset, sidebands, just skip
            i if guard_bands && (i >= 59 || i <= 5 || i == 32) => {}

            // pilot tones
            i if guard_bands && (i == 6 || i == 25 || i == 39 || i == 58) => {
                phase_offset = phase_offset + angle(input[i] / Complex64::new(1.0, 0.0));
                // phase_offset = phase_offset + angle(input[i] / hk[i]);
            }

            _ => {
                samples_counted += 1;
                output.push(next);
            }
        }
    }

    phase_offset /= pilot_count as f64;

    // go back to all the samples we pushed in and correct them retroactively.
    output
        .iter_mut()
        .rev()
        .take(samples_counted)
        .for_each(|f| *f = *f * (Complex64::new(0.0, -1.0) * phase_offset).exp())
}

pub fn demodulate(stream: Vec<Complex64>, scheme: ModulationScheme) -> Vec<u8> {
    let mut out = Vec::new();

    let sliced = stream.into_boxed_slice();
    let (out_chunks, remainder) = sliced.as_chunks::<8>();

    assert_eq!(remainder.len(), 0);

    for chunk in out_chunks {
        match scheme {
            ModulationScheme::Bpsk => {
                let mut bools = [false; 8];
                chunk
                    .iter()
                    .zip(bools.iter_mut())
                    .for_each(|(sample, slot)| *slot = sample.re > 0.0);
                out.push(crate::utils::bools_to_u8(bools));
            }
            ModulationScheme::Qpsk => {
                let mut bools = [false; 16];
                for (idx, &Complex64 { re, im }) in chunk.iter().enumerate() {
                    let bid = idx * 2;
                    let (l, r) = match (re, im) {
                        (_, _) if (re >= 0.0 && im >= 0.0) => (true, true),
                        (_, _) if (re >= 0.0 && im <= 0.0) => (true, false),
                        (_, _) if (re < 0.0 && im > 0.0) => (false, true),
                        (_, _) if (re < 0.0 && im < 0.0) => (false, false),
                        _ => (false, false),
                    };
                    bools[bid] = l;
                    bools[bid + 1] = r;
                }
                for bool_arr in bools.array_chunks::<8>() {
                    out.push(crate::utils::bools_to_u8(*bool_arr));
                }
                // out.push(crate::utils::bools_to_u8(bools[0..8]));
                // out.push(crate::utils::bools_to_u8(bools[8..]));
            }
            ModulationScheme::Qam => {}
        }
    }

    out
}

pub fn split_into_chunks(samples: Vec<Complex64>) -> Vec<[Complex64; 80]> {
    let mut samples = samples.into_boxed_slice();

    let (chunks, remainder) = samples.as_chunks_mut::<80>();
    let mut chunk_vec = Vec::from(chunks);

    if remainder.len() > 0 {
        chunk_vec.push(pad_chunk(remainder));
    }

    chunk_vec
}

/// split into chunks
pub fn pad_chunk(remainder: &[Complex64]) -> [Complex64; 80] {
    let mut out = [Complex64::default(); 80];
    out[..remainder.len()].copy_from_slice(remainder);
    out
}

pub fn estimate_channel(locking_blocks: &[[Complex64; 80]]) -> [Complex64; 64] {
    assert_eq!(locking_blocks.len(), 5);
    let mut hk = [Complex64::default(); 64];

    let training = transmitter::training_signals::<80>();

    for block in locking_blocks.iter() {
        let mut corrected = unprefix_block(block);
        corrected.div_by_other(&training);
        for (id, sample) in corrected.iter().enumerate() {
            hk[id] += sample;
        }
    }

    hk.div_by(locking_blocks.len() as f64);

    hk
}

pub fn frequency_correction(left: &[Complex64; 80], right: &[Complex64; 80]) -> f64 {
    let mut out = [0.0; 80];

    left.iter()
        .zip(right.iter())
        .enumerate()
        .for_each(|(idx, (l, r))| out[idx] = angle(r / l));

    (((out.iter().sum::<f64>()) / 80.0) / 80.0).abs()
}

fn angle(Complex64 { re, im }: Complex64) -> f64 {
    let y = im;
    let x = re;
    y.atan2(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angle_is_ok() {
        // should be -0.7854
        dbg!(angle(Complex64 { re: 1.0, im: -1.0 }));
    }

    #[test]
    fn angle_matches_matlab() {
        // should be -0.7854
        dbg!(angle(Complex64 {
            re: -0.9301897,
            im: 0.366309
        }));

        let a = dbg!(angle(Complex64 {
            re: -0.870127724245302,
            im: 0.0849499100296296
        }));

        dbg!(a / 80.0);

        let a = Complex64 {
            re: 1.562529741252829,
            im: -1.660641994738211,
        };
        let b = Complex64 {
            re: -2.2353334900267217,
            im: 0.45001690562988267,
        };

        dbg!(angle(a / b));
    }

    #[test]
    fn lags() {
        let samples = [1, 2, 3, 4, 5, 6].to_signal();
        let mut lags = -1 * samples.len() as i32..samples.len() as i32;

        let _o = dbg!(lags.nth(2));
        // dbg!(lags.collect::<Vec<_>>());
    }

    #[test]
    fn corr() {
        let samples = [1.0, 2.0, 3.0, 4.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0].to_signal();

        let g = samples.xcorr_fft([1, 2, 3, 4].to_signal());
        dbg!(g);
    }

    fn im(a: i32, b: i32) -> Complex64 {
        Complex64::new(a as f64, b as f64)
    }

    #[test]
    fn assign() {
        // shorthand for re/im syntax
        let cmplx = im(1, -3);

        dbg!(cmplx);
    }
}
