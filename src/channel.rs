use std::f64::consts::PI;

use crate::signals::*;
use num::complex::Complex64;
use rand::Rng;

// Original channel
const _TMP: [f64; 10] = [0.0, -0.1, 1.0, -0.1, 0.05, -0.01, 0.0, 0.0, 0.0, 0.0];

// Original channel expanded
const _TMP2: [f64; 12] = [
    -0.0000, -0.1912, 0.9316, 0.2821, -0.1990, 0.1630, -0.1017, 0.0544, -0.0261, 0.0090, 0.0000,
    -0.0034,
];

// Original channel expanded onto a 64 wide block
//
// This code generated the channe:
//```
//    let mut h = [Complex64::default(); 64];
//    h.iter_mut()
//        .skip(7)
//        .zip(TMP2.iter())
//        .for_each(|(slot, t1)| *slot = Complex64::new(*t1, 0.0));
//```
pub const CHANNEL: [f64; 64] = [
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, -0.0000, -0.1912, 0.9316, 0.2821, -0.1990, 0.1630, -0.1017,
    0.0544, -0.0261, 0.0090, 0.0000, -0.0034, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
];
#[optargs::optfn]
pub fn channel(
    transmission: Vec<num::complex::Complex64>,
    snr: Option<f64>,
    timing_error: Option<bool>,
) -> Vec<num::complex::Complex64> {
    let mut rng = rand::thread_rng();

    let snr = 10_f64.powf((snr.unwrap_or_else(|| 30.0) / 10.0) as f64);

    let h = CHANNEL.clone().to_signal();

    log::debug!("Starting convolve");
    let mut output = transmission.convolve(&h);
    log::debug!("Convolve complete");

    if timing_error.unwrap_or(false) {
        // a problematic value
        // let f_delta = 0.045;

        // The original code divided fdetla by 64
        // Unfortunately, since we use an 80 long block size, this will roll over outside what shmidl cox can handle
        let f_delta = PI * (rng.gen_range(0.0..1.0) / 80.0);

        dbg!(f_delta);

        for (idx, y) in output.iter_mut().enumerate() {
            // Translation of this line:
            let comp: Complex64 = Complex64::new(0.0, 1.0) * f_delta * ((idx + 1) as f64);
            *y = *y * comp.exp();
        }
    }

    // Noise
    let noise_var = output.variance() / snr;
    for y in output.iter_mut() {
        let noise = (0.5 * noise_var).sqrt()
            * Complex64::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0));
        *y += noise;
    }

    output
}

#[test]
fn channel_works() {
    let samples = [1, 2, 3, 4, 5, 6, 7, 8].to_vec().to_signal();
    let out = channel(samples, None, None);

    dbg!(out);
}

#[test]
fn channel_works_timing() {
    let samples = [1, 2, 3, 4, 5, 6, 7, 8].to_vec().to_signal();
    let out = channel(samples, None, Some(true));

    dbg!(out);
}

#[test]
fn channel_makes_sense() {
    let samples = (0..128).map(|_| (1, -1)).collect::<Vec<_>>().to_signal();

    let _out = channel(samples, None, Some(true));
    // dbg!(out.reals());

    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //   -0.0000 + 0.0000i
    //   -0.1912 + 0.1912i
    //    0.7404 - 0.7404i
    //    1.0225 - 1.0225i
    //    0.8234 - 0.8234i
    //    0.9864 - 0.9864i
    //    0.8847 - 0.8847i
    //    0.9391 - 0.9391i
    //    0.9130 - 0.9130i
    //    0.9220 - 0.9220i
    //    0.9220 - 0.9220i
    //    0.9186 - 0.9186i
    //    0.9186 - 0.9186i
    //    0.9186 - 0.9186i
    //    0.9186 - 0.9186i
    //    0.9186 - 0.9186i
    //    0.9186 - 0.9186i
    //    1.1098 - 1.1098i
    //    0.1782 - 0.1782i
    //   -0.1039 + 0.1039i
    //    0.0952 - 0.0952i
    //   -0.0678 + 0.0678i
    //    0.0339 - 0.0339i
    //   -0.0205 + 0.0205i
    //    0.0056 - 0.0056i
    //   -0.0034 + 0.0034i
    //   -0.0034 + 0.0034i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
    //    0.0000 + 0.0000i
}

#[test]
fn basic_math() {
    let f_delta = 0.048486623519511635;

    for i in 1..=10 {
        dbg!((Complex64::new(0.0, 1.0) * f_delta * (i as f64)).exp());
    }
}
