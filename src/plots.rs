use num_complex::Complex32;
use textplots::*;

use crate::{IntoSignal, SignalMut, SignalSlice};

pub fn stem_plot(samples: SignalSlice) {
    let pts = samples
        .iter()
        .enumerate()
        .map(|(idx, f)| (idx as f32, f.re as f32))
        // .map(|(idx, f)| (idx as f64, f.re as f64))
        .collect::<Vec<_>>();

    // use cubic_spline::{Point, Points, SplineOpts, TryFrom};

    // let opts = SplineOpts::new().tension(0.5).num_of_segments(100);

    // let mut points = Points::try_from(&pts).unwrap();
    // let pts: Vec<_> = points
    //     .calc_spline(&opts)
    //     .unwrap()
    //     .into_inner()
    //     .into_iter()
    //     .map(|f| (f.x as f32, f.y as f32))
    //     .collect();

    textplots::Chart::new(180, 60, 0.0, samples.len() as f32)
        .lineplot(&Shape::Lines(&pts))
        .display();
}

// Draw the channel impulse function just for funsies
#[test]
fn test_original() {
    draw_channel_plot()
}

pub fn draw_channel_plot() {
    let vals = [
        -0.0000, -0.1912, 0.9316, 0.2821, -0.1990, 0.1630, -0.1017, 0.0544, -0.0261, 0.0090,
        0.0000, -0.0034,
    ]
    .iter()
    .map(|&f| Complex32::new(f, 0.0))
    .collect::<Vec<_>>();

    stem_plot(vals);
}
