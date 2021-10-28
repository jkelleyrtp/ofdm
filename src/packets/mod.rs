//! Useful tools for encoding packets

use std::fs::{File, OpenOptions};

use image::codecs::gif::*;
use image::Rgba;
use std::io;
use std::io::prelude::*;
use std::io::Read;

use image::gif::{GifDecoder, GifEncoder};
use image::{AnimationDecoder, ImageDecoder};
use serde::{Deserialize, Serialize};

use crate::packets::colors::COLORMAP;

pub mod colors;
pub mod compression;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Header {
    // Allow receivers to peak how long the transmission is
    // This allows variably encoded transmission lengths for sending arbitrary data
    // The packet length determines how many *bytes* was used to create the original transmisison data
    pub packet_length: u128,
}

impl Header {
    pub fn new(packet_length: u128) -> Self {
        Self { packet_length }
    }
}

#[test]
fn header_size() {
    let sample_header = Header::new(100);
    let bytes = bincode::serialize(&sample_header).unwrap();
    let decoded: Header = bincode::deserialize(&bytes).unwrap();
    dbg!(decoded);
}

/// A small script/utility that writes the dancing gif to 1-byte color bytes
#[test]
fn image_to_custom_colorspace() {
    //

    // let mut bits = include_bytes!("../../support/dancing_smaller.gif");
    let mut bits = include_bytes!("../../support/dancing_super_small.gif");

    let (dims, out_bytes) = gif_to_bytestream(bits);

    for (id, frame) in out_bytes.into_iter().enumerate() {
        let mut out_file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(false)
            .open(format!("support/dancing_{}.bytes", id))
            .unwrap();

        // let first_frame = out_bytes.first().unwrap();
        out_file.write_all(&frame).unwrap();
    }

    // dbg!(out_bytes.len());
}

pub fn gif_to_bytestream(bytes: &[u8]) -> ((u32, u32), Vec<Vec<u8>>) {
    let mut decoder = GifDecoder::new(bytes).unwrap();

    let dims = decoder.dimensions();

    let frames = decoder.into_frames();
    let g = frames.collect_frames().expect("error_decoding");

    let mut out_bytes: Vec<Vec<u8>> = Vec::new();
    for frame in g {
        let mut raw_bytes = frame.into_buffer().into_raw().into_iter();

        let mut frame_bytes = Vec::with_capacity(raw_bytes.len());

        while let (Some(r), Some(g), Some(b), Some(a)) = (
            raw_bytes.next(),
            raw_bytes.next(),
            raw_bytes.next(),
            raw_bytes.next(),
        ) {
            let approx = colors::COLORMAP.get_closest(r, g, b);
            frame_bytes.push(approx.color_id);
        }

        out_bytes.push(frame_bytes);
    }

    (dims, out_bytes)
}

#[test]
fn load_custom_color_pallete() {
    // let raw_bytes = include_bytes!("../../support/dancing.bytes");
    // let dims = (400, 400);

    // // let mut rbga_buffer = Vec::with_capacity(raw_bytes.len() * 3);

    // // for byte in raw_bytes {
    // //     let color = COLORMAP.get(*byte);
    // //     rbga_buffer.push(color.rgb.r);
    // //     rbga_buffer.push(color.rgb.g);
    // //     rbga_buffer.push(color.rgb.b);
    // // }

    // use minifb::{Key, ScaleMode, Window, WindowOptions};

    // // let mut buf = vec![0; raw_bytes];
    // let u32_buffer: Vec<u32> = raw_bytes
    //     .iter()
    //     .map(|byte| {
    //         let color = COLORMAP.get(*byte);
    //         let colors::CustomRgb { r, g, b } = color.rgb;

    //         ((r as u32) << 16) | ((g as u32) << 8) | b as u32
    //     })
    //     .collect();

    // let width = dims.0;
    // let height = dims.1;

    // let mut window = Window::new(
    //     "Noise Test - Press ESC to exit",
    //     width as usize,
    //     height as usize,
    //     WindowOptions {
    //         resize: true,
    //         scale_mode: ScaleMode::Center,
    //         ..WindowOptions::default()
    //     },
    // )
    // .expect("Unable to open Window");

    // while window.is_open() && !window.is_key_down(Key::Escape) {
    //     window
    //         .update_with_buffer(&u32_buffer, width as usize, height as usize)
    //         .unwrap();
    // }
}
