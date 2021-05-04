#![allow(non_upper_case_globals)]
use colored::Colorize;
use ofdm::*;
use tap::{Pipe, Tap};

const transmission_bytes: &[u8] = include_bytes!("../support/dancing.bytes");
const num_bytes: usize = transmission_bytes.len();
const guard_bands: bool = true;
const timing_error: bool = true;
const ecc_enabled: bool = true;

fn main() {
    ofdm::logging::set_up_logging("ofdm");
    let i = transmission_bytes.clone().into_iter();
    let source_data = utils::create_transmission_bytes(&mut i.map(|f| *f));

    (&source_data)
        // 1) Encode the data
        .pipe(|data| ofdm::encode!(data, guard_bands))
        .tap(|_| {
            dbg!("asd");
        })
        // 2) Pass through the channel
        .pipe(|transmission| ofdm::channel!(transmission, snr: 30.0, timing_error))
        .tap(|_| {
            dbg!("asd");
        })
        // 3) Receive and decode the samples
        .pipe(|samples| ofdm::decode!(samples, guard_bands).expect("Failed to decode"))
        // 5) print out the analysis
        .pipe(|received_data| {
            // Print the bit data to the terminal
            // utils::debug_data(source_data.as_ref(), &received_data);

            // Compare the sent data to the original
            dbg!(utils::Analysis::new(source_data.as_ref(), &received_data));
            // Convert the bytes back into the original message
            // println!(
            //     "{}",
            let mut received_iter = received_data.into_iter();
            let color_buf = utils::decipher_transmision_colorspace(&mut received_iter, ecc_enabled)
                .expect("Failed to decode text from transmission");
            let dims = (50, 50);

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
            // );
        });
}
