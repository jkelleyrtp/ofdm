//! Important functions for doing signal processing not normally included in Rust.
//! These are included manually so they can be swapped out with the realtime implementation.

use std::ops::Div;

use num_complex::{Complex, Complex32};
use rustfft::FftPlanner;
mod impls;
pub use impls::*;

impl<T: AsMut<[Complex32]>> SignalMut for T {}
impl<T: AsRef<[Complex32]>> SignalRef for T {}
pub type SignalConst<const LEN: usize> = [Complex32; LEN];
pub type SignalVec = Vec<Complex32>;
pub type SignalSlice<'a> = &'a [Complex32];

// Allow ergonomic conversions of arrays and vecs into signals
pub trait IntoSignal {
    type SignalOutput: PartialEq;
    fn to_signal(self) -> Self::SignalOutput;
}

// Algorithms that modify a signal vector in place
pub trait SignalMut: AsMut<[Complex32]> {
    /// Perform the fast-foruier transform in place
    fn fft(&mut self) {
        let signal = self.as_mut();

        let mut planner = rustfft::FftPlanner::new();
        let fft = planner.plan_fft_forward(signal.len());
        fft.process(signal);
    }

    /// Perform the inverse fast-foruier transform
    fn ifft(&mut self) {
        let signal = self.as_mut();

        let mut planner = rustfft::FftPlanner::new();
        let fft = planner.plan_fft_inverse(signal.len());
        fft.process(signal);
    }

    /// Shift the data using an fft_shift approach
    fn fft_shift(&mut self) {
        let signal = self.as_mut();

        let len = signal.len();
        let mid = (len as f32 + 1.0) / 2 as f32;

        let mut samples = Vec::with_capacity(len);
        samples.copy_from_slice(signal);
        let (l, r) = samples.split_at_mut(mid.floor() as usize);

        r.iter()
            .chain(l.iter())
            .zip(signal.iter_mut())
            .for_each(|(n, o)| *o = *n);
    }

    /// Invert the fft_shift
    fn ifft_shift(&mut self) {
        let signal = self.as_mut();

        let len = signal.len();
        let mid = (len as f32) / 2 as f32;

        let mut samples = Vec::with_capacity(len);
        samples.copy_from_slice(signal);
        let (l, r) = samples.split_at_mut(mid.floor() as usize);

        r.iter()
            .chain(l.iter())
            .zip(signal.iter_mut())
            .for_each(|(n, o)| *o = *n);
    }
}

pub trait SignalRef: AsRef<[Complex32]> {
    /// Return the index of the largest correlationan and the correlations themselves
    fn xcorr(&self, other: impl AsRef<[Complex32]>) -> (usize, SignalVec) {
        todo!()
    }

    fn convolve(&self, kernel: impl AsRef<[Complex32]>) -> SignalVec {
        let sample = self.as_ref();
        let kernel = kernel.as_ref();

        let vec = sample.len();
        let ker = kernel.len();

        // never let the kernel fail
        if ker > vec {
            return kernel.convolve(self);
        }

        let result_len = sample.len() + kernel.len() - 1;
        let mut conv = vec![Complex32::default(); result_len];

        for i in 0..(vec + ker - 1) {
            let u_i = if i > vec { i - ker } else { 0 };
            let u_f = std::cmp::min(i, vec - 1);

            if u_i == u_f {
                conv[i] += sample[u_i] * kernel[(i - u_i)];
            } else {
                for u in u_i..(u_f + 1) {
                    if i - u < ker {
                        conv[i] += sample[u] * kernel[(i - u)];
                    }
                }
            }
        }
        conv
    }

    fn variance(&self) -> Complex32 {
        let signal = self.as_ref();

        let data_mean = self.mean();
        signal
            .iter()
            .map(|value| data_mean - (*value as Complex32))
            .map(|diff| diff * diff)
            .sum::<Complex32>()
            / (signal.len() as f32)
    }

    fn mean(&self) -> Complex32 {
        let signal = self.as_ref();

        let mut sum: Complex32 = signal.iter().sum();
        let len = signal.len() as f32;
        sum.re /= len;
        sum.im /= len;
        sum
    }
}

// /// These methods are currently implemented for const slices
// /// Currently, they are CPU only, but in the future we'd like to enable SMD and GPU (with Cuda)
// impl<const LEN: usize> Signal for [Complex32; LEN] {
//     // Inspiration taken from:
//     // https://github.com/scott97/fyrp/blob/master/rust/bubble-lib/src/xcorr.rs
//     fn xcorr(&self, other: &[Complex32]) -> (usize, Vec<Complex<f32>>) {
//         let mut out = vec![Complex32::default(); self.len()];

//         // ... yes this is O(n * k)
//         // We can implement an SMD version of this or just go for the FFT
//         // With small enough k it's not too bad
//         for idx_self in 0..self.len() {
//             for idx_other in 0..other.len() {
//                 let peak = idx_self + idx_other;
//                 out[idx_self] += self.get(peak).unwrap_or(&Complex::default()) * other[idx_other];
//             }
//         }

