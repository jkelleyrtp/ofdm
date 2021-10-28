use std::usize;

use num::complex::Complex64;

use crate::SignalRef;

pub trait GetBitAt {
    fn get_bit_at(self, n: u8) -> bool;
    fn to_bools(self) -> [bool; 8];
}

impl GetBitAt for u8 {
    fn get_bit_at(self, n: u8) -> bool {
        if n < 8 {
            self & (1 << n) != 0
        } else {
            false
        }
    }

    fn to_bools(self) -> [bool; 8] {
        let mut out = [false; 8];
        for (bit, o) in out.iter_mut().enumerate() {
            *o = self & (1 << bit) != 0;
        }
        out
    }
}

pub fn bools_to_u8(bools: [bool; 8]) -> u8 {
    let mut out: u8 = 0;
    for (i, &b) in bools.iter().rev().enumerate() {
        out |= (b as u8) << (7 - i);
    }
    out
}

#[derive(Debug, PartialEq)]
pub struct Analysis {
    pub num_errs: u32,
    pub num_block_errs: u32,
    pub err_rate: f64,
}
impl Analysis {
    pub fn new(left: &[u8], right: &[u8]) -> Self {
        assert_eq!(left.len(), right.len());

        let (num_errs, num_block_errs) =
            left.iter()
                .zip(right.iter())
                .fold((0, 0), |(acc_errs, acc_block_errs), (a, b)| {
                    // Count the number of blocks broken and the number of errs
                    if a != b {
                        let different_bits = (a ^ b).count_ones();
                        (acc_errs + different_bits, acc_block_errs + 1)
                    } else {
                        (acc_errs, acc_block_errs)
                    }
                });

        let err_rate = num_errs as f64 / (left.len() as f64 * 8.0);

        Self {
            num_errs,
            num_block_errs,
            err_rate,
        }
    }
}

const CORPUS: &'static str = r#"
I met a traveller from an antique land,
Who said—“Two vast and trunkless legs of stone
Stand in the desert. . . . Near them, on the sand,
Half sunk a shattered visage lies, whose frown,
And wrinkled lip, and sneer of cold command,
Tell that its sculptor well those passions read
Which yet survive, stamped on these lifeless things,
The hand that mocked them, and the heart that fed;
And on the pedestal, these words appear:
My name is Ozymandias, King of Kings;
Look on my Works, ye Mighty, and despair!
Nothing beside remains. Round the decay
Of that colossal Wreck, boundless and bare
The lone and level sands stretch far away.
"#;

pub fn create_transmission_text(msg_bytes: usize, ecc: bool) -> Vec<u8> {
    if !ecc {
        return CORPUS.clone().bytes().cycle().take(msg_bytes).collect();
    }

    let mut body_iter = CORPUS.clone().bytes().cycle().take(msg_bytes);
    create_transmission_bytes(&mut body_iter)
}

pub fn create_transmission_bytes(byte_iter: &mut impl Iterator<Item = u8>) -> Vec<u8> {
    // We encode all of our data in blocks that are at minimum 256 bytes.
    // 223 bytes are data and 32 bytes are parity
    //
    // n = 255, k = 223, s = 8
    // 2t = 32, t = 16
    //
    // This means we can have 16 block failures for every 255 bytes
    // https://www.cs.cmu.edu/~guyb/realworld/reedsolomon/reed_solomon_codes.html
    use reed_solomon::*;

    let encoder = Encoder::new(32);
    let mut outstream: Vec<u8> = Vec::new();
    let mut byte_idx = 0;
    let mut scratch_buf = [0_u8; 223];
    loop {
        match byte_iter.next() {
            Some(next_byte) => {
                scratch_buf[byte_idx] = next_byte;
                byte_idx += 1;
                if byte_idx == 223 {
                    outstream.extend(&encoder.encode(&scratch_buf)[..]);
                    scratch_buf.fill(0);
                    byte_idx = 0;
                }
            }
            None => {
                // commit what we have and write into the stream
                // Fill with random data to prevent weird fft issues
                // scratch_buf[byte_idx..]
                //     .iter_mut()
                //     .for_each(|f| *f = rand::random());

                outstream.extend(encoder.encode(&scratch_buf).iter());
                break;
            }
        }
    }

    outstream
}

pub fn decipher_transmission_text(num_bytes: usize, data: Vec<u8>, ecc: bool) -> Option<String> {
    if !ecc {
        return String::from_utf8(data).ok();
    }

    let mut body_iter = data.into_iter();

    let mut outstream = decipher_transmission_bytes(&mut body_iter)?;

    let _ = outstream.split_off(num_bytes);
    String::from_utf8(outstream).ok()
}

pub fn decipher_transmission_bytes(byte_iter: &mut impl Iterator<Item = u8>) -> Option<Vec<u8>> {
    use reed_solomon::*;
    let decoder = Decoder::new(32);
    let mut outstream: Vec<u8> = Vec::new();
    let mut byte_idx = 0;
    let mut scratch_buf = [0_u8; 255];

    loop {
        match byte_iter.next() {
            Some(next_byte) => {
                scratch_buf[byte_idx] = next_byte;
                byte_idx += 1;
                if byte_idx == 255 {
                    let decoded = decoder.correct(&scratch_buf, None).ok()?;
                    outstream.extend(decoded.data());
                    scratch_buf.fill(0);
                    byte_idx = 0;
                }
            }
            None => {
                let decoded = decoder.correct(&scratch_buf, None).ok()?;
                outstream.extend(decoded.data());
                break;
            }
        }
    }

    Some(outstream)
}

