//! Important functions for doing signal processing not normally included in Rust.
//! These are included manually so they can be swapped out with the realtime implementation.

use std::ops::Div;

use num_complex::{Complex, Complex32};
use rustfft::FftPlanner;

pub trait Signal {
    /// Perform the fast-foruier transform in place
    fn fft(&mut self) {}

    /// Perform the inverse fast-foruier transform
    fn ifft(&mut self);

    /// Shift the data using an fft_shift approach
    fn fft_shift(&mut self);

    /// Return the index of the largest correlationan and the correlations themselves
    fn xcorr(&self, other: &[Complex32]) -> (usize, Vec<Complex32>) {
        todo!()
    }

    fn variance(&self) -> Complex32 {
        todo!()
    }

    fn convolve(&self, other: &[Complex32]) -> Vec<Complex32>;
    fn mean(&self) -> Complex32;
}

/// These methods are currently implemented for const slices
/// Currently, they are CPU only, but in the future we'd like to enable SMD and GPU (with Cuda)
impl<const LEN: usize> Signal for [Complex32; LEN] {
    fn fft(&mut self) {
        let mut planner = rustfft::FftPlanner::new();
        let fft = planner.plan_fft_forward(LEN);
        fft.process(self);
    }

    fn ifft(&mut self) {
        let mut planner = rustfft::FftPlanner::new();
        let fft = planner.plan_fft_inverse(LEN);
        fft.process(self);
    }

    fn fft_shift(&mut self) {
        todo!()
    }

    // Inspiration taken from:
    // https://github.com/scott97/fyrp/blob/master/rust/bubble-lib/src/xcorr.rs
    fn xcorr(&self, other: &[Complex32]) -> (usize, Vec<Complex<f32>>) {
        let mut out = vec![Complex32::default(); self.len()];

        // ... yes this is O(n * k)
        // We can implement an SMD version of this or just go for the FFT
        // With small enough k it's not too bad
        for idx_self in 0..self.len() {
            for idx_other in 0..other.len() {
                let peak = idx_self + idx_other;
                out[idx_self] += self.get(peak).unwrap_or(&Complex::default()) * other[idx_other];
            }
        }

        let max = Complex32::default();
        let mut idx_max = 0;

        // a bit wasteful to do an extra pass
        for (idx, val) in out.iter().enumerate() {
            if val.norm_sqr() > max.norm_sqr() {
                idx_max = idx;
            }
        }

        (idx_max, out)
    }

    fn mean(&self) -> Complex32 {
        (self as &[Complex32]).mean()
    }

    fn convolve(&self, other: &[Complex32]) -> Vec<Complex32> {
        (self as &[Complex32]).convolve(other)
    }
}

impl Signal for &[Complex32] {
    fn variance(&self) -> Complex32 {
        let data_mean = self.mean();
        self.iter()
            .map(|value| data_mean - (*value as Complex32))
            .map(|diff| diff * diff)
            .sum::<Complex32>()
            / (self.len() as f32)
    }

    fn mean(&self) -> Complex32 {
        let mut sum: Complex32 = self.iter().sum();
        let len = self.len() as f32;
        sum.re /= len;
        sum.im /= len;
        sum
    }

    fn convolve(&self, other: &[Complex32]) -> Vec<Complex32> {
        convolution_safe(self, other)
    }
    fn ifft(&mut self) {}
    fn fft_shift(&mut self) {}
}

pub fn convolution_safe<'a>(sample: &[Complex32], kernel: &[Complex32]) -> Vec<Complex32> {
    let vec = sample.len();
    let ker = kernel.len();

    if ker > vec {
        return convolution_safe(kernel, sample);
    }

    // if ker == 0 || ker > vec {
    //     panic!("convolve_full expects `self.len() >= kernel.len() > 0`, received {} and {} respectively.",vec,ker);
    // }

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

#[cfg(test)]
mod tests {
    use super::*;
    use rustfft::num_traits::Zero;
    use rustfft::{algorithm::Radix4, num_traits::One};

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
        let x = [Complex::from(1.0), Complex::from(2.0), Complex::from(3.0)];
        let h = [Complex::from(4.0), Complex::from(5.0)];
        let expected = [
            Complex32::from(14.0),
            Complex32::from(23.0),
            Complex32::from(12.0),
        ];
        let (idx, lags) = x.xcorr(&h);
        dbg!(idx);
        assert_eq!(lags, expected);

        // more complex
        let x = [
            Complex32::one(),
            Complex32::one(),
            Complex32::zero(),
            Complex32::zero(),
            Complex32::one(),
            Complex32::one(),
            Complex32::zero(),
            Complex32::zero(),
        ];

        let h = [
            Complex32::one(),
            Complex32::one(),
            Complex32::zero(),
            Complex32::zero(),
        ];

        let expected = [
            Complex::from(2.0),
            Complex::from(1.0),
            Complex::zero(),
            Complex::from(1.0),
            Complex::from(2.0),
            Complex::from(1.0),
            Complex::zero(),
            Complex::zero(),
        ];
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