//         let max = Complex32::default();
//         let mut idx_max = 0;

//         // a bit wasteful to do an extra pass
//         for (idx, val) in out.iter().enumerate() {
//             if val.norm_sqr() > max.norm_sqr() {
//                 idx_max = idx;
//             }
//         }

//         (idx_max, out)
//     }

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use super::*;
    use rustfft::num_traits::Zero;
    use rustfft::{algorithm::Radix4, num_traits::One};

    #[test]
    fn ifftshift_works() {
        let mut vals1 = [
            (1.0, 0.0),
            (2.0, 0.0),
            (3.0, 0.0),
            (4.0, 0.0),
            (5.0, 0.0),
            (6.0, 0.0),
            (7.0, 0.0),
        ]
        .to_signal();
        vals1.fft_shift();
        vals1.ifft_shift();
        dbg!(vals1);
    }

    #[test]
    fn fftshift_works() {
        let mut vals1 = [
            (1.0, 0.0),
            (2.0, 0.0),
            (3.0, 0.0),
            (4.0, 0.0),
            (5.0, 0.0),
            (6.0, 0.0),
            (7.0, 0.0),
        ]
        .to_signal();

        vals1.fft_shift();
        dbg!(vals1);
    }

    #[test]
    fn fft_shift_demo_even() {
        let mut data = [1, 2, 3, 4, 5, 6, 7];

        let mid = (data.len() as f32 + 1.0) / 2 as f32;

        let (l, r) = data.split_at_mut(mid.floor() as usize);

        let out: [i32; 7] = r
            .into_iter()
            .chain(l.into_iter())
            .map(|f| *f)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        dbg!(out);
    }

    #[test]
    fn fft_shift_demo_odd() {
        let mut data = [1, 2, 3, 4, 5, 6];

        let mid = (data.len() as f32 + 1.0) / 2 as f32;

        let (l, r) = data.split_at_mut(mid.floor() as usize);

        let out: [i32; 6] = r
            .into_iter()
            .chain(l.into_iter())
            .map(|f| *f)
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        dbg!(out);
    }

    #[test]
    fn conv_works() {
        let vals1 = [
            Complex32::new(1.0, 0.0),
            Complex32::new(2.0, 0.0),
            Complex32::new(3.0, 0.0),
        ];
        let vals2 = [
            Complex32::new(4.0, 0.0),
            Complex32::new(5.0, 0.0),
            Complex32::new(6.0, 0.0),
        ];
        let out = vals2.convolve(&vals1);
        dbg!(out);
    }

    #[test]
    fn mean_works() {
        let vals = [
            Complex32::new(1.0, 1.0),
            Complex32::new(1.0, 2.0),
            Complex32::new(1.0, 3.0),
        ];

        assert!(vals.mean() == Complex32::new(1.0, 2.0))
    }

    // #[rustfmt::skip]
    #[test]
    fn xcorr_works() {
        let x = [1, 2, 3].to_signal();
        let h = [4.0, 5.0].to_signal();
        let expected = [14.0, 23.0, 12.0].to_signal();

        let (idx, lags) = x.xcorr(&h);

        dbg!(idx);
        assert_eq!(lags, expected);

        // more complex
        let x = [1, 1, 0, 0, 1, 1, 0, 0].to_signal();

        let h = [1, 1, 0, 0].to_signal();

        let expected = [2.0, 1.0, 0.0, 1.0, 2.0, 1.0, 0.0, 0.0].to_signal();

        let (idx, lags) = x.xcorr(&h);
        dbg!(idx);
        assert_eq!(lags, expected);
    }

    #[test]
    fn special_xcorr() {
        use rustfft::algorithm::Radix4;
        use rustfft::num_complex::Complex;
        use rustfft::Fft;

        // xcorr implemented but with ffts to be more efficient
        fn xcorr2(a: &mut [Complex32], b: &mut [Complex32]) {
            let n = a.len();
            let fft = Radix4::new(n, rustfft::FftDirection::Forward);
            let ifft = Radix4::new(n, rustfft::FftDirection::Inverse);
            // let mut A: Vec<Complex<f64>> = vec![Complex::new(0.0, 0.0); n];
            // let mut B: Vec<Complex<f64>> = vec![Complex::new(0.0, 0.0); n];

            fft.process(a);
            fft.process(b);

            b.iter_mut().for_each(|c| *c = c.conj());
            // fft.process(&mut ac[..], &mut A);
            // fft.process(&mut bc[..], &mut B);

            // B = B.iter().map(|c| c.conj()).collect();

            let mut ab: Vec<Complex<f32>> = a.iter().zip(b.iter()).map(|(x, y)| x * y).collect();

            ifft.process(&mut ab);

            let max = ab
                .iter()
                .map(|c| c.norm())
                .fold(0.0, |m, x| if x > m { x } else { m });
            // let anorm = a.iter().map(|x| x * x).sum::<f64>().sqrt();
            // let bnorm = b.iter().map(|x| x * x).sum::<f64>().sqrt();

            // max / (anorm * bnorm * (n as f64))
        }
    }
}
