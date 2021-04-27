//! Important functions for doing signal processing not normally included in Rust.
//! These are included manually so they can be swapped out with the realtime implementation.

use num_complex::Complex32;

fn xcorr() {}
fn convolve() {}

use basic_dsp::*;

/// Perform a fast fourier transform with constantly known array sizes
pub fn fft<const LEN: usize>(data: &[Complex32; LEN]) -> [Complex32; LEN] {
    let g = Vec::from(data.clone());
}
fn ifft() {}
