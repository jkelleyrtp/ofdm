use std::convert::TryInto;

use kdtree::kdtree::*;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

// https://jonasjacek.github.io/colors/
// We use the top 255 colors.
// Each byte we send corresponds to this color
pub static COLORMAP: Lazy<ColorMap> = Lazy::new(ColorMap::new);

pub struct ColorMap {
    colors: [ColorEntry; 256],
    tree: Kdtree<RgbColorPoint>,
}

impl ColorMap {
    fn new() -> Self {
        let map: Vec<ColorEntry> =
            serde_json::from_str(include_str!("../../support/colors.json")).unwrap();
        let colors: [ColorEntry; 256] = map.try_into().unwrap();

        let mut pts = colors
            .iter()
            .enumerate()
            .map(|(idx, color)| {
                RgbColorPoint::new(idx as u8, color.rgb.r, color.rgb.g, color.rgb.b)
            })
            .collect::<Vec<_>>();

        let tree = Kdtree::new(&mut pts);

        Self { colors, tree }
    }

    pub fn get(&self, id: u8) -> &ColorEntry {
        &self.colors[id as usize]
    }

    pub fn get_closest(&self, r: u8, g: u8, b: u8) -> &ColorEntry {
        let neighbor = self.tree.nearest_search(&RgbColorPoint::new(0, r, g, b));
        &self.colors[neighbor.id as usize]
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ColorEntry {
    pub color_id: u8,
    pub hex_string: String,
    pub rgb: CustomRgb,
    pub name: String,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct CustomRgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct RgbColorPoint {
    color_pos: [f64; 3],
    id: u8,
}

impl RgbColorPoint {
    fn new(id: u8, r: u8, g: u8, b: u8) -> Self {
        Self {
            color_pos: [r as f64, g as f64, b as f64],
            id,
        }
    }
}

impl KdtreePointTrait for RgbColorPoint {
    #[inline]
    fn dims(&self) -> &[f64] {
        &self.color_pos
    }
}

#[test]
fn color_space_works() {
    assert_eq!(&COLORMAP.get(0).name, "Black");
    assert_eq!(&COLORMAP.get(1).name, "Maroon");
}

#[test]
fn color_get() {
    let color = COLORMAP.get_closest(0, 215, 215);
    dbg!(color);
}
