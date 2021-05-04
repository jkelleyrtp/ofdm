use super::IntoSignal;
use num::complex::Complex64;

impl IntoSignal for Vec<i32> {
    type SignalOutput = Vec<Complex64>;

    fn to_signal(self) -> Self::SignalOutput {
        self.into_iter()
            .map(|f| Complex64::new(f as f64, 0.0))
            .collect()
    }
}

impl IntoSignal for Vec<f64> {
    type SignalOutput = Vec<Complex64>;

    fn to_signal(self) -> Self::SignalOutput {
        self.into_iter().map(|f| Complex64::new(f, 0.0)).collect()
    }
}

// Only casting of arrays only with known types
impl<const LEN: usize> IntoSignal for [i32; LEN] {
    type SignalOutput = [Complex64; LEN];
    fn to_signal(self) -> Self::SignalOutput {
        let mut out = [Complex64::default(); LEN];

        out.iter_mut()
            .zip(self.iter())
            .for_each(|(slot, &sample)| *slot = Complex64::new(sample as f64, 0.0));

        out
    }
}

// Only casting of arrays only with known types
impl<const LEN: usize> IntoSignal for [f64; LEN] {
    type SignalOutput = [Complex64; LEN];
    fn to_signal(self) -> Self::SignalOutput {
        let mut out = [Complex64::default(); LEN];

        out.iter_mut()
            .zip(self.iter())
            .for_each(|(slot, &sample)| *slot = Complex64::new(sample as f64, 0.0));

        out
    }
}

// Only casting of arrays only with known types
impl<const LEN: usize> IntoSignal for [(f64, f64); LEN] {
    type SignalOutput = [Complex64; LEN];
    fn to_signal(self) -> Self::SignalOutput {
        let mut out = [Complex64::default(); LEN];

        out.iter_mut()
            .zip(self.iter())
            .for_each(|(slot, &sample)| *slot = Complex64::new(sample.0, sample.1));

        out
    }
}

// Only casting of arrays only with known types
impl<const LEN: usize> IntoSignal for [(i32, i32); LEN] {
    type SignalOutput = [Complex64; LEN];
    fn to_signal(self) -> Self::SignalOutput {
        let mut out = [Complex64::default(); LEN];

        out.iter_mut()
            .zip(self.iter())
            .for_each(|(slot, &sample)| *slot = Complex64::new(sample.0 as f64, sample.1 as f64));

        out
    }
}

// Only casting of arrays only with known types
impl IntoSignal for Vec<(i32, i32)> {
    type SignalOutput = Vec<Complex64>;
    fn to_signal(self) -> Self::SignalOutput {
        self.into_iter()
            .map(|(l, r)| Complex64::new(l as f64, r as f64))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::signals::{IntoSignal, SignalMut};

    #[test]
    fn convert_integers_to_signal() {
        let mut g = [1, 2, 3, 4].to_signal();
        g.fft();

        let (_l, _r) = g.split_at(10);
    }

    fn convert_integer_pairs_to_signal() {
        let _g = [
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
