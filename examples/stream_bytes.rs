use colored::Colorize;
use num::complex::Complex64;
use ofdm::*;
use std::io::prelude::*;

use image::codecs::gif::*;
use image::Rgba;
use std::io::prelude::*;
use std::io::Read;
use std::{fs::File, io};

const guard_bands: bool = true;
const ecc_enabled: bool = true;

fn main() {
    let files = [
        "support/dancing_0.bytes",
        "support/dancing_1.bytes",
        "support/dancing_2.bytes",
        "support/dancing_3.bytes",
        "support/dancing_4.bytes",
        "support/dancing_5.bytes",
        "support/dancing_6.bytes",
        "support/dancing_7.bytes",
    ];

    for (idx, file) in files.iter().enumerate() {
        let mut opened = File::open(file).unwrap();
        let mut bytes = Vec::new();
        opened.read_to_end(&mut bytes).unwrap();

        // let data = utils::create_transmission_text(num_bytes, ecc_enabled);
        let i = bytes.into_iter();
        let data = utils::create_transmission_bytes(&mut i.map(|f| f));

        let samples: Vec<Complex64> = ofdm::encode!(data: data.as_ref(), guard_bands);

        let mut file = File::create(format!("data/tx_dance{}.dat", idx)).unwrap();

        file.write_all(&utils::sig_to_bytes(samples)).unwrap();
    }
}
