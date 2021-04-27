use std::io::Read;

use num_complex::Complex32;

use super::*;

pub trait GetBitAt {
    fn get_bit_at(self, n: u8) -> bool;
}

impl GetBitAt for u8 {
    fn get_bit_at(self, n: u8) -> bool {
        if n < 8 {
            self & (1 << n) != 0
        } else {
            false
        }
    }
}

pub struct Analysis {
    pub num_errs: u32,
    pub err_rate: f32,
}
impl Analysis {
    pub fn new(left: &[u8], right: &[u8]) -> Self {
        let num_errs = left
            .iter()
            .zip(right.iter())
            .fold(0, |acc, (a, b)| if a != b { acc + 1 } else { acc });

        Self {
            num_errs,
            err_rate: num_errs as f32 / left.len() as f32,
        }
    }
}

pub fn create_transmission(packet_size: u32) -> Vec<u8> {
    let text = "abc123";
    let g = text.as_bytes();
    String::from(text).into_bytes();
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_bit_at_is_right() {
        let val: u8 = 255;
        (0..8).for_each(|pos| assert_eq!(val.get_bit_at(pos), true));

        let val: u8 = 0;
        (0..8).for_each(|pos| assert_eq!(val.get_bit_at(pos), false));

        let val: u8 = 127;
        (0..7).for_each(|pos| assert_eq!(val.get_bit_at(pos), true));
        assert_eq!(val.get_bit_at(7), false);
    }

    #[test]
    fn errs_is_right() {
        let Analysis { num_errs, err_rate } = Analysis::new(&[1, 0, 1, 0], &[1, 0, 1, 0]);
        assert_eq!((num_errs, err_rate), (0, 0.0));

        let Analysis { num_errs, err_rate } = Analysis::new(&[1, 0, 0, 0], &[1, 0, 1, 0]);
        assert_eq!((num_errs, err_rate), (1, 0.25));

        let Analysis { num_errs, err_rate } = Analysis::new(&[0, 0, 0, 0], &[1, 0, 1, 0]);
        assert_eq!((num_errs, err_rate), (2, 0.50));
    }

    #[test]
    fn create_transmission_cycle() {}
}
