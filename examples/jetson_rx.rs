//!
use std::{
    borrow::Borrow,
    ops::Deref,
    sync::{Arc, RwLock},
};

use anyhow::{Context, Result};
use num::complex::Complex64;
use ofdm::*;
use tap::Pipe;
use uhd::{self, StreamCommand, StreamCommandType, StreamTime, TuneRequest, Usrp};

const CHANNEL_SELECT: usize = 0;
const SAMPLE_RATE: f64 = 1e6;
const NUM_SAMPLES: usize = 3_000_000;
const FREQUENCY: f64 = 915e6;
// const FREQUENCY: f64 = 2.4e9;

const guard_bands: bool = true;
const modulation: ModulationScheme = ModulationScheme::Bpsk;
// const modulation: ModulationScheme = ModulationScheme::Qpsk;

type SharedBuffer = Vec<Complex64>;

fn main() -> Result<()> {
    ofdm::logging::set_up_logging("jetson_rx");
    let (send_ready, recv_ready) = std::sync::mpsc::sync_channel::<SharedBuffer>(1);

    let sync_channel = std::thread::spawn(move || -> Result<()> {
        // we will use this thread for the radio business

        let mut usrp = Usrp::find("serial=30C628D")
            .context("Failed to open device list")?
            .drain(..)
            .next()
            .context("Failed to find a valid USRP to attach to")?
            .pipe(|addr| Usrp::open(&addr))
            .context("Failed to find properly open the USRP")?;

        usrp.set_rx_sample_rate(SAMPLE_RATE, CHANNEL_SELECT)?;
        usrp.set_rx_antenna("TX/RX", CHANNEL_SELECT)?;
        usrp.set_rx_frequency(&TuneRequest::with_frequency(FREQUENCY), CHANNEL_SELECT)?;
        usrp.set_rx_gain(150.0, CHANNEL_SELECT, "")?;

        let mut receiver = usrp.get_rx_stream(&uhd::StreamArgs::<Complex64>::new("fc32"))?;

        for i in 0..100 {
            log::debug!("Starting capture {} ", i);
            let mut chan = vec![Complex64::default(); NUM_SAMPLES];
            receiver.receive_simple(chan.as_mut())?;
            log::debug!("Capture {} buf1 finished", i);
            send_ready.send(chan).unwrap();
            // log::debug!("Buffer sent");
        }

        log::debug!("All capturing finished");
        Ok(())
    });

    let dims = (24, 24);
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

    let mut display_samples = vec![0; 892];
    loop {
        if let Ok(samples) = recv_ready.try_recv() {
            log::debug!("starting decode...");
            let res_data = ofdm::decode!(samples, guard_bands, modulation);
            if res_data.is_err() {
                log::debug!("failed to decode");
                continue;
            }
            let data = res_data.unwrap();

            let mut bytes_iter = data.into_iter();
            // let decoded = crate::utils::decipher_transmision_colorspace(500, data, true);
            // let decoded = crate::utils::decipher_transmission_text(900, data, true);

            // if let Some(text) = decoded {
            //     use colored::Colorize;
            //     println!("{}", text.green());
            // todo!("I didn't think we get this far {}", text);
            // }
            log::debug!("bytes: {}", bytes_iter.len());
            let decoded = crate::utils::decipher_transmision_colorspace(&mut bytes_iter, true);
            if let Some(img) = decoded {
                dbg!(img.len());
                log::debug!("We got something? {}", img.len());
                if img.len() != 892 {
                    continue;
                }

                display_samples = img;
            } else {
                log::debug!("Decoding image failed");
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(16));
        window
            .update_with_buffer(&display_samples, width as usize, height as usize)
            .unwrap();
    }

    Ok(())
}
