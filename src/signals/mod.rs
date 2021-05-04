//! Important functions for doing signal processing not normally included in Rust.
//! These are included manually so they can be swapped out with the realtime implementation.

use num::complex::Complex64;

mod impls;
pub use impls::*;

// Blanket implement our signal traits for anything that can be casted as a Slice of Complex
// This enables both primitives and custom wrapper types
impl<T: AsRef<[Complex64]>> SignalRef for T {}
impl<T: AsMut<[Complex64]> + SignalRef> SignalMut for T {}

pub type SignalConst<const LEN: usize> = [Complex64; LEN];
pub type SignalVec = Vec<Complex64>;
pub type SignalSlice<'a> = &'a [Complex64];

// Allow ergonomic conversions of arrays and vecs into signals
pub trait IntoSignal {
    type SignalOutput: PartialEq;
    fn to_signal(self) -> Self::SignalOutput;
}

// Algorithms that modify a signal vector in place
pub trait SignalMut: AsMut<[Complex64]> + AsRef<[Complex64]> {
    /// Perform the fast-foruier transform in place
    fn fft(&mut self) -> &mut Self {
        let mut signal = self.as_mut();
        signal.fft_len(signal.len());
        self
    }

    /// Perform the inverse fast-foruier transformlace
    fn ifft(&mut self) -> &mut Self {
        let mut signal = self.as_mut();
        signal.ifft_len(signal.len());
        self
    }

    // FFT but with a custom len parameter
    fn fft_len(&mut self, len: usize) -> &mut Self {
        rustfft::FftPlanner::new()
            .plan_fft_forward(len)
            .process(self.as_mut());
        self
    }

    /// ifft but with a custom len
    fn ifft_len(&mut self, len: usize) -> &mut Self {
        rustfft::FftPlanner::new()
            .plan_fft_inverse(len)
            .process(self.as_mut());

        // Matlab expects a normalization of the ifft
        // perhaps we shouldn't?
        self.normalize_by(1.0 / len as f64);
        self
    }

    /// Shift the data using an fft_shift approach
    fn fft_shift(&mut self) -> &mut Self {
        let signal = self.as_mut();

        let len = signal.len();
        let mid = (len as f64 + 1.0) / 2 as f64;

        let mut samples = vec![Complex64::default(); len];
        samples.copy_from_slice(signal);
        let (l, r) = samples.split_at_mut(mid.floor() as usize);

        r.iter()
            .chain(l.iter())
            .zip(signal.iter_mut())
            .for_each(|(n, o)| *o = *n);

        self
    }

    /// Invert the fft_shift
    fn ifft_shift(&mut self) -> &mut Self {
        let signal = self.as_mut();

        let len = signal.len();
        let mid = (len as f64) / 2 as f64;

        let mut samples = vec![Complex64::default(); len];
        samples.copy_from_slice(signal);
        let (l, r) = samples.split_at_mut(mid.floor() as usize);

        r.iter()
            .chain(l.iter())
            .zip(signal.iter_mut())
            .for_each(|(n, o)| *o = *n);
        self
    }

    fn div_by(&mut self, by: f64) -> &mut Self {
        let signal = self.as_mut();
        for val in signal.iter_mut() {
            val.re /= by;
            val.im /= by;
        }
        self
    }

    /// Divides two arrays with lossyness
    /// Does not check if both arrays are the same size
    fn div_by_other(&mut self, other: &impl SignalRef) -> &mut Self {
        for (l, r) in self.as_mut().iter_mut().zip(other.as_ref().iter()) {
            *l = *l / *r;
        }
        self
    }

    /// Divides two arrays with lossyness
    /// Does not check if both arrays are the same size
    fn mul_by_other(&mut self, other: &impl SignalRef) -> &mut Self {
        for (l, r) in self.as_mut().iter_mut().zip(other.as_ref().iter()) {
            *l = *l * *r;
        }
        self
    }

    /// Perform the complex conjugate in place
    fn conj(&mut self) -> &mut Self {
        for i in self.as_mut() {
            *i = i.conj();
        }
        self
    }

    fn normalize_by(&mut self, scale: f64) -> &mut Self {
        let signal = self.as_mut();
        for val in signal.iter_mut() {
            *val *= scale;
        }
        self
    }

    fn abs(&mut self) -> &mut Self {
        for i in self.as_mut() {
            *i = Complex64::new(i.norm(), 0.0);
        }
        self
    }
}

