/*!
Miscellaneous utilities grab-bag.

 */

#![crate_name="calx"]
#![feature(core, collections, std_misc)]
#![feature(plugin, custom_attribute, unboxed_closures, slice_patterns)]
#![feature(custom_derive)]
#![plugin(rand_macros)]

#[no_link] extern crate rand_macros;
extern crate collections;
extern crate rustc_serialize;
extern crate time;
extern crate rand;
extern crate num;
extern crate image;
extern crate glutin;
#[macro_use]
extern crate glium;

use std::num::{Float};
use std::env;
use std::path::{Path, PathBuf};
use std::num::Wrapping;

pub use rgb::{ToColor, FromColor, Rgba, color};
pub use geom::{V2, V3, Rect, RectIter};
pub use img::{color_key};
pub use atlas::{AtlasBuilder, Atlas, AtlasItem};
pub use dijkstra::{DijkstraNode, Dijkstra};
pub use hex::{HexGeom, Dir6, HexFov};
pub use rng::{EncodeRng, RngExt};

mod atlas;
mod dijkstra;
mod geom;
mod hex;
mod img;
mod primitive;
mod rgb;
mod rng;

pub mod backend;
pub mod text;
pub mod timing;
pub mod vorud;

/// Clamp a value to range.
pub fn clamp<C: PartialOrd+Copy>(mn: C, mx: C, x: C) -> C {
    if x < mn { mn }
    else if x > mx { mx }
    else { x }
}

/// Deterministic noise.
pub fn noise(n: i32) -> f32 {
    let n = Wrapping(n);
    let n = (n << 13) ^ n;
    let m = (n * (n * n * Wrapping(15731) + Wrapping(789221)) + Wrapping(1376312589))
        & Wrapping(0x7fffffff);
    let Wrapping(m) = m;
    1.0 - m as f32 / 1073741824.0
}

/// Convert probability to a log odds deciban value.
///
/// Log odds correspond to the Bayesian probability for a hypothesis that
/// has decibans * 1/10 log_2(10) bits of evidence in favor of it. They're
/// a bit like rolling a d20 but better.
pub fn to_log_odds(p: f32) -> f32 {
    10.0 * (p / (1.0 - p)).log(10.0)
}

/// Convert a log odds deciban value to the corresponding probability.
pub fn from_log_odds(db: f32) -> f32 {
    (1.0 - 1.0 / (1.0 + 10.0.powf(db / 10.0)))
}

/// Rectangle anchoring points.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Anchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Top,
    Left,
    Right,
    Bottom,
    Center
}

#[cfg(target_os = "macos")]
/// Return the application data directory path for the current platform.
pub fn app_data_path(app_name: &str) -> PathBuf {
    Path::new(
        format!("{}/Library/Application Support/{}",
                env::var("HOME").unwrap(), app_name))
    .to_path_buf()
}

#[cfg(target_os = "windows")]
/// Return the application data directory path for the current platform.
pub fn app_data_path(_app_name: &str) -> PathBuf {
    use std::env;

    // Unless the Windows app was installed with an actual installer instead
    // of just being a portable .exe file, it shouldn't go around creating
    // strange directories but just use the local directory instead.

    // Path::new(
    // format!("{}\\{}", env::var("APPDATA").unwrap(), app_name))
    // .to_path_buf();

    match env::current_exe() {
        Ok(mut p) => { p.pop(); p }
        // If couldn't get self exe path, just use the local relative path and
        // hope for the best.
        _ => Path::new(".").to_path_buf()
    }
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
/// Return the application data directory path for the current platform.
pub fn app_data_path(app_name: &str) -> PathBuf {
    Path::new(
        &format!("{}/.config/{}", env::var("HOME").unwrap(), app_name))
    .to_path_buf()
}

#[cfg(test)]
mod test {
    #[test]
    fn test_noise() {
        use super::noise;

        for i in 0i32..100 {
            assert!(noise(i) >= -1.0 && noise(i) <= 1.0);
        }
    }

    #[test]
    fn test_log_odds() {
        use super::{to_log_odds, from_log_odds};
        assert_eq!(from_log_odds(0.0), 0.5);
        assert_eq!(to_log_odds(0.5), 0.0);

        assert_eq!((from_log_odds(-5.0) * 100.0) as i32, 24);
        assert_eq!(to_log_odds(0.909091) as i32, 10);
    }
}
