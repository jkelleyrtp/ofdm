#![feature(array_chunks)]
#![allow(incomplete_features)]
#![feature(const_generics)]
#![feature(const_evaluatable_checked)]
#![feature(slice_as_chunks)]
//! This program implements lab3 OFDM using the Rust programming language.

mod channel;
pub use channel::*;

mod original;
pub use original::*;

mod receiver;
pub use receiver::*;

mod signals;
pub use signals::*;

mod transmitter;
pub use transmitter::*;

pub mod utils;

pub mod logging;

pub mod plots;

pub mod packets;

#[cfg(test)]
mod tests {
    use super::*;

    use tap::Pipe;

    #[test]
    fn demodulate_works() {
        use crate::transmitter::modulate;

        let input_text = "alskdjas";

        let final_text = input_text
            .as_bytes()
            .pipe(|a| modulate(a, &ModulationScheme::Qpsk))
            .pipe(|a| demodulate(a, ModulationScheme::Qpsk))
            .pipe(String::from_utf8)
            .unwrap();

        assert_eq!(input_text, final_text)
    }

    #[test]
    fn encoding_works() {
        let data = "alskdjas";
        encode(data.as_bytes(), Some(true), None);
    }
}