pub trait SignalRef: AsRef<[Complex64]> {
    // Inspiration taken from:
    // https://github.com/scott97/fyrp/blob/master/rust/bubble-lib/src/xcorr.rs
    //
    /// Return the index of the largest correlationan and the correlations themselves
    ///
    /// This algorithm is O(n^2).
    /// Prefer the xcorr_fft over this one.
    /// Eventually this will be swapped out with xcorr_fft
    // fn xcorr(&self, other: impl AsRef<[Complex64]>) -> (usize, SignalVec) {
    //     let sig = self.as_ref();
    //     let other = other.as_ref();

    //     let mut out = vec![Complex64::default(); sig.len()];

    //     // ... yes this is O(n * k)
    //     // We can implement an SMD version of this or just go for the FFT
    //     // With small enough k it's not too bad
    //     for idx_self in 0..sig.len() {
    //         for idx_other in 0..other.len() {
    //             let peak = idx_self + idx_other;
    //             out[idx_self] += sig.get(peak).unwrap_or(&Complex64::default()) * other[idx_other];
    //         }
    //     }

    //     let max = Complex64::default();
    //     let mut idx_max = 0;

    //     // a bit wasteful to do an extra pass
    //     for (idx, val) in out.iter().enumerate() {
    //         if val.norm_sqr() > max.norm_sqr() {
    //             idx_max = idx;
    //         }
    //     }

    //     (idx_max, out)
    // }

    fn xcorr_fft(&self, other: impl AsRef<[Complex64]>) -> (usize, SignalVec) {
        let mut a = self.as_ref().to_vec();
        let mut b = other.as_ref().to_vec();

        let a_len = a.len();
        let b_len = b.len();

        // http://matlab.izmiran.ru/help/toolbox/signal/xcorr.html
        let pad_to = 2 * a_len - 1;
        a.extend(vec![Complex64::default(); pad_to - a_len].into_iter());
        b.extend(vec![Complex64::default(); pad_to - b_len].into_iter());

        // https://stackoverflow.com/questions/7396814/cross-correlation-in-matlab-without-using-the-inbuilt-function
        // The xcorr of two values is the product of the ffts where one is conjugated
        // It's a bit dense :x
        a.fft().mul_by_other(b.fft().conj()).ifft().fft_shift();

        let out = a;

        let mut max = Complex64::default();
        let mut idx_max = 0;

        // a bit wasteful to do an extra pass
        for (idx, val) in out.iter().enumerate() {
            if val.norm_sqr() > max.norm_sqr() {
                idx_max = idx;
                max = *val;
            }
        }

        (idx_max, out)
    }

    fn convolve(&self, kernel: impl AsRef<[Complex64]>) -> SignalVec {
        let mut a = self.as_ref().to_vec();
        let mut b = kernel.as_ref().to_vec();

        let a_len = a.len();
        let b_len = b.len();

        // http://matlab.izmiran.ru/help/toolbox/signal/xcorr.html
        let pad_to = a_len + b_len - 1;
        a.extend(vec![Complex64::default(); pad_to - a_len].into_iter());
        b.extend(vec![Complex64::default(); pad_to - b_len].into_iter());

        //https://www.mathworks.com/matlabcentral/answers/38066-difference-between-conv-ifft-fft-when-doing-convolution
        // ifft(fft(a, 14) .* fft(b, 14))

        a.fft().mul_by_other(b.fft()).ifft();

        a
    }

    fn variance(&self) -> Complex64 {
        let signal = self.as_ref();

        let data_mean = self.mean();
        signal
            .iter()
            .map(|value| data_mean - (*value as Complex64))
            .map(|diff| diff * diff)
            .sum::<Complex64>()
            / (signal.len() as f64)
    }

    fn mean(&self) -> Complex64 {
        let signal = self.as_ref();

        let mut sum: Complex64 = signal.iter().sum();
        let len = signal.len() as f64;
        sum.re /= len;
        sum.im /= len;
        sum
    }

    fn reals(&self) -> Vec<f64> {
        let signal = self.as_ref();
        signal.iter().map(|f| f.re).collect()
    }

    fn imag(&self) -> Vec<f64> {
        let signal = self.as_ref();
        signal.iter().map(|f| f.im).collect()
    }

    fn idmax(&self) -> usize {
        let sig = self.as_ref();
        let mut idx_max = 0;
        let max = Complex64::default();
        for (idx, val) in sig.iter().enumerate() {
            if val.norm_sqr() > max.norm_sqr() {
                idx_max = idx;
            }
        }
        idx_max
    }
}

