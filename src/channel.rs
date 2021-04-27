use num_complex::Complex32;
use tap::Pipe;

const ARR: [f32; 10] = [0.0, -0.1, 1.0, -0.1, 0.05, -0.01, 0.0, 0.0, 0.0, 0.0];

#[optargs::optfn]
pub fn channel(transmission: Vec<Complex32>, snr: Option<f32>) -> Vec<Complex32> {
    let out_vec = Vec::<Complex32>::with_capacity(transmission.capacity());

    // convolve

    todo!()
}
