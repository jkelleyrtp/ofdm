//! Loads a gif, pre-encodes it, and periodically sends it

use image::codecs::gif::*;
use image::Rgba;
use std::io::prelude::*;
use std::io::Read;
use std::{fs::File, io};

use image::gif::{GifDecoder, GifEncoder};
use image::{AnimationDecoder, ImageDecoder};

use ofdm::packets::{
    colors::{self, COLORMAP},
    gif_to_bytestream,
};

use anyhow::Result;

fn main() -> anyhow::Result<()> {
    let mut raw_gif = File::open("support/dancing_really_small.gif")?;

    let mut bytes = Vec::new();

    raw_gif.read_to_end(&mut bytes)?;

    let (dims, out_bytes) = gif_to_bytestream(&bytes);

    loop {
        println!("asd");
        std::thread::sleep(std::time::Duration::from_millis(125));
    }

    Ok(())
}
