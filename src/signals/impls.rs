use super::IntoSignal;
use num_complex::Complex32;

impl IntoSignal for Vec<i32> {
    type SignalOutput = Vec<Complex32>;

    fn to_signal(self) -> Self::SignalOutput {
        self.into_iter()
            .map(|f| Complex32::new(f as f32, 0.0))
            .collect()
    }
}

impl IntoSignal for Vec<f32> {
    type SignalOutput = Vec<Complex32>;

    fn to_signal(self) -> Self::SignalOutput {
        self.into_iter().map(|f| Complex32::new(f, 0.0)).collect()
    }
}

// Only casting of arrays only with known types
impl<const LEN: usize> IntoSignal for [i32; LEN] {
    type SignalOutput = [Complex32; LEN];
    fn to_signal(self) -> Self::SignalOutput {
        let mut out = [Complex32::default(); LEN];

        out.iter_mut()
            .zip(self.iter())
            .for_each(|(slot, &sample)| *slot = Complex32::new(sample as f32, 0.0));

        out
    }
}

// Only casting of arrays only with known types
impl<const LEN: usize> IntoSignal for [f32; LEN] {
    type SignalOutput = [Complex32; LEN];
    fn to_signal(self) -> Self::SignalOutput {
        let mut out = [Complex32::default(); LEN];

        out.iter_mut()
            .zip(self.iter())
            .for_each(|(slot, &sample)| *slot = Complex32::new(sample as f32, 0.0));

        out
    }
}

// Only casting of arrays only with known types
impl<const LEN: usize> IntoSignal for [(f32, f32); LEN] {
    type SignalOutput = [Complex32; LEN];
    fn to_signal(self) -> Self::SignalOutput {
        let mut out = [Complex32::default(); LEN];

        out.iter_mut()
            .zip(self.iter())
            .for_each(|(slot, &sample)| *slot = Complex32::new(sample.0, sample.1));

        out
    }
}

// Only casting of arrays only with known types
impl<const LEN: usize> IntoSignal for [(i32, i32); LEN] {
    type SignalOutput = [Complex32; LEN];
    fn to_signal(self) -> Self::SignalOutput {
        let mut out = [Complex32::default(); LEN];

        out.iter_mut()
            .zip(self.iter())
            .for_each(|(slot, &sample)| *slot = Complex32::new(sample.0 as f32, sample.1 as f32));

        out
    }
}

#[cfg(test)]
mod tests {
    use crate::signals::{IntoSignal, SignalMut, SignalRef};

    #[test]
    fn convert_integers_to_signal() {
        let mut g = [1, 2, 3, 4].to_signal();
        g.fft();

        let (l, r) = g.split_at(10);
    }

    fn convert_integer_pairs_to_signal() {
        let mut g = [
            (0, 1),
            (2, 3),
            (4, 5),
            (6, 7),
            (8, 9),
            (0, -1),
            (2, -3),
            (4, -5),
            (6, -7),
            (8, -9),
        ];
    }
}
