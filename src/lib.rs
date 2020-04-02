//! # minidisplay
//!
//! A small Rust library that enumerates the system's displays / monitors.
//!
//! Implemented for Windows only.
//!
//! ## Dependencies
//!
//! [`bitflags`](http://crates.io/crates/bitflags).
//!
//! On Windows, [`winapi`](http://crates.io/crates/winapi).

mod display_info;
mod displays;
mod rectangle;

#[cfg(windows)]
mod win;

#[macro_use]
extern crate bitflags;

pub use display_info::{
    closest_dimensions, ClosestDimensionsFlags, ConnectionType, DisplayInfo, DisplayMode,
    DisplayRects, UpscaleMode,
};
pub use displays::{AdjacencyInfo, DisplayInfoFull, DisplayInfoIter, Displays};
pub use rectangle::{ClipRectFlags, Dimensions, Position, Rectangle};

#[cfg(windows)]
pub use win::DisplayInfoWin as DisplayInfoPlatform;
