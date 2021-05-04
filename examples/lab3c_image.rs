#![allow(non_upper_case_globals)]
#![feature(slice_as_chunks)]
use colored::Colorize;
use num::complex::Complex64;
use ofdm::*;
use std::fs::File;
use std::io::prelude::*;

// Just enough for the whole poem
const transmission_bytes: &[u8] = include_bytes!("../support/dancing.bytes");
// const num_bytes: usize = 500;
const guard_bands: bool = true;
const ecc_enabled: bool = true;

fn transmit(path: &str) {
    dbg!(transmission_bytes.len());
    // let data = utils::create_transmission_text(num_bytes, ecc_enabled);
    let i = transmission_bytes.clone().into_iter();
    let data = utils::create_transmission_bytes(&mut i.map(|f| *f));

    let samples: Vec<Complex64> = ofdm::encode!(data: data.as_ref(), guard_bands);

    crate::plots::stem_plot(&samples);

    let mut file = File::create(path).unwrap();
    file.write_all(&utils::sig_to_bytes(samples)).unwrap();
}

fn receive(path: &str, start: Option<usize>, stop: Option<usize>) {
    let mut file = File::open(path).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();
    let samples = unsafe { utils::bytes_to_sig(buf) };

    // let samples = samples[56868..68883].to_vec();
    // let samples = samples[1480846..1503259].to_vec();
    let samples = samples[start.unwrap_or(0)..stop.unwrap_or_else(|| samples.len())].to_vec();
    // let samples = samples[1476048..1503284].to_vec();
    // let samples = samples[815000..samples.len() - 172000].to_vec();
    dbg!(samples.len());

    let received_data = ofdm::decode!(samples, guard_bands).expect("Failed to decode");

    // Debug the output
    let i = transmission_bytes.clone().into_iter();
    let source_data = utils::create_transmission_bytes(&mut i.map(|f| *f));
    // let source_data = utils::create_transmission_text(num_bytes, ecc_enabled);

    // Print the data to the terminal
    // utils::debug_data(source_data.as_ref(), &received_data[..]);

    // Compare the sent data to the original
    dbg!(utils::Analysis::new(source_data.as_ref(), &received_data));

    let mut received_iter = received_data.into_iter();

    log::debug!(" in len: {}", received_iter.len());
    let color_buf = utils::decipher_transmision_colorspace(&mut received_iter, ecc_enabled)
        .expect("Failed to decode text from transmission");

    log::debug!(" colors len: {}", color_buf.len());
    let dims = (24, 24);
    // let dims = (30, 30);
    // let dims = (50, 50);

    use minifb::{Key, ScaleMode, Window, WindowOptions};
    let width = dims.0;
    let height = dims.1;

    let mut window = Window::new(
        "Noise Test - Press ESC to exit",
        width as usize,
        height as usize,
        WindowOptions {
            resize: true,
            scale_mode: ScaleMode::AspectRatioStretch,
            borderless: true,
            scale: minifb::Scale::FitScreen,
            ..WindowOptions::default()
        },
    )
    .expect("Unable to open Window");

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(&color_buf, width as usize, height as usize)
            .unwrap();
    }
    // Convert the bytes back into the original message
    // println!(
    //     "{}",
    //     utils::decipher_transmission_text(num_bytes, received_data, ecc_enabled)
    //         .unwrap()
    //         .green()
    // );
}

/// Run lab3c
#[derive(argh::FromArgs)]
struct CmdArgs {
    /// create the transmission file
    #[argh(option)]
    transmit: Option<String>,

    /// decode the samples from file
    #[argh(option)]
    receive: Option<String>,

    /// start slice of the received data
    #[argh(option)]
    start: Option<usize>,

    /// stop slice of received data
    #[argh(option)]
    stop: Option<usize>,
}

fn main() {
    ofdm::logging::set_up_logging("lab3c_image");
    let cfg: CmdArgs = argh::from_env();
    match (cfg.transmit, cfg.receive) {
        (Some(t), None) => transmit(&t),
        (None, Some(t)) => receive(&t, cfg.start, cfg.stop),
        _ => panic!("Not a valid argument combination, specify transmit or receive, but not both"),
    }
}
