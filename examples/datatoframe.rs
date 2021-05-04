//! Sketch out a proof of concept of gettting data to the screen
//! The jetsons will send this data to each other

use minifb::{Key, ScaleMode, Window, WindowOptions};
use ofdm::packets::colors;

fn main() {
    let raw_bytes = include_bytes!("../support/dancing.bytes");
    let dims = (50, 50);

    let u32_buffer: Vec<u32> = raw_bytes
        .iter()
        .map(|byte| {
            let color = colors::COLORMAP.get(*byte);
            let colors::CustomRgb { r, g, b } = color.rgb;

            ((r as u32) << 16) | ((g as u32) << 8) | b as u32
        })
        .collect();

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
            .update_with_buffer(&u32_buffer, width as usize, height as usize)
            .unwrap();
    }
}
