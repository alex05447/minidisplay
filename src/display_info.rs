#![allow(clippy::too_many_arguments)]

use std::fmt::{Display, Formatter};

use crate::{Dimensions, Rectangle};

/// Describes the display's upscaling mode.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum UpscaleMode {
    Unknown,
    Center,
    Stretch,
}

impl Display for UpscaleMode {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        use UpscaleMode::*;

        match self {
            Unknown => write!(f, "<unknown>"),
            Center => write!(f, "center"),
            Stretch => write!(f, "stretch"),
        }
    }
}

/// Describes the display's physical connection type.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ConnectionType {
    Unknown,
    VGA,
    DVI,
    HDMI,
    DisplayPort,
    Internal,
}

impl Display for ConnectionType {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        use ConnectionType::*;

        match self {
            Unknown => write!(f, "<unknown>"),
            VGA => write!(f, "VGA"),
            DVI => write!(f, "DVI"),
            HDMI => write!(f, "HDMI"),
            DisplayPort => write!(f, "DisplayPort"),
            Internal => write!(f, "internal"),
        }
    }
}

/// Describes a display's supported fullscreen display mode.
#[derive(Clone, Copy, Debug)]
pub struct DisplayMode {
    /// Display mode dimensions.
    pub dimensions: Dimensions,
    /// Refresh rate in Hz.
    pub refresh_rate: u32,
    /// Refresh rate numerator, such that numerator/denominator gives the refresh rate in Hz.
    pub refresh_rate_num: u32,
    /// Refresh rate denominator, such that numerator/denominator gives the refresh rate in Hz.
    pub refresh_rate_denom: u32,
    /// Display mode upscale mode.
    pub upscale_mode: UpscaleMode,
}

/// Describes the display's rectangles w.r.t. the virtual display.
#[derive(Clone, Copy, Debug)]
pub struct DisplayRects {
    /// Display (non-work, a.k.a. full) rectangle w.r.t. the virtual display.
    pub virtual_rect: Rectangle,
    /// Display work (e.g. with the taskbar subtracted) rectangle w.r.t. the virtual display.
    pub work_rect: Rectangle,
}

/// Describes a single enumerated system display.
#[derive(Clone, Debug)]
pub struct DisplayInfo {
    /// Display's friendly name, if any.
    pub name: Option<String>,
    /// Whether the display is the system's primary display.
    pub is_primary: bool,
    /// The display's rectangles w.r.t. the virtual display.
    pub rects: DisplayRects,
    /// The display's physical connection type.
    pub connection: ConnectionType,
    /// The display's current display mode.
    pub current_mode: DisplayMode,
    /// The display's preferred display mode.
    pub preferred_mode: DisplayMode,
    /// The display's supported (fullscreen) display modes.
    /// At least one display mode is supported by any enumerated display.
    pub display_modes: Vec<DisplayMode>,
    /// The dimensions of the smallest (by area) of the display's supported display modes.
    pub min_dimensions: Dimensions,
    /// The display's DPI scale value.
    /// `1.0` is the default and means no scaling.
    /// Higher values like `1.25`, `1.5`, `2.0` mean higher zoom.
    pub dpi_scale: f32,
}

impl DisplayInfo {
    pub(crate) fn new(
        name: Option<String>,
        is_primary: bool,
        rects: DisplayRects,
        connection: ConnectionType,
        current_mode: DisplayMode,
        preferred_mode: DisplayMode,
        display_modes: Vec<DisplayMode>,
        dpi_scale: f32,
    ) -> Self {
        let min_dimensions = DisplayInfo::calc_min_dimensions(&display_modes);

        Self {
            name,
            is_primary,
            rects,
            connection,
            current_mode,
            preferred_mode,
            display_modes,
            min_dimensions,
            dpi_scale,
        }
    }

    /// Returns the [`dimensions`] of the display's [`display mode`] closest to provided `dimensions`
    /// based on provided `flags`.
    ///
    /// [`dimensions`]: struct.Dimensions.html
    /// [`display mode`]: struct.DisplayMode.html
    pub fn closest_dimensions(
        &self,
        dimensions: Dimensions,
        flags: ClosestDimensionsFlags,
    ) -> Dimensions {
        closest_dimensions(&self.display_modes, dimensions, flags)
    }

    /// Returns the dimensions of the smallest (by area) display mode from a non-empty array of `display_modes`.
    fn calc_min_dimensions(display_modes: &[DisplayMode]) -> Dimensions {
        debug_assert!(!display_modes.is_empty());

        let mut min_area = std::u32::MAX;
        let mut found = None;

        for (index, mode) in display_modes.iter().enumerate() {
            let area = mode.dimensions.area();

            if area < min_area {
                min_area = area;
                found.replace(index);
            }
        }

        display_modes[found.expect("Failed to calculate the minimum display mode dimensions.")]
            .dimensions
    }
}

/// Determines which display mode to pick when looking for one
/// with closest dimensions to provided value.
pub enum ClosestDimensionsFlags {
    /// Pick the display mode with closest (by area) dimensions to provided value,
    /// both smaller or larger.
    Closest,
    /// Pick the display mode with closest (by area) dimensions to provided value,
    /// and additionally not wider/taller.
    ClosestSmallerOrEqual,
}

/// Returns the [`dimensions`] of the [`display mode`] closest to provided `dimensions`
/// based on provided `flags`.
///
/// [`dimensions`]: struct.Dimensions.html
/// [`display mode`]: struct.DisplayMode.html
pub fn closest_dimensions(
    display_modes: &[DisplayMode],
    dimensions: Dimensions,
    flags: ClosestDimensionsFlags,
) -> Dimensions {
    debug_assert!(!display_modes.is_empty());

    let area = dimensions.area();

    let mut min_difference = std::u32::MAX;
    let mut found = None;
    let mut found_smaller = None;

    for (index, mode) in display_modes.iter().enumerate() {
        let mode_area = mode.dimensions.area();
        let area_difference = if mode_area > area {
            mode_area - area
        } else {
            area - mode_area
        };

        if area_difference < min_difference {
            min_difference = area_difference;
            found.replace(index);

            match flags {
                ClosestDimensionsFlags::Closest => {}
                ClosestDimensionsFlags::ClosestSmallerOrEqual => {
                    if (mode.dimensions.width <= dimensions.width)
                        && (mode.dimensions.height <= dimensions.height)
                    {
                        found_smaller.replace(index);
                    }
                }
            }
        }
    }

    let found = found.expect("Failed to find a display mode with closest dimensions.");
    let found_smaller = found_smaller.unwrap_or(found);

    let found = match flags {
        ClosestDimensionsFlags::Closest => found,
        ClosestDimensionsFlags::ClosestSmallerOrEqual => found_smaller,
    };

    display_modes[found].dimensions
}
