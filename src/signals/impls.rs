use super::IntoSignal;
use num_complex::Complex32;
use std::ops::Deref;

mod constunsized {
    use super::*;

    // impl<const LEN: usize, T> super::super::SignalMut for T
    // where
    //     T: AsMut<[Complex32; LEN]>,
    //     [(); LEN]: ,
    // {
    // }
}

// ====== SignalConst
pub use sigconst::*;
mod sigconst {
    use super::*;

    #[derive(Debug, PartialEq)]
    pub struct SignalConst<const LEN: usize> {
        pub inner: [Complex32; LEN],
    }

    impl<const LEN: usize> SignalConst<LEN> {
        pub fn new() -> Self {
            Self {
                inner: [Complex32::default(); LEN],
            }
        }
    }

    // Asmut/ref
    impl<const LEN: usize> AsRef<[Complex32]> for SignalConst<LEN> {
        fn as_ref(&self) -> &[Complex32] {
            &self.inner
        }
    }
    impl<const LEN: usize> AsMut<[Complex32]> for SignalConst<LEN> {
        fn as_mut(&mut self) -> &mut [Complex32] {
            &mut self.inner
        }
    }

    // Deref/derefmut for iterators
    impl<const LEN: usize> std::ops::Deref for SignalConst<LEN> {
        type Target = [Complex32; LEN];
        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }
    impl<const LEN: usize> std::ops::DerefMut for SignalConst<LEN> {
        fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
            &mut self.inner
        }
    }
}

// ====== Signal Slices =======
pub use sigslice::*;
mod sigslice {
    use super::*;
    #[derive(Debug, PartialEq)]
    pub struct SignalSlice<'a> {
        inner: &'a [Complex32],
    }

    // Asmut/ref
    impl AsRef<[Complex32]> for SignalSlice<'_> {
        fn as_ref(&self) -> &[Complex32] {
            &self.inner
        }
    }
    impl AsMut<[Complex32]> for SignalSlice<'_> {
        fn as_mut(&mut self) -> &mut [Complex32] {
            &mut self.inner
        }
    }

    // Deref/derefmut for iterators
    impl std::ops::Deref for SignalSlice<'_> {
        type Target = [Complex32];
        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }
    impl std::ops::DerefMut for SignalSlice<'_> {
        fn deref_mut(&mut self) -> &mut <Self as Deref>::Target {
            &mut self.inner
        }
    }
}

// ====== Boxed Signal Slices =======
pub use sigboxed::*;
mod sigboxed {
    use super::*;

    #[derive(Debug, PartialEq)]
    pub struct SignalBoxed {
        inner: Box<[Complex32]>,
    }

    impl std::ops::Deref for SignalBoxed {
        type Target = Box<[Complex32]>;

        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }
}

// ====== Signal Vectors =======
pub use sigvec::*;
mod sigvec {
    use super::*;
    #[derive(Debug, PartialEq)]
    pub struct SignalVec {
        pub inner: Vec<Complex32>,
    }

    impl std::ops::Deref for SignalVec {
        type Target = Vec<Complex32>;

        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }

    // Asmut/ref
    impl AsRef<[Complex32]> for SignalVec {
        fn as_ref(&self) -> &[Complex32] {
            &self.inner
        }
    }
    impl AsMut<[Complex32]> for SignalVec {
        fn as_mut(&mut self) -> &mut [Complex32] {
            &mut self.inner
        }
    }
}

mod conversions {
    use super::*;
    impl<'a> IntoSignal for &'a [Complex32] {
        type SignalOutput = SignalSlice<'a>;
        fn to_signal(self) -> Self::SignalOutput {
            SignalSlice { inner: self }
        }
    }

    impl IntoSignal for Vec<i32> {
        type SignalOutput = SignalVec;

        fn to_signal(self) -> Self::SignalOutput {
            SignalVec {
                inner: self.to_signal().inner.to_vec(),
            }
        }
    }

    impl IntoSignal for Vec<f32> {
        type SignalOutput = SignalVec;

        fn to_signal(self) -> Self::SignalOutput {
            SignalVec {
                inner: self.to_signal().inner.to_vec(),
            }
        }
    }

    // Only casting of arrays only with known types
    impl<const LEN: usize> IntoSignal for [i32; LEN] {
        type SignalOutput = SignalConst<LEN>;
        fn to_signal(self) -> Self::SignalOutput {
            let mut out = [Complex32::default(); LEN];

            out.iter_mut()
                .zip(self.iter())
                .for_each(|(slot, &sample)| *slot = Complex32::new(sample as f32, 0.0));

            SignalConst { inner: out }
        }
    }

    // Only casting of arrays only with known types
    impl<const LEN: usize> IntoSignal for [f32; LEN] {
        type SignalOutput = SignalConst<LEN>;
        fn to_signal(self) -> Self::SignalOutput {
            let mut out = [Complex32::default(); LEN];

            out.iter_mut()
                .zip(self.iter())
                .for_each(|(slot, &sample)| *slot = Complex32::new(sample as f32, 0.0));

            SignalConst { inner: out }
        }
    }

    // Only casting of arrays only with known types
    impl<const LEN: usize> IntoSignal for [(f32, f32); LEN] {
        type SignalOutput = SignalConst<LEN>;
        fn to_signal(self) -> Self::SignalOutput {
            let mut out = [Complex32::default(); LEN];

            out.iter_mut()
                .zip(self.iter())
                .for_each(|(slot, &sample)| *slot = Complex32::new(sample.0, sample.1));

            SignalConst { inner: out }
        }
    }

    // Only casting of arrays only with known types
    impl<const LEN: usize> IntoSignal for [(i32, i32); LEN] {
        type SignalOutput = SignalConst<LEN>;
        fn to_signal(self) -> Self::SignalOutput {
            let mut out = [Complex32::default(); LEN];

            out.iter_mut().zip(self.iter()).for_each(|(slot, &sample)| {
                *slot = Complex32::new(sample.0 as f32, sample.1 as f32)
            });

            SignalConst { inner: out }
        }
    }

    // Only casting of arrays only with known types
    impl<const LEN: usize> IntoSignal for [Complex32; LEN] {
        type SignalOutput = SignalConst<LEN>;
        fn to_signal(self) -> Self::SignalOutput {
            SignalConst { inner: self }
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
}

mod compares {
    use super::*;
    use crate::signals::*;

    impl<const LEN: usize> PartialEq<super::SignalVec> for SignalConst<LEN> {
        fn eq(&self, other: &super::SignalVec) -> bool {
            let a = self.as_ref();
            let b = other.as_ref();
            a == b
        }
    }

    impl<const LEN: usize> PartialEq<SignalConst<LEN>> for super::SignalVec {
        fn eq(&self, other: &SignalConst<LEN>) -> bool {
            let a = self.as_ref();
            let b = other.as_ref();
            a == b
        }
    }
}