// trait SignalSlicesOwned<const SLI: usize, const LEN: usize> {
//     fn flatten(self) -> [Complex64; SLI * LEN];
// }
// impl<const SLI: usize, const LEN: usize> SignalSlicesOwned<SLI, LEN> for [[Complex64; LEN]; SLI] {
//     fn flatten(self) -> [Complex64; SLI * LEN] {}
// }

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use super::*;

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

        let mid = (data.len() as f64 + 1.0) / 2 as f64;

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

        let mid = (data.len() as f64 + 1.0) / 2 as f64;

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
            Complex64::new(1.0, 0.0),
            Complex64::new(2.0, 0.0),
            Complex64::new(3.0, 0.0),
        ];
        let vals2 = [
            Complex64::new(4.0, 0.0),
            Complex64::new(5.0, 0.0),
            Complex64::new(6.0, 0.0),
        ];
        let out = vals2.convolve(&vals1);
        dbg!(out);
    }

    #[test]
    fn mean_works() {
        let vals = [
            Complex64::new(1.0, 1.0),
            Complex64::new(1.0, 2.0),
            Complex64::new(1.0, 3.0),
        ];

        assert!(vals.mean() == Complex64::new(1.0, 2.0))
    }

    // // #[rustfmt::skip]
    // #[test]
    // fn xcorr_works() {
    //     let x = [1, 2, 3].to_signal();
    //     let h = [4.0, 5.0].to_signal();
    //     let expected = [14.0, 23.0, 12.0].to_signal();

    //     let (idx, lags) = x.xcorr(&h);

    //     dbg!(idx);
    //     assert_eq!(lags, expected);

    //     // more complex
    //     let x = [1, 1, 0, 0, 1, 1, 0, 0].to_signal();

    //     let h = [1, 1, 0, 0].to_signal();

    //     let expected = [2.0, 1.0, 0.0, 1.0, 2.0, 1.0, 0.0, 0.0].to_signal();

    //     let (idx, lags) = x.xcorr(&h);
    //     dbg!(idx);
    //     assert_eq!(lags, expected);
    // }

    #[test]
    fn xcorr_fft_works() {
        let x = [1, 2, 3].to_signal();
        let h = [4.0, 5.0].to_signal();
        let expected = [14.0, 23.0, 12.0].to_signal();

        let (idx, lags) = x.xcorr_fft(&h);

        dbg!(idx);
        assert_eq!(lags, expected);

        // more complex
        let x = [1, 1, 0, 0, 1, 1, 0, 0].to_signal();

        let h = [1, 1, 0, 0].to_signal();

        let expected = [2.0, 1.0, 0.0, 1.0, 2.0, 1.0, 0.0, 0.0].to_signal();

        let (idx, lags) = x.xcorr_fft(&h);
        dbg!(idx);
        assert_eq!(lags, expected);
    }

    #[test]
    fn xcorr_algo() {
        let a = [1, 2, 3, 4].to_signal().to_vec();
        let b = [5, 6, 7, 8].to_signal().to_vec();

        let g = a.xcorr_fft(b);
        dbg!(g);
        let a = [1, 2, 3, 4].to_signal().to_vec();
        let b = [5, 6, 7].to_signal().to_vec();

        let g = a.xcorr_fft(b);
        dbg!(g);

        let a = [1, 2, 3].to_signal().to_vec();
        let b = [5, 6, 7, 8].to_signal().to_vec();

        let g = a.xcorr_fft(b);
        dbg!(g);
    }

    #[test]
    fn fft_test() {
        let mut sig = [-1, 1, 1, -1, 1, -1, 1, -1].to_signal();

        crate::plots::stem_plot(&sig);
        sig.ifft();
        crate::plots::stem_plot(&sig);
        dbg!(sig);
    }

    #[test]
    fn arrayfire_test() {
        crate::logging::set_up_logging("ofdm");

        use arrayfire as af;

        af::set_device(0);
        af::info();
        let samples = 100000;
        let dims = af::Dim4::new(&[samples, 1, 1, 1]);

        let values = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
            .clone()
            .iter()
            .cycle()
            .take(100000)
            .cloned()
            .collect::<Vec<_>>()
            .to_signal();

        // WiFi from scratch with Rust and Nvidia Jetson

        let signal = af::Array::new(&values, dims);
        // let signal = Array::new(&values, dims);
        // let signal = Array::new(&values, dims);

        // af_print!("signal", signal);

        // Used length of input signal as norm_factor
        let t1 = std::time::Instant::now();
        log::debug!("starting fft");
        let output = af::fft(&signal, 0.1, samples as i64);

        dbg!(output.is_linear());

        log::debug!("fft done: {:#?} taken", std::time::Instant::now() - t1);

        // af_print!("Output", output);
    }
}