pub fn decipher_transmision_colorspace(
    bytes_iter: &mut impl ExactSizeIterator<Item = u8>,
    ecc: bool,
) -> Option<Vec<u32>> {
    use crate::packets::colors;

    let data = if ecc {
        log::debug!("ECC enabled, {}", bytes_iter.len());
        decipher_transmission_bytes(bytes_iter)?
    } else {
        bytes_iter.collect()
    };

    let out = data
        .iter()
        .map(|byte| {
            let color = colors::COLORMAP.get(*byte);
            let colors::CustomRgb { r, g, b } = color.rgb;

            ((r as u32) << 16) | ((g as u32) << 8) | b as u32
        })
        .collect();
    Some(out)
}

pub fn debug_data(left: &[u8], right: &[u8]) {
    left.iter()
        .zip(right)
        .enumerate()
        .for_each(|(idx, (sent, received))| {
            use colored::*;
            let out = format!("> {:} | {:#012b} \n     | {:#012b}", idx, sent, received);
            match sent == received {
                true => println!("{}", out.green()),
                false => println!("{}", out.red()),
            }
        });
}

pub fn trim_to(mut received: Vec<u8>, block_size: usize) -> Vec<u8> {
    // Only take as many samples as was sent
    let _ = received.split_off(block_size);
    received
}

/// Convert encoded complex numbers to a byte stream for use in writing to files
pub fn sig_to_bytes(received: Vec<Complex64>) -> Vec<u8> {
    let mut out = Vec::new();
    for sample in received {
        out.extend_from_slice(&(sample.re as f32).to_ne_bytes());
        out.extend_from_slice(&(sample.im as f32).to_ne_bytes());
    }
    out
}

/// Convert a byte stream from the USRP into a stream of Complex64
pub unsafe fn bytes_to_sig(input: Vec<u8>) -> Vec<Complex64> {
    let mut out = Vec::new();

    dbg!(input.len());
    for chunk in input.chunks_exact(8) {
        // panic if we can't take 8 bytes for the complex
        assert_eq!(chunk.len(), 8);
        let re = f32::from_ne_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        let im = f32::from_ne_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
        out.push(Complex64 {
            re: re as f64,
            im: im as f64,
        });
    }

    out
}

pub fn write_to_numpy_file(data: &[Complex64], filename: &'static str) -> anyhow::Result<()> {
    let reals = data.reals();
    let imag = data.imag();

    npy::to_file(format!("data/simulated/{}_reals.npy", filename), reals)?;
    npy::to_file(format!("data/simulated/{}_imag.npy", filename), imag)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::IntoSignal;

    use super::*;

    #[test]
    fn write_to_file() {
        let data = [(6, -1), (5, -2), (4, -3), (3, -4), (2, -5), (1, -6)]
            .to_vec()
            .to_signal();

        write_to_numpy_file(&data, "test").unwrap();
    }

    #[test]
    fn get_bit_at_is_right() {
        let val: u8 = 255;
        (0..8).for_each(|pos| assert_eq!(val.get_bit_at(pos), true));

        let val: u8 = 0;
        (0..8).for_each(|pos| assert_eq!(val.get_bit_at(pos), false));

        let val: u8 = 127;
        (0..7).for_each(|pos| assert_eq!(val.get_bit_at(pos), true));
        assert_eq!(val.get_bit_at(7), false);
    }

    #[test]
    fn errs_is_right() {
        let Analysis {
            num_errs,
            err_rate,
            num_block_errs,
        } = Analysis::new(&[1, 0, 1, 0], &[1, 0, 1, 0]);
        assert_eq!((num_errs, err_rate), (0, 0.0));

        let Analysis {
            num_errs,
            err_rate,
            num_block_errs,
        } = Analysis::new(&[1, 0, 0, 0], &[1, 0, 1, 0]);
        assert_eq!((num_errs, err_rate), (1, 0.25));

        let Analysis {
            num_errs,
            err_rate,
            num_block_errs,
        } = Analysis::new(&[0, 0, 0, 0], &[1, 0, 1, 0]);
        assert_eq!((num_errs, err_rate), (2, 0.50));
    }

    #[test]
    fn create_transmission_cycle() {}

    #[test]
    fn bools_and_back() {
        for num in 0..100_u8 {
            let bools = num.to_bools();
            assert_eq!(bools_to_u8(bools), num);
        }
    }

    #[test]
    fn bytes_and_back() {
        let sig = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9].to_vec().to_signal();

        let bytes = sig_to_bytes(sig);

        let outsig = unsafe { bytes_to_sig(bytes) };

        dbg!(outsig);
    }

    #[test]
    fn ecc_works() {
        use reed_solomon::*;

        // 8 + 248 = 256
        let enc = Encoder::new(8);
        let mut body = CORPUS
            .clone()
            .bytes()
            .cycle()
            .take(249)
            // .take(255)
            // .take(223)
            .collect::<Vec<u8>>();
        let out = enc.encode(&body);
        out.to_vec();
    }

    #[test]
    fn ecc_packets() {
        let ecced = create_transmission_text(1024, true);

        // dbg!(&ecced);
        dbg!(ecced.len());

        let text = decipher_transmission_text(1024, ecced, true);
        dbg!(text);
    }
}
