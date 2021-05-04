use num::complex::Complex64;
use textplots::*;

use crate::IntoSignal;

pub fn stem_plot(samples: &[Complex64]) {
    let pts = samples
        .iter()
        .enumerate()
        // .map(|(idx, f)| (idx as f64, f.re as f64))
        .map(|(idx, f)| (idx as f64, f.re as f64))
        .collect::<Vec<_>>();

    use cubic_spline::{Points, SplineOpts, TryFrom};

    let opts = SplineOpts::new().tension(0.5).num_of_segments(100);

    let points = Points::try_from(&pts).unwrap();
    let pts: Vec<_> = points
        .calc_spline(&opts)
        .unwrap()
        .into_inner()
        .into_iter()
        .map(|f| (f.x as f32, f.y as f32))
        .collect();

    textplots::Chart::new(240, 60, 0.0, samples.len() as f32)
        .lineplot(&Shape::Lines(&pts))
        .display();
}

pub fn constellation(samples: &[Complex64]) {
    let pts = samples
        .iter()
        .enumerate()
        // .map(|(idx, f)| (idx as f64, f.re as f64))
        .map(|(idx, f)| (f.re as f32, f.im as f32))
        .collect::<Vec<_>>();

    // use cubic_spline::{Points, SplineOpts, TryFrom};

    // let opts = SplineOpts::new().tension(0.5).num_of_segments(100);

    // let points = Points::try_from(&pts).unwrap();
    // let pts: Vec<_> = points
    //     .calc_spline(&opts)
    //     .unwrap()
    //     .into_inner()
    //     .into_iter()
    //     .map(|f| (f.x as f32, f.y as f32))
    //     .collect();

    textplots::Chart::new(240, 240, -2.0, 2.0)
        .lineplot(&Shape::Points(&pts))
        .ymax(2.0)
        .ymin(-2.0)
        .display();
}

// Draw the channel impulse function just for funsies
#[test]
fn test_original() {
    draw_channel_plot()
}

pub fn draw_channel_plot() {
    let vals = crate::CHANNEL.to_signal();

    stem_plot(&vals);
}
#[test]
pub fn draw_channel_plot2() {
    let vals = crate::CHANNEL.to_signal();

    stem_plot(&vals);
}

#[test]
pub fn draw_channel_plot3() {
    let mut vals = crate::CHANNEL.to_signal();
    use crate::signals::SignalMut;
    vals.fft();

    stem_plot(&vals);
}
