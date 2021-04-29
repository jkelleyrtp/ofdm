use std::{io::Read, ops::IndexMut, usize};

use num_complex::Complex32;
use uhd::alloc_boxed_slice;

use super::*;

pub trait GetBitAt {
    fn get_bit_at(self, n: u8) -> bool;
    fn to_bools(self) -> [bool; 8];
}

impl GetBitAt for u8 {
    fn get_bit_at(self, n: u8) -> bool {
        if n < 8 {
            self & (1 << n) != 0
        } else {
            false
        }
    }

    fn to_bools(self) -> [bool; 8] {
        let mut out = [false; 8];
        for (bit, o) in out.iter_mut().enumerate() {
            *o = self & (1 << bit) != 0;
        }
        out
    }
}

pub fn bools_to_u8(bools: [bool; 8]) -> u8 {
    let mut out: u8 = 0;
    for (i, &b) in bools.iter().rev().enumerate() {
        out |= (b as u8) << (7 - i);
    }
    out
}

#[derive(Debug, PartialEq)]
pub struct Analysis {
    pub num_errs: u32,
    pub err_rate: f32,
}
impl Analysis {
    pub fn new(left: &[u8], right: &[u8]) -> Self {
        assert_eq!(left.len(), right.len());

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

const CORPUS: &'static str = r#"
I met a traveller from an antique land,
Who said—“Two vast and trunkless legs of stone
Stand in the desert. . . . Near them, on the sand,
Half sunk a shattered visage lies, whose frown,
And wrinkled lip, and sneer of cold command,
Tell that its sculptor well those passions read
Which yet survive, stamped on these lifeless things,
The hand that mocked them, and the heart that fed;
And on the pedestal, these words appear:
My name is Ozymandias, King of Kings;
Look on my Works, ye Mighty, and despair!
Nothing beside remains. Round the decay
Of that colossal Wreck, boundless and bare
The lone and level sands stretch far away.
"#;

pub fn create_transmission<const LEN: usize>() -> Box<[u8; LEN]> {
    let mut out = alloc_boxed_slice::<u8, LEN>();

    CORPUS
        .as_bytes()
        .iter()
        .cycle()
        .take(out.len())
        .zip(out.iter_mut())
        .for_each(|(l, r)| *r = *l);

    out
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

    #[test]
    fn bools_and_back() {
        for num in 0..100_u8 {
            let bools = num.to_bools();
            assert_eq!(bools_to_u8(bools), num);
        }
    }
}
