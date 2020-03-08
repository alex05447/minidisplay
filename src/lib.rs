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
pub use displays::{AdjacencyInfo, Displays};
pub use rectangle::{ClipRectFlags, Dimensions, Position, Rectangle};

#[cfg(windows)]
pub use win::DisplayInfoWin as DisplayInfoPlatform;
