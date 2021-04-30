use std::f32::consts::PI;

use crate::signals::*;
use num_complex::Complex32;
use rand::{Rng, SeedableRng};
use tap::Pipe;

const TMP: [f32; 10] = [0.0, -0.1, 1.0, -0.1, 0.05, -0.01, 0.0, 0.0, 0.0, 0.0];

pub const TMP2: [f32; 12] = [
    -0.0000, -0.1912, 0.9316, 0.2821, -0.1990, 0.1630, -0.1017, 0.0544, -0.0261, 0.0090, 0.0000,
    -0.0034,
];

#[optargs::optfn]
pub fn channel(transmission: SignalVec, snr: Option<f32>, timing_error: Option<bool>) -> SignalVec {
    // let mut rng = rand::thread_rng();
    let mut rng = rand::rngs::StdRng::seed_from_u64(9999);

    let snr = 10_f32.powf((snr.unwrap_or_else(|| 30.0) / 10.0) as f32);

    let mut h = [Complex32::default(); 64];

    h.iter_mut()
        .skip(7)
        .zip(TMP2.iter())
        .for_each(|(slot, t1)| *slot = Complex32::new(*t1, 0.0));

    // Convolve
    let mut output = transmission.convolve(&h);

    if timing_error.unwrap_or_default() {
        let f_delta = PI * (rng.gen_range(0.0..1.0) / 64.0);
        for (idx, y) in output.iter_mut().enumerate() {
            // Translation of this line:
            // y = y .* exp(1i * f_delta * [1:length(y)]);
            *y = *y * (Complex32::new(0.0, 1.0) * f_delta * (idx as f32)).exp();
        }
    }

    // Noise
    let noise_var = output.variance() / snr;
    for y in output.iter_mut() {
        *y = *y + (0.5 * noise_var).sqrt() * Complex32::new(rng.gen(), rng.gen());
    }

    output
}

#[test]
fn channel_works() {
    let samples = SignalVec {
        inner: [1, 2, 3, 4, 5, 6, 7, 8].to_signal().inner.to_vec(),
    };

    let out = channel(samples, None, None);

    dbg!(out);
}
